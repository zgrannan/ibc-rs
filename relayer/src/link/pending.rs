use core::iter::Iterator;
use core::time::Duration;
use std::time::Instant;
use tracing::{debug, error, trace, trace_span};
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc::events::IbcEvent;
use crate::chain::requests::{QueryTxHash, QueryTxRequest};
use crate::chain::tracking::TrackingId;
use crate::error::Error as RelayerError;
use crate::link::{error::LinkError, RelayPath};
use crate::telemetry;
use crate::util::queue::Queue;
use crate::{
    chain::handle::ChainHandle,
    link::{
        operational_data::OperationalData, relay_sender::AsyncReply, RelaySummary,
        TxHashes,
    },
};
pub const TIMEOUT: Duration = Duration::from_secs(300);
/// A wrapper over an [`OperationalData`] that is pending.
/// Additionally holds all the necessary information
/// to query for confirmations:
///     - hashes for all transactions in that op. data,
///     - the target chain to query for confirmations,
///     - timestamp to track time-outs and declare an
///         operational data as pending.
#[derive(Clone)]
pub struct PendingData {
    pub original_od: OperationalData,
    pub tx_hashes: TxHashes,
    pub submit_time: Instant,
    pub error_events: Vec<IbcEvent>,
}
impl PendingData {
    #[prusti_contracts::trusted]
    pub fn tracking_id(&self) -> TrackingId {
        self.original_od.tracking_id
    }
}
/// Stores all pending data
/// and tries to confirm them asynchronously.
pub struct PendingTxs<Chain> {
    pub chain: Chain,
    pub channel_id: ChannelId,
    pub port_id: PortId,
    pub counterparty_chain_id: ChainId,
    pub pending_queue: Queue<PendingData>,
}
impl<Chain> PendingTxs<Chain> {
    #[prusti_contracts::trusted]
    pub fn new(
        chain: Chain,
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_chain_id: ChainId,
    ) -> Self {
        Self {
            chain,
            channel_id,
            port_id,
            counterparty_chain_id,
            pending_queue: Queue::new(),
        }
    }
}
impl<Chain: ChainHandle> PendingTxs<Chain> {
    #[prusti_contracts::trusted]
    pub fn chain_id(&self) -> ChainId {
        self.chain.id()
    }
    /// Insert a new pending transaction to the back of the queue.
    #[prusti_contracts::trusted]
    pub fn insert_new_pending_tx(&self, r: AsyncReply, od: OperationalData) {
        let mut tx_hashes = Vec::new();
        let mut error_events = Vec::new();
        for response in r.responses.into_iter() {
            if response.code.is_err() {
                let span = trace_span!(
                    "inserting new pending txs", chain = % self.chain_id(),
                    counterparty_chain = % self.counterparty_chain_id, port = % self
                    .port_id, channel = % self.channel_id,
                );
                let _guard = span.enter();
                trace!("putting error response in error event queue: {:?} ", response);
                let error_event = IbcEvent::ChainError(
                    format!(
                        "deliver_tx on chain {} for Tx hash {} reports error: code={:?}, log={:?}",
                        self.chain_id(), response.hash, response.code, response.log
                    ),
                );
                error_events.push(error_event);
            } else {
                tx_hashes.push(response.hash);
            }
        }
        let u = PendingData {
            original_od: od,
            tx_hashes: TxHashes(tx_hashes),
            submit_time: Instant::now(),
            error_events,
        };
        self.pending_queue.push_back(u);
    }
    #[prusti_contracts::trusted]
    fn check_tx_events(
        &self,
        tx_hashes: &TxHashes,
    ) -> Result<Option<Vec<IbcEvent>>, RelayerError> {
        let mut all_events = Vec::new();
        for hash in &tx_hashes.0 {
            let mut events = self
                .chain
                .query_txs(QueryTxRequest::Transaction(QueryTxHash(*hash)))?;
            if events.is_empty() {
                return Ok(None);
            } else {
                all_events.append(&mut events)
            }
        }
        Ok(Some(all_events))
    }
    /// Try and process one pending transaction within the given timeout duration if one
    /// is available.
    ///
    /// A `resubmit` closure is provided when the clear interval for packets is 0. If this closure
    /// is provided, the pending transactions that fail to process within the given timeout duration
    /// are resubmitted following the logic specified by the closure.
    #[prusti_contracts::trusted]
    pub fn process_pending<ChainA: ChainHandle, ChainB: ChainHandle>(
        &self,
        timeout: Duration,
        relay_path: &RelayPath<ChainA, ChainB>,
        resubmit: Option<impl FnOnce(OperationalData) -> Result<AsyncReply, LinkError>>,
    ) -> Result<Option<RelaySummary>, LinkError> {
        if let Some(pending) = self.pending_queue.pop_front() {
            let tx_hashes = &pending.tx_hashes;
            let submit_time = &pending.submit_time;
            if tx_hashes.0.is_empty() {
                return Ok(Some(RelaySummary::from_events(pending.error_events)));
            }
            let span = trace_span!(
                "processing pending tx", chain = % self.chain_id(), counterparty_chain =
                % self.counterparty_chain_id, port = % self.port_id, channel = % self
                .channel_id,
            );
            let _guard = span.enter();
            trace!("trying to confirm {} ", tx_hashes);
            let relay_summary = match self.check_tx_events(tx_hashes) {
                Ok(None) => {
                    trace!("transaction is not yet committed: {} ", tx_hashes);
                    if submit_time.elapsed() > timeout {
                        error!("timed out while confirming {}", tx_hashes);
                        match resubmit {
                            Some(f) => {
                                let new_od = relay_path
                                    .regenerate_operational_data(pending.original_od.clone());
                                trace!("regenerated operational data for {}", tx_hashes);
                                match new_od.map(f) {
                                    Some(Ok(reply)) => {
                                        self.insert_new_pending_tx(reply, pending.original_od);
                                        Ok(None)
                                    }
                                    Some(Err(e)) => {
                                        self.pending_queue.push_back(pending);
                                        Err(e)
                                    }
                                    None => Ok(None),
                                }
                            }
                            None => Ok(None),
                        }
                    } else {
                        self.pending_queue.push_back(pending);
                        Ok(None)
                    }
                }
                Ok(Some(mut events)) => {
                    debug!(
                        tracking_id = % pending.tracking_id(), elapsed = ? pending
                        .submit_time.elapsed(), tx_hashes = % tx_hashes,
                        "transactions confirmed",
                    );
                    telemetry!(
                        tx_confirmed, tx_hashes.0.len(), pending.tracking_id(), & self
                        .chain.id(), & self.channel_id, & self.port_id, & self
                        .counterparty_chain_id
                    );
                    events.extend(pending.error_events);
                    Ok(Some(RelaySummary::from_events(events)))
                }
                Err(e) => {
                    error!(
                        "error querying for tx hashes {}: {}. will retry again later",
                        tx_hashes, e
                    );
                    self.pending_queue.push_back(pending);
                    Err(LinkError::relayer(e))
                }
            };
            if !self.pending_queue.is_empty() {
                trace!("total pending transactions left: {}", self.pending_queue.len());
            }
            relay_summary
        } else {
            Ok(None)
        }
    }
}

