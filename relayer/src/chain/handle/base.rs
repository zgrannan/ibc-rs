use core::fmt::Debug;

use crossbeam_channel as channel;
use serde::{Serialize, Serializer};

use ibc::{
    core::{
        ics02_client::client_consensus::{AnyConsensusState, AnyConsensusStateWithHeight},
        ics02_client::client_state::{AnyClientState, IdentifiedAnyClientState},
        ics02_client::events::UpdateClient,
        ics02_client::header::AnyHeader,
        ics02_client::misbehaviour::MisbehaviourEvidence,
        ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd},
        ics03_connection::version::Version,
        ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd},
        ics04_channel::packet::{PacketMsgType, Sequence},
        ics23_commitment::{commitment::CommitmentPrefix, merkle::MerkleProof},
        ics24_host::identifier::ChainId,
        ics24_host::identifier::ChannelId,
        ics24_host::identifier::{ClientId, ConnectionId, PortId},
    },
    events::IbcEvent,
    proofs::Proofs,
    query::{QueryBlockRequest, QueryTxRequest},
    signer::Signer,
    Height,
};

use crate::{
    account::Balance,
    chain::{
        client::ClientSettings,
        endpoint::ChainStatus,
        requests::{
            IncludeProof, QueryChannelClientStateRequest, QueryChannelRequest,
            QueryChannelsRequest, QueryClientConnectionsRequest, QueryClientStateRequest,
            QueryClientStatesRequest, QueryConnectionChannelsRequest, QueryConnectionRequest,
            QueryConnectionsRequest, QueryConsensusStateRequest, QueryConsensusStatesRequest,
            QueryHostConsensusStateRequest, QueryNextSequenceReceiveRequest,
            QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementsRequest,
            QueryPacketCommitmentRequest, QueryPacketCommitmentsRequest, QueryPacketReceiptRequest,
            QueryUnreceivedAcksRequest, QueryUnreceivedPacketsRequest,
            QueryUpgradedClientStateRequest, QueryUpgradedConsensusStateRequest,
        },
        tracking::TrackedMsgs,
    },
    config::ChainConfig,
    connection::ConnectionMsgType,
    error::Error,
    keyring::KeyEntry,
};

use super::{reply_channel, ChainHandle, ChainRequest, HealthCheck, ReplyTo};

pub struct BaseChainHandle {
    /// Chain identifier
    chain_id: ChainId,

    /// The handle's channel for sending requests to the runtime
    runtime_sender: channel::Sender<ChainRequest>,
}

impl BaseChainHandle {
    fn send<F, O>(&self, f: F) -> Result<O, Error>
    where
        F: FnOnce(ReplyTo<O>) -> ChainRequest,
        O: Debug,
    {
        unimplemented!()
    }
}

impl ChainHandle for BaseChainHandle {
    fn add_key(&self, key_name: String, key: KeyEntry) -> Result<(), Error> {
        self.send(|reply_to| ChainRequest::AddKey {
            key_name,
            key,
            reply_to,
        })
    }

