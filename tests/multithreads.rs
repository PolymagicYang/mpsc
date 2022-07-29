use mpsc::{async_channel, sync_channel};
use std::collections::HashSet;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, Clone)]
struct SimpleTest {}

#[derive(Debug, Clone)]
struct UsizeTest {
    key: usize,
}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect(&self, _: &SimpleTest) -> bool {
        true
    }
}

impl mpsc::HyperKey for UsizeTest {
    fn collision_detect(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[test]
fn naive_async_test() {
    // ugly implementation.
    let chan = Box::leak(Box::new(mpsc::Channel::<SimpleTest, usize>::new()));
    let sender = async_channel::Sender { chan };
    let receiver = async_channel::Receiver { chan };

    let _: Vec<_> = (0..1000)
        .map(|i| {
            // sequential test.
            let sender = sender.clone();

            thread::spawn(move || sender.send(vec![SimpleTest {}], i))
                .join()
                .unwrap()
        })
        .collect();

    let _: Vec<_> = (0..1000)
        .map(|i| {
            assert_eq!(i, receiver.recv().unwrap().val);
        })
        .collect();

    let out_of_order = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let _: Vec<_> = (0..1000)
        .map(|i| {
            let out_of_order = out_of_order.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                let mut vec = out_of_order.lock().unwrap();
                sender.send(vec![SimpleTest {}], i);
                vec.push(i);
            })
        })
        .collect();

    for i in 0..1000 {
        let vec = out_of_order.lock().unwrap();
        if i >= vec.len() {
            // ugly commands.
            break;
        }
        assert_eq!(vec[i], receiver.recv().unwrap().val);
    }
}

#[test]
fn async_simple_key_test() {
    // key send order: 2 2 3 1 3 6 7 8
    // receive order should be: 2 -> 3 -> 1 -> 6 -> 7 -> 8
    // then drops 2
    // receive should be: 2
    let chan = Box::leak(Box::new(mpsc::Channel::<UsizeTest, usize>::new()));
    let sender = async_channel::Sender { chan };
    let receiver = async_channel::Receiver { chan };

    let test_cases = vec![2, 2, 3, 1, 3, 6, 7, 8];
    for elem in test_cases {
        let sender = sender.clone();
        let _handle = thread::spawn(move || {
            sender.send(vec![UsizeTest { key: elem }], elem);
        })
        .join()
        .unwrap();
    }

    let msg1 = receiver.recv().unwrap();
    let msg2 = receiver.recv().unwrap();
    let msg3 = receiver.recv().unwrap();
    let msg4 = receiver.recv().unwrap();
    let msg5 = receiver.recv().unwrap();
    let msg6 = receiver.recv().unwrap();

    assert_eq!(2, msg1.val);
    assert_eq!(3, msg2.val);
    assert_eq!(1, msg3.val);
    assert_eq!(6, msg4.val);
    assert_eq!(7, msg5.val);
    assert_eq!(8, msg6.val);

    drop(msg1);
    assert_eq!(2, receiver.recv().unwrap().val);
    drop(msg2);
    assert_eq!(3, receiver.recv().unwrap().val);
}

#[test]
fn fifo_test() {
    let chan = Box::leak(Box::new(mpsc::Channel::<UsizeTest, usize>::new()));
    let sender = async_channel::Sender { chan };
    let receiver = async_channel::Receiver { chan };

    let order = Arc::new(Mutex::new(vec![]));
    let test_cases = vec![2, 2, 3, 1, 3, 6, 7, 8];
    for elem in test_cases {
        let sender = sender.clone();
        let order = order.clone();
        let _handle = thread::spawn(move || {
            sender.send(vec![UsizeTest { key: elem }], elem);
            order.lock().unwrap().push(elem);
        });
    }

    let mut results = vec![];
    for _ in 0..6 {
        results.push(receiver.recv().unwrap());
    }

    let mut hash_set = HashSet::new();
    for i in 0..6 {
        hash_set.insert(results[i].val);
    }
    // make sure no collision.
    assert_eq!(hash_set.len(), results.len());
}

#[test]
fn naive_sync_test() {
    let chan = Box::leak(Box::new(mpsc::Channel::<SimpleTest, usize>::new()));
    let sender = sync_channel::Sender { chan };
    let receiver = sync_channel::Receiver { chan };
    // simple test:
    thread::spawn(move || sender.send(vec![SimpleTest {}], 1));
    assert_eq!(1, receiver.recv().unwrap().val);
}
