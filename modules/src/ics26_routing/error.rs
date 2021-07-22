
use anomaly::{BoxError, Context};
use thiserror::Error;
use prusti_contracts::*;

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
//     #[error("error raised by message handler")]
    HandlerRaisedError,

//     #[error("error raised by the keeper functionality in message handler")]
    KeeperRaisedError,

//     #[error("unknown type URL {0}")]
    UnknownMessageTypeUrl(String),

//     #[error("the message is malformed and cannot be decoded")]
    MalformedMessageBytes,
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
unreachable!() //         Context::new(self, Some(source.into()))
    }
}
