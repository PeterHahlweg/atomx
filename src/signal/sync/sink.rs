use std::{
    cell::Cell,
    sync::atomic::Ordering
};
use crate::signal::{
    Signal,
    loom::{Arc, atomic::AtomicU32}
};
use super::source::Source;


#[derive(Clone)]
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
    acks: Arc<AtomicU32>,
    last_id: Cell<u64>,
}

impl<T> Sink<T>  where T: Clone + Sync + Send + Default {
    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink {
            signal: source.inner.signal(),
            acks: source.acks.clone(),
            last_id: Cell::new(0),
        }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&self) -> T {
        let (value, id) = self.signal.value();
        self.acknowledge(id);
        value
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        let id = self.signal.process(closure);
        self.acknowledge(id)
    }

    pub fn changed(&self) -> bool {
        self.signal.box_id() != self.last_id.get()
    }

    /// When the signal is synced, this is used to inform the Sender that the Sink has been received
    /// the signal.
    fn acknowledge(&self, id: u64) {
        if self.last_id.get() != id {
            self.acks.fetch_sub(1, Ordering::SeqCst);
        }
        self.last_id.set(id)
    }
}