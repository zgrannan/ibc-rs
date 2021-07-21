use anomaly::{BoxError, Context};
use thiserror::Error;
use prusti_contracts::*;

pub type Error = anomaly::Error<Kind>;

#[derive(Clone, Debug, Error)]
pub enum Kind {
    #[error("port unknown")]
    UnknownPort,
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}
