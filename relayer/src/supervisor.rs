use std::{
    collections::HashMap,
    fmt,
    thread::{self, JoinHandle},
    time::Duration,
};

use anomaly::BoxError;
use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, info, trace, warn};

use crate::channel::ChannelSide;
use ibc::ics02_client::client_state::ClientState;
use ibc::ics02_client::events::UpdateClient;
use ibc::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::{
    events::VecIbcEvents,
    ics04_channel::events::{OpenAck, OpenConfirm, OpenInit, OpenTry},
};

use ibc::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::{
    events::IbcEvent,
    ics02_client::events::NewBlock,
    ics03_connection::connection::State as ConnectionState,
    ics04_channel::{
        channel::State as ChannelState,
        events::{CloseInit, SendPacket, TimeoutPacket, WriteAcknowledgement},
    },
    ics24_host::identifier::{ChainId, ChannelId, PortId},
    Height,
};
use ibc_proto::ibc::core::channel::v1::QueryChannelsRequest;

use crate::channel::Channel as RelayChannel;
use crate::foreign_client::ForeignClient;

use crate::{
    chain::handle::ChainHandle,
    event::monitor::EventBatch,
    link::{Link, LinkParameters},
};

/// A command for a [`Worker`].
pub enum WorkerCmd {
    /// A batch of packet events need to be relayed
    IbcEvents { batch: EventBatch },
    /// A batch of [`NewBlock`] events need to be relayed
    NewBlock { height: Height, new_block: NewBlock },
}

/// Handle to a [`Worker`], for sending [`WorkerCmd`]s to it.
pub struct WorkerHandle {
    tx: Sender<WorkerCmd>,
    thread_handle: JoinHandle<()>,
}

impl WorkerHandle {
    /// Send a batch of packet events to the worker.
    pub fn send_events(
        &self,
        height: Height,
        events: Vec<IbcEvent>,
        chain_id: ChainId,
    ) -> Result<(), BoxError> {
        let batch = EventBatch {
            height,
            events,
            chain_id,
        };

        trace!("supervisor sends {:?}", batch);
        self.tx.send(WorkerCmd::IbcEvents { batch })?;
        Ok(())
    }

    /// Send a batch of [`NewBlock`] event to the worker.
    pub fn send_new_block(&self, height: Height, new_block: NewBlock) -> Result<(), BoxError> {
        self.tx.send(WorkerCmd::NewBlock { height, new_block })?;
        Ok(())
    }

    /// Wait for the worker thread to finish.
    pub fn join(self) -> thread::Result<()> {
        self.thread_handle.join()
    }
}

/// A pair of [`ChainHandle`]s.
#[derive(Clone)]
pub struct ChainHandlePair {
    pub a: Box<dyn ChainHandle>,
    pub b: Box<dyn ChainHandle>,
}

impl ChainHandlePair {
    /// Swap the two handles.
    pub fn swap(self) -> Self {
        Self {
            a: self.b,
            b: self.a,
        }
    }
}

/// The supervisor listens for events on a pair of chains,
/// and dispatches the events it receives to the appropriate
/// worker, based on the [`Object`] associated with each event.
pub struct Supervisor {
    chains: ChainHandlePair,
    workers: HashMap<Object, WorkerHandle>,
}

impl fmt::Display for Supervisor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} <-> {}]", self.chains.a.id(), self.chains.b.id(),)
    }
}

impl Supervisor {
    /// Spawn a supervisor which listens for events on the two given chains.
    pub fn spawn(
        chain_a: Box<dyn ChainHandle>,
        chain_b: Box<dyn ChainHandle>,
    ) -> Result<Self, BoxError> {
        let chains = ChainHandlePair {
            a: chain_a,
            b: chain_b,
        };

        Ok(Self {
            chains,
            workers: HashMap::new(),
        })
    }

