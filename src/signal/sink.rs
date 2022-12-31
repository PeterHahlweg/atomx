use super::{*, loom::Arc};
use haphazard::HazardPointer;


// Sink
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
    guard: HazardPointer<'static>
}


// impl Sink
impl<T> Sink<T> where T: Clone + Sync + Send + Default {

    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink { signal: source.signal(), guard: HazardPointer::new() }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&mut self) -> T {
        self.signal.value(&mut self.guard).0
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&mut self, closure: &mut dyn FnMut(&T)) {
        self.signal.modify(&mut self.guard, closure);
    }

}

impl<T> Clone for Sink<T> where T: Clone + Sync + Send + Default {
    fn clone(&self) -> Self {
        Sink { signal: self.signal.clone(), guard: HazardPointer::new() }
    }
}


