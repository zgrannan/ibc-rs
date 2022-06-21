use alloc::sync::Arc;
use core::convert::TryFrom;
use ibc::core::ics23_commitment::merkle::MerkleProof;

use tokio::runtime::Runtime as TokioRuntime;

use ibc::core::ics02_client::client_consensus::{
    AnyConsensusState, AnyConsensusStateWithHeight, ConsensusState,
};
use ibc::core::ics02_client::client_state::{
    AnyClientState, ClientState, IdentifiedAnyClientState,
};
use ibc::core::ics02_client::header::Header;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd, State};
use ibc::core::ics03_connection::version::{get_compatible_versions, Version};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::packet::{PacketMsgType, Sequence};
use ibc::core::ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::events::IbcEvent;
use ibc::proofs::{ConsensusProof, Proofs};
use ibc::query::{QueryBlockRequest, QueryTxRequest};
use ibc::signer::Signer;
use ibc::timestamp::Timestamp;
use ibc::Height as ICSHeight;
use tendermint_rpc::endpoint::broadcast::tx_sync::Response as TxResponse;

use crate::account::Balance;
use crate::chain::client::ClientSettings;
use crate::chain::requests::{
    QueryChannelClientStateRequest, QueryChannelRequest, QueryChannelsRequest,
    QueryClientConnectionsRequest, QueryClientStateRequest, QueryClientStatesRequest,
    QueryConnectionChannelsRequest, QueryConnectionRequest, QueryConnectionsRequest,
    QueryConsensusStateRequest, QueryConsensusStatesRequest, QueryHostConsensusStateRequest,
    QueryNextSequenceReceiveRequest, QueryPacketAcknowledgementsRequest,
    QueryPacketCommitmentsRequest, QueryUnreceivedAcksRequest, QueryUnreceivedPacketsRequest,
    QueryUpgradedClientStateRequest, QueryUpgradedConsensusStateRequest,
};
use crate::chain::tracking::TrackedMsgs;
use crate::config::ChainConfig;
use crate::connection::ConnectionMsgType;
use crate::error::{Error, QUERY_PROOF_EXPECT_MSG};
use crate::keyring::{KeyEntry, KeyRing};

use super::requests::{
    IncludeProof, QueryPacketAcknowledgementRequest, QueryPacketCommitmentRequest,
    QueryPacketReceiptRequest,
};

pub enum HealthCheck {
    Healthy,
    Unhealthy(Box<Error>),
}

pub struct ChainStatus {
    pub height: ICSHeight,
    pub timestamp: Timestamp,
}

/// Defines a blockchain as understood by the relayer
pub trait ChainEndpoint: Sized {
    /// Type of light blocks for this chain
    type LightBlock: Send + Sync;

    /// Type of headers for this chain
    type Header: Header;

    /// Type of consensus state for this chain
    type ConsensusState: ConsensusState;

    /// Type of the client state for this chain
    type ClientState: ClientState;

    /// Constructs the chain
    fn bootstrap(config: ChainConfig, rt: Arc<TokioRuntime>) -> Result<Self, Error>;

    /// Returns the chain's identifier
    fn id(&self) -> &ChainId;

    /// Shutdown the chain runtime
    fn shutdown(self) -> Result<(), Error>;

    /// Perform a health check
    fn health_check(&self) -> Result<HealthCheck, Error>;

    /// Returns the chain's keybase
    fn keybase(&self) -> &KeyRing;

    /// Returns the chain's keybase, mutably
    fn keybase_mut(&mut self) -> &mut KeyRing;