    fn create_workers(&mut self) -> Result<(), BoxError> {
        let req = QueryChannelsRequest {
            pagination: ibc_proto::cosmos::base::query::pagination::all(),
        };

        // For each opened channel spawn:
        // - the path worker for AtoB (used to clear packets) and
        // - the client worker for the A side client of the channel's connection (used for refresh)
        // TODO - we should move to one supervisor, operating on a set of chains:
        // `for chain_a in self.chains.clone().iter() {`, lookup chain b in registry based
        // on the client chain_id, etc.
        for chains in [self.chains.clone(), self.chains.clone().swap()].iter() {
            let channels: Vec<IdentifiedChannelEnd> = chains.a.query_channels(req.clone())?;
            for channel in channels.iter() {
                // TODO next - more thought needed here as optimistic sends will fail to clear
                // maybe instead just check if there are pending packets...but the new problem
                // will be the reference height for clear vs events, etc.

                // get the channel's connection
                let connection_id =
                    channel
                        .channel_end
                        .connection_hops()
                        .first()
                        .ok_or_else(|| {
                            format!("no connection hops for channel '{}'", channel.channel_id)
                        })?;
                let connection = chains.a.query_connection(&connection_id, Height::zero())?;

                // get the client used by the connection and check that the other end is on chain b
                let client_id = connection.client_id();
                let client = chains.a.query_client_state(client_id, Height::zero())?;
                if client.chain_id() != chains.b.id() {
                    continue;
                }
                // Only clear packets for opened channels
                // if !channel
                //     .channel_end
                //     .state_matches(&ibc::ics04_channel::channel::State::Open)
                // {
                // create the channel object and spawn worker to finish handshake
                let channel_object = Object::Channel(Channel {
                    dst_chain_id: chains.b.id(),
                    src_chain_id: chains.a.id(),
                    src_channel_id: channel.channel_id.clone(),
                    src_port_id: channel.port_id.clone(),
                    connection_id: connection_id.clone(),
                });

                debug!(
                    "create workers: creating a worker for object {:?}",
                    channel_object
                );
                let worker = Worker::spawn(chains.clone(), channel_object.clone());
                self.workers.entry(channel_object).or_insert(worker);

                //     continue;
                // }

            //     //Only clear packets for opened channels
            //     if channel
            //         .channel_end
            //         .state_matches(&ibc::ics04_channel::channel::State::Open)
            //     {
            //         // create the client object and spawn worker
            //         let client_object = Object::Client(Client {
            //             dst_client_id: client_id.clone(),
            //             dst_chain_id: chains.a.id(),
            //             src_chain_id: client.chain_id(),
            //         });
            //         let worker = Worker::spawn(chains.clone(), client_object.clone());
            //         self.workers.entry(client_object).or_insert(worker);

            //         // create the path object and spawn worker
            //         let path_object = Object::UnidirectionalChannelPath(UnidirectionalChannelPath {
            //             dst_chain_id: chains.b.id(),
            //             src_chain_id: chains.a.id(),
            //             src_channel_id: channel.channel_id.clone(),
            //             src_port_id: channel.port_id.clone(),
            //         });
            //         let worker = Worker::spawn(chains.clone(), path_object.clone());
            //         self.workers.entry(path_object).or_insert(worker);
            //    }
            }
        }
        Ok(())
    }

    /// Run the supervisor event loop.
    pub fn run(mut self) -> Result<(), BoxError> {
        let subscription_a = self.chains.a.subscribe()?;
        let subscription_b = self.chains.b.subscribe()?;

        self.create_workers()?;

        loop {
            for batch in subscription_a.try_iter() {
                debug!("process batch on chain {:?}", self.chains.a.id());
                self.process_batch(self.chains.a.clone(), batch.unwrap_or_clone())?;
            }

            for batch in subscription_b.try_iter() {
                debug!("process batch on chain {:?}", self.chains.b.id());
                self.process_batch(self.chains.b.clone(), batch.unwrap_or_clone())?;
            }

            std::thread::sleep(Duration::from_millis(600));
        }
    }

    /// Process a batch of events received from a chain.
    fn process_batch(
        &mut self,
        src_chain: Box<dyn ChainHandle>,
        batch: EventBatch,
    ) -> Result<(), BoxError> {
        assert_eq!(src_chain.id(), batch.chain_id);

        let height = batch.height;
        let chain_id = batch.chain_id.clone();

        let direction = if chain_id == self.chains.a.id() {
            Direction::AtoB
        } else {
            assert_eq!(chain_id, self.chains.b.id());
            Direction::BtoA
        };

        let mut collected = collect_events(src_chain.as_ref(), batch);

        for (object, events) in collected.per_object.drain() {
            if events.is_empty() {
                continue;
            }

            debug!(
                "[{}] chain {} sent {} for object {:?}, direction {:?}",
                self,
                chain_id,
                VecIbcEvents(events.clone()),
                object,
                direction
            );

            if let Some(worker) = self.worker_for_object(object, direction) {
                worker.send_events(height, events, chain_id.clone())?;
            }
        }

        if let Some(IbcEvent::NewBlock(new_block)) = collected.new_block {
            for (object, worker) in self.workers.iter() {
                match object {
                    Object::UnidirectionalChannelPath(p) => {
                        if p.src_chain_id == src_chain.id() {
                            worker.send_new_block(height, new_block)?;
                        }
                    }
                    //Object::Client(_) => {}
                    Object::Channel(_) => {}
                }
            }
        }

        Ok(())
    }

