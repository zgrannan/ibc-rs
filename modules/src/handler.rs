use crate::events::IbcEvent;
use std::marker::PhantomData;
#[cfg(feature="prusti")]
use prusti_contracts::*;

pub type HandlerResult<T, E> = Result<HandlerOutput<T>, E>;

#[derive(Clone)]
pub struct HandlerOutput<T> {
    pub result: T,
    pub log: Vec<String>,
    pub events: Vec<IbcEvent>,
}

impl<T> HandlerOutput<T> {
#[cfg_attr(feature="prusti", trusted)]
    pub fn builder() -> HandlerOutputBuilder<T> {
        HandlerOutputBuilder::new()
    }
}

#[derive(Clone, Default)]
pub struct HandlerOutputBuilder<T> {
    log: Vec<String>,
    events: Vec<IbcEvent>,
    marker: PhantomData<T>,
}

impl<T> HandlerOutputBuilder<T> {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn new() -> Self {
        Self {
            log: vec![],
            events: vec![],
            marker: PhantomData,
        }
    }

    // These seem to be upsetting Prusti
    /*
    pub fn with_log(mut self, log: impl Into<Vec<String>>) -> Self {
        self.log.append(&mut log.into());
        self
    }

    pub fn log(&mut self, log: impl Into<String>) {
        self.log.push(log.into());
    }
    */

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn with_log<U>(mut self, log: U) -> Self {
        // self.log.append(&mut log.into());
        self
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn log<U>(&mut self, log: U) {
        // self.log.push(log.into());
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn with_events(mut self, mut events: Vec<IbcEvent>) -> Self {
        self.events.append(&mut events);
        self
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn emit(&mut self, event: IbcEvent) {
        self.events.push(event);
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn with_result(self, result: T) -> HandlerOutput<T> {
        HandlerOutput {
            result,
            log: self.log,
            events: self.events,
        }
    }
}
