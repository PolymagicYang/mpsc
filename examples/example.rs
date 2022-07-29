use mpsc::{async_channel, sync_channel, Channel, HyperKey};
use std::thread;

// You must implement HyperKey for your own type to use mpsc.
#[derive(Clone, Debug)]
struct SimplKey {
    key: u32,
}

impl HyperKey for SimplKey {
    fn collision_detect(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

fn async_example() {
    let chan = Box::leak(Box::new(mpsc::Channel::<SimplKey, usize>::new()));
    let sender = async_channel::Sender { chan };
    let receiver = async_channel::Receiver { chan };

    let sender = sender.clone();
    let _handle = thread::spawn(move || sender.send(vec![SimplKey { key: 2 }], 1));

    assert_eq!(receiver.recv().unwrap().val, 1);
}

fn sync_example() {
    let chan = Box::leak(Box::new(mpsc::Channel::<SimplKey, usize>::new()));
    let sender = sync_channel::Sender { chan };
    let receiver = sync_channel::Receiver { chan };

    let sender = sender.clone();
    let _handle = thread::spawn(move || sender.send(vec![SimplKey { key: 2 }], 1));

    assert_eq!(receiver.recv().unwrap().val, 1);
}

fn main() {
    async_example();
    sync_example();
}
