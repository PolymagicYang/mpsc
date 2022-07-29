use std::sync::{Arc, Mutex};

use crate::HyperKey;

#[derive(Debug)]
pub(crate) struct Filter<K: HyperKey + Clone> {
    active_keys: Arc<Mutex<Vec<K>>>,
}

impl<K: HyperKey + Clone> Filter<K> {
    pub(crate) fn contains(&self, k: &Vec<K>) -> bool {
        let res: Vec<bool> = (0..k.len())
            .map(|index| {
                self.active_keys
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|elem| k[index].collision_detect(elem))
            })
            .collect();

        res.iter().any(|elem| *elem)
    }

    pub(crate) fn put(&self, k: &[K]) {
        self.active_keys.lock().unwrap().append(&mut Vec::from(k));
    }

    pub(crate) fn pop(&self, k: &K) {
        let mut guard = self.active_keys.lock().unwrap();
        let index = guard
            .iter()
            .position(|elem| elem.collision_detect(k))
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

#[test]
fn filter_test() {
    #[derive(Clone)]
    struct SimpleKey {
        key: usize,
    }

    impl HyperKey for SimpleKey {
        fn collision_detect(&self, other: &Self) -> bool {
            if self.key == other.key {
                true
            } else {
                false
            }
        }
    }

    let filter = Filter::default();
    let k1 = SimpleKey { key: 1 };
    filter.put(&vec![k1]);
}
