use atomx::signal;
#[derive(Clone)]
struct Dummy { data: [u64;16], checksum: u64 }

impl Dummy {
    fn gen_checksum(&self) -> u64 {
        let mut checksum = 0;
        for x in self.data {
            checksum = (checksum << 1) ^ x
        };
        checksum
    }

    pub fn default_a() -> Self {
        let mut dummy = Dummy { data: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15], checksum: 0};
        dummy.checksum = dummy.gen_checksum();
        dummy
    }

    pub fn default_b() -> Self {
        let mut dummy = Dummy { data: [16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1], checksum: 0 };
        dummy.checksum = dummy.gen_checksum();
        dummy
    }

    pub fn default_c() -> Self {
        let mut dummy = Dummy { data: [5,12,16,8,2,13,11,1,7,4,14,15,6,9,10,3], checksum: 0 };
        dummy.checksum = dummy.gen_checksum();
        dummy
    }
}


impl Default for Dummy {
    fn default() -> Self {
        Self::default_a()
    }
}

impl Dummy {
    pub fn verify(&self) {
        let last_value = self.data[self.data.len()-1];
        let checksum0 = self.gen_checksum();
        let checksum1 = self.checksum;
        if checksum0 != checksum1{
            println!("{:?} fail, last_value:{}", std::thread::current().id(), last_value);
            assert!(checksum0 != checksum1);
        }
    }
}

#[test]
fn data_integrity_on_concurrent_access_sync_signal() {
    let (mut source, sink) = signal::sync::create::<Dummy>();
    let mut handles = vec![];

    // run the consumer
    for _thread in 0..8 {
        let sink = sink.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..1000000 {
                sink.process(&mut |dummy|{
                    dummy.verify();
                });
            }
        }));
    }

    // run the producer
    let t2 = std::thread::spawn(move || {
        let mut idx = 0;
        loop {
            // this is a zero copy write, we directly modifying the underlying memory here
            let state = source.modify(&mut |dummy| {
                *dummy = match idx%3 {
                    2 => Dummy::default_a(),
                    1 => Dummy::default_b(),
                    _ => Dummy::default_c(),
                };
                for x in &mut dummy.data {
                    *x += 1;
                }
                dummy.checksum = dummy.gen_checksum();
            });
            use signal::sync::State::*;
            match state {
                AllGone => break,
                Receiving => {},//unreachable!("not a sync::signal"),
                Ready => {},
            }

            idx += 1;
        }
    });

    drop(sink);
    for handle in handles {
        handle.join().expect("Couldn't join on the associated thread");
    }
    t2.join().expect("Couldn't join on the associated thread 2");

}


#[test]
fn data_integrity_on_concurrent_access_signal() {
    let (mut source, sink) = signal::create::<Dummy>();
    let mut handles = vec![];

    // run the consumer
    for _thread in 0..8 {
        let sink = sink.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..1000000 {
                if i == 0 {continue}
                sink.process(&mut |dummy|{
                    dummy.verify();
                });
            }
        }));
    }

    // run the producer
    let t2 = std::thread::spawn(move || {
        let mut idx = 0;
        loop {
            // this is a zero copy write, we directly modifying the underlying memory here
            let state = source.modify(&mut |dummy| {
                *dummy = match idx%3 {
                    2 => Dummy::default_a(),
                    1 => Dummy::default_b(),
                    _ => Dummy::default_c(),
                };
            });

            use signal::sync::State::*;
            match state {
                AllGone => break,
                Receiving => {},
                Ready => {},
            }
            idx += 1;
        }
    });

    drop(sink);
    for handle in handles {
        handle.join().expect("Couldn't join on the associated thread");
    }
    t2.join().expect("Couldn't join on the associated thread 2");

}