    /// Get a handle to the worker in charge of handling events associated
    /// with the given [`Object`].
    ///
    /// This function will spawn a new [`Worker`] if one does not exists already.
    ///
    /// The `direction` parameter indicates in which direction the worker should
    /// relay events.
    fn worker_for_object(&mut self, object: Object, direction: Direction) -> Option<&WorkerHandle> {
        if self.workers.contains_key(&object) {
            Some(&self.workers[&object])
        } else {
            let chains = match direction {
                Direction::AtoB => self.chains.clone(),
                Direction::BtoA => self.chains.clone().swap(),
            };

            if object.src_chain_id() != &chains.a.id() || object.dst_chain_id() != &chains.b.id() {
                trace!(
                    "object {:?} is not relevant to worker for chains {}/{}",
                    object,
                    chains.a.id(),
                    chains.b.id()
                );

                return None;
            }

            debug!(
                "process_batch -- worker for object creating a worker for object {:?}",
                object
            );

            let worker = Worker::spawn(chains, object.clone());
            let worker = self.workers.entry(object).or_insert(worker);
            Some(worker)
        }
    }
}

/// The direction in which a [`Worker`] should relay events.
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    /// From chain A to chain B.
    AtoB,
    /// From chain B to chain A.
    BtoA,
}

/// A worker processes batches of events associated with a given [`Object`].
pub struct Worker {
    chains: ChainHandlePair,
    rx: Receiver<WorkerCmd>,
}

impl fmt::Display for Worker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} <-> {}]", self.chains.a.id(), self.chains.b.id(),)
    }
}

impl Worker {
    /// Spawn a worker which relay events pertaining to `object` between two `chains`.
    pub fn spawn(chains: ChainHandlePair, object: Object) -> WorkerHandle {
        let (tx, rx) = crossbeam_channel::unbounded();

        debug!(
            "[{}] Spawned worker with chains a:{} and b:{} for object {:#?} ",
            object.short_name(),
            chains.a.id(),
            chains.b.id(),
            object,
        );

        let worker = Self { chains, rx };
        let thread_handle = std::thread::spawn(move || worker.run(object));

        WorkerHandle { tx, thread_handle }
    }

    /// Run the worker event loop.
    fn run(self, object: Object) {
        let result = match object.clone() {
            Object::UnidirectionalChannelPath(path) => self.run_uni_chan_path(path),
            //Object::Client(client) => self.run_client(client),
            Object::Channel(channel) => self.run_channel(channel),
        };

        if let Err(e) = result {
            eprintln!("[{}] worker error: {}", object.short_name(), e);
        }
        info!("[{}] worker exits", object.short_name());
    }

    /// Run the event loop for events associated with a [`Client`].
    fn run_client(self, client: Client) -> Result<(), BoxError> {
        let mut client = ForeignClient::restore(
            &client.dst_client_id,
            self.chains.a.clone(),
            self.chains.b.clone(),
        );

        info!("[{}] running client worker for {}", self, client);

        info!(
            "[{}] running initial misbehaviour detection for {}",
            self, client
        );

        // initial check for evidence of misbehaviour for all updates
        if !client
            .detect_misbehaviour_and_submit_evidence(None)?
            .is_empty()
        {
            return Ok(());
        }

        info!(
            "[{}] running client worker (misbehaviour and refresh) for {}",
            self, client
        );
        loop {
            if let Ok(WorkerCmd::IbcEvents { batch }) = self.rx.try_recv() {
                trace!("[{}] client receives batch {:?}", client, batch);

                for event in batch.events {
                    if let IbcEvent::UpdateClient(update) = event {
                        debug!("[{}] client updated", client);
                        let result = client
                            .detect_misbehaviour_and_submit_evidence(Some(update))
                            .map_err(|e| {
                                format!(
                                    "[{}] could not run misbehaviour detection for {}: {}",
                                    self, client, e
                                )
                            })?;
                        if result.is_empty() {
                            break;
                        }
                    }
                }
            }

            client.refresh()?;
            thread::sleep(Duration::from_millis(600))
        }
    }

