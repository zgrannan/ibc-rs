#[cfg(feature="prusti")]
use prusti_contracts::*;
use ibc::events::IbcEvent;

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug))]
pub struct RelaySummary {
    pub events: Vec<IbcEvent>,
    // errors: todo!(),
    // timings: todo!(),
}

impl RelaySummary {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn empty() -> Self {
        Self { events: vec![] }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn from_events(events: Vec<IbcEvent>) -> Self {
        Self { events }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn extend(&mut self, other: RelaySummary) {
        self.events.extend(other.events)
    }
}
