use crate::signal::sync::State;

use super::{*, loom::Arc};

// Source
pub struct Source<T:Send + Default> {
    pub(super) signal: Arc<Signal<T>>,
    pub(super) id: usize,
}

// impl Source
impl<T:Send> Source<T> where T: Clone + Sync + Default {
    pub fn from(value: T) -> Self {
        Source {
            signal: Arc::new(Signal::new(value)),
            id: 0
        }
    }

    pub fn send(&mut self, signal: &T) -> State {
        self.store(signal);
        self.id = self.signal.swap(self.id);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> State {
        let data = unsafe{self.signal.write_ptr(self.id).as_mut().expect("always valid ptr")};
        closure(data);
        self.id = self.signal.swap(self.id);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    pub(super) fn signal(&self) -> Arc<Signal<T>> {
        self.signal.clone()
    }

    fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.signal) as u32 -1
    }

    // Update the value of store, without allocating memory. The given data will be cloned once.
    fn store(&mut self, value: &T) {
        let data = unsafe{self.signal.write_ptr(self.id).as_mut().expect("always valid ptr")};
        *data = value.clone();
    }

    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }

}


