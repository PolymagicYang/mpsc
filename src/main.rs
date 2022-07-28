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
    println!("{:?}", receiver.recv().unwrap());
}

#[derive(Debug)]
struct Test {}

impl HyperKey for Test {
    fn collision_detect<Test>(&self, k: &Test) -> bool {
        true
    }
}
