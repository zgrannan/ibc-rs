#[cfg(feature="prusti")]
use prusti_contracts::*;

use std::fmt;
use std::time::Instant;

use prost_types::Any;
use tracing::{info, warn};

use ibc::events::IbcEvent;
use ibc::Height;

use crate::link::error::LinkError;
use crate::link::RelayPath;

#[derive(Clone, Copy, PartialEq)]
pub enum OperationalDataTarget {
    Source,
    Destination,
}

impl fmt::Display for OperationalDataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationalDataTarget::Source => write!(f, "Source"),
            OperationalDataTarget::Destination => write!(f, "Destination"),
        }
    }
}

/// A packet messages that is prepared for sending to a chain, but has not been sent yet.
/// Comprises both the proto-encoded packet message, alongside the event which generated it.
#[cfg_attr(feature="prusti", derive(PrustiClone))]
#[cfg_attr(not(feature="prusti"), derive(Clone))]
pub struct TransitMessage {
    pub event: IbcEvent,
    pub msg: Any,
}

/// Holds all the necessary information for handling a set of in-transit messages.
///
/// Each `OperationalData` item is uniquely identified by the combination of two attributes:
///     - `target`: represents the target of the packet messages, either source or destination chain,
///     - `proofs_height`: represents the height for the proofs in all the messages.
///       Note: this is the height at which the proofs are queried. A client consensus state at
///       `proofs_height + 1` must exist on-chain in order to verify the proofs.
#[cfg_attr(feature="prusti", derive(PrustiClone))]
#[cfg_attr(not(feature="prusti"), derive(Clone))]
pub struct OperationalData {
    pub proofs_height: Height,
    pub batch: Vec<TransitMessage>,
    pub target: OperationalDataTarget,
    /// Stores the time when the clients on the target chain has been updated, i.e., when this data
    /// was scheduled. Necessary for packet delays.
    pub scheduled_time: Instant,
}

impl OperationalData {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn new(proofs_height: Height, target: OperationalDataTarget) -> Self {
        OperationalData {
            proofs_height,
            batch: vec![],
            target,
            scheduled_time: Instant::now(),
        }
    }

    #[cfg_attr(feature="prusti", trusted_skip)]
    pub fn events(&self) -> Vec<IbcEvent> {
        self.batch.iter().map(|gm| gm.event.clone()).collect()
    }

    /// Returns all the messages in this operational data, plus prepending the client update message
    /// if necessary.
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn assemble_msgs(&self, relay_path: &RelayPath) -> Result<Vec<Any>, LinkError> {
        if self.batch.is_empty() {
            warn!("assemble_msgs() method call on an empty OperationalData!");
            return Ok(vec![]);
        }

        let mut msgs: Vec<Any> = self.batch.iter().map(|gm| gm.msg.clone()).collect();

        // For zero delay we prepend the client update msgs.
        if relay_path.zero_delay() {
            let update_height = self.proofs_height.increment();

            info!(
                "[{}] prepending {} client update @ height {}",
                relay_path, self.target, update_height
            );

            // Fetch the client update message. Vector may be empty if the client already has the header
            // for the requested height.
            let mut client_update_opt = match self.target {
                OperationalDataTarget::Source => {
                    relay_path.build_update_client_on_src(update_height)?
                }
                OperationalDataTarget::Destination => {
                    relay_path.build_update_client_on_dst(update_height)?
                }
            };

            if let Some(client_update) = client_update_opt.pop() {
                msgs.insert(0, client_update);
            }
        }

        info!(
            "[{}] assembled batch of {} message(s)",
            relay_path,
            msgs.len()
        );

        Ok(msgs)
    }
}

impl fmt::Display for OperationalData {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Op.Data [->{} @{}; {} event(s) & msg(s) in batch]",
            self.target,
            self.proofs_height,
            self.batch.len(),
        )
    }
}
