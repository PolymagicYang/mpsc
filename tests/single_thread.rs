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
    let _ = (0..100000).map(|i| {
        sender.send(SimpleTest {}, i).unwrap();
    });
    let _ = (0..100000).map(|i| {
        assert_eq!(i, receiver.recv().unwrap().val);
    });
    let _ = (0..100000).map(|i| {
        sender.send(SimpleTest {}, i).unwrap();
        assert_eq!(i, receiver.recv().unwrap().val)
    });
}