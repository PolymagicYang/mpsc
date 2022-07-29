use mpsc::{async_channel, sync_channel};

#[derive(Debug, Clone)]
struct SimpleTest {}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect(&self, _: &SimpleTest) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct UsizeTest {
    key: usize,
}

impl mpsc::HyperKey for UsizeTest {
    fn collision_detect(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[test]
fn navive_async_test() {
    let chan = mpsc::Channel::<SimpleTest, usize>::new();
    let sender = async_channel::Sender { chan: &chan };
    let receiver = async_channel::Receiver { chan: &chan };
    let _ = (0..100000).map(|i| {
        sender.send(vec![SimpleTest {}], i);
    });
    let _ = (0..100000).map(|i| {
        assert_eq!(i, receiver.recv().unwrap().val);
    });
    let _ = (0..100000).map(|i| {
        sender.send(vec![SimpleTest {}], i);
        assert_eq!(i, receiver.recv().unwrap().val)
    });
}

#[test]
fn collision_test() {
    let chan = mpsc::Channel::<UsizeTest, usize>::new();
    let sender = async_channel::Sender { chan: &chan };
    let receiver = async_channel::Receiver { chan: &chan };

    sender.send(vec![UsizeTest { key: 1 }], 1);
    sender.send(vec![UsizeTest { key: 1 }], 3);
    sender.send(vec![UsizeTest { key: 2 }], 2);

    let msg1 = receiver.recv().unwrap();
    let msg2 = receiver.recv().unwrap();
    let msg3 = receiver.recv();

    assert_eq!(msg1.val, 1);
    assert_eq!(msg2.val, 2);
    assert!(msg3.is_err());
    drop(msg1);
    let msg4 = receiver.recv().unwrap();
    assert_eq!(msg4.val, 3);
}

#[test]
fn naive_sync_test() {
    let chan = mpsc::Channel::<SimpleTest, usize>::new();
    let _sender = sync_channel::Sender { chan: &chan };
    // Yeah, it blocks!
    // sender.send(SimpleTest {}, 1).unwrap();
}
