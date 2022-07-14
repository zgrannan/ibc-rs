use core::fmt;
use ibc::events::IbcEvent;
#[derive(Clone, Debug)]
pub struct RelaySummary {
    pub events: Vec<IbcEvent>,
}
impl RelaySummary {
    #[prusti_contracts::trusted]
    pub fn empty() -> Self {
        Self { events: vec![] }
    }
    #[prusti_contracts::trusted]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    #[prusti_contracts::trusted]
    pub fn from_events(events: Vec<IbcEvent>) -> Self {
        Self { events }
    }
    #[prusti_contracts::trusted]
    pub fn extend(&mut self, other: RelaySummary) {
        self.events.extend(other.events)
    }
}
impl fmt::Display for RelaySummary {
    #[prusti_contracts::trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RelaySummary events: ")?;
        for e in &self.events {
            write!(f, "{}; ", e)?
        }
        write!(f, "total events = {}", self.events.len())
    }
}

