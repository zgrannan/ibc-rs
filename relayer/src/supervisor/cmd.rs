#[cfg(feature="prusti")]
use prusti_contracts::*;
use crossbeam_channel::Sender;
use ibc::ics24_host::identifier::ChainId;

use crate::config::ChainConfig;

use super::dump_state::SupervisorState;

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug))]
pub enum ConfigUpdate {
    Add(ChainConfig),
    Remove(ChainId),
    Update(ChainConfig),
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug))]
pub enum SupervisorCmd {
    UpdateConfig(ConfigUpdate),
    DumpState(Sender<SupervisorState>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CmdEffect {
    ConfigChanged,
    Nothing,
}

impl CmdEffect {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (CmdEffect::ConfigChanged, _) => CmdEffect::ConfigChanged,
            (_, CmdEffect::ConfigChanged) => CmdEffect::ConfigChanged,
            _ => self,
        }
    }
}
