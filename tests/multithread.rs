use mpsc;
use std::thread;

#[derive(Debug)]
struct SimpleTest {}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect<Test>(&self, _: &Test) -> bool {
        true
    }
}

#[test]
fn navive_test() {
    // ugly implementation.
    let chan = Box::leak(Box::new(mpsc::Channel::<SimpleTest, usize>::new()));
    let sender = mpsc::Sender {
        chan
    };
    let receiver = mpsc::Receiver {
        chan 
    };

    let _: Vec<_> = (0..100).map(|i| {
        let sender = sender.clone();
        thread::spawn(move || {
            sender.send(SimpleTest {}, i)
        })
        .join()
        .unwrap()
    }).collect();
        
    let _: Vec<_> = (0..100).map(|i| {
        assert_eq!(i, receiver.recv().unwrap().val);
    }).collect();
 
    for i in 0..100 {
        sender.send(SimpleTest {}, i).unwrap();
        assert_eq!(i, receiver.recv().unwrap().val)
    }
}