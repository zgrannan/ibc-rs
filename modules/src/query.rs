use tendermint::abci::transaction::Hash;

use crate::ics02_client::client_consensus::QueryClientEventRequest;
use crate::ics04_channel::channel::QueryPacketEventDataRequest;

/// Used for queries and not yet standardized in channel's query.proto
#[derive(Clone)]
pub enum QueryTxRequest {
    Packet(QueryPacketEventDataRequest),
    Client(QueryClientEventRequest),
    Transaction(QueryTxHash),
}

#[derive(Clone)]
pub struct QueryTxHash(pub Hash);
