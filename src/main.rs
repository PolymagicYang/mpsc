use mpsc::{async_channel::Receiver, async_channel::Sender, Channel, HyperKey};
fn main() {
    let chan = Channel::<Test, usize>::new();
    let sender = Sender { chan: &chan };
    let receiver = Receiver { chan: &chan };

    sender.send(Test {}, 1);
    println!("{:?}", receiver.recv().unwrap());
}

#[derive(Debug, Clone)]
struct Test {}

impl HyperKey for Test {
    fn collision_detect<Test>(&self, _k: Test) -> bool {
        true
    }
}
