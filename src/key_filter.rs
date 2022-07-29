use std::{
    sync::{self, Arc, Mutex},
    thread,
};

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

    pub(crate) fn pop(&self, k: &[K]) {
        let mut guard = self.active_keys.lock().unwrap();
        let _indexes: Vec<_> = k
            .iter()
            .map(|key| {
                let index = guard
                    .iter()
                    .position(|elem| elem.collision_detect(key))
                    .unwrap();
                guard.remove(index);
            })
            .collect();
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

#[derive(Clone)]
struct SimpleKey {
    key: usize,
}

impl HyperKey for SimpleKey {
    fn collision_detect(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[test]
fn single_test() {
    let filter = Filter::default();
    let mut keys = vec![];
    for i in 1..=100 {
        keys.push(SimpleKey { key: i as usize });
    }
    filter.put(&keys);
    for i in 1..=100 {
        assert!(filter.contains(&vec![SimpleKey { key: i as usize }]));
    }
}

#[test]
fn multikeys_test() {
    let filter = Filter::default();
    let mut keys = vec![];
    for i in 1..=100 {
        keys.push(SimpleKey { key: i as usize });
    }
    filter.put(&keys);

    for i in 1..=100 {
        let mut temp_keys = vec![];
        for j in 1..=i {
            temp_keys.push(SimpleKey { key: j as usize });
        }
        assert!(filter.contains(&temp_keys));
    }
}

#[test]
fn pop_test() {
    let filter = Filter::default();
    let mut keys = vec![];
    for i in 1..=100 {
        keys.push(SimpleKey { key: i as usize });
    }
    filter.put(&keys);

    // pop one.
    for i in 1..=100 {
        filter.pop(&vec![SimpleKey { key: i as usize }]);
        assert!(!filter.contains(&vec![SimpleKey { key: i as usize }]));
    }

    // pop many.
    filter.put(&keys);
    for i in (1..=100).step_by(10) {
        let mut temp = vec![];
        for j in i..i + 10 {
            temp.push(SimpleKey { key: j as usize });
        }
        filter.pop(&temp);
        assert!(!filter.contains(&temp), "failed to pop many");
    }
}

#[test]
fn multithreads_test() {
    let filter = Filter::default();

    let mut joins = vec![];
    for i in 1..=10 {
        let filter = filter.clone();
        joins.push(std::thread::spawn(move || {
            filter.put(&vec![SimpleKey { key: i as usize }])
        }));
    }
    let _join: Vec<_> = joins.into_iter().map(|handler| handler.join()).collect();

    for i in 1..=10 {
        assert!(filter.contains(&vec![SimpleKey { key: i as usize }]));
    }
}
