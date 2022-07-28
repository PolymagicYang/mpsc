#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]

pub mod async_channel;
pub mod sync_channel;

use std::fmt::Debug;
use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering};
use std::ptr;

#[derive(Debug, Clone)]
pub struct SendError;

#[derive(Debug, Clone)]
pub struct RecvError;

/// Channel acts as a linked-list to hold the msg.
pub struct Channel<K: HyperKey, V> {
    /// head as a start point, no one could delete it.
    head: AtomicPtr<Node<K, V>>,
    tail: AtomicPtr<Node<K, V>>,
}

impl<K, V> Channel<K, V>
where 
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    /// Just like create a Linked-List.
    /// Init state:
    /// Channel head ---    tail ---
    ///                |           |
    ///                |           |
    ///               Node <--------    
    ///
    ///
    /// Channel head ---    tail -----------
    ///                |                   |
    ///                |                   |
    ///               Node ---> Node ---> null_ptr
    pub fn new() -> Channel<K, V> {
        let node = Node::default();
        let node_ptr = Box::leak(Box::new(node));
        Self {
            head: AtomicPtr::new(
                node_ptr
            ),
            tail: AtomicPtr::new(
                node_ptr
            )
        }
    }
    
    /// try to occupy the tail node.
    fn send(&self, key: K, val: V) -> Result<(), SendError> {
        // try to append a new node to the Channel.
        // todo: optimize the channel appending.
        let new_node = Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(Some(Msg {
                key,
                val,
                status: false 
            }))),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false)
        };
        let new_node = Box::into_raw(Box::new(new_node));

        let mut tail = unsafe { 
            &*(self.tail.load(Ordering::Acquire))
        };

        let expected: *mut Node<K, V> = ptr::null_mut();

        loop {
            // can not depend on the tail, so AcqRel.
            match tail.next.compare_exchange_weak(expected, new_node, Ordering::AcqRel, Ordering::Relaxed) {
                // append successully.
                Ok(_) => break,
                // curr is not null.
                Err(curr) => tail = unsafe { &*curr }
            }
        }

        self.tail.store(new_node, Ordering::Release);
        Ok(())
    }
    
    fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        let head = unsafe { 
            // only one thread modifies the head, so it's not important to choose a right ordering.
            &*self.head.load(Ordering::Relaxed) 
        };
        
        loop {
            if head.next.load(Ordering::Acquire) == ptr::null_mut() {
                // block.
                // todo: sleep.
                continue
            };

            let msg_node_ptr = head.next.load(Ordering::Acquire);
            let msg_node = unsafe {
                &*msg_node_ptr
            };
            
            let msg_opt = unsafe { &*msg_node.data };
            
            match msg_opt {
                Some(_) => { 
                    // todo: more functional format.
                    //
                    // let msg = unsafe {
                        // take the value out.
                    //    Box::from_raw(msg_node.data.get()).unwrap()
                    //};
                    let msg = unsafe { Box::from_raw(msg_node.data).take().unwrap() };

                    let temp_next = msg_node.next.load(Ordering::Acquire);
                    // send is fater than recv, so I think it's ok, but we confirm it later.
                    head.next.store(temp_next, Ordering::Release);
                    if temp_next == ptr::null_mut() {
                        self.tail.store(self.head.load(Ordering::Relaxed), Ordering::Release);
                    } 

                    // free the msg node.
                    let _ = unsafe { Box::from_raw(msg_node_ptr) };
                    return Ok(msg) 
                },
                None => return Err(RecvError {}),
            }
        }
    }
    
    fn recv_sync(&self) -> Result<Msg<K, V>, RecvError> {
        let head = unsafe { 
            // only one thread modifies the head, so it's not important to choose a right ordering.
            &*self.head.load(Ordering::Relaxed) 
        };
        
        loop {
            if head.next.load(Ordering::Acquire) == ptr::null_mut() {
                // block.
                // todo: sleep.
                continue
            };

            let msg_node_ptr = head.next.load(Ordering::Acquire);
            let msg_node = unsafe {
                &*msg_node_ptr
            };
            
            let msg_opt = unsafe { &*msg_node.data };
            
            match msg_opt {
                Some(_) => { 
                    // todo: more functional format.
                    //
                    // let msg = unsafe {
                        // take the value out.
                    //    Box::from_raw(msg_node.data.get()).unwrap()
                    //};
                    let msg = unsafe { Box::from_raw(msg_node.data).take().unwrap() };

                    let temp_next = msg_node.next.load(Ordering::Acquire);
                    // send is fater than recv, so I think it's ok, but we confirm it later.
                    head.next.store(temp_next, Ordering::Release);
                    if temp_next == ptr::null_mut() {
                        self.tail.store(self.head.load(Ordering::Relaxed), Ordering::Release);
                    } 

                    // free the msg node.
                    msg_node.is_destroy.store(true, Ordering::Release);
                    return Ok(msg) 
                },
                None => return Err(RecvError {}),
            }
        }
    }
    
    fn send_sync(&self, key: K, val: V) -> Result<(), SendError> {
        // naive implementation.
        let new_node = Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(Some(Msg {
                key,
                val,
                status: false 
            }))),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false)
        };
        let new_node = Box::into_raw(Box::new(new_node));

        let mut tail = unsafe { 
            &*(self.tail.load(Ordering::Acquire))
        };

        let expected: *mut Node<K, V> = ptr::null_mut();

        loop {
            // can not depend on the tail, so AcqRel.
            match tail.next.compare_exchange_weak(expected, new_node, Ordering::AcqRel, Ordering::Relaxed) {
                // append successully.
                Ok(_) => break,
                // curr is not null.
                Err(curr) => tail = unsafe { &*curr }
            }
        }

        self.tail.store(new_node, Ordering::Release);
        
        loop {
            let stored_node = unsafe {
                &*new_node
            };
            while !stored_node.is_destroy.load(Ordering::Acquire) {
                // todo: sleep
            }
            // todo: verify the correctness.
            // difference bettween sync and async version:
            // sync: sender's duty to drop the node.
            // async: receiver's duty to drop the node.
            let _  = Box::from(new_node);
            break
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Node<K, V>
where
    K: HyperKey
{
    /// Node needs to be shared across the thread boundary, so the AtomicOpt is nessessary.
    next: AtomicPtr<Node<K, V>>,
    
    /// The msg to be shared.
    data: *mut Option<Msg<K, V>>,

    /// like a lock to occupy the location.
    /// after droping the msg, we can destroy this given node.
    is_destroy: AtomicBool,
        
    /// acts as a lock.
    is_hold: AtomicBool
}

impl<K: HyperKey, V> Default for Node<K, V> {
    fn default() -> Self {
        Self {
            next: AtomicPtr::new(ptr::null_mut()),
            data: Box::into_raw(Box::new(None)),
            is_destroy: AtomicBool::new(false),
            is_hold: AtomicBool::new(false),
        }
    }
}

pub trait HyperKey {
    // todo: add macro support.
    fn collision_detect<K>(&self, k: &K) -> bool; 
}  

#[derive(Debug)]
pub struct Msg<K, T>
where 
    K: HyperKey 
{
    pub key: K,
    pub val: T,
    // true is active, else false.
    status: bool  
}


fn filter_fn() -> bool {
    todo!("try to implement a user-defined fn")
}

impl<K, V> Drop for Msg<K, V>
where
    K: HyperKey
{
    fn drop(&mut self) {
    }
}

struct Filter<K: HyperKey> {
    active_keys: Vec<K>
}

impl<K: HyperKey> Filter<K> {
    fn contains(&self, k: K) -> bool {
        self.active_keys.iter().any(|elem| k.collision_detect(elem))
    }
}



