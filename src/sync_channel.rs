use crate::{HyperKey, Channel, Msg, RecvError, SendError};
use std::fmt::Debug;

pub struct SyncSender<'a, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    pub chan: &'a Channel<K, V>
}

pub struct SyncReceiver<'a, K, V>
where
    K: HyperKey + Send,
    V: Send
{
    pub chan: &'a Channel<K, V> 
}

impl<K, V> SyncReceiver<'_, K, V>
where
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    pub fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        self.chan.recv_sync() 
    }
}

impl<K, V> SyncSender<'_, K, V>
where 
    K: HyperKey + Send + Debug,
    V: Send + Debug
{
    /// a simpl wrapper for sending.
    pub fn send(&self, key: K, val: V) -> Result<(), SendError> {
        self.chan.send_sync(key, val)
    }
}