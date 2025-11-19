use std::sync::atomic::Ordering;

use crate::signal::{
    loom::{Arc, atomic::AtomicU32}
};
use super::{
    State, Sink
};

pub struct Source<T> where T: Clone + Sync + Send + Default {
    pub (super) inner: crate::signal::Source<T>,
    pub (super) acks: Arc<AtomicU32>,
}

impl<T> Source<T> where T: Clone + Sync + Send + Default {

    pub fn from(value: T) -> Self {
        Source {
            inner: crate::signal::Source::from(value),
            acks: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }

    fn try_sync(&self) -> State {
        use State::*;
        match self.sink_count() {
            1.. => match self.acks_count() {
                1.. => Receiving,
                0   => {
                    self.reset_acks(self.sink_count());
                    Ready
                }
            },
            0   => AllGone
        }
    }

    pub fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.inner.signal) as u32 -1
    }

    pub fn send(&mut self, signal: &T) -> State {
        use State::*;
        let state = self.try_sync();
        if state == Ready {
            self.inner.send(signal);
        }
        state
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> State {
        use State::*;
        let state = self.try_sync();
        if state == Ready {
            self.inner.memory.write_in_place(closure);
            self.inner.signal.swap(&mut self.inner.memory);
        }
        state
    }

    fn reset_acks(&self, acks: u32) {
        self.acks.store(acks, Ordering::Release)
    }

    fn acks_count(&self) -> u32 {
        self.acks.load(Ordering::Acquire)
    }

    /// Check if the given data equals the last published data.
    pub fn equals_last(&self, data: &T) -> bool where T: PartialEq {
        self.inner.equals_last(data)
    }

}