use mpsc::{Channel, Sender, HyperKey, Receiver}; 
fn main() {
    let chan = Channel::<Test, usize>::new();
    let sender = Sender {
        chan: &chan
    };
    let receiver = Receiver {
        chan: &chan 
    };
    
    sender.send(Test {}, 1).unwrap();
    sender.send(Test {}, 2).unwrap();
    sender.send(Test {}, 3).unwrap();
    sender.send(Test {}, 4).unwrap();
    println!("{:?}", receiver.recv().unwrap());
    println!("{:?}", receiver.recv().unwrap());
    println!("{:?}", receiver.recv().unwrap());
}

#[derive(Debug)]
struct Test {}

impl HyperKey for Test {
    fn collision_detect<Test>(&self, k: &Test) -> bool {
        true
    }
}
