use anomaly::{BoxError, Context};
use thiserror::Error;
use prusti_contracts::*;

use crate::ics24_host::error::ValidationKind;

pub type Error = anomaly::Error<Kind>;

impl std::fmt::Display for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::fmt::Debug for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}


#[derive(Clone, Error)]
pub enum Kind {
//     #[error("invalid trusting period")]
    InvalidTrustingPeriod,

//    #[error("invalid client state trust threshold")]
    InvalidTrustThreshold,

//    #[error("invalid unbonding period")]
    InvalidUnboundingPeriod,

//     #[error("invalid address")]
    InvalidAddress,

//     #[error("invalid header, failed basic validation")]
    InvalidHeader,

//     #[error("validation error")]
    ValidationError,

//     #[error("invalid raw client state")]
    InvalidRawClientState,

//     #[error("invalid chain identifier: raw value {0} with underlying validation error: {1}")]
    InvalidChainId(String, ValidationKind),

//     #[error("invalid raw height")]
    InvalidRawHeight,

//     #[error("invalid raw client consensus state")]
    InvalidRawConsensusState,

//     #[error("invalid raw header")]
    InvalidRawHeader,

//     #[error("invalid raw misbehaviour")]
    InvalidRawMisbehaviour,
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
unreachable!() //         Context::new(self, Some(source.into()))
    }
}
