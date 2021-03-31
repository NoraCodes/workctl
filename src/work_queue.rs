//! `WorkQueue` is a purely safe work queue, suitable for fairly distributing
//! work to any number of worker threads from any number of controllers.

use std::sync::{Mutex, Arc};
use std::collections::VecDeque;
use std::thread;

use crate::sync_flag::SyncFlagRx;

/// A generic work queue for any work element that is Send.
/// This queue is symmetric, in that any thread with a copy of it can
/// add work or remove work.
///
/// # Examples
///
/// The general usage pattern is to create a queue in a controller/main thread
/// and clone it into other threads.
///
/// ```
/// use workctl::WorkQueue;
/// use std::thread;
///
/// // Create a WorkQueue to share work into other threads.
/// let mut wq = WorkQueue::new();
///
/// // Make a clone of the queue (this is like Arc, it's creating another
/// // reference to the actual underlying queue).
/// // This gets moved into the spawned thread.
/// let mut thread_wq = wq.clone();
/// let handle = thread::spawn(move || {
///    loop {
///         if let Some(work) = thread_wq.pull_work() {
///             // Do some work!
///             println!("Got work {} in spawned thread.", work);
///             break;
///         } else {
///             thread::yield_now();
///         }
///    }
/// });
///
/// wq.push_work(1337);
///
/// handle.join().unwrap();
/// ```
///
/// # Panics
/// The functions on this type will panic if the underlying mutex became poisoned;
/// that is, if there was a panic during the execution of any mutex-acquiring function.
/// This is pretty unlikely.
#[derive(Clone)]
pub struct WorkQueue<T: Send> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send> WorkQueue<T> {
    /// Creates a new, empty WorkQueue with the default capacity.
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(VecDeque::new())) }
    }

    /// Creates a new, empty WorkQueue with at least the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))) }
    }

    /// Blocks the current thread until it can check if work is available,
    /// then acquires the work data and removes it from the queue.
    ///
    /// Returns `None` if there is currently no work in the queue.
    pub fn pull_work(&mut self) -> Option<T> {
        // Try to lock the internal mutex. This will block or fail.
        if let Ok(mut queue) = self.inner.lock() {
            // Try to get an element from the queue
            queue.pop_front()
        } else {
            panic!("WorkQueue::pull_work() tried to lock a poisoned mutex.");
        }
    }

    /// Blocks the current thread until it can add work to the queue, adding that
    /// work at the end of the queue.
    ///
    /// Returns the number of elements in the queue after inserting that work.
    pub fn push_work(&mut self, work_element: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work_element);
            queue.len()
        } else {
            panic!("WorkQueue::push_work() tried to lock a poisoned mutex.");
        }
    }

    /// Blocks the current thread until either some work is available or `run_flag`
    /// becomes false.
    ///
    /// Returns either a piece of work or None, signifying that no more work is
    /// expected because `run_flag` is false.
    ///
    /// # Examples
    ///
    /// Based on the example for the type, above, we can create a worker that
    /// looks for jobs until it's told to stop.
    ///
    /// ```
    /// use workctl::{WorkQueue, new_syncflag};
    /// use std::thread;
    ///
    /// let mut wq = WorkQueue::new();
    ///
    /// let (mut run_tx, run_rx) = new_syncflag(true);
    ///
    /// let mut thread_wq = wq.clone();
    /// let handle = thread::spawn(move || {
    ///     let mut work_done = 0;
    ///     // Wait until either there is work or the worker should quit
    ///     while let Some(work) = thread_wq.wait(&run_rx) {
    ///         // Do some work!
    ///         println!("Got work {} in spawned thread.", work);
    ///         work_done += 1;
    ///     }
    ///     assert_eq!(work_done, 2, 
    ///         "Expected worker to do 2 work; it did {}.", work_done);
    /// });
    ///
    /// // Put some work in the queue.
    /// wq.push_work(1337);
    /// wq.push_work(1024);
    ///
    /// // Wait a bit.
    /// thread::sleep_ms(1000);
    ///
    /// // Tell the worker to stop looking for work
    /// run_tx.set(false);
    ///
    /// // This work won't get done.
    /// wq.push_work(1776);
    ///
    /// handle.join().unwrap();
    /// ```
    ///
    /// 
    pub fn wait(&mut self, run_flag: &SyncFlagRx) -> Option<T> {
        while run_flag.get() {
            if let Some(w) = self.pull_work() {
                return Some(w);
            }
            thread::yield_now();
        }

        return None;
    }

    /// Blocks the current thread until it can examine the queue, returning the
    /// number of work elements that remain in the queue.
    pub fn len(&self) -> usize {
        if let Ok(queue) = self.inner.lock() {
            queue.len()
        } else {
            panic!("WorkQueue::len() tried to lock a poisoned mutex.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkQueue;
    #[test]
    fn add_and_remove() {
        let mut wq: WorkQueue<i32> = WorkQueue::new();

        assert_eq!(
            wq.len(),
            0,
            "Expected queue to be created empty, it was {} long.",
            wq.len()
        );

        wq.push_work(0);
        let len_after_2_pushes = wq.push_work(1);

        assert_eq!(
            len_after_2_pushes,
            2,
            "Expected queue to have 2 elements, it was {} long.",
            len_after_2_pushes,
        );

        wq.pull_work();
        let work = wq.pull_work();

        assert_eq!(
            work.unwrap(),
            1,
            "Expected to pull 1 second since 1 was pushed second. Instead got {}.",
            work.unwrap(),
        );

        let work = wq.pull_work();
        assert_eq!(
            work,
            None,
            "Expected to get None when pulling from an empty queue; instead, got {:?}.",
            work,
        );
    }
}