    /// Run the event loop for events associated with a [`Channel`].
    fn run_channel(self, channel: Channel) -> Result<(), BoxError> {
        let done = '🥳';

        let a_chain = self.chains.a.clone();
        let b_chain = self.chains.b.clone();

        //TODO chage: the chain already queried for this info
        let connection = self
            .chains
            .a
            .query_connection(&channel.connection_id, Height::zero())?;
        let a_channel = self.chains.a.query_channel(
            &channel.src_port_id,
            &channel.src_channel_id,
            Height::zero(),
        )?;

        let mut state = &ibc::ics04_channel::channel::State::Uninitialized;

        let mut b_channel = Default::default();

        
        let counterparty_channel_id = if a_channel.remote.channel_id.is_none() {  
            Default::default()
        } else {
            b_channel = self.chains.b.query_channel(
                &a_channel.remote.port_id.clone(),
                &a_channel.remote.channel_id.clone().unwrap(),
                Height::zero(),
            )?;
            state = &b_channel.state;
            a_channel.remote.channel_id.clone().unwrap()
        };

        let mut handshake_channel = RelayChannel {
            ordering: a_channel.ordering().clone(),
            a_side: ChannelSide::new(
                a_chain.clone(),
                connection.client_id().clone(),
                channel.connection_id.clone(),
                channel.src_port_id.clone(),
                channel.src_channel_id.clone(),
            ),
            b_side: ChannelSide::new(
                b_chain.clone(),
                connection.counterparty().client_id().clone(),
                connection.counterparty().connection_id().unwrap().clone(),
                a_channel.remote.port_id.clone(),
                counterparty_channel_id.clone(),
            ),
            connection_delay: connection.delay_period(),
            version: Some(a_channel.version.clone()),
        };

        let mut stage = 0; //Nothing started

        debug!(
            "\n [{}] initial handshake_channel is {:?}  \n ",
            channel.short_name(),
            handshake_channel
        );

        if a_channel.state_matches(&ibc::ics04_channel::channel::State::Init) {
            if a_channel.remote.channel_id.is_none() {

                let req = QueryChannelsRequest {
                    pagination: ibc_proto::cosmos::base::query::pagination::all(),
                };

                let mut found = false; 
                let channels: Vec<IdentifiedChannelEnd> = b_chain.query_channels(req.clone())?;
                for chan in channels.iter() { 
                    if chan.channel_end.remote.channel_id.is_some() && 
                        chan.channel_end.remote.channel_id.clone().unwrap() ==  channel.src_channel_id.clone() {
                           
                            debug!("[{}] found a pair channel {} on chain {}",channel.short_name(),chan.channel_id, handshake_channel.b_side.chain_id());
                            found = true; 
                            break;
                    }
                }
                stage = 1; // channel in Init 

                if !found {
                    println!(
                        "\n [{}] sends build_chan_open_try_and_send \n on handshake_channel {:?}  channel in state Init \n",
                        channel.short_name(),
                        handshake_channel
                    );

                    match handshake_channel.build_chan_open_try_and_send() {
                        Err(e) => {
                            debug!("Failed ChanTry {:?}: {:?}", handshake_channel.b_side, e);
                        }
                        Ok(event) => {
                            println!("{}  {} => {:#?}\n", done, b_chain.id(), event);
                        }
                    }
                }
            }
        } else {
            if a_channel.state_matches(&ibc::ics04_channel::channel::State::TryOpen) {
                
                stage = 2; //channel is in Try Open 

                if a_channel.remote.channel_id.is_some() {

                    //Try chanOpenTry on b_chain
                    debug!("[{}] chain {} has channel {} in state TryOpen with counterparty {} in state {} \n", channel.short_name(), a_chain.id(), channel.src_channel_id.clone(), counterparty_channel_id.clone(), state);

                    if !b_channel.state_matches(&ibc::ics04_channel::channel::State::Open) {
                       
                        debug!(
                            "\n [{}] sends build_chan_open_ack_and_send \n on handshake_channel {:?}",
                            channel.short_name(), 
                            handshake_channel
                        );

                        match handshake_channel.build_chan_open_ack_and_send() {
                            Err(e) => {
                                debug!("Failed ChanAck {:?}: {:?}", handshake_channel.b_side, e);
                            }
                            Ok(event) => {
                                // handshake_channel.b_side.channel_id = extract_channel_id(&event)?.clone();
                                println!("{}  {} => {:#?}\n", done, b_chain.id(), event);
                            }
                        }

                    }//TODO else either counter party channel is more advanced or another channel closed the hanshake  
                } //TODO else error
            } else {
               
                match (a_channel.state().clone(), state) {
                    (
                        ibc::ics04_channel::channel::State::Open,
                        ibc::ics04_channel::channel::State::TryOpen,
                    ) => {
                        stage = 3; // channel is Open 
                        debug!(
                            "[{}] chain {} has channel {} in state Open counterparty TryOpen \n",
                            channel.short_name(),
                            a_chain.id(),
                            channel.src_channel_id.clone()
                        );

                        // Confirm to b_chain
                        debug!(
                            "[{}] sends build_chan_open_confirm_and_send \n on handshake_channel {:?}",
                            channel.short_name(),
                            handshake_channel
                        );

                        match handshake_channel.build_chan_open_confirm_and_send() {
                            Err(e) => {
                                debug!(
                                    "Failed OpenConfirm {:?}: {:?}",
                                    handshake_channel.b_side, e
                                );
                            }
                            Ok(event) => {
                                println!("{}  {} => {:#?}\n", done, b_chain.id(), event);
                            }
                        }
                    }
                    (
                        ibc::ics04_channel::channel::State::Open,
                        ibc::ics04_channel::channel::State::Open,
                    ) => {

                      //  stage = 3; //Channel is Open
                        println!(
                            "[{}]{}  {}  {}  Channel handshake finished for {:#?}\n",
                            channel.short_name(),
                            done,
                            done,
                            done,
                            &channel.src_channel_id,
                        );
                        return Ok(());
                    }
                    _ => {
                        debug!("[{}] \n Error Unimplemented handshake case \n", channel.short_name())
                    }
                }
            }
        };

        loop {
            if let  Ok(WorkerCmd::IbcEvents { batch })  = self.rx.try_recv() {
                // Ok(cmd)
                // match cmd {
                //     WorkerCmd::IbcEvents { batch } => {
                        for event in batch.events {
                            match event {
                                IbcEvent::OpenInitChannel(_open_init) => {}

                                IbcEvent::OpenTryChannel(_open_try) => {}

                                IbcEvent::OpenAckChannel(open_ack) => {
                                    debug!(" \n [{}] {} channel handshake OpenAck  from {:?} {} channel from event OpenAck \n", 
                                        channel.short_name(),
                                        handshake_channel.a_side.channel_id(),
                                        handshake_channel.a_side.chain_id(),
                                        open_ack.channel_id().clone().unwrap()
                                    );


                                    if stage >=3 {
                                        debug! ("[{}] channel in stage >= OpenAck that is it already processed its OpenAck event ", channel.short_name());
                                        continue;
                                    }
                                    stage = 3 ;
                                

                                    let a_channel = self.chains.a.query_channel(
                                        &channel.src_port_id,
                                        &channel.src_channel_id,
                                        Height::zero(),
                                    )?;

                                    let b_channel = self.chains.b.query_channel(
                                        &a_channel.remote.port_id.clone(),
                                        &a_channel.remote.channel_id.clone().unwrap(),
                                        Height::zero(),
                                    )?;

                                    debug!(
                                        "[{}] hanshake_channel b_side channel id is {}",
                                        channel.short_name(),
                                        handshake_channel.b_side.channel_id()
                                    );

                                    if a_channel
                                        .state_matches(&ibc::ics04_channel::channel::State::Open)
                                    {
                                        debug!(
                                            " [{}] channel {} is OPEN \n",
                                            channel.short_name(),
                                            channel.src_channel_id
                                        );
                                        if a_channel.remote.channel_id.is_some() {
                                            if b_channel.state_matches(
                                                &ibc::ics04_channel::channel::State::TryOpen,
                                            ) {
                                                debug!(
                                                    " [{}] channel {} is TryOPEN \n ",
                                                    channel.short_name(),
                                                    a_channel.remote.channel_id.clone().unwrap()
                                                );


                                                debug!(
                                                    "\n [{}] sends build_chan_open_confirm_and_send \n on handshake_channel {:?}",
                                                    channel.short_name(),
                                                    handshake_channel
                                                );  

                                                debug!("[{}] writting b_side before open_confirm {} ", channel.short_name(),handshake_channel.b_side.channel_id);

                                                handshake_channel.b_side.channel_id = a_channel.remote.channel_id.clone().unwrap();
                        
                                                debug!("[{}] writting b_side after open_confirm {} ", channel.short_name(),handshake_channel.b_side.channel_id);

                                                let event = handshake_channel
                                                    .build_chan_open_confirm_and_send()?;
                                                println!(
                                                    "{}  {} => {:#?}\n",
                                                    done,
                                                    b_chain.id(),
                                                    event
                                                );
                                            }
                                        } //TODO else error
                                    }

                                
                                }
                                IbcEvent::OpenConfirmChannel(open_confirm) => {
                                    debug!("[{}] {} channel handshake OpenConfirm [{}] channel from event OpenConfirm {} ", 
                                    channel.short_name(),
                                    handshake_channel.a_side.channel_id(),
                                    handshake_channel.a_side.chain_id(),
                                    open_confirm.channel_id().clone().unwrap()
                                );
                                   

                                    let a_channel = self.chains.a.query_channel(
                                        &channel.src_port_id,
                                        &channel.src_channel_id,
                                        Height::zero(),
                                    )?;

                                    let b_channel = self.chains.b.query_channel(
                                        &a_channel.remote.port_id.clone(),
                                        &a_channel.remote.channel_id.clone().unwrap(),
                                        Height::zero(),
                                    )?;

                                    if stage >=4 {
                                        debug! ("[{}] channel in stage >= OpenConfirm that is it already processed its OpenTry event ", channel.short_name());
                                        return Ok(());
                                    }
                                    stage = 4;


                                    if a_channel
                                        .state_matches(&ibc::ics04_channel::channel::State::Open)
                                    {
                                        if a_channel.remote.channel_id.is_some() {
                                            if b_channel.state_matches(
                                                &ibc::ics04_channel::channel::State::Open,
                                            ) {
                                                println!(
                                                "{}  {}  {}  Channel handshake finished for {:#?}\n",
                                                done, done, done,  &channel.src_channel_id,
                                            );
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                                IbcEvent::CloseInitChannel(_) => {}
                                IbcEvent::CloseConfirmChannel(_) => {}
                                _ => {}
                            }
                        }
                   // }

                    // WorkerCmd::NewBlock {
                    //     height: _,
                    //     new_block: _,
                    // } => {
                    //     debug!("\n new block \n ");
                    // } //link.a_to_b.clear_packets(height)?,

                    // _ => {}
               // }
            }
        }
    }

    /// Run the event loop for events associated with a [`UnidirectionalChannelPath`].
    fn run_uni_chan_path(self, path: UnidirectionalChannelPath) -> Result<(), BoxError> {
        let mut link = Link::new_from_opts(
            self.chains.a.clone(),
            self.chains.b.clone(),
            LinkParameters {
                src_port_id: path.src_port_id,
                src_channel_id: path.src_channel_id,
            },
        )?;

        if link.is_closed()? {
            warn!("channel is closed, exiting");
            return Ok(());
        }

        loop {
            if let Ok(cmd) = self.rx.try_recv() {
                match cmd {
                    WorkerCmd::IbcEvents { batch } => {
                        link.a_to_b.update_schedule(batch)?;
                        // Refresh the scheduled batches and execute any outstanding ones.
                    }
                    WorkerCmd::NewBlock {
                        height,
                        new_block: _,
                    } => link.a_to_b.clear_packets(height)?,
                }
            }

            // Refresh the scheduled batches and execute any outstanding ones.
            link.a_to_b.refresh_schedule()?;
            link.a_to_b.execute_schedule()?;

            thread::sleep(Duration::from_millis(100))
        }
    }
}

/// Client
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Client {
    /// Destination chain identifier.
    pub dst_chain_id: ChainId,

    /// Source channel identiier.
    pub dst_client_id: ClientId,

    /// Source chain identifier.
    pub src_chain_id: ChainId,
}

impl Client {
    pub fn short_name(&self) -> String {
        format!(
            "{} -> {}:{}",
            self.src_chain_id, self.dst_chain_id, self.dst_client_id
        )
    }
}

/// A unidirectional path from a source chain, channel and port.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Channel {
    /// Destination chain identifier.
    pub dst_chain_id: ChainId,

    /// Source chain identifier.
    pub src_chain_id: ChainId,

    /// Source channel identiier.
    pub src_channel_id: ChannelId,

    /// Source port identiier.
    pub src_port_id: PortId,

    /// Source connection_id
    pub connection_id: ConnectionId,
}

impl Channel {
    pub fn short_name(&self) -> String {
        format!(
            "{}/{}:{} -> {}",
            self.src_channel_id, self.src_port_id, self.src_chain_id, self.dst_chain_id,
        )
    }
}

/// A unidirectional path from a source chain, channel and port.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnidirectionalChannelPath {
    /// Destination chain identifier.
    pub dst_chain_id: ChainId,

    /// Source chain identifier.
    pub src_chain_id: ChainId,

    /// Source channel identiier.
    pub src_channel_id: ChannelId,

    /// Source port identiier.
    pub src_port_id: PortId,
}

impl UnidirectionalChannelPath {
    pub fn short_name(&self) -> String {
        format!(
            "{}/{}:{} -> {}",
            self.src_channel_id, self.src_port_id, self.src_chain_id, self.dst_chain_id,
        )
    }
}

/// An object determines the amount of parallelism that can
/// be exercised when processing [`IbcEvent`] between
/// two chains. For each [`Object`], a corresponding
/// [`Worker`] is spawned and all [`IbcEvent`]s mapped
/// to an [`Object`] are sent to the associated [`Worker`]
/// for processing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Object {
    // /// See [`Client`].
    // Client(Client),
    /// See [`Channel`].
    Channel(Channel),
    /// See [`UnidirectionalChannelPath`].
    UnidirectionalChannelPath(UnidirectionalChannelPath),
}

// impl From<Client> for Object {
//     fn from(c: Client) -> Self {
//         Self::Client(c)
//     }
// }

impl From<Channel> for Object {
    fn from(c: Channel) -> Self {
        Self::Channel(c)
    }
}

impl From<UnidirectionalChannelPath> for Object {
    fn from(p: UnidirectionalChannelPath) -> Self {
        Self::UnidirectionalChannelPath(p)
    }
}

impl Object {
    pub fn src_chain_id(&self) -> &ChainId {
        match self {
           // Self::Client(ref client) => &client.src_chain_id,
            Self::Channel(ref channel) => &channel.src_chain_id,
            Self::UnidirectionalChannelPath(ref path) => &path.src_chain_id,
        }
    }