    /// Sends one or more transactions with `msgs` to chain and
    /// synchronously wait for it to be committed.
    fn send_messages_and_wait_commit(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<IbcEvent>, Error>;

    /// Sends one or more transactions with `msgs` to chain.
    /// Non-blocking alternative to `send_messages_and_wait_commit` interface.
    fn send_messages_and_wait_check_tx(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<TxResponse>, Error>;

    fn get_signer(&mut self) -> Result<Signer, Error>;

    fn config(&self) -> ChainConfig;

    fn add_key(&mut self, key_name: &str, key: KeyEntry) -> Result<(), Error>;

    /// Return the version of the IBC protocol that this chain is running, if known.
    fn ibc_version(&self) -> Result<Option<semver::Version>, Error>;

    // Queries

    /// Query the balance of the given account for the denom used to pay tx fees.
    /// If no account is given, behavior must be specified, e.g. retrieve it from configuration file.
    fn query_balance(&self, key_name: Option<String>) -> Result<Balance, Error>;

    fn query_commitment_prefix(&self) -> Result<CommitmentPrefix, Error>;

    /// Query the latest height and timestamp the application is at
    fn query_application_status(&self) -> Result<ChainStatus, Error>;

    /// Performs a query to retrieve the state of all clients that a chain hosts.
    fn query_clients(
        &self,
        request: QueryClientStatesRequest,
    ) -> Result<Vec<IdentifiedAnyClientState>, Error>;

    /// Performs a query to retrieve the state of the specified light client. A
    /// proof can optionally be returned along with the result.
    fn query_client_state(
        &self,
        request: QueryClientStateRequest,
        include_proof: IncludeProof,
    ) -> Result<(AnyClientState, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve the consensus state for a specified height
    /// `consensus_height` that the specified light client stores.
    fn query_consensus_state(
        &self,
        request: QueryConsensusStateRequest,
        include_proof: IncludeProof,
    ) -> Result<(AnyConsensusState, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve all the consensus states that the specified
    /// light client stores.
    fn query_consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
    ) -> Result<Vec<AnyConsensusStateWithHeight>, Error>;

    fn query_upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
    ) -> Result<(AnyClientState, MerkleProof), Error>;

    fn query_upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
    ) -> Result<(AnyConsensusState, MerkleProof), Error>;

    /// Performs a query to retrieve the identifiers of all connections.
    fn query_connections(
        &self,
        request: QueryConnectionsRequest,
    ) -> Result<Vec<IdentifiedConnectionEnd>, Error>;

    /// Performs a query to retrieve the identifiers of all connections.
    fn query_client_connections(
        &self,
        request: QueryClientConnectionsRequest,
    ) -> Result<Vec<ConnectionId>, Error>;

    /// Performs a query to retrieve the connection associated with a given
    /// connection identifier. A proof can optionally be returned along with the
    /// result.
    fn query_connection(
        &self,
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
    ) -> Result<(ConnectionEnd, Option<MerkleProof>), Error>;

    /// Performs a query to retrieve all channels associated with a connection.
    fn query_connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error>;

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
    ) -> Result<(Vec<Sequence>, ICSHeight), Error>;

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
    ) -> Result<(Vec<Sequence>, ICSHeight), Error>;

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

    /// Performs a query to retrieve `nextSequenceRecv` stored at path
    /// `path::SeqRecvsPath` as defined in ICS-4. A proof can optionally be
    /// returned along with the result.
    fn query_next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
    ) -> Result<(Sequence, Option<MerkleProof>), Error>;

    fn query_txs(&self, request: QueryTxRequest) -> Result<Vec<IbcEvent>, Error>;

    fn query_blocks(
        &self,
        request: QueryBlockRequest,
    ) -> Result<(Vec<IbcEvent>, Vec<IbcEvent>), Error>;

    fn query_host_consensus_state(
        &self,
        request: QueryHostConsensusStateRequest,
    ) -> Result<Self::ConsensusState, Error>;

    fn build_client_state(
        &self,
        height: ICSHeight,
        settings: ClientSettings,
    ) -> Result<Self::ClientState, Error>;

    fn build_consensus_state(
        &self,
        light_block: Self::LightBlock,
    ) -> Result<Self::ConsensusState, Error>;

}
