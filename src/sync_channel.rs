use crate::{HyperKey, Channel, Msg, RecvError, SendError};
use std::fmt::Debug;

pub struct Sender<'a, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
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

impl<K, V> Receiver<'_, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    /// # Errors
    /// will return `SendError` if data structure panics in the Channel.
    ///
    pub fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        self.chan.recv_sync() 
    }
}

impl<K, V> Sender<'_, K, V>
where 
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    /// # Errors
    /// will return `SendError` if inner structures panics.
    /// 
    pub fn send(&self, key: K, val: V) -> Result<(), SendError> {
        self.chan.send_sync(key, val)
    }
}

impl<K, V> Clone for Sender<'_, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    fn clone(&self) -> Self {
        Self { chan: <&Channel<K, V>>::clone(&self.chan) }
    }
}