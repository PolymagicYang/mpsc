use mpsc::{async_channel, sync_channel};

#[derive(Debug)]
struct SimpleTest {}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect<Test>(&self, _: &Test) -> bool {
        true
    }
}

#[test]
fn navive_async_test() {
    let chan = mpsc::Channel::<SimpleTest, usize>::new();
    let sender = async_channel::Sender {
        chan: &chan
    };
    let receiver = async_channel::Receiver {
        chan: &chan 
    };
    let _ = (0..100000).map(|i| {
        sender.send(SimpleTest {}, i);
    });
    let _ = (0..100000).map(|i| {
        assert_eq!(i, receiver.recv().unwrap().val);
    });
    let _ = (0..100000).map(|i| {
        sender.send(SimpleTest {}, i);
        assert_eq!(i, receiver.recv().unwrap().val)
    });
}

#[test]
fn naive_sync_test() {
    let chan = mpsc::Channel::<SimpleTest, usize>::new();
    let _sender = sync_channel::Sender {
        chan: &chan
    };
    // Yeah, it blocks!
    // sender.send(SimpleTest {}, 1).unwrap();
}