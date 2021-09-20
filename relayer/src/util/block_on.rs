//! Utility function to execute a future synchronously

#[cfg(feature="prusti")]
use prusti_contracts::*;

use futures::Future;

/// Spawns a new tokio runtime and use it to block on the given future.
#[cfg_attr(feature="prusti", trusted_skip)]
pub fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
