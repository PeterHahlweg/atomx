use crate::source;
use crate::sink;
use crate::loom::Arc;
use crate::loom::atomic::{AtomicU32, Ordering};


#[derive(Debug, PartialEq, Eq)]
pub enum SyncState {
    AllGone,
    Receiving,
    Ready
}

pub struct Sink<T> where T: Clone + Sync + Send + Default {
    inner: sink::Sink<T>,
    acks: Arc<AtomicU32>,
}

impl<T> Sink<T>  where T: Clone + Sync + Send + Default {
    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink {
            inner: sink::Sink::from(&source.inner),
            acks: source.acks.clone()
        }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&self) -> T {
        let (value, id) = self.inner.signal.value();
        self.acknowledge(id);
        value
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        let id = self.inner.signal.modify(closure);
        self.acknowledge(id)
    }

    /// When the signal is synced, this is used to inform the Sender that the Sink has been received
    /// the signal.
    fn acknowledge(&self, id: u64) {
        if self.inner.last_id.get() != id {
            self.acks.fetch_sub(1, Ordering::SeqCst);
        }
        self.inner.last_id.set(id)
    }

    pub fn changed(&self) -> bool {
        self.inner.signal.box_id() != self.inner.last_id.get()
    }
}


// TODO: fix wrapper type and inner type of source, moved sync related functions
pub struct Source<T> where T: Clone + Sync + Send + Default {
    inner: source::Source<T>,
    acks: Arc<AtomicU32>,
    on_received: Option<Box<dyn FnMut(&T)>>,
}

impl<T> Source<T> where T: Clone + Sync + Send + Default {
    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }


    pub fn on_received(&mut self, closure: impl FnMut(&T) + 'static) {
        self.on_received = Some(Box::new(closure))
    }

    fn call_on_received(&mut self) {
        match &mut self.on_received {
            Some(callback) => callback(self.inner.store.as_ref().expect("always some value")),
            None => {}
        }
    }

    fn try_sync(&self) -> SyncState {
        use SyncState::*;
        match self.sink_count() {
            1.. => match self.acks_count() {
                1.. => Receiving,
                0   => {
                    self.reset_acks(self.sink_count());
                    Ready
                }
            },
            0   => AllGone
        }
    }

    fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.inner.signal) as u32 -1
    }

    pub fn send(&mut self, signal: &T) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            self.inner.send(signal);
            self.call_on_received();
        }
        state
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            closure(self.inner.store.as_mut().expect("always valid data"));
            self.inner.swap();
            self.call_on_received();
        }
        state
    }

    fn reset_acks(&self, acks: u32) {
        self.acks.store(acks, Ordering::SeqCst)
    }

    fn acks_count(&self) -> u32 {
        self.acks.load(Ordering::SeqCst)
    }

}

pub mod signal {
    use crate::source;
    use super::{Source, Sink};
    use crate::loom::Arc;
    use crate::loom::atomic::AtomicU32;

    /// Create a pair of source and sink, which are performing a handshake.
    /// This handshake guaranties, that the source will not update the value until all sinks have
    /// seen the value.
    /// This is probably in most cases a special use case and increases the overhead of the
    /// communication.
    /// In a case where the source can not send, the control is given back to the user.
    pub fn create<T>() -> (Source<T>, Sink<T>) where T: Send + Sync + Clone + Default {
        let source = Source{
            inner: source::Source::with_sync(T::default()),
            acks: Arc::new(AtomicU32::new(0)),
            on_received: None
        };
        let sink = Sink::from(&source);
        (source, sink)
    }
}

#[test]
fn assume_on_received_provides_expected_value() {
    use crate::synced;

    // create a synced signal
    let (mut src, snk) = synced::signal::create();
    let snk2 = src.sink();
    snk2.changed();
    src.on_received(|value| assert!(0.eq(value))); // assert i32 default, which is 0
    // send will call on_received, if set and signal is synced
    src.send(&1); // should give 0 in on_received, which is call in send
    src.on_received(|value| assert!(1.eq(value))); // overwrite, and expect 1 for next call
    snk.receive();
    snk2.receive();
    src.send(&2); // should give 1 in on_received
}