    fn query_upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
    ) -> Result<(AnyClientState, MerkleProof), Error> {
        self.send(|reply_to| ChainRequest::QueryUpgradedClientState { request, reply_to })
    }

    fn query_upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
    ) -> Result<(AnyConsensusState, MerkleProof), Error> {
        self.send(|reply_to| ChainRequest::QueryUpgradedConsensusState { request, reply_to })
    }

    fn query_commitment_prefix(&self) -> Result<CommitmentPrefix, Error> {
        self.send(|reply_to| ChainRequest::QueryCommitmentPrefix { reply_to })
    }

    fn query_connection(
        &self,
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
    ) -> Result<(ConnectionEnd, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryConnection {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_connections(
        &self,
        request: QueryConnectionsRequest,
    ) -> Result<Vec<IdentifiedConnectionEnd>, Error> {
        self.send(|reply_to| ChainRequest::QueryConnections { request, reply_to })
    }

    fn query_connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error> {
        self.send(|reply_to| ChainRequest::QueryConnectionChannels { request, reply_to })
    }

    fn query_next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
    ) -> Result<(Sequence, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryNextSequenceReceive {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_channels(
        &self,
        request: QueryChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error> {
        self.send(|reply_to| ChainRequest::QueryChannels { request, reply_to })
    }

    fn query_channel(
        &self,
        request: QueryChannelRequest,
        include_proof: IncludeProof,
    ) -> Result<(ChannelEnd, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryChannel {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
    ) -> Result<Option<IdentifiedAnyClientState>, Error> {
        self.send(|reply_to| ChainRequest::QueryChannelClientState { request, reply_to })
    }

    fn build_header(
        &self,
        trusted_height: Height,
        target_height: Height,
        client_state: AnyClientState,
    ) -> Result<(AnyHeader, Vec<AnyHeader>), Error> {
        self.send(|reply_to| ChainRequest::BuildHeader {
            trusted_height,
            target_height,
            client_state,
            reply_to,
        })
    }

    fn build_client_state(
        &self,
        height: Height,
        settings: ClientSettings,
    ) -> Result<AnyClientState, Error> {
        self.send(|reply_to| ChainRequest::BuildClientState {
            height,
            settings,
            reply_to,
        })
    }

    fn build_consensus_state(
        &self,
        trusted: Height,
        target: Height,
        client_state: AnyClientState,
    ) -> Result<AnyConsensusState, Error> {
        self.send(|reply_to| ChainRequest::BuildConsensusState {
            trusted,
            target,
            client_state,
            reply_to,
        })
    }

    fn check_misbehaviour(
        &self,
        update_event: UpdateClient,
        client_state: AnyClientState,
    ) -> Result<Option<MisbehaviourEvidence>, Error> {
        self.send(|reply_to| ChainRequest::BuildMisbehaviour {
            client_state,
            update_event,
            reply_to,
        })
    }

    fn build_connection_proofs_and_client_state(
        &self,
        message_type: ConnectionMsgType,
        connection_id: &ConnectionId,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(Option<AnyClientState>, Proofs), Error> {
        self.send(
            |reply_to| ChainRequest::BuildConnectionProofsAndClientState {
                message_type,
                connection_id: connection_id.clone(),
                client_id: client_id.clone(),
                height,
                reply_to,
            },
        )
    }

    fn build_channel_proofs(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        height: Height,
    ) -> Result<Proofs, Error> {
        self.send(|reply_to| ChainRequest::BuildChannelProofs {
            port_id: port_id.clone(),
            channel_id: *channel_id,
            height,
            reply_to,
        })
    }

    fn build_packet_proofs(
        &self,
        packet_type: PacketMsgType,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        height: Height,
    ) -> Result<Proofs, Error> {
        self.send(|reply_to| ChainRequest::BuildPacketProofs {
            packet_type,
            port_id: port_id.clone(),
            channel_id: *channel_id,
            sequence,
            height,
            reply_to,
        })
    }

    fn query_packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketCommitment {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
    ) -> Result<(Vec<Sequence>, Height), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketCommitments { request, reply_to })
    }

    fn query_packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketReceipt {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
    ) -> Result<Vec<Sequence>, Error> {
        self.send(|reply_to| ChainRequest::QueryUnreceivedPackets { request, reply_to })
    }

    fn query_packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketAcknowledgement {
            request,
            include_proof,
            reply_to,
        })
    }

    fn query_packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
    ) -> Result<(Vec<Sequence>, Height), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketAcknowledgements { request, reply_to })
    }

    fn query_unreceived_acknowledgements(
        &self,
        request: QueryUnreceivedAcksRequest,
    ) -> Result<Vec<Sequence>, Error> {
        self.send(|reply_to| ChainRequest::QueryUnreceivedAcknowledgement { request, reply_to })
    }

    fn query_txs(&self, request: QueryTxRequest) -> Result<Vec<IbcEvent>, Error> {
        self.send(|reply_to| ChainRequest::QueryPacketEventDataFromTxs { request, reply_to })
    }

    fn query_blocks(
        &self,
        request: QueryBlockRequest,
    ) -> Result<(Vec<IbcEvent>, Vec<IbcEvent>), Error> {
        self.send(|reply_to| ChainRequest::QueryPacketEventDataFromBlocks { request, reply_to })
    }

    fn query_host_consensus_state(
        &self,
        request: QueryHostConsensusStateRequest,
    ) -> Result<AnyConsensusState, Error> {
        self.send(|reply_to| ChainRequest::QueryHostConsensusState { request, reply_to })
    }
}
