use anomaly::BoxError;
use tracing::debug;

use ibc::{
    ics02_client::client_state::{ClientState, IdentifiedAnyClientState},
    ics03_connection::connection::{IdentifiedConnectionEnd, State as ConnectionState},
    ics04_channel::channel::State as ChannelState,
};

use crate::{
    chain::handle::ChainHandle,
    config::Config,
    object::{Channel, Client, Connection, Object, Packet},
    registry::Registry,
    supervisor::scan::{ChainScan, ClientScan},
    worker::WorkerMap,
};

use super::{
    scan::{ChannelScan, ConnectionScan, Scan},
    RwArc,
};

/// A context for spawning workers within the [`crate::supervisor::Supervisor`].
pub struct SpawnContext<'a> {
    config: &'a RwArc<Config>,
    registry: &'a mut Registry,
    workers: &'a mut WorkerMap,
}

impl<'a> SpawnContext<'a> {
    pub fn new(
        config: &'a RwArc<Config>,
        registry: &'a mut Registry,
        workers: &'a mut WorkerMap,
    ) -> Self {
        Self {
            config,
            registry,
            workers,
        }
    }

    pub fn spawn_workers(&mut self, scan: Scan) {
        for (_, chain_scan) in scan.chains {
            self.spawn_chain_workers(chain_scan);
        }
    }

    pub fn spawn_chain_workers(&mut self, chain_scan: ChainScan) {
        let ChainScan { chain, clients } = chain_scan;

        debug!(chain.id = %chain.id(), "spawning workers");

        for (_, client) in clients {
            self.spawn_client_workers(chain.clone(), client);
        }
    }

    pub fn spawn_client_workers(&mut self, chain: Box<dyn ChainHandle>, client_scan: ClientScan) {
        let ClientScan {
            client,
            connections,
        } = client_scan;

        debug!(chain.id = %chain.id(), client.id = %client.client_id, "spawning client workers");

        for (_, connection) in connections {
            self.spawn_connection_workers(chain.clone(), &client, connection);
        }
    }

    pub fn spawn_connection_workers(
        &mut self,
        chain: Box<dyn ChainHandle>,
        client: &IdentifiedAnyClientState,
        connection_scan: ConnectionScan,
    ) -> Result<(), BoxError> {
        let ConnectionScan {
            connection,
            counterparty,
            channels,
        } = connection_scan;

        let handshake_enabled = self.handshake_enabled();

        let counterparty_chain = self
            .registry
            .get_or_spawn(&client.client_state.chain_id())?;

        let conn_state_src = connection.connection_end.state;
        let conn_state_dst = counterparty
            .as_ref()
            .map_or(ConnectionState::Uninitialized, |c| c.state);

        debug!(
            "connection {} on chain {} is: {:?}, state on dest. chain ({}) is: {:?}",
            connection.connection_id,
            chain.id(),
            conn_state_src,
            counterparty_chain.id(),
            conn_state_dst
        );

        if conn_state_src.is_open() && conn_state_dst.is_open() {
            debug!(
                "connection {} on chain {} is already open, not spawning Client worker",
                connection.connection_id,
                chain.id()
            );

            // nothing to do
        } else if !conn_state_dst.is_open()
            && conn_state_dst.less_or_equal_progress(conn_state_src)
            && handshake_enabled
        {
            // create worker for connection handshake that will advance the remote state
            let connection_object = Object::Connection(Connection {
                dst_chain_id: client.client_state.chain_id(),
                src_chain_id: chain.id(),
                src_connection_id: connection.connection_id.clone(),
            });

            self.workers
                .spawn(
                    connection_object.clone(),
                    chain.clone(),
                    counterparty_chain.clone(),
                )
                .then(|| {
                    debug!(
                        "spawning Connection worker: {}",
                        connection_object.short_name()
                    );
                });
        }

        if !connection.connection_end.is_open() {
            debug!(
                "connection {} not open, skip workers for channels over this connection",
                connection.connection_id
            );

            return Ok(());
        }

        match counterparty {
            None => {
                debug!(
                    "no counterparty for connection {}",
                    connection.connection_id
                );
                return Ok(());
            }
            Some(counterparty) => {
                if !counterparty.is_open() {
                    debug!(
                        "connection {} not open, skip workers for channels over this connection",
                        connection.connection_id
                    );

                    debug!(
                        "drop connection {} because its counterparty is not open",
                        connection.connection_id
                    );

                    return Ok(());
                }

                for channel_scan in channels {
                    self.spawn_channel_workers(
                        chain.clone(),
                        counterparty_chain.clone(),
                        &client,
                        channel_scan,
                    );
                }
            }
        }

        Ok(())
    }

    fn spawn_channel_workers(
        &mut self,
        chain: Box<dyn ChainHandle>,
        counterparty_chain: Box<dyn ChainHandle>,
        client: &IdentifiedAnyClientState,
        channel_scan: ChannelScan,
    ) {
        let ChannelScan {
            channel,
            counterparty,
        } = channel_scan;

        let chan_state_src = channel.channel_end.state;
        let chan_state_dst = counterparty
            .as_ref()
            .map_or(ChannelState::Uninitialized, |c| c.state);

        debug!(
            "channel {} on chain {} is: {}; state on dest. chain ({}) is: {}",
            channel.channel_id,
            chain.id(),
            chan_state_src,
            counterparty_chain.id(),
            chan_state_dst
        );

        if chan_state_src.is_open() && chan_state_dst.is_open() {
            // spawn the client worker
            let client_object = Object::Client(Client {
                dst_client_id: client.client_id.clone(),
                dst_chain_id: chain.id(),
                src_chain_id: client.client_state.chain_id(),
            });

            self.workers
                .spawn(
                    client_object.clone(),
                    counterparty_chain.clone(),
                    chain.clone(),
                )
                .then(|| debug!("spawned Client worker: {}", client_object.short_name()));

            // TODO: Only start the Packet worker if there are outstanding packets or ACKs.
            //       https://github.com/informalsystems/ibc-rs/issues/901

            // create the path object and spawn worker
            let path_object = Object::Packet(Packet {
                dst_chain_id: counterparty_chain.id(),
                src_chain_id: chain.id(),
                src_channel_id: channel.channel_id,
                src_port_id: channel.port_id,
            });

            self.workers
                .spawn(
                    path_object.clone(),
                    chain.clone(),
                    counterparty_chain.clone(),
                )
                .then(|| debug!("spawned Path worker: {}", path_object.short_name()));
        } else if !chan_state_dst.is_open()
            && chan_state_dst.less_or_equal_progress(chan_state_src)
            && self.handshake_enabled()
        {
            // create worker for channel handshake that will advance the remote state
            let channel_object = Object::Channel(Channel {
                dst_chain_id: counterparty_chain.id(),
                src_chain_id: chain.id(),
                src_channel_id: channel.channel_id,
                src_port_id: channel.port_id,
            });

            self.workers
                .spawn(channel_object.clone(), chain, counterparty_chain)
                .then(|| debug!("spawned Channel worker: {}", channel_object.short_name()));
        }
    }

    fn handshake_enabled(&self) -> bool {
        self.config
            .read()
            .expect("poisoned lock")
            .handshake_enabled()
    }
}
