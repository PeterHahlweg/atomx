use crate::signal::sync::State;
use super::{*, loom::Arc, memory::Memory};

use std::pin::Pin;

// Source
pub struct Source<T:Send + Default> {
    pub(super) signal: Arc<Signal<T>>,
    pub(super) memory: Pin<Box<Memory<T>>>
}

impl<T:Send> Source<T> where T: Clone + Sync + Default {
    /// Create a new source from a given value.
    pub fn from(value: T) -> Self {
        let mut memory: Pin<Box<Memory<T>>> =  Memory::new(value);
        let pointer = memory.new_read_ptr();
        Source {
            signal: Arc::new(Signal::new(pointer)),
            memory
        }
    }

    /// Modify the current data and publish the changes to the sinks. The data will be cloned once.
    pub fn send(&mut self, data: &T) -> State {
        self.memory.write(data);
        self.signal.swap(&mut self.memory);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    /// Modify the current data with zero copy and publish the changes to the sinks.
    pub fn modify(&mut self, closure: &mut dyn FnMut(&mut T)) -> State {
        self.memory.write_in_place(closure);
        self.signal.swap(&mut self.memory);
        match self.sink_count() {
            0 => State::AllGone,
            _ => State::Ready
        }
    }

    /// Access the current data without publishing the change to the sinks.
    pub fn access(&mut self, closure: &mut dyn FnMut(&mut T)) {
        self.memory.write_in_place(closure)
    }

    pub(super) fn signal(&self) -> Arc<Signal<T>> {
        self.signal.clone()
    }

    /// Returns the number of current sinks connected to the source.
    pub fn sink_count(&self) -> u32 {
        // the expectation here is, that this count does not change often
        Arc::strong_count(&self.signal) as u32 -1
    }

    /// Return a new sink form source.
    pub fn sink(&self) -> Sink<T> {
        Sink::from(self)
    }

    /// Check if the given data equals the last published data.
    pub fn equals_last(&self, data: &T) -> bool where T: PartialEq {
        self.memory.equals_current(data)
    }
}


