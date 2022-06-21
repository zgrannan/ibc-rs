use core::fmt::Display;
use core::mem;
use core::time::Duration;
use crossbeam_channel::{bounded, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use tracing::{debug, error, warn};

use crate::util::lock::LockExt;

/**
   A task handle holds the endpoints for stopping or waiting for a
   background task to terminate.

   A holder of `TaskHandle` can explicitly stop the background task by
   calling [`shutdown`](TaskHandle::shutdown) or
   [`shutdown_and_wait`](TaskHandle::shutdown_and_wait).

   Otherwise, when the `TaskHandle` is dropped, it will stop the background
   task and wait for the background task to terminate before returning.
*/
pub struct TaskHandle {
    shutdown_sender: Sender<()>,
    stopped: Arc<RwLock<bool>>,
    join_handle: DropJoinHandle,
}

/**
   A wrapper to [`std::thread::JoinHandle`] so that the handle is joined
   when it is dropped.
*/
struct DropJoinHandle(Option<thread::JoinHandle<()>>);

/**
   A wrapper around the error type returned by a background task step
   function to indicate whether the background task should be terminated
   because of the error.
*/
pub enum TaskError<E> {
    /**
       Inform the background task runner that an ignorable error has occured,
       and the background task runner should log the error and then continue
       execution.
    */
    Ignore(E),

    /**
       Inform the background task runner that a fatal error has occured,
       and the background task runner should log the error and then abort
       execution.
    */
    Fatal(E),
}

pub enum Next {
    Continue,
    Abort,
}

impl TaskHandle {
    /**
       Wait for the background task to terminate.

       Note that because the background tasks are meant to run forever,
       this would likely never return unless errors occurred or if
       the step runner returns [`Next::Abort`] to abort prematurely.
    */
    pub fn join(mut self) {
        if let Some(handle) = mem::take(&mut self.join_handle.0) {
            let _ = handle.join();
        }
    }

    /**
       Send the shutdown signal to the background task without waiting
       for it to terminate.

       Note that the waiting will still happen when the [`TaskHandle`] is
       dropped.

       This can be used to shutdown multiple tasks in parallel, and then
       wait for them to all terminate concurrently.
    */
    pub fn shutdown(&self) {
        let _ = self.shutdown_sender.send(());
    }

    /**
       Send the shutdown signal and wait for the task to terminate.

       This is done implicitly by the [`TaskHandle`] when it is dropped.
    */
    pub fn shutdown_and_wait(self) {
        let _ = self.shutdown_sender.send(());
    }

    /**
       Check whether a background task has been stopped prematurely.
    */
    pub fn is_stopped(&self) -> bool {
        *self.stopped.acquire_read()
    }
}

impl Drop for DropJoinHandle {
    fn drop(&mut self) {
        if let Some(handle) = mem::take(&mut self.0) {
            let _ = handle.join();
        }
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        let _ = self.shutdown_sender.send(());
    }
}
