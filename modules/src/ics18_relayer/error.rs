use crate::ics24_host::identifier::ClientId;
use crate::Height;
use prusti_contracts::*;
use anomaly::{BoxError, Context};
use thiserror::Error;

pub type Error = anomaly::Error<Kind>;

impl std::fmt::Debug for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::fmt::Display for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}



#[derive(Clone, Error)]
pub enum Kind {
//     #[error("client state on destination chain not found, (client id: {0})")]
    ClientStateNotFound(ClientId),

//     #[error("the client on destination chain is already up-to-date (client id: {0}, source height: {1}, dest height: {2})")]
    ClientAlreadyUpToDate(ClientId, Height, Height),

//     #[error("the client on destination chain is at a higher height (client id: {0}, source height: {1}, dest height: {2})")]
    ClientAtHigherHeight(ClientId, Height, Height),

//     #[error("transaction processing by modules failed")]
    TransactionFailed,
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
unreachable!() //         Context::new(self, Some(source.into()))
    }
}
