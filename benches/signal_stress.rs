use criterion::{criterion_group, criterion_main, Criterion};
use std::thread;

fn signal_sink_read_pressure(c: &mut Criterion) {
    let (mut source, sink) = atomx::signal::create::<u64>();
    for _thread in 0..100 {
        let s = sink.clone();
        let mut x = 0;
        thread::spawn(move ||{
            for _i in 0..10_000_000 {
                s.process(&mut |value|{
                    x = *value;
                })
            }
        });
    }

    c.bench_function("atomx::signal write/read stress test", |b| b.iter(|| {
        source.modify(&mut |value| { *value += 1});
    }));
}

criterion_group!(benches, signal_sink_read_pressure);
criterion_main!(benches);