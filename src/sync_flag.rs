//! `SyncFlag`s, or syncronization flags, are one-way Boolean values that can
//! be shared across threads, sending messages from a single producer to
//! any number of consumers.
//!
//! Like `std::sync::mpsc`, `SyncFlag`s are created in pairs: a transmitter
//! that can't be duplicated and a receiver that can be duplicated any number
//! of times. `SyncFlagTx` can be used in, for instance, a controller/main
//! thread, which can pass clones of the corresponding `SyncFlagRx` to the
//! worker threads it spawns in order to control them.
//!
//! # Examples
//!
//! ```
//! use workctl::new_syncflag;
//! use std::thread;
//! 
//! // Create a new SyncFlag set to communicate with the spawned thread.
//! let (mut tx, rx) = new_syncflag(true);
//! 
//! // This isn't technically needed in this case, but if we were spawning more
//! // than one thread we'd create a clone for each.
//! let thread_rx = rx.clone();
//! thread::spawn(move || {
//!     // Do nothing as long as the sync flag is true. Really, you'd do work here.
//!     while thread_rx.get() {
//!         thread::yield_now();
//!     }
//!     println!("Thread got signal to close.");
//! });
//!
//! // Do something here, like maybe adding work to a WorkQueue
//!
//! // The program has completed, so set the SyncFlag to false.
//! tx.set(false);
//! ```

use std::sync::{Arc, Mutex};

/// `SyncFlagTx` is the transmitting (mutable) half of a Single Producer,
/// Multiple Consumer Boolean (e.g. the opposite of `std::sync::mpsc`).
/// A single controller can use this to send info to any number of worker
/// threads, for instance.
///
/// `SyncFlagTx` is not Clone because it should only exist in one place.
///
/// # Panics
/// The functions on this type will panic if the underlying mutex became poisoned; 
/// that is, if there was a panic during the execution of any mutex-acquiring 
/// function. This is pretty unlikely.
pub struct SyncFlagTx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagTx {
    /// Sets the interior value of the `SyncFlagTx` which will be read by any
    /// `SyncFlagRx` that exist for this SyncFlag.
    pub fn set(&mut self, state: bool) {
        if let Ok(mut v) = self.inner.lock() {
            // The * (deref operator) means assigning to what's inside the
            // MutexGuard, not the guard itself (which would be silly)
            *v = state;
        } else {
            panic!("SyncFlagTx::set() tried to lock a poisoned mutex.");
        }
    }
}

/// `SyncFlagRx` is the receiving (immutable) half of a Single Producer,
/// Multiple Consumer Boolean (e.g. the opposite of `std::sync::mpsc`).
/// An number of worker threads can use this to get info from a single
/// controller, for instance.
///
/// `SyncFlagRx` is Clone so it can be shared across threads.
///
/// # Panics
/// The functions on this type will panic if the underlying mutex became poisoned; 
/// that is, if there was a panic during the execution of any mutex-acquiring 
/// function. This is pretty unlikely.

#[derive(Clone)]
pub struct SyncFlagRx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagRx {
    /// Gets the interior state of the `SyncFlagRx` to whatever the corresponding
    /// `SyncFlagTx` last set it to.
    ///
    /// # Errors
    /// If the underlying mutex is poisoned this might return an error.
    pub fn get(&self) -> bool {
        if let Ok(v) = self.inner.lock() {
            // Deref the MutexGuard to get at the bool inside
            *v
        } else {
            panic!("SyncFlagRx::get() tried to lock a poisoned mutex.");
        }
    }
}

/// Create a new `SyncFlagTx` and `SyncFlagRx` that can be used to share a bool 
/// across a number of threads.
pub fn new_syncflag(initial_state: bool) -> (SyncFlagTx, SyncFlagRx) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagTx { inner: state.clone() };
    let rx = SyncFlagRx { inner: state.clone() };

    return (tx, rx);
}