    pub fn dst_chain_id(&self) -> &ChainId {
        match self {
          //  Self::Client(ref client) => &client.dst_chain_id,
            Self::Channel(ref channel) => &channel.dst_chain_id,
            Self::UnidirectionalChannelPath(ref path) => &path.dst_chain_id,
        }
    }

    pub fn short_name(&self) -> String {
        match self {
           // Self::Client(ref client) => client.short_name(),
            Self::Channel(ref channel) => channel.short_name(),
            Self::UnidirectionalChannelPath(ref path) => path.short_name(),
        }
    }

    // /// Build the object associated with the given [`UpdateClient`] event.
    // pub fn for_update_client(
    //     e: &UpdateClient,
    //     dst_chain: &dyn ChainHandle,
    // ) -> Result<Self, BoxError> {
    //     let client_state = dst_chain.query_client_state(e.client_id(), Height::zero())?;
    //     if client_state.refresh_time().is_none() {
    //         return Err(format!(
    //             "client '{}' on chain {} does not require refresh",
    //             e.client_id(),
    //             dst_chain.id()
    //         )
    //         .into());
    //     }

    //     let src_chain_id = client_state.chain_id();

    //     Ok(Client {
    //         dst_client_id: e.client_id().clone(),
    //         dst_chain_id: dst_chain.id(),
    //         src_chain_id,
    //     }
    //     .into())
    // }

