pub mod sink;
pub mod source;
pub mod sync;
pub mod loom;

use haphazard::{AtomicPtr, HazardPointer};

pub use source::Source;
pub use sink::Sink;

pub fn create<T>() -> (Source<T>, Sink<T>) where T: Send + Sync + Clone + Default {
    let source = Source::from(T::default());
    let sink = Sink::from(&source);
    (source, sink)
}

struct Signal<T: Send> {
    ptr: Option<AtomicPtr<T>>, // Option required to retire on drop
}

// impl Signal
impl<T: Clone+Default+Send+Sync> Signal<T> {
    fn new(value: T) -> Self {
        Signal {
            ptr: Some(AtomicPtr::from(Box::new(value)))
        }
    }

    fn swap(&self, value: Box<T>) -> Box<T> {
        let replaced = match &self.ptr {
            Some(ptr) => ptr.swap(value).expect("replaced box"),
            None => unreachable!(),
        };
        let replaced_ptr = replaced.into_inner().as_ptr();

        // Safety: - ptr can not be null as it is extracted from NotNull<T>
        //         - ptr is immediately consumed in this function
        //         - ptr types are matching, ensured by function signature
        //
        // Cost:   - no heap allocation happens here for data
        //         - only a "empty" Box will be allocated and takes ownership of ptr and the
        //           associated  data
        unsafe { Box::<T>::from_raw(replaced_ptr) }
    }

    fn value(&self) -> (T, u64) {
        let mut val = T::default();
        let id = self.modify(&mut |value| {
            val = value.clone()
        });
        (val, id)
    }

    fn modify(&self, closure: &mut dyn FnMut(&T)) -> u64 {
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
impl<T: Send> Drop for Signal<T> {
    fn drop(&mut self) {
        // Safety:
        // - AtomicPtr has used the global domain, as required by haphazard::AtomicPtr::retire
        // - AtomicPtr is only used in signal
        let aptr = self.ptr.take().expect("always some AtomicPtr");
        unsafe{ aptr.retire() };
    }
}

use std::fmt::Debug;
impl<T> Debug for Signal<T> where T: Send {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal").field("ptr", &self.ptr).field("guard", &"invisible").finish()
    }
}


#[test]
fn read_source_value() {
    let src = Signal::new(5);
    let mut counter = 0;
    src.modify(&mut |val|{
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
                                         size_of::<Sink<u32>>() +
                                         (size_of::<u32>() *2)
    );

}