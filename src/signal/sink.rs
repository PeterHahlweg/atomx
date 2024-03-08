use super::{*, loom::{Arc, atomic::AtomicU64}};
use std::sync::atomic::Ordering;

// Sink
#[derive(Default)]
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
    last_id: AtomicU64
}


// impl Sink
impl<T> Sink<T> where T: Clone + Sync + Send + Default {

    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink { signal: source.signal(), last_id: AtomicU64::from(0) }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&self) -> T {
        self.last_id.store(self.signal.box_id(), Ordering::Release);
        self.signal.value().0
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        self.last_id.store(self.signal.box_id(), Ordering::Release);
        self.signal.process(closure);
    }

    /// Check if sink is connected.
    pub fn is_connected(&self) -> bool {
        Arc::strong_count(&self.signal) > 0
    }

    /// Check if source has changed the signal, since last read.
    pub fn changed(&self) -> bool {
       self.signal.box_id() != self.last_id.load(Ordering::Acquire)
    }

}

impl<T> Clone for Sink<T> where T: Clone + Sync + Send + Default {
    fn clone(&self) -> Self {
        Sink { signal: self.signal.clone(), last_id: AtomicU64::default()}
    }
}


