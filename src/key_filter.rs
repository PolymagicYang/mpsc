use std::sync::{Arc, Mutex};

use crate::HyperKey;

#[derive(Debug)]
pub(crate) struct Filter<K: HyperKey + Clone> {
    active_keys: Arc<Mutex<Vec<K>>>,
}

impl<K: HyperKey + Clone> Filter<K> {
    fn contains(&self, k: &K) -> bool {
        self.active_keys
            .lock()
            .unwrap()
            .iter()
            .any(|elem| k.collision_detect(elem))
    }

    fn put(&self, k: &K) {
        self.active_keys.lock().unwrap().push(k.clone());
    }

    fn pop(&self, k: &K) {
        let mut guard = self.active_keys.lock().unwrap();
        let index = guard
            .iter()
            .position(|elem| elem.collision_detect(k.clone()))
            .unwrap();
        guard.remove(index);
    }
}

impl<K> Default for Filter<K>
where
    K: HyperKey + Clone,
{
    fn default() -> Self {
        Self {
            active_keys: std::sync::Arc::default(),
        }
    }
}

impl<K> Clone for Filter<K>
where
    K: HyperKey + Clone,
{
    fn clone(&self) -> Self {
        Self {
            active_keys: self.active_keys.clone(),
        }
    }
}
