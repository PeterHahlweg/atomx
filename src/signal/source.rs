use crate::synced::SyncState;

use super::*;
use loom::Arc;

// Source
pub struct Source<T:Send> {
    pub(super) signal: Arc<Signal<T>>,
    pub(super) store: Option<Box<T>>,
}

// impl Source
impl<T:Send> Source<T> where T: Clone + Sync + Default {
    pub fn from(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
        }
    }

    pub fn with_sync(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
        }
    }

    pub fn send(&mut self, signal: &T) -> SyncState {
        self.store(signal);
        self.swap();
        SyncState::Ready
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> SyncState {
        closure(self.store.as_mut().expect("always valid data"));
        self.swap();
        SyncState::Ready
    }

    pub(super) fn signal(&self) -> Arc<Signal<T>> {
        self.signal.clone()
    }

    // Update the value of store, without allocating memory. The given data will be cloned once.
    fn store(&mut self, value: &T) {
        match &mut self.store {
            Some(store) => { // store new value on heap, without reallocation
                **store = value.clone(); // this clone is required, to overwrite the last value
            },
            None => unreachable!()
        }
    }

    /// Swaps the signal value with the value in store.
    pub(super) fn swap(&mut self) {
        // - it is safe to unwrap here as long as we take the value out of store
        //   and afterwards put some new value in
        // - as long as at no other point the value of store is taken
        let new = self.store.take().unwrap();
        self.store = Some(self.signal.swap(new));
    }

    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }

}


