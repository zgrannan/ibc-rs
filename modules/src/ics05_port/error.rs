use anomaly::{BoxError, Context};
use thiserror::Error;
use prusti_contracts::*;

pub type Error = anomaly::Error<Kind>;

impl std::fmt::Display for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}
impl std::fmt::Debug for Kind {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}


#[derive(Clone, Error)]
pub enum Kind {
//     #[error("port unknown")]
    UnknownPort,
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
panic!("No") //         Context::new(self, Some(source.into()))
    }
}
