use alloc::collections::VecDeque;
use std::sync::{Arc, RwLock};
use crate::util::lock::LockExt;
/// A lightweight wrapper type to RefCell<VecDeque<T>> so that
/// we can safely mutate it with regular reference instead of
/// mutable reference. We only expose subset of VecDeque methods
/// that does not return any inner reference, so that the RefCell
/// can never panic caused by simultaneous borrow and borrow_mut.
pub struct Queue<T>(Arc<RwLock<VecDeque<T>>>);
impl<T> Queue<T> {
    #[prusti_contracts::trusted]
    pub fn new() -> Self {
        Queue(Arc::new(RwLock::new(VecDeque::new())))
    }
    #[prusti_contracts::trusted]
    pub fn pop_front(&self) -> Option<T> {
        self.0.acquire_write().pop_front()
    }
    #[prusti_contracts::trusted]
    pub fn pop_back(&self) -> Option<T> {
        self.0.acquire_write().pop_back()
    }
    #[prusti_contracts::trusted]
    pub fn push_back(&self, val: T) {
        self.0.acquire_write().push_back(val)
    }
    #[prusti_contracts::trusted]
    pub fn push_front(&self, val: T) {
        self.0.acquire_write().push_front(val)
    }
    #[prusti_contracts::trusted]
    pub fn len(&self) -> usize {
        self.0.acquire_read().len()
    }
    #[prusti_contracts::trusted]
    pub fn is_empty(&self) -> bool {
        self.0.acquire_read().is_empty()
    }
    #[prusti_contracts::trusted]
    pub fn into_vec(self) -> VecDeque<T> {
        self.0.acquire_write().drain(..).collect()
    }
    #[prusti_contracts::trusted]
    pub fn replace(&self, queue: VecDeque<T>) {
        *self.0.acquire_write() = queue;
    }
    #[prusti_contracts::trusted]
    pub fn take(&self) -> VecDeque<T> {
        self.0.acquire_write().drain(..).collect()
    }
}
impl<T: Clone> Queue<T> {
    #[prusti_contracts::trusted]
    pub fn clone_vec(&self) -> VecDeque<T> {
        self.0.acquire_read().clone()
    }
}
impl<T> Default for Queue<T> {
    #[prusti_contracts::trusted]
    fn default() -> Self {
        Self::new()
    }
}
impl<T> From<VecDeque<T>> for Queue<T> {
    #[prusti_contracts::trusted]
    fn from(deque: VecDeque<T>) -> Self {
        Queue(Arc::new(RwLock::new(deque)))
    }
}

