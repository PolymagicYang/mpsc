use crate::{Channel, HyperKey, Msg, RecvError, SendError};
use std::fmt::Debug;

pub struct Sender<'a, K, V>
where
    K: HyperKey + Send + Clone,
    V: Send,
{
    pub chan: &'a Channel<K, V>,
}

pub struct Receiver<'a, K, V>
where
    K: HyperKey + Send + Clone,
    V: Send,
{
    pub chan: &'a Channel<K, V>,
}

impl<K, V> Sender<'_, K, V>
where
    K: HyperKey + Send + Debug + Clone,
    V: Send + Debug,
{
    /// # Errors
    /// will return `SendError` if data structure panics in the Channel.
    ///
    pub fn send(&self, keys: Vec<K>, val: V) {
        self.chan.send(keys, val);
    }
}

impl<K, V> Receiver<'_, K, V>
where
    K: HyperKey + Send + Debug + Clone,
    V: Send + Debug,
{
    /// # Errors
    /// will return `RecvError` if inner data structure panics.
    pub fn recv(&self) -> Result<Msg<K, V>, RecvError> {
        self.chan.recv()
    }
}

impl<K, V> Clone for Sender<'_, K, V>
where
    K: HyperKey + Send + Debug + Clone,
    V: Send + Debug,
{
    fn clone(&self) -> Self {
        Self {
            chan: <&Channel<K, V>>::clone(&self.chan),
        }
    }
}
