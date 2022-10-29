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
pub struct Sink<T: Clone + Sync + Send + Default> {
    signal: Arc<Signal<T>>,
    last: Cell<u64>,
    is_synced: bool
}

// impl Sink
impl<T> Sink<T> where T: Clone + Sync + Send + Default {
    pub fn from(source: &Source<T>) -> Self {
        Sink {
            signal: source.signal(),
            last: Cell::new(0),
            is_synced: source.is_synced(),
        }
    }

    pub fn receive(&self) -> T {
        let (value, id) = self.signal.value();
        if self.is_synced { self.acknowledge(id) }
        value
    }

    pub fn process(&self, closure: &mut dyn FnMut(&T)) {
        let id = self.signal.modify(closure);
        if self.is_synced { self.acknowledge(id) }
    }

    fn acknowledge(&self, id: u64) {
        if self.last.get() != id {
            self.signal.decrement_acks()
        }
        self.last.set(id)
    }

    pub fn changed(&self) -> bool {
        (! self.is_synced) || (self.signal.id() != self.last.get())
    }

    pub fn is_synced(&self) -> bool {
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
fn changed_if_not_synced() {
    use crate::signal;
    // create a synced signal
    let (mut src, snk) = signal::create_synced::<f32>();
    src.send(&0.0);
    snk.receive();
    assert!( ! snk.changed()); // because received latest and sync is enabled
}