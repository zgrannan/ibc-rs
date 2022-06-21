use alloc::sync::Arc;
use std::thread;

use crossbeam_channel as channel;
use tokio::runtime::Runtime as TokioRuntime;
use tracing::error;

use ibc::{
    core::{
        ics02_client::{
            client_consensus::{AnyConsensusState, AnyConsensusStateWithHeight, ConsensusState},
            client_state::{AnyClientState, ClientState, IdentifiedAnyClientState},
            events::UpdateClient,
            header::{AnyHeader, Header},
            misbehaviour::MisbehaviourEvidence,
        },
        ics03_connection::{
            connection::{ConnectionEnd, IdentifiedConnectionEnd},
            version::Version,
        },
        ics04_channel::{
            channel::{ChannelEnd, IdentifiedChannelEnd},
            packet::{PacketMsgType, Sequence},
        },
        ics23_commitment::{commitment::CommitmentPrefix, merkle::MerkleProof},
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    events::IbcEvent,
    proofs::Proofs,
    query::{QueryBlockRequest, QueryTxRequest},
    signer::Signer,
    Height,
};

use crate::{
    account::Balance,
    config::ChainConfig,
    connection::ConnectionMsgType,
    error::Error,
    event::{
        bus::EventBus,
    },
    keyring::KeyEntry
};

use super::{
    client::ClientSettings,
    endpoint::{ChainEndpoint, ChainStatus, HealthCheck},
    handle::{ChainHandle, ChainRequest, ReplyTo},
    requests::{
        IncludeProof, QueryChannelClientStateRequest, QueryChannelRequest, QueryChannelsRequest,
        QueryClientConnectionsRequest, QueryClientStateRequest, QueryClientStatesRequest,
        QueryConnectionChannelsRequest, QueryConnectionRequest, QueryConnectionsRequest,
        QueryConsensusStateRequest, QueryConsensusStatesRequest, QueryHostConsensusStateRequest,
        QueryNextSequenceReceiveRequest, QueryPacketAcknowledgementRequest,
        QueryPacketAcknowledgementsRequest, QueryPacketCommitmentRequest,
        QueryPacketCommitmentsRequest, QueryPacketReceiptRequest, QueryUnreceivedAcksRequest,
        QueryUnreceivedPacketsRequest, QueryUpgradedClientStateRequest,
        QueryUpgradedConsensusStateRequest,
    },
    tracking::TrackedMsgs,
};

pub struct Threads {
    pub chain_runtime: thread::JoinHandle<()>,
    pub event_monitor: Option<thread::JoinHandle<()>>,
}

pub struct ChainRuntime<Endpoint: ChainEndpoint> {
    /// The specific chain this runtime runs against
    chain: Endpoint,

    /// The sender side of a channel to this runtime. Any `ChainHandle` can use this to send
    /// chain requests to this runtime
    request_sender: channel::Sender<ChainRequest>,

    /// The receiving side of a channel to this runtime. The runtime consumes chain requests coming
    /// in through this channel.
    request_receiver: channel::Receiver<ChainRequest>,

    #[allow(dead_code)]
    rt: Arc<TokioRuntime>, // Making this future-proof, so we keep the runtime around.
}

impl<Endpoint> ChainRuntime<Endpoint>
where
    Endpoint: ChainEndpoint + Send + 'static,
{

    fn run(mut self) -> Result<(), Error> {
        Ok(())
    }

    fn health_check(&mut self, reply_to: ReplyTo<HealthCheck>) -> Result<(), Error> {
        let result = self.chain.health_check();
        reply_to.send(result).map_err(Error::send)
    }

    fn send_messages_and_wait_commit(
        &mut self,
        tracked_msgs: TrackedMsgs,
        reply_to: ReplyTo<Vec<IbcEvent>>,
    ) -> Result<(), Error> {
        let result = self.chain.send_messages_and_wait_commit(tracked_msgs);
        reply_to.send(result).map_err(Error::send)
    }

    fn send_messages_and_wait_check_tx(
        &mut self,
        tracked_msgs: TrackedMsgs,
        reply_to: ReplyTo<Vec<tendermint_rpc::endpoint::broadcast::tx_sync::Response>>,
    ) -> Result<(), Error> {
        let result = self.chain.send_messages_and_wait_check_tx(tracked_msgs);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_balance(
        &self,
        key_name: Option<String>,
        reply_to: ReplyTo<Balance>,
    ) -> Result<(), Error> {
        let balance = self.chain.query_balance(key_name);
        reply_to.send(balance).map_err(Error::send)
    }

    fn query_application_status(&self, reply_to: ReplyTo<ChainStatus>) -> Result<(), Error> {
        let latest_timestamp = self.chain.query_application_status();
        reply_to.send(latest_timestamp).map_err(Error::send)
    }

    fn get_signer(&mut self, reply_to: ReplyTo<Signer>) -> Result<(), Error> {
        let result = self.chain.get_signer();
        reply_to.send(result).map_err(Error::send)
    }

    fn get_config(&self, reply_to: ReplyTo<ChainConfig>) -> Result<(), Error> {
        let result = Ok(self.chain.config());
        reply_to.send(result).map_err(Error::send)
    }

    fn add_key(
        &mut self,
        key_name: String,
        key: KeyEntry,
        reply_to: ReplyTo<()>,
    ) -> Result<(), Error> {
        let result = self.chain.add_key(&key_name, key);
        reply_to.send(result).map_err(Error::send)
    }

    fn ibc_version(&mut self, reply_to: ReplyTo<Option<semver::Version>>) -> Result<(), Error> {
        let result = self.chain.ibc_version();
        reply_to.send(result).map_err(Error::send)
    }

    /// Constructs a client state for the given height
    fn build_client_state(
        &self,
        height: Height,
        settings: ClientSettings,
        reply_to: ReplyTo<AnyClientState>,
    ) -> Result<(), Error> {
        let client_state = self
            .chain
            .build_client_state(height, settings)
            .map(|cs| cs.wrap_any());

        reply_to.send(client_state).map_err(Error::send)
    }

    fn query_clients(
        &self,
        request: QueryClientStatesRequest,
        reply_to: ReplyTo<Vec<IdentifiedAnyClientState>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_clients(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_client_connections(
        &self,
        request: QueryClientConnectionsRequest,
        reply_to: ReplyTo<Vec<ConnectionId>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_client_connections(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_client_state(
        &self,
        request: QueryClientStateRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(AnyClientState, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let res = self
            .chain
            .query_client_state(request, include_proof)
            .map(|(cs, proof)| (cs.wrap_any(), proof));

        reply_to.send(res).map_err(Error::send)
    }

    fn query_upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
        reply_to: ReplyTo<(AnyClientState, MerkleProof)>,
    ) -> Result<(), Error> {
        let result = self
            .chain
            .query_upgraded_client_state(request)
            .map(|(cl, proof)| (cl.wrap_any(), proof));

        reply_to.send(result).map_err(Error::send)
    }

    fn query_consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
        reply_to: ReplyTo<Vec<AnyConsensusStateWithHeight>>,
    ) -> Result<(), Error> {
        let consensus_states = self.chain.query_consensus_states(request);
        reply_to.send(consensus_states).map_err(Error::send)
    }

    fn query_consensus_state(
        &self,
        request: QueryConsensusStateRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(AnyConsensusState, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let res = self.chain.query_consensus_state(request, include_proof);

        reply_to.send(res).map_err(Error::send)
    }

    fn query_upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
        reply_to: ReplyTo<(AnyConsensusState, MerkleProof)>,
    ) -> Result<(), Error> {
        let result = self
            .chain
            .query_upgraded_consensus_state(request)
            .map(|(cs, proof)| (cs.wrap_any(), proof));

        reply_to.send(result).map_err(Error::send)
    }

    fn query_commitment_prefix(&self, reply_to: ReplyTo<CommitmentPrefix>) -> Result<(), Error> {
        let prefix = self.chain.query_commitment_prefix();
        reply_to.send(prefix).map_err(Error::send)
    }

    fn query_connection(
        &self,
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(ConnectionEnd, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let connection_end = self.chain.query_connection(request, include_proof);
        reply_to.send(connection_end).map_err(Error::send)
    }

    fn query_connections(
        &self,
        request: QueryConnectionsRequest,
        reply_to: ReplyTo<Vec<IdentifiedConnectionEnd>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_connections(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
        reply_to: ReplyTo<Vec<IdentifiedChannelEnd>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_connection_channels(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_channels(
        &self,
        request: QueryChannelsRequest,
        reply_to: ReplyTo<Vec<IdentifiedChannelEnd>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_channels(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_channel(
        &self,
        request: QueryChannelRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(ChannelEnd, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_channel(request, include_proof);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
        reply_to: ReplyTo<Option<IdentifiedAnyClientState>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_channel_client_state(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_packet_commitment(request, include_proof);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
        reply_to: ReplyTo<(Vec<Sequence>, Height)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_packet_commitments(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_packet_receipt(request, include_proof);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
        reply_to: ReplyTo<Vec<Sequence>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_unreceived_packets(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let result = self
            .chain
            .query_packet_acknowledgement(request, include_proof);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
        reply_to: ReplyTo<(Vec<Sequence>, Height)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_packet_acknowledgements(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_unreceived_acknowledgement(
        &self,
        request: QueryUnreceivedAcksRequest,
        reply_to: ReplyTo<Vec<Sequence>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_unreceived_acknowledgements(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Sequence, Option<MerkleProof>)>,
    ) -> Result<(), Error> {
        let result = self
            .chain
            .query_next_sequence_receive(request, include_proof);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_txs(
        &self,
        request: QueryTxRequest,
        reply_to: ReplyTo<Vec<IbcEvent>>,
    ) -> Result<(), Error> {
        let result = self.chain.query_txs(request);
        reply_to.send(result).map_err(Error::send)
    }

    fn query_blocks(
        &self,
        request: QueryBlockRequest,
        reply_to: ReplyTo<(Vec<IbcEvent>, Vec<IbcEvent>)>,
    ) -> Result<(), Error> {
        let result = self.chain.query_blocks(request);

        reply_to.send(result).map_err(Error::send)?;

        Ok(())
    }

    fn query_host_consensus_state(
        &self,
        request: QueryHostConsensusStateRequest,
        reply_to: ReplyTo<AnyConsensusState>,
    ) -> Result<(), Error> {
        let result = self
            .chain
            .query_host_consensus_state(request)
            .map(|h| h.wrap_any());

        reply_to.send(result).map_err(Error::send)?;

        Ok(())
    }
}
