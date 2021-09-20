#[cfg(feature="prusti")]
use prusti_contracts::*;

use ibc::{ics02_client::events::NewBlock, Height};

use crate::event::monitor::EventBatch;

/// A command for a [`Worker`].
#[cfg_attr(feature="prusti", derive(PrustiDebug,PrustiClone))]
#[cfg_attr(not(feature="prusti"), derive(Debug,Clone))]
pub enum WorkerCmd {
    /// A batch of packet events need to be relayed
    IbcEvents { batch: EventBatch },

    /// A new block has been committed
    NewBlock { height: Height, new_block: NewBlock },

    /// Trigger a pending packets clear
    ClearPendingPackets,

    /// Shutdown the worker
    Shutdown,
}
