use super::*;
use loom::Arc;

// Source
pub struct Source<T:Send> {
    signal: Arc<Signal<T>>,
    store: Option<Box<T>>,
    pub(super) is_synced: bool,
    on_received: Option<Box<dyn FnMut(&T)>>
}

// impl Source
impl<T:Send> Source<T> where T: Clone + Sync + Default {
    pub fn from(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
            is_synced: false,
            on_received: None
        }
    }

    pub fn with_sync(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            store: Some(Box::new(T::default())),
            is_synced: true,
            on_received: None
        }
    }

    pub fn send(&mut self, signal: &T) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            self.store(signal);
            self.swap();
            self.call_on_received();
        }
        state
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> SyncState {
        use SyncState::*;
        let state = self.try_sync();
        if state == Ready {
            closure(self.store.as_mut().expect("always valid data"));
            self.swap();
            self.call_on_received();
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

    pub fn on_received(&mut self, closure: impl FnMut(&T) + 'static) {
        self.on_received = Some(Box::new(closure))
    }

    fn call_on_received(&mut self) {
        if self.is_synced() {
            match &mut self.on_received {
                Some(callback) => callback(self.store.as_ref().expect("always some value")),
                None => {}
            }
        }
    }

}

#[test]
fn assume_on_received_is_not_executed_if_not_synced() {
    use crate::signal;
    // create a signal
    let (mut src, snk) = signal::create();
    let snk2 = Sink::from(&src);
    src.send(&1);
    src.on_received(|_| panic!()); // shall never be called, as signal is not synced
    snk.receive();
    snk2.receive();
    src.send(&2); // send will call on_received, if set and signal is synced
}

#[test]
fn assume_on_received_provides_expected_value() {
    use crate::signal;

    // create a synced signal
    let (mut src, snk) = signal::create().sync();
    let snk2 = Sink::from(&src);
    src.on_received(|value| assert_eq!(*value, 0)); // assert i32 default, which is 0
    // send will call on_received, if set and signal is synced
    src.send(&1); // should give 0 in on_received, which is call in send
    src.on_received(|value| assert_eq!(*value, 1)); // overwrite, and expect 1 for next call
    snk.receive();
    snk2.receive();
    src.send(&2); // should give 1 in on_received
}

