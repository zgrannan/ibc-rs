use crossbeam_channel as channel;
use ibc::core::ics02_client::client_consensus::{AnyConsensusState, AnyConsensusStateWithHeight};
use ibc::core::ics02_client::client_state::{AnyClientState, IdentifiedAnyClientState};
use ibc::core::ics02_client::events::UpdateClient;
use ibc::core::ics02_client::misbehaviour::MisbehaviourEvidence;
use ibc::core::ics03_connection::connection::IdentifiedConnectionEnd;
use ibc::core::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::core::ics04_channel::packet::{PacketMsgType, Sequence};
use ibc::core::ics23_commitment::merkle::MerkleProof;
use ibc::query::QueryTxRequest;
use ibc::{
    core::ics02_client::header::AnyHeader,
    core::ics03_connection::connection::ConnectionEnd,
    core::ics03_connection::version::Version,
    core::ics04_channel::channel::ChannelEnd,
    core::ics23_commitment::commitment::CommitmentPrefix,
    core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
    events::IbcEvent,
    proofs::Proofs,
    query::QueryBlockRequest,
    signer::Signer,
    Height,
};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tracing::debug;

use crate::account::Balance;
use crate::chain::client::ClientSettings;
use crate::chain::endpoint::{ChainStatus, HealthCheck};
use crate::chain::handle::{ChainHandle, ChainRequest};
use crate::chain::requests::{
    IncludeProof, QueryChannelClientStateRequest, QueryChannelRequest, QueryChannelsRequest,
    QueryClientConnectionsRequest, QueryClientStateRequest, QueryClientStatesRequest,
    QueryConnectionChannelsRequest, QueryConnectionRequest, QueryConnectionsRequest,
    QueryConsensusStateRequest, QueryConsensusStatesRequest, QueryHostConsensusStateRequest,
    QueryNextSequenceReceiveRequest, QueryPacketAcknowledgementRequest,
    QueryPacketAcknowledgementsRequest, QueryPacketCommitmentRequest,
    QueryPacketCommitmentsRequest, QueryPacketReceiptRequest, QueryUnreceivedAcksRequest,
    QueryUnreceivedPacketsRequest, QueryUpgradedClientStateRequest,
    QueryUpgradedConsensusStateRequest,
};
use crate::chain::tracking::TrackedMsgs;
use crate::config::ChainConfig;
use crate::error::Error;
use crate::util::lock::LockExt;
use crate::{connection::ConnectionMsgType, keyring::KeyEntry};

#[derive(Debug, Clone)]
pub struct CountingChainHandle<Handle> {
    inner: Handle,
    metrics: Arc<RwLock<HashMap<String, u64>>>,
}

impl<Handle> CountingChainHandle<Handle> {
    pub fn new(handle: Handle) -> Self {
        Self {
            inner: handle,
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn inner(&self) -> &Handle {
        &self.inner
    }

    pub fn metrics(&self) -> RwLockReadGuard<'_, HashMap<String, u64>> {
        self.metrics.acquire_read()
    }

    fn inc_metric(&self, key: &str) {
        let mut metrics = self.metrics.acquire_write();
        if let Some(entry) = metrics.get_mut(key) {
            *entry += 1;
        } else {
            metrics.insert(key.to_string(), 1);
        }
    }
}

impl<Handle: Serialize> Serialize for CountingChainHandle<Handle> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}
