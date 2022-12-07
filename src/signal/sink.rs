use super::*;
use loom::Arc;
use std::cell::Cell;

#[derive(Debug, PartialEq, Eq)]
pub enum SinkState {
    Consuming,
    Finished,
    Ready
}

// Sink
#[derive(Clone, Debug)]
pub struct Sink<T> where T: Clone + Sync + Send + Default {
    signal: Arc<Signal<T>>,
    last_id: Cell<u64>,

    /// This is a regular bool, as the sync behavior can only be defined at object creation,
    /// such that multiple signals can exist with different sync behavior.
    is_synced: bool
}

pub trait SinkSync {
    /// In case the receiver needs to know if the synced signal has changed but without reading the
    /// value, this function will read the memory id and compare it to the latest one. If they
    /// differ, a change has happened. This will not guaranty that the value actually changed,
    /// but the sender send a new signal which may be different.
    ///
    /// In case the signal is not synced, this function will return always true.
    fn changed(&self) -> bool;

    /// Indicates if the signal is synchronized.
    fn is_synced(&self) -> bool;
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
            is_synced: source.is_synced(),
        }
    }

    /// Returns a copy of the received signal value.
    /// This is especially useful for small or primitive types. If the signal data is to expansive
    /// to copy have a look at [process].
    pub fn receive(&self) -> T {
        let (value, id) = self.signal.value();
        if self.is_synced() { self.acknowledge(id) }
        value
    }

    /// In contrast to [receive] this function allows the consumer to directly access the data via
    /// an immutable reference given by a closure. This could drastically reduce memory usage, but
    /// creates back pressure onto the sender if processing takes to much time (even if not
    /// synced).
    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        let id = self.signal.modify(closure);
        if self.is_synced() { self.acknowledge(id) }
    }

    /// When the signal is synced, this is used to inform the Sender that the Sink has been received
    /// the signal.
    fn acknowledge(&self, id: u64) {
        if self.last_id.get() != id {
            self.signal.acknowledge()
        }
        self.last_id.set(id)
    }

}

impl<T> SinkSync for Sink<T>  where T: Clone + Sync + Send + Default {

    fn changed(&self) -> bool {
        (! self.is_synced) || (self.signal.id() != self.last_id.get())
    }

    fn is_synced(&self) -> bool {
        self.is_synced
    }
}

#[test]
fn changed_is_true_on_create() {
    use crate::signal;
    // create a not synced signal
    let (_, snk) = signal::create::<f32>();
    assert!(snk.changed()); // because all data is new
}
#[test]
fn changed_is_true_if_create_synced() {
    use crate::signal;
    // create a not synced signal
    let (_, snk) = signal::create_synced::<f32>();
    assert!(snk.changed()); // because all data is new
}

#[test]
fn changed_is_always_true_if_not_synced() {
    use crate::signal;
    // create a not synced signal
    let (mut src, snk) = signal::create::<f32>();
    src.send(&0.0);
    snk.receive();
    assert!(snk.changed()); // because sync is disabled

}

#[test]
fn changed_if_synced() {
    use crate::signal;
    // create a synced signal
    let (mut src, snk) = signal::create_synced::<f32>();
    src.send(&0.0);
    snk.receive();
    assert!( ! snk.changed()); // because received latest and sync is enabled
}