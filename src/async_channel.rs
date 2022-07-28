use crate::{HyperKey, Channel, SendError, RecvError, Msg};
use std::{fmt::Debug};

pub struct Sender<'a, K, V>
where
    K: HyperKey + Send,
    V: Send
{
    pub chan: &'a Channel<K, V>
}

pub struct Receiver<'a, K, V> 
where 
    K: HyperKey + Send,
    V: Send 
{
    pub chan: &'a Channel<K, V>
}

impl<K, V> Sender<'_, K, V>
where 
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    /// a simpl wrapper for sending.
    pub fn send(&self, key: K, val: V) -> Result<(), SendError> {
        self.chan.send(key, val)
    }
}

impl<K, V> Receiver<'_, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    pub fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        self.chan.recv() 
    }
}

impl<K, V> Clone for Sender<'_, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    fn clone(&self) -> Self {
        Self { chan: self.chan.clone() }
    }
}
