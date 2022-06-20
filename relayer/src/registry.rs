//! Registry for keeping track of [`ChainHandle`]s indexed by a `ChainId`.

use alloc::collections::btree_map::BTreeMap as HashMap;
use alloc::sync::Arc;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use tokio::runtime::Runtime as TokioRuntime;
use tracing::{trace, warn};

use ibc::core::ics24_host::identifier::ChainId;

use crate::{
    chain::handle::ChainHandle,
    config::Config,
    util::lock::RwArc,
};

/// Registry for keeping track of [`ChainHandle`]s indexed by a `ChainId`.
///
/// The purpose of this type is to avoid spawning multiple runtimes for a single `ChainId`.
#[derive(Debug)]
pub struct Registry<Chain: ChainHandle> {
    config: Config,
    handles: HashMap<ChainId, Chain>,
    rt: Arc<TokioRuntime>,
}

#[derive(Clone)]
pub struct SharedRegistry<Chain: ChainHandle> {
    pub registry: RwArc<Registry<Chain>>,
}

impl<Chain: ChainHandle> Registry<Chain> {
    /// Construct a new [`Registry`] using the provided [`Config`]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            handles: HashMap::new(),
            rt: Arc::new(TokioRuntime::new().unwrap()),
        }
    }

    /// Return the size of the registry, i.e., the number of distinct chain runtimes.
    pub fn size(&self) -> usize {
        self.handles.len()
    }

    /// Return an iterator overall the chain handles managed by the registry.
    pub fn chains(&self) -> impl Iterator<Item = &Chain> {
        self.handles.values()
    }

    /// Shutdown the runtime associated with the given chain identifier.
    pub fn shutdown(&mut self, chain_id: &ChainId) {
        if let Some(handle) = self.handles.remove(chain_id) {
            if let Err(e) = handle.shutdown() {
                warn!(chain = %chain_id, "chain runtime might have failed to shutdown properly: {}", e);
            }
        }
    }
}

impl<Chain: ChainHandle> SharedRegistry<Chain> {
    pub fn new(config: Config) -> Self {
        let registry = Registry::new(config);

        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    pub fn shutdown(&self, chain_id: &ChainId) {
        self.write().shutdown(chain_id)
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Registry<Chain>> {
        self.registry.write().unwrap()
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Registry<Chain>> {
        self.registry.read().unwrap()
    }
}
