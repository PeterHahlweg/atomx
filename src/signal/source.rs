use super::*;
use loom::Arc;

// Source
pub struct Source<T:Send> {
    signal: Arc<Signal<T>>,
    store: Option<Box<T>>,
    is_synced: bool
}

// impl Source
impl<T:Send> Source<T> where T: Clone + Sync + Default {
    pub fn from(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
            is_synced: false
        }
    }

    pub fn with_sync(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
            is_synced: true
        }
    }

    pub fn send(&mut self, signal: &T) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            self.store(signal);
            self.swap()
        }
        state
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            closure(self.store.as_mut().expect("always valid data"));
            self.swap()
        }
        state
    }

    fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.signal) as u32 -1
    }

    pub fn is_synced(&self) -> bool {
        self.is_synced
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
    fn swap(&mut self) {
        // - it is safe to unwrap here as long as we take the value out of store
        //   and afterwards put some new value in
        // - as long as at no other point the value of store is taken
        let new = self.store.take().unwrap();
        self.store = Some(self.signal.swap(new));
    }

    fn try_sync(&self) -> SyncState {
        use SyncState::*;
        let mut state = Ready;
        if self.is_synced() {
            state = match self.sink_count() {
                1.. => match self.signal.acks_count() {
                    1.. => Receiving,
                    0   => {
                        self.signal.reset_acks(self.sink_count());
                        Ready
                    }
                },
                0   => AllGone
            };
        }
        state
    }

}