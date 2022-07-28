use mpsc;

#[derive(Debug)]
struct SimpleTest {}

impl mpsc::HyperKey for SimpleTest {
    fn collision_detect<Test>(&self, _: &Test) -> bool {
        true
    }
}

#[test]
fn navive_test() {
    let chan = mpsc::Channel::<SimpleTest, usize>::new();
    let sender = mpsc::Sender {
        chan: &chan
    };
    let receiver = mpsc::Receiver {
        chan: &chan 
    };
    for i in 0..100000 {
        sender.send(SimpleTest {}, i).unwrap();
    }
    for i in 0..100000 {
        assert_eq!(i, receiver.recv().unwrap().val);
    }
    for i in 0..100000 {
        sender.send(SimpleTest {}, i).unwrap();
        assert_eq!(i, receiver.recv().unwrap().val)
    }
}