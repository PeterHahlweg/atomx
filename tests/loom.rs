#![cfg(loom)]

use atomx::signal;
use atomx::signal::sync::State;
use loom::thread;

#[test]
fn loom_signal_ack(){
    loom::model(|| {
        let (mut sender, sink1) = signal::sync::create::<f32>();
        let sink2 = sender.sink();
        let sink3 = sender.sink();
        let input1 = 0.1;
        let input2 = 0.2;
        sender.send(&input1);
        assert_eq!(State::Receiving, sender.send(&input2));

        // clock generator module
        let t1 = thread::spawn( move ||{
            assert_eq!(input1, sink1.receive());
        });

        // clock consumer module
        let t2 = thread::spawn( move || {
            assert_eq!(input1, sink2.receive());
        });

        t1.join().expect("completion");
        t2.join().expect("completion");
        assert_eq!(State::Receiving, sender.send(&input2));

        sink3.receive();
        assert_eq!(State::Ready, sender.send(&input2));

        drop(sink3);
        assert_eq!(State::AllGone, sender.send(&input2));
    });
}

