pub mod sink;
pub mod source;
pub mod sync;
pub mod loom;
pub mod memory;
pub use source::Source;
pub use sink::Sink;

use memory::*;
use haphazard::{AtomicPtr, HazardPointer};
use std::{fmt::Debug, pin::Pin};


pub fn create<T>() -> (Source<T>, Sink<T>) where T: Send + Sync + Clone + Default {
    let source = Source::from(T::default());
    let sink = Sink::from(&source);
    (source, sink)
}

#[derive(Default)]
struct Signal<T: Send + Default> {
    ptr: Option<AtomicPtr<T>>, // Option required to retire on drop
}

// impl Signal
impl<T: Clone+Default+Send+Sync> Signal<T> {
    fn new(pointer: AtomicPtr<T>) -> Self {
        Signal {
            ptr: Some(pointer)
        }
    }

    fn swap(&self, memory: &mut Pin<Box<Memory<T>>>) {
        if let Some(ptr) = &self.ptr {
            // No retire needed here - we're just toggling between pre-allocated slots
            memory.swap(ptr)
        }
    }

    fn value(&self) -> (T, u64) {
        let mut val = T::default();
        let id = self.process(&mut |value| {
            val = value.clone()
        });
        (val, id)
    }

    fn process(&self, closure: &mut dyn FnMut(&T)) -> u64 {
        match &self.ptr {
            Some(ptr) => {
                let mut guard = HazardPointer::new();
                let val = ptr.safe_load(&mut guard).expect("not null");
                closure(val);
                ptr.load_ptr() as u64
            }
            None => unreachable!()
        }
    }

    fn box_id(&self) -> u64 {
        match &self.ptr {
            Some(ptr) => ptr.load_ptr() as u64,
            None => unreachable!(),
        }
    }

}

// drop Signal
impl<T: Send + Default> Drop for Signal<T> {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr.take() {
            ptr.swap(Box::<T>::default());
            // Safety:
            // - AtomicPtr has used the global domain, as required by haphazard::AtomicPtr::retire
            // - AtomicPtr is only used in signal
            unsafe{ ptr.retire() };
        }
    }
}

impl<T> Debug for Signal<T> where T: Send + Default {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal").field("ptr", &self.ptr).field("guard", &"invisible").finish()
    }
}


#[test]
fn read_source_value() {
    let mut memory = Memory::new(5);
    let signal = Signal::new(memory.new_read_ptr());
    let mut counter = 0;
    signal.process(&mut |val|{
        counter += val
    });
    assert_eq!(counter, 5);
}

#[test]
fn signal_does_not_panic_on_immediate_drop() {
    let mut memory = Memory::new(false);
    let signal = Signal::new(memory.new_read_ptr());
    drop(signal);
}

#[test]
fn sink_is_connected_when_source_exists() {
    let (source, sink) = super::signal::create::<bool>();
    assert!(sink.is_connected(), "Sink should be connected when source exists");
    drop(source);
}

#[test]
fn sink_is_not_connected_after_source_dropped() {
    let sink = {
        let (source, sink) = super::signal::create::<bool>();
        assert!(sink.is_connected(), "Sink should be connected when source exists");
        drop(source);
        sink
    };
    assert!(!sink.is_connected(), "Sink should not be connected after source is dropped");
}

#[test]
fn sink_and_source_does_not_panic_on_immediate_drop() {
    let (source, sink) = super::signal::create::<bool>();
    drop(source);
    drop(sink);
}

#[test]
fn source_and_sinks_are_connected() {
    let (mut source, sink1) = super::signal::create::<bool>();
    let sink2 = source.sink();
    for i in 0..10 {
        let v = i%2 == 1;
        source.send(&v);
        assert_eq!(sink1.receive(), v);
        assert_eq!(sink2.receive(), v);
    }
}

#[test]
#[ignore = "only show sizes"]
fn sizes() {
    use super::signal::{Signal, Source, Sink};
    use std::mem::size_of;
    println!("size_of signal");
    println!("Sink<u32>:         {:3}B", size_of::<Sink<u32>>());
    println!("Source<u32>:       {:3}B", size_of::<Source<u32>>());
    println!("Signal<u32>:       {:3}B", size_of::<Signal<u32>>());
    println!("signal cost:       {:3}B", size_of::<Signal<u32>>() +
                                         size_of::<Source<u32>>() +
                                         size_of::<Sink<u32>>()
    );

}