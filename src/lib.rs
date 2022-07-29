#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

pub mod async_channel;
pub mod key_filter;
pub mod sync_channel;

use std::fmt::Debug;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SendError;

#[derive(Debug, Clone)]
pub struct RecvError;

/// Channel acts as a linked-list to hold the msg.
pub struct Channel<K, V>
where
    K: Clone + HyperKey,
{
    /// AtomicPtr impls Send + Sync, so Channel is Send + Sync by default.
    /// head as a start point, no one could delete it.
    head: AtomicPtr<Node<K, V>>,
    tail: AtomicPtr<Node<K, V>>,
    filter: key_filter::Filter<K>,
}

impl<K, V> Default for Channel<K, V>
where
    K: HyperKey + Send + Debug + Clone,
    V: Send + Debug,
{
    fn default() -> Self {
        let node = Node::default();
        let node_ptr = Box::leak(Box::new(node));
        Self {
            head: AtomicPtr::new(node_ptr),
            tail: AtomicPtr::new(node_ptr),
            filter: key_filter::Filter::default(),
        }
    }
}

impl<K, V> Channel<K, V>
where
    K: HyperKey + Send + Debug + Clone,
    V: Send + Debug,
{
    /// Just like create a Linked-List.
    /// Init state:
    /// # Channel head ---    tail ---
    /// #                |           |
    /// #                |           |
    /// #               Node <--------    
    ///
    ///
    /// # Channel head ---    tail -----------
    /// #                |                   |
    /// #                |                   |
    /// #               Node ---> Node ---> `null_ptr`
    ///
    #[must_use]
    pub fn new() -> Channel<K, V> {
        let node = Node::default();
        let node_ptr = Box::leak(Box::new(node));
        Self {
            head: AtomicPtr::new(node_ptr),
            tail: AtomicPtr::new(node_ptr),
            filter: key_filter::Filter::default(),
        }
    }

    /// try to occupy the tail node.
    fn send(&self, keys: Vec<K>, val: V) {
        // try to append a new node to the Channel.
        // todo: optimize the channel appending.
        let new_node = Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(Some(Msg {
                keys,
                val,
                filter: self.filter.clone(),
            }))),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false),
        };
        let new_node = Box::into_raw(Box::new(new_node));

        let mut tail = unsafe { &*(self.tail.load(Ordering::SeqCst)) };

        let expected: *mut Node<K, V> = ptr::null_mut();

        loop {
            // can not depend on the tail, so AcqRel.
            match tail.next.compare_exchange_weak(
                expected,
                new_node,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                // append successully.
                Ok(_) => break,
                // curr is not null.
                Err(curr) => tail = unsafe { &*curr },
            }
        }

        self.tail.store(new_node, Ordering::SeqCst);
    }

    fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        let head = unsafe {
            // only one thread modifies the head, so it's not important to choose a right ordering.
            &*self.head.load(Ordering::SeqCst)
        };

        while head.next.load(Ordering::SeqCst).is_null() {
            // spin
        }

        let mut curr_node = unsafe { &*head.next.load(Ordering::SeqCst) };

        loop {
            if curr_node.is_destroy.load(Ordering::SeqCst) {
                // try to destroy the node.
                // 1. check next, if next is null, return to the head.next
                // 2. next is not null, curr becomes curr.next
                if curr_node.next.load(Ordering::SeqCst).is_null() {
                    return Err(RecvError);
                }
                curr_node = unsafe { &*curr_node.next.load(Ordering::SeqCst) };
                continue;
            }

            let msg_opt = unsafe { &*curr_node.data };

            match msg_opt {
                Some(_) => {
                    // todo: more functional format.
                    //
                    // let msg = unsafe {
                    // take the value out.
                    //    Box::from_raw(msg_node.data.get()).unwrap()

                    let msg = unsafe { Box::from_raw(curr_node.data).unwrap() };

                    if self.filter.contains(&msg.keys) {
                        Box::into_raw(Box::new(msg));
                        if curr_node.next.load(Ordering::SeqCst).is_null() {
                            // collisions are detected, and no more avaliable msg can be read.
                            return Err(RecvError);
                        }
                        curr_node = unsafe { &*curr_node.next.load(Ordering::SeqCst) };
                        // avoids double free.
                        continue;
                    }

                    curr_node.is_destroy.store(true, Ordering::SeqCst);
                    self.filter.put(&msg.keys);

                    return Ok(msg);
                }
                None => return Err(RecvError {}),
            }
        }
    }

    fn recv_sync(&self) -> Result<Msg<K, V>, RecvError> {
        self.recv()
    }

    fn send_sync(&self, keys: Vec<K>, val: V) -> Result<(), SendError> {
        // naive implementation.
        let new_node = Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(Some(Msg {
                keys,
                val,
                filter: self.filter.clone(),
            }))),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false),
        };
        let new_node = Box::into_raw(Box::new(new_node));

        let mut tail = unsafe { &*(self.tail.load(Ordering::Acquire)) };

        let expected: *mut Node<K, V> = ptr::null_mut();

        loop {
            // can not depend on the tail, so AcqRel.
            match tail.next.compare_exchange_weak(
                expected,
                new_node,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                // append successully.
                Ok(_) => break,
                // curr is not null.
                Err(curr) => tail = unsafe { &*curr },
            }
        }

        self.tail.store(new_node, Ordering::Release);

        let stored_node = unsafe { &*new_node };
        let beginning_park = Instant::now();
        let time_out = Duration::from_secs(5);

        while !stored_node.is_destroy.load(Ordering::Acquire) {
            let elapsed = beginning_park.elapsed();
            if time_out - elapsed < Duration::from_secs(0) {
                return Err(SendError);
            }
            std::hint::spin_loop();
        }
        // todo: verify the correctness.
        // difference bettween sync and async version:
        // sync: sender's duty to drop the node.
        // async: receiver's duty to drop the node.
        //
        // let head = unsafe { &*self.head.load(Ordering::Relaxed) };
        // let drop = unsafe { Box::from_raw(new_node) };

        // head.next.store(drop.next.load(Ordering::Acquire), Ordering::Release);
        Ok(())
    }
}

#[derive(Debug)]
struct Node<K, V>
where
    K: HyperKey + Clone,
{
    /// Node needs to be shared across the thread boundary, so the AtomicOpt is nessessary.
    next: AtomicPtr<Node<K, V>>,

    /// The msg to be shared.
    data: *mut Option<Msg<K, V>>,

    /// like a lock to occupy the location.
    /// after droping the msg, we can destroy this given node.
    is_destroy: AtomicBool,

    is_hold: AtomicBool,
}

impl<K, V> Default for Node<K, V>
where
    K: Clone + HyperKey,
{
    fn default() -> Self {
        Self {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(None)),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false),
        }
    }
}

pub trait HyperKey<OtherKey = Self> {
    // todo: add macro support.
    fn collision_detect(&self, other: &OtherKey) -> bool;
}

#[derive(Debug)]
pub struct Msg<K, T>
where
    K: HyperKey + Clone,
{
    pub keys: Vec<K>,
    pub val: T,
    // true is active, else false.
    filter: key_filter::Filter<K>,
}

impl<K, V> Drop for Msg<K, V>
where
    K: HyperKey + Clone,
{
    fn drop(&mut self) {
        self.filter.pop(&self.keys);
    }
}

pub trait Detector {}

// todo: impl Drop for Channel to release memory.
