#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]

use std::cell::UnsafeCell;
use std::sync::atomic::AtomicPtr;
use std::ptr;

/// Channel acts as a linked-list to hold the msg.
pub(crate) struct Channel<K: HyperKey, V> {
    head: AtomicPtr<Node<K, V>>,
    tail: AtomicPtr<Node<K, V>>,
}

impl<K, V> Channel<K, V>
where 
    K: HyperKey 
{
    /// Just like create a Linked-List.
    /// Init state:
    /// Channel head ---    tail ---
    ///                |           |
    ///                |           |
    ///               Node  ->   Node
    pub(crate) fn new() -> Channel<K, V> {
        let tail_node = Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data: UnsafeCell::new(None),
        };
        let tail_ptr = Box::leak(Box::new(tail_node));
        let head_node = Node {
            next: AtomicPtr::new(tail_ptr),
            data: UnsafeCell::new(None),
        };
        Self {
            head: AtomicPtr::new(
                Box::leak(Box::new(head_node))
            ),
            tail: AtomicPtr::new(
                tail_ptr
            )
        }
    }
}

struct Node<K, V>
where
    K: HyperKey 
{
    /// Node needs to be shared across the thread boundary, so the AtomicOpt is nessessary.
    next: AtomicPtr<Node<K, V>>,
    
    /// The msg to be shared.
    data: UnsafeCell<Option<Msg<K, V>>>
}

trait HyperKey {
    // todo: add macro support.
    fn collision_detect(&self) -> bool; 
}  

struct Msg<K, T>
where 
    K: HyperKey 
{
    key: K,
    val: T,
    status: bool  
}

pub struct Sender {
     
}

pub struct Receiver {
    
}


