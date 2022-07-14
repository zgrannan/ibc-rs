use core::fmt;
use tendermint_rpc::endpoint::broadcast::tx_sync;
use tracing::info;
use ibc::events::{IbcEvent, PrettyEvents};
use crate::chain::handle::ChainHandle;
use crate::chain::tracking::TrackedMsgs;
use crate::link::error::LinkError;
use crate::link::RelaySummary;
pub trait SubmitReply {
    /// Creates a new, empty instance, i.e., comprising zero replies.
    fn empty() -> Self;
    /// Counts the number of replies that this instance contains.
    fn len(&self) -> usize;
}
impl SubmitReply for RelaySummary {
    #[prusti_contracts::trusted]
    fn empty() -> Self {
        RelaySummary::empty()
    }
    #[prusti_contracts::trusted]
    fn len(&self) -> usize {
        self.events.len()
    }
}
/// Captures the ability to submit messages to a chain.
pub trait Submit {
    type Reply: SubmitReply;
    fn submit(
        target: &impl ChainHandle,
        msgs: TrackedMsgs,
    ) -> Result<Self::Reply, LinkError>;
}
/// Synchronous sender
pub struct SyncSender;
impl Submit for SyncSender {
    type Reply = RelaySummary;
    #[prusti_contracts::trusted]
    fn submit(
        target: &impl ChainHandle,
        msgs: TrackedMsgs,
    ) -> Result<Self::Reply, LinkError> {
        let tx_events = target
            .send_messages_and_wait_commit(msgs)
            .map_err(LinkError::relayer)?;
        info!("[Sync->{}] result {}\n", target.id(), PrettyEvents(& tx_events));
        let ev = tx_events
            .clone()
            .into_iter()
            .find(|event| matches!(event, IbcEvent::ChainError(_)));
        match ev {
            Some(ev) => Err(LinkError::send(ev)),
            None => Ok(RelaySummary::from_events(tx_events)),
        }
    }
}
pub struct AsyncReply {
    pub responses: Vec<tx_sync::Response>,
}
impl SubmitReply for AsyncReply {
    #[prusti_contracts::trusted]
    fn empty() -> Self {
        Self { responses: vec![] }
    }
    #[prusti_contracts::trusted]
    fn len(&self) -> usize {
        self.responses.len()
    }
}
pub struct AsyncSender;
impl Submit for AsyncSender {
    type Reply = AsyncReply;
    #[prusti_contracts::trusted]
    fn submit(
        target: &impl ChainHandle,
        msgs: TrackedMsgs,
    ) -> Result<Self::Reply, LinkError> {
        let a = target
            .send_messages_and_wait_check_tx(msgs)
            .map_err(LinkError::relayer)?;
        let reply = AsyncReply { responses: a };
        info!("[Async~>{}] {}\n", target.id(), reply);
        Ok(reply)
    }
}
impl fmt::Display for AsyncReply {
    #[prusti_contracts::trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "response(s): {}", self.responses.len())?;
        self.responses.iter().try_for_each(|r| write!(f, "; {:?}:{}", r.code, r.hash))
    }
}

