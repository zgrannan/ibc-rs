//! Capabilities: this is a placeholder.
#[cfg(feature="prusti")]
use prusti_contracts::*;

#[derive(Clone)]
pub struct Capability {
    index: u64,
}

impl Capability {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn new() -> Capability {
        Self { index: 0x0 }
    }
}

impl Default for Capability {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn default() -> Self {
        Self::new()
    }
}