    /// Build the object associated with the given [`SendPacket`] event.
    pub fn for_send_packet(e: &SendPacket, src_chain: &dyn ChainHandle) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.packet.source_channel, &e.packet.source_port)?;

        Ok(UnidirectionalChannelPath {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.packet.source_channel.clone(),
            src_port_id: e.packet.source_port.clone(),
        }
        .into())
    }

    /// Build the object associated with the given [`WriteAcknowledgement`] event.
    pub fn for_write_ack(
        e: &WriteAcknowledgement,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id = get_counterparty_chain(
            src_chain,
            &e.packet.destination_channel,
            &e.packet.destination_port,
        )?;

        Ok(UnidirectionalChannelPath {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.packet.destination_channel.clone(),
            src_port_id: e.packet.destination_port.clone(),
        }
        .into())
    }

    /// Build the object associated with the given [`TimeoutPacket`] event.
    pub fn for_timeout_packet(
        e: &TimeoutPacket,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.packet.source_channel, &e.packet.source_port)?;

        Ok(UnidirectionalChannelPath {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.src_channel_id().clone(),
            src_port_id: e.src_port_id().clone(),
        }
        .into())
    }

    /// Build the object associated with the given [`CloseInit`] event.
    pub fn for_close_init_channel(
        e: &CloseInit,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id = get_counterparty_chain(src_chain, e.channel_id(), &e.port_id())?;

        Ok(UnidirectionalChannelPath {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.channel_id().clone(),
            src_port_id: e.port_id().clone(),
        }
        .into())
    }
    /// Build the object associated with the given [`OpenInit`] event.
    pub fn for_open_init_channel(
        e: &OpenInit,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.channel_id().clone().unwrap(), &e.port_id())?;

        Ok(Channel {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.channel_id().clone().unwrap(),
            src_port_id: e.port_id().clone(),
            connection_id: e.connection_id().clone(),
        }
        .into())
    }
    /// Build the object associated with the given [`OpenTry`] event.
    pub fn for_open_try_channel(
        e: &OpenTry,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.channel_id().clone().unwrap(), &e.port_id())?;

        debug!(
            " in for_open_try_channel dst_chain_id {} src_chain_id {} with event {:?} ",
            dst_chain_id,
            src_chain.id(),
            e
        );

        Ok(Channel {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.channel_id().clone().unwrap(),
            src_port_id: e.port_id().clone(),
            connection_id: e.connection_id().clone(),
        }
        .into())
    }

    pub fn for_open_ack_channel(
        e: &OpenAck,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.channel_id().clone().unwrap(), &e.port_id())?;

        debug!(
            " in for_open_ack_channel dst_chain_id {} src_chain_id {} with event {:?} ",
            dst_chain_id,
            src_chain.id(),
            e
        );

        Ok(Channel {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.channel_id().clone().unwrap(),
            src_port_id: e.port_id().clone(),
            connection_id: e.connection_id().clone(),
        }
        .into())
    }

    pub fn for_open_confirm_channel(
        e: &OpenConfirm,
        src_chain: &dyn ChainHandle,
    ) -> Result<Self, BoxError> {
        let dst_chain_id =
            get_counterparty_chain(src_chain, &e.channel_id().clone().unwrap(), &e.port_id())?;

        debug!(
            " in for_open_confirm_channel dst_chain_id {} src_chain_id {} with event {:?} ",
            dst_chain_id,
            src_chain.id(),
            e
        );

        Ok(Channel {
            dst_chain_id,
            src_chain_id: src_chain.id(),
            src_channel_id: e.channel_id().clone().unwrap(),
            src_port_id: e.port_id().clone(),
            connection_id: e.connection_id().clone(),
        }
        .into())
    }
}

