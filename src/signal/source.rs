use crate::signal::sync::State;

use super::{*, loom::Arc, memory::Memory};

// Source
pub struct Source<T:Send + Default> {
    pub(super) signal: Arc<Signal<T>>,
    pub(super) memory: Memory<T>
}

// impl Source
impl<T:Send> Source<T> where T: Clone + Sync + Default {
    pub fn from(value: T) -> Self {
        let mut memory = Memory::new(value);
        Source {
            signal: Arc::new(Signal::new(&mut memory)),
            memory
        }
    }

    pub fn send(&mut self, data: &T) -> State {
        self.memory.write(data);
        self.signal.swap(&mut self.memory);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> State {
        self.memory.write_in_place(closure);
        self.signal.swap(&mut self.memory);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    pub(super) fn signal(&self) -> Arc<Signal<T>> {
        self.signal.clone()
    }

    pub fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.signal) as u32 -1
    }

    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }

}


