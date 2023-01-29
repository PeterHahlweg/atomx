pub mod sink;
pub mod source;
pub mod sync;
pub mod loom;

use haphazard::{AtomicPtr, HazardPointer, raw::Pointer};

pub use source::Source;
pub use sink::Sink;

pub fn create<T>() -> (Source<T>, Sink<T>) where T: Send + Sync + Clone + Default {
    let source = Source::from(T::default());
    let sink = Sink::from(&source);
    (source, sink)
}

struct Signal<T: Send + Default> {
    ptr: Option<AtomicPtr<T>>, // Option required to retire on drop
    slot: [T;2],
    id: AtomicU8,
    _marker: PhantomPinned
}

// impl Signal
impl<T: Clone+Default+Send+Sync> Signal<T> {
    fn new(value: T) -> Self {
        let mut signal = Signal {
            ptr: None,
            slot: [T::default(), value],
            id: AtomicU8::from(1), // defult state, writer usees slot[0], reader uses slot[1]
            _marker: PhantomPinned
        };
        let read_id = signal.id.load(Ordering::SeqCst) as usize;
        let read_ptr = &mut (signal.slot[read_id]) as *mut T;
        // Safety - ptr points to a valid memory location and this memory is pinned
        signal.ptr = Some(AtomicPtr::from(unsafe{Box::from_raw(read_ptr)}));
        signal
    }

    fn slot_ptr(&self, id: usize) -> *mut T {
        let const_ptr = &(self.slot[id]) as *const T;
        unsafe{std::mem::transmute::<*const T, *mut T>(const_ptr)}
    }

    fn write_ptr(&self, id: usize) -> *mut T {
        self.slot_ptr(id)
    }

    fn swap(&self, read_id: usize) -> usize {
        // swap slot
        let write_id = read_id^1;
        // Safty: - this is a valid mutable pointer
        //        - the caller has to make shure it is valid
        let read_ptr = unsafe{Pointer::from_raw(self.slot_ptr(read_id))};

        match &self.ptr {
            Some(ptr) => ptr.swap(read_ptr).expect("replaced box"),
            None => unreachable!(),
        };
        write_id
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
        // Safety:
        // - AtomicPtr has used the global domain, as required by haphazard::AtomicPtr::retire
        // - AtomicPtr is only used in signal
        let ptr = self.ptr.take().expect("always some AtomicPtr");
        ptr.swap(Box::<T>::default());
        unsafe{ ptr.retire() };
    }
}

use std::{fmt::Debug, marker::PhantomPinned, sync::atomic::{AtomicU8, Ordering}};
impl<T> Debug for Signal<T> where T: Send + Default {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal").field("ptr", &self.ptr).field("guard", &"invisible").finish()
    }
}


#[test]
fn read_source_value() {
    let signal = Signal::new(5);
    let mut counter = 0;
    signal.process(&mut |val|{
        counter += val
    });
    assert_eq!(counter, 5);
}

#[test]
fn signal_does_not_panic_on_immediate_drop() {
    let signal = Signal::new(false);
    drop(signal);
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