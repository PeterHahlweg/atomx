use super::*;
use loom::Arc;
use std::cell::Cell;

// Sink
#[derive(Clone, Debug)]
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    pub(super) signal: Arc<Signal<T>>,
    pub(super) last_id: Cell<u64>,
}


// impl Sink
impl<T> Sink<T> where T: Clone + Sync + Send + Default {

    /// Creates a new Sink from the given Source.
    /// This is the only way to create a Sink. That's necessary to guaranty that both share the
    /// same sync property.
    pub fn from(source: &Source<T>) -> Self {
        Sink {
            signal: source.signal(),
            last_id: Cell::new(0),
        }
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
        self.signal.modify(closure);
    }

}


#[test]
fn changed_is_true_if_create_synced() {
    // create a synced signal
    let (_, snk) = synced::signal::create::<f32>();
    assert!(snk.changed()); // because all data is new
}


#[test]
fn changed_if_synced() {
    // create a synced signal
    let (mut src, snk) = synced::signal::create::<f32>();
    src.send(&0.0);
    snk.receive();
    assert!( ! snk.changed()); // because received latest and sync is enabled
}