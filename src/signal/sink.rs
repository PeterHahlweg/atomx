use super::{*, loom::Arc};

// Sink
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
}


// impl Sink
impl<T> Sink<T> where T: Clone + Sync + Send + Default {

    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink { signal: source.signal() }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&self) -> T {
        self.signal.value().0
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        self.signal.process(closure);
    }

}

impl<T> Clone for Sink<T> where T: Clone + Sync + Send + Default {
    fn clone(&self) -> Self {
        Sink { signal: self.signal.clone() }
    }
}


