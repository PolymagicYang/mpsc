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

    let _: Vec<_> = (0..1000).map(|i| {
        // sequential test.
        let sender = sender.clone();

        thread::spawn(move || {
            sender.send(SimpleTest {}, i)
        })
        .join()
        .unwrap()
    }).collect();
    
    let _: Vec<_> = (0..1000).map(|i| {
        assert_eq!(i, receiver.recv().unwrap().val);
    }).collect();
    
    let out_of_order = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let _: Vec<_> = (0..1000).map(|i| {
        let out_of_order = out_of_order.clone();
        let sender = sender.clone();
        thread::spawn(move || {
            out_of_order.lock().unwrap().push(i);
            sender.send(SimpleTest {}, i).unwrap();
        })
    }).collect();

    for i in 0..1000 {
        let vec = out_of_order.lock().unwrap();
        assert_eq!(vec[i], receiver.recv().unwrap().val)
    }
}