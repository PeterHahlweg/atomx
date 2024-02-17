use std::sync::atomic::{Ordering, AtomicU64};
use crate::signal::{
    Signal,
    loom::{Arc, atomic::AtomicU32}
};
use super::source::Source;

#[derive(Default)]
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
    acks: Arc<AtomicU32>,
    last_id: AtomicU64,
}

impl<T> Sink<T>  where T: Clone + Sync + Send + Default {
    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink {
            signal: source.inner.signal(),
            acks: source.acks.clone(),
            last_id: AtomicU64::new(0),
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

    /// Check if source have changed the signal, since last acknowledge.
    pub fn changed(&self) -> bool {
        self.signal.box_id() != self.last_id.load(Ordering::Acquire)
    }

    /// When the signal is synced, this is used to inform the Sender that the Sink has been received
    /// the signal.
    fn acknowledge(&self, id: u64) {
        if self.last_id.load(Ordering::Acquire) != id {
            self.acks.fetch_sub(1, Ordering::AcqRel);
        }
        self.last_id.store(id, Ordering::Release)
    }
}

impl<T> Clone for Sink<T> where T: Clone + Sync + Send + Default {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            acks: self.acks.clone(),
            last_id: AtomicU64::default()
        }
    }
}
