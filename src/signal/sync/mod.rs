pub mod sink;
pub mod source;

pub use source::Source;
pub use sink::Sink;

use crate::signal::loom::{Arc, atomic::AtomicU32};

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    AllGone,
    Receiving,
    Ready
}

/// Create a pair of source and sink, which are performing a handshake.
/// This handshake guaranties, that the source will not update the value until all sinks have
/// seen the value.
/// This is probably in most cases a special use case and increases the overhead of the
/// communication.
/// In a case where the source can not send, the control is given back to the user.
pub fn create<T>() -> (Source<T>, Sink<T>) where T: Send + Sync + Clone + Default {
    let source = Source {
        inner: crate::signal::Source::from(T::default()),
        acks: Arc::new(AtomicU32::new(0)),
    };
    let sink = Sink::from(&source);
    (source, sink)
}

#[test]
fn changed_is_true_if_create_synced() {
    let (_, snk) = crate::signal::sync::create::<f32>();
    assert!(snk.changed()); // because all data is new
}


#[test]
fn changed_if_synced() {
    let (mut src, snk) = crate::signal::sync::create::<f32>();
    src.send(&0.0);
    snk.receive();
    assert!( ! snk.changed()); // because received latest value already
}

#[test]
fn sync_sink_is_connected_when_source_exists() {
    let (source, sink) = crate::signal::sync::create::<bool>();
    assert!(sink.is_connected(), "Sync sink should be connected when source exists");
    drop(source);
}

#[test]
fn sync_sink_is_not_connected_after_source_dropped() {
    let sink = {
        let (source, sink) = crate::signal::sync::create::<bool>();
        assert!(sink.is_connected(), "Sync sink should be connected when source exists");
        drop(source);
        sink
    };
    assert!(!sink.is_connected(), "Sync sink should not be connected after source is dropped");
}

#[test]
#[ignore = "only show sizes"]
fn sizes() {
    use crate::signal::sync::{Source, Sink};
    use crate::signal::Signal;
    use std::mem::size_of;
    println!("size_of synced signal");
    println!("Sink<u32>:         {:3}B", size_of::<Sink<u32>>());
    println!("Source<u32>:       {:3}B", size_of::<Source<u32>>());
    println!("Signal<u32>:       {:3}B", size_of::<Signal<u32>>());
    println!("signal cost:       {:3}B", size_of::<Signal<u32>>() +
                                         size_of::<Source<u32>>() +
                                         size_of::<Sink<u32>>() +
                                         (size_of::<u32>() *2)
    );
}