use alloc::sync::Arc;
use core::fmt::{self, Debug};

use crossbeam_channel as channel;
use serde::Serialize;

use ibc::{
    core::{
        ics02_client::{
            client_consensus::{AnyConsensusState, AnyConsensusStateWithHeight},
            client_state::{AnyClientState, IdentifiedAnyClientState},
            events::UpdateClient,
            header::AnyHeader,
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
        ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
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
    keyring::KeyEntry,
};

use super::{
    client::ClientSettings,
    endpoint::{ChainStatus, HealthCheck},
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

mod base;

pub use base::BaseChainHandle;

/// A pair of [`ChainHandle`]s.
#[derive(Clone)]
pub struct ChainHandlePair<ChainA: ChainHandle, ChainB: ChainHandle> {
    pub a: ChainA,
    pub b: ChainB,
}

impl<ChainA: ChainHandle, ChainB: ChainHandle> ChainHandlePair<ChainA, ChainB> {
    /// Swap the two handles.
    pub fn swap(self) -> ChainHandlePair<ChainB, ChainA> {
        ChainHandlePair {
            a: self.b,
            b: self.a,
        }
    }
}

pub type ReplyTo<T> = channel::Sender<Result<T, Error>>;
pub type Reply<T> = channel::Receiver<Result<T, Error>>;

pub fn reply_channel<T>() -> (ReplyTo<T>, Reply<T>) {
    channel::bounded(1)
}

/// Requests that a `ChainHandle` may send to a `ChainRuntime`.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ChainRequest {
    Shutdown {
        reply_to: ReplyTo<()>,
    },

    HealthCheck {
        reply_to: ReplyTo<HealthCheck>,
    },

    SendMessagesAndWaitCommit {
        tracked_msgs: TrackedMsgs,
        reply_to: ReplyTo<Vec<IbcEvent>>,
    },

    SendMessagesAndWaitCheckTx {
        tracked_msgs: TrackedMsgs,
        reply_to: ReplyTo<Vec<tendermint_rpc::endpoint::broadcast::tx_sync::Response>>,
    },

    Config {
        reply_to: ReplyTo<ChainConfig>,
    },

    Signer {
        reply_to: ReplyTo<Signer>,
    },

    GetKey {
        reply_to: ReplyTo<KeyEntry>,
    },

    AddKey {
        key_name: String,
        key: KeyEntry,
        reply_to: ReplyTo<()>,
    },

    IbcVersion {
        reply_to: ReplyTo<Option<semver::Version>>,
    },

    QueryBalance {
        key_name: Option<String>,
        reply_to: ReplyTo<Balance>,
    },

    QueryApplicationStatus {
        reply_to: ReplyTo<ChainStatus>,
    },

    QueryClients {
        request: QueryClientStatesRequest,
        reply_to: ReplyTo<Vec<IdentifiedAnyClientState>>,
    },

    BuildHeader {
        trusted_height: Height,
        target_height: Height,
        client_state: AnyClientState,
        reply_to: ReplyTo<(AnyHeader, Vec<AnyHeader>)>,
    },

    BuildClientState {
        height: Height,
        settings: ClientSettings,
        reply_to: ReplyTo<AnyClientState>,
    },

    BuildConsensusState {
        trusted: Height,
        target: Height,
        client_state: AnyClientState,
        reply_to: ReplyTo<AnyConsensusState>,
    },

    BuildMisbehaviour {
        client_state: AnyClientState,
        update_event: UpdateClient,
        reply_to: ReplyTo<Option<MisbehaviourEvidence>>,
    },

    BuildConnectionProofsAndClientState {
        message_type: ConnectionMsgType,
        connection_id: ConnectionId,
        client_id: ClientId,
        height: Height,
        reply_to: ReplyTo<(Option<AnyClientState>, Proofs)>,
    },

    QueryClientState {
        request: QueryClientStateRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(AnyClientState, Option<MerkleProof>)>,
    },

    QueryClientConnections {
        request: QueryClientConnectionsRequest,
        reply_to: ReplyTo<Vec<ConnectionId>>,
    },

    QueryConsensusState {
        request: QueryConsensusStateRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(AnyConsensusState, Option<MerkleProof>)>,
    },

    QueryConsensusStates {
        request: QueryConsensusStatesRequest,
        reply_to: ReplyTo<Vec<AnyConsensusStateWithHeight>>,
    },

    QueryUpgradedClientState {
        request: QueryUpgradedClientStateRequest,
        reply_to: ReplyTo<(AnyClientState, MerkleProof)>,
    },

    QueryUpgradedConsensusState {
        request: QueryUpgradedConsensusStateRequest,
        reply_to: ReplyTo<(AnyConsensusState, MerkleProof)>,
    },

    QueryCommitmentPrefix {
        reply_to: ReplyTo<CommitmentPrefix>,
    },

    QueryCompatibleVersions {
        reply_to: ReplyTo<Vec<Version>>,
    },

    QueryConnection {
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(ConnectionEnd, Option<MerkleProof>)>,
    },

    QueryConnections {
        request: QueryConnectionsRequest,
        reply_to: ReplyTo<Vec<IdentifiedConnectionEnd>>,
    },

    QueryConnectionChannels {
        request: QueryConnectionChannelsRequest,
        reply_to: ReplyTo<Vec<IdentifiedChannelEnd>>,
    },

    QueryChannels {
        request: QueryChannelsRequest,
        reply_to: ReplyTo<Vec<IdentifiedChannelEnd>>,
    },

    QueryChannel {
        request: QueryChannelRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(ChannelEnd, Option<MerkleProof>)>,
    },

    QueryChannelClientState {
        request: QueryChannelClientStateRequest,
        reply_to: ReplyTo<Option<IdentifiedAnyClientState>>,
    },

    QueryNextSequenceReceive {
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Sequence, Option<MerkleProof>)>,
    },

    BuildChannelProofs {
        port_id: PortId,
        channel_id: ChannelId,
        height: Height,
        reply_to: ReplyTo<Proofs>,
    },

    BuildPacketProofs {
        packet_type: PacketMsgType,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        height: Height,
        reply_to: ReplyTo<Proofs>,
    },

    QueryPacketCommitment {
        request: QueryPacketCommitmentRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    },

    QueryPacketCommitments {
        request: QueryPacketCommitmentsRequest,
        reply_to: ReplyTo<(Vec<Sequence>, Height)>,
    },

    QueryPacketReceipt {
        request: QueryPacketReceiptRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    },

    QueryUnreceivedPackets {
        request: QueryUnreceivedPacketsRequest,
        reply_to: ReplyTo<Vec<Sequence>>,
    },

    QueryPacketAcknowledgement {
        request: QueryPacketAcknowledgementRequest,
        include_proof: IncludeProof,
        reply_to: ReplyTo<(Vec<u8>, Option<MerkleProof>)>,
    },

    QueryPacketAcknowledgements {
        request: QueryPacketAcknowledgementsRequest,
        reply_to: ReplyTo<(Vec<Sequence>, Height)>,
    },

    QueryUnreceivedAcknowledgement {
        request: QueryUnreceivedAcksRequest,
        reply_to: ReplyTo<Vec<Sequence>>,
    },

    QueryPacketEventDataFromTxs {
        request: QueryTxRequest,
        reply_to: ReplyTo<Vec<IbcEvent>>,
    },

    QueryPacketEventDataFromBlocks {
        request: QueryBlockRequest,
        reply_to: ReplyTo<(Vec<IbcEvent>, Vec<IbcEvent>)>,
    },

    QueryHostConsensusState {
        request: QueryHostConsensusStateRequest,
        reply_to: ReplyTo<AnyConsensusState>,
    },
}

pub trait ChainHandle: Send + Sync + 'static {
    fn add_key(&self, key_name: String, key: KeyEntry) -> Result<(), Error>;

    fn query_upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
    ) -> Result<(AnyClientState, MerkleProof), Error>;

    fn query_upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
    ) -> Result<(AnyConsensusState, MerkleProof), Error>;

    fn query_commitment_prefix(&self) -> Result<CommitmentPrefix, Error>;

    /// Performs a query to retrieve the connection associated with a given
    /// connection identifier. A proof can optionally be returned along with the
    /// result.
    fn query_connection(
        &self,
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
    ) -> Result<(ConnectionEnd, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve the identifiers of all connections.
    fn query_connections(
        &self,
        request: QueryConnectionsRequest,
    ) -> Result<Vec<IdentifiedConnectionEnd>, Error>;

    /// Performs a query to retrieve all channels associated with a connection.
    fn query_connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error>;

    /// Performs a query to retrieve `nextSequenceRecv` stored at path
    /// `path::SeqRecvsPath` as defined in ICS-4. A proof can optionally be
    /// returned along with the result.
    fn query_next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
    ) -> Result<(Sequence, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve all the channels of a chain.
    fn query_channels(
        &self,
        request: QueryChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error>;

    /// Performs a query to retrieve the channel associated with a given channel
    /// identifier. A proof can optionally be returned along with the result.
    fn query_channel(
        &self,
        request: QueryChannelRequest,
        include_proof: IncludeProof,
    ) -> Result<(ChannelEnd, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve the client state for the channel associated
    /// with a given channel identifier.
    fn query_channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
    ) -> Result<Option<IdentifiedAnyClientState>, Error>;

    fn build_header(
        &self,
        trusted_height: Height,
        target_height: Height,
        client_state: AnyClientState,
    ) -> Result<(AnyHeader, Vec<AnyHeader>), Error>;

    /// Constructs a client state at the given height
    fn build_client_state(
        &self,
        height: Height,
        settings: ClientSettings,
    ) -> Result<AnyClientState, Error>;

    /// Constructs a consensus state at the given height
    fn build_consensus_state(
        &self,
        trusted: Height,
        target: Height,
        client_state: AnyClientState,
    ) -> Result<AnyConsensusState, Error>;

    fn check_misbehaviour(
        &self,
        update: UpdateClient,
        client_state: AnyClientState,
    ) -> Result<Option<MisbehaviourEvidence>, Error>;

    fn build_connection_proofs_and_client_state(
        &self,
        message_type: ConnectionMsgType,
        connection_id: &ConnectionId,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(Option<AnyClientState>, Proofs), Error>;

    fn build_channel_proofs(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        height: Height,
    ) -> Result<Proofs, Error>;

    fn build_packet_proofs(
        &self,
        packet_type: PacketMsgType,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        height: Height,
    ) -> Result<Proofs, Error>;

    /// Performs a query to retrieve a stored packet commitment hash, stored on
    /// the chain at path `path::CommitmentsPath`. A proof can optionally be
    /// returned along with the result.
    fn query_packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve all the packet commitments hashes
    /// associated with a channel. Returns the corresponding packet sequence
    /// numbers and the height at which they were retrieved.
    fn query_packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
    ) -> Result<(Vec<Sequence>, Height), Error>;

    /// Performs a query to retrieve a given packet receipt, stored on the chain at path
    /// `path::CommitmentsPath`. A proof can optionally be returned along with the result.
    fn query_packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error>;

    /// Performs a query about which IBC packets in the specified list has not
    /// been received. Returns the sequence numbers of the packets that were not
    /// received.
    ///
    /// For example, given a request with the sequence numbers `[5,6,7,8]`, a
    /// response of `[7,8]` would indicate that packets 5 & 6 were received,
    /// while packets 7, 8 were not.
    fn query_unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
    ) -> Result<Vec<Sequence>, Error>;

    /// Performs a query to retrieve a stored packet acknowledgement hash,
    /// stored on the chain at path `path::AcksPath`. A proof can optionally be
    /// returned along with the result.
    fn query_packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve all the packet acknowledgements associated
    /// with a channel. Returns the corresponding packet sequence numbers and
    /// the height at which they were retrieved.
    fn query_packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
    ) -> Result<(Vec<Sequence>, Height), Error>;

    /// Performs a query about which IBC packets in the specified list has not
    /// been acknowledged. Returns the sequence numbers of the packets that were not
    /// acknowledged.
    ///
    /// For example, given a request with the sequence numbers `[5,6,7,8]`, a
    /// response of `[7,8]` would indicate that packets 5 & 6 were acknowledged,
    /// while packets 7, 8 were not.
    fn query_unreceived_acknowledgements(
        &self,
        request: QueryUnreceivedAcksRequest,
    ) -> Result<Vec<Sequence>, Error>;

    fn query_txs(&self, request: QueryTxRequest) -> Result<Vec<IbcEvent>, Error>;

    fn query_blocks(
        &self,
        request: QueryBlockRequest,
    ) -> Result<(Vec<IbcEvent>, Vec<IbcEvent>), Error>;

    fn query_host_consensus_state(
        &self,
        request: QueryHostConsensusStateRequest,
    ) -> Result<AnyConsensusState, Error>;
}
