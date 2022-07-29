use mpsc::{async_channel, sync_channel};
use std::thread;

#[derive(Debug)]
struct SimpleTest {}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect<Test>(&self, _: &Test) -> bool {
        true
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

            thread::spawn(move || sender.send(SimpleTest {}, i))
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
                sender.send(SimpleTest {}, i);
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
fn naive_sync_test() {
    let chan = Box::leak(Box::new(mpsc::Channel::<SimpleTest, usize>::new()));
    let sender = sync_channel::Sender { chan };
    let receiver = sync_channel::Receiver { chan };
    // simple test:
    thread::spawn(move || sender.send(SimpleTest {}, 1));
    assert_eq!(1, receiver.recv().unwrap().val);
}