/// Describes the result of [`collect_events`].
#[derive(Clone, Debug)]
pub struct CollectedEvents {
    /// The height at which these events were emitted from the chain.
    pub height: Height,
    /// The chain from which the events were emitted.
    pub chain_id: ChainId,
    /// [`NewBlock`] event collected from the [`EventBatch`].
    pub new_block: Option<IbcEvent>,
    /// Mapping between [`Object`]s and their associated [`IbcEvent`]s.
    pub per_object: HashMap<Object, Vec<IbcEvent>>,
}

impl CollectedEvents {
    pub fn new(height: Height, chain_id: ChainId) -> Self {
        Self {
            height,
            chain_id,
            new_block: Default::default(),
            per_object: Default::default(),
        }
    }

    /// Whether the collected events include a [`NewBlock`] event.
    pub fn has_new_block(&self) -> bool {
        self.new_block.is_some()
    }
}

/// Collect the events we are interested in from an [`EventBatch`],
/// and maps each [`IbcEvent`] to their corresponding [`Object`].
pub fn collect_events(src_chain: &dyn ChainHandle, batch: EventBatch) -> CollectedEvents {
    let mut collected = CollectedEvents::new(batch.height, batch.chain_id);

    for event in batch.events {
        match event {
            IbcEvent::NewBlock(_) => {
                collected.new_block = Some(event);
            }
            // IbcEvent::UpdateClient(ref update) => {
            //     if let Ok(object) = Object::for_update_client(update, src_chain) {
            //         collected.per_object.entry(object).or_default().push(event);
            //     }
            // }
            IbcEvent::SendPacket(ref packet) => {
                if let Ok(object) = Object::for_send_packet(packet, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::TimeoutPacket(ref packet) => {
                if let Ok(object) = Object::for_timeout_packet(packet, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::WriteAcknowledgement(ref packet) => {
                if let Ok(object) = Object::for_write_ack(packet, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::CloseInitChannel(ref packet) => {
                if let Ok(object) = Object::for_close_init_channel(packet, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::OpenInitChannel(ref open_init) => {
                if let Ok(object) = Object::for_open_init_channel(open_init, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::OpenTryChannel(ref open_try) => {
                if let Ok(object) = Object::for_open_try_channel(open_try, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::OpenAckChannel(ref open_ack) => {
                if let Ok(object) = Object::for_open_ack_channel(open_ack, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            IbcEvent::OpenConfirmChannel(ref open_confirm) => {
                if let Ok(object) = Object::for_open_confirm_channel(open_confirm, src_chain) {
                    collected.per_object.entry(object).or_default().push(event);
                }
            }
            _ => (),
        }
    }

    collected
}

// TODO: Memoize this result
fn get_counterparty_chain(
    src_chain: &dyn ChainHandle,
    src_channel_id: &ChannelId,
    src_port_id: &PortId,
) -> Result<ChainId, BoxError> {
    trace!(
        chain_id = %src_chain.id(),
        src_channel_id = %src_channel_id,
        src_port_id = %src_port_id,
        "getting counterparty chain"
    );

    let src_channel = src_chain.query_channel(src_port_id, src_channel_id, Height::zero())?;
    if src_channel.state_matches(&ChannelState::Uninitialized) {
        return Err(format!("missing channel '{}' on source chain", src_channel_id).into());
    }

    let src_connection_id = src_channel
        .connection_hops()
        .first()
        .ok_or_else(|| format!("no connection hops for channel '{}'", src_channel_id))?;

    let src_connection = src_chain.query_connection(&src_connection_id, Height::zero())?;
    if src_connection.state_matches(&ConnectionState::Uninitialized) {
        return Err(format!("missing connection '{}' on source chain", src_connection_id).into());
    }

    let client_id = src_connection.client_id();
    let client_state = src_chain.query_client_state(client_id, Height::zero())?;

    trace!(
        chain_id=%src_chain.id(), src_channel_id=%src_channel_id, src_port_id=%src_port_id,
        "counterparty chain: {}", client_state.chain_id()
    );

    Ok(client_state.chain_id())
}
