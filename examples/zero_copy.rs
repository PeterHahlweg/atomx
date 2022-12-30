use atomx::signal;

#[derive(Default)]
struct Dummy { id: usize }

impl Clone for Dummy {
    fn clone(&self) -> Self {
        // panic if clone would be called at any time in the code
        panic!("clone");
    }
}

fn main() {
    let (mut source, mut sink) = signal::sync::create::<Dummy>();
    source.modify(&mut |dummy| {dummy.id = 1});

    // run the consumer
    let t1 = std::thread::spawn(move || {
        let mut stop = false;
        while ! stop {
            // this is a zero copy read, we directly accessing the underlying memory here
            sink.process(&mut |dummy|{
                if dummy.id > 1000 {
                    stop = true
                }
            });
        }
    });

    // run the producer
    let t2 = std::thread::spawn(move || {
        loop {
            // this is a zero copy write, we directly modifying the underlying memory here
            let state = source.modify(&mut |dummy| {
                if dummy.id % 100 == 0 { println!("\nsrc dummy.id: {}", dummy.id) }
                else { print!(".") }
                // - counter intuitive, dummy id needs to be increased by 2
                // - because the signal has two memory slots and write will modify in place
                // - in combination with incrementing the id in one slot beforehand this
                //   will give an id incrementing by 1 each cycle
                dummy.id += 2;
            });
            use signal::sync::State::*;
            match state {
                AllGone => break,
                Receiving => (),
                Ready => ()
            }
        }
    });

    t1.join().expect("Couldn't join on the associated thread 1");
    t2.join().expect("Couldn't join on the associated thread 2");

}
