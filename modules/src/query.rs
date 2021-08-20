use tendermint::abci::transaction::Hash;

use crate::ics02_client::client_consensus::QueryClientEventRequest;
use crate::ics04_channel::channel::QueryPacketEventDataRequest;
use prusti_contracts::*;

/// Used for queries and not yet standardized in channel's query.proto
#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub enum QueryTxRequest {
    Packet(QueryPacketEventDataRequest),
    Client(QueryClientEventRequest),
    Transaction(QueryTxHash),
}

#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
pub struct QueryTxHash(pub Hash);
