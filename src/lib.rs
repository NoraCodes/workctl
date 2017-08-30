//! `workctl` provides a set of higher-level abstractions for controlling 
//! concurrent/parallel programs. These abstractions are especially focused on
//! the "controller/worker" paradigm, in which one or a few "controller" 
//! threads determine what work needs to be done and use `WorkQueue`s and 
//! `SyncFlag`s to communicate that to many "worker" threads.
//!
//! # Examples
//!
//! Here is a typical example using a `WorkQueue`, a `SyncFlag`, and a `std::sync::mpsc`.
//! This is somewhat more complex than is required for processing a list of numbers, but
//! it illustrates the principle. When looking at this example, imagine that you might
//! 
//! * have a mechanism by which some of the worker threads can add new work or, 
//! * that the control thread (or another thread) expects to produce work _forever_,
//! as in a server, for instance. 
//!
//! The `SyncFlag` can then be used at any future time to
//! gracefully shut down all the worker threads, e.g. when the controller gets 
//! `SIGTERM`.
//!
//! ```
//! use std::thread;
//! use workctl::{WorkQueue, new_syncflag};
//!
//! // Create a new work queue to schedule pieces of work; in this case, i32s.
//! // The type annotation is not strictly needed.
//! let mut queue: WorkQueue<i32> = WorkQueue::new();
//!
//! // Create a channel for the worker threads to send messages back on.
//! use std::sync::mpsc::channel;
//! let (results_tx, results_rx) = channel();
//!
//! // Create a SyncFlag to share whether or not the worker threads should
//! // keep waiting on jobs.
//! let (mut more_jobs_tx, more_jobs_rx) = new_syncflag(true);
//!
//! // This Vec is just for the controller to keep track of the worker threads.
//! let mut thread_handles = Vec::new();
//!
//! // Spawn 4 workers.
//! for _ in 0..4 {
//!     // Create clones of the various control mechanisms for the new thread.
//!     let mut t_queue = queue.clone();
//!     let t_results_tx = results_tx.clone();
//!     let t_more_jobs = more_jobs_rx.clone();
//!
//!     let handle = thread::spawn(move || {
//!         // Loop until the controller says to stop.
//!         while t_more_jobs.get() {
//!             // Try to get a piece of work to do.
//!             if let Some(work_input) = t_queue.pull_work() {
//!                 // Do some work. Totally contrived in this case.
//!                 let result = work_input % 1024;
//!                 // Send the results of the work to the main thread. 
//!                 t_results_tx.send((work_input, result)).unwrap();
//!             } else {
//!                 thread::yield_now();
//!             }
//!         }
//!     });
//!     
//!     // Add the handle to the vec of handles
//!     thread_handles.push(handle);
//! }
//!
//! // Put some work in the queue.
//! let mut total_work = 0;
//! for _ in 0..10 {
//!     queue.push_work(1023);
//!     total_work += 1;
//! }
//!
//! for _ in 0..10 {
//!     queue.push_work(1024);
//!     total_work += 1;
//! }
//!
//!
//! // Now, receive all the results.
//! let mut results = Vec::new();
//! while total_work > 0 {
//!     // In reality, you'd do something with these results.
//!     let r = results_rx.recv().unwrap();
//!     total_work -= 1;
//!     results.push(r);
//! }
//!
//! 
//!
//! // All the work is done, so tell the workers to stop looking for work.
//! more_jobs_tx.set(false);
//!
//! // Join all the threads.
//! for thread_handle in thread_handles {
//!     thread_handle.join().unwrap();
//! }
//!
//! assert_eq!(results.len(), 20);
//! ```
pub mod work_queue;
pub use work_queue::WorkQueue;

pub mod sync_flag;
pub use sync_flag::new_syncflag;

