use std::sync::Arc;
use std::{sync::atomic::{ AtomicBool, Ordering::*, AtomicU32 }};


pub trait Signal<T> {
    fn state(&self) -> T;
    fn into_arc(self) -> Arc<Self>;
}

pub trait SignalEmitter<T> {
    fn set(&self, val: T);
}

// Boolean Signal

#[derive(Debug, Default)]
pub struct BooleanSignal {
    state: AtomicBool,
}

impl BooleanSignal {
    pub fn new(val: bool) -> Self {
        BooleanSignal {
            state: AtomicBool::new(val)
        }
    }

    pub fn is_true(&self) -> bool {
        true == self.state.load(Acquire)
    }

    pub fn is_false(&self) -> bool {
        false == self.state.load(Acquire)
    }
}

impl Signal<bool> for BooleanSignal {

    fn state(&self) -> bool {
        self.state.load(Acquire)
    }

    fn into_arc(self) -> Arc<BooleanSignal> {
        Arc::new(self)
    }
}

impl SignalEmitter<bool> for BooleanSignal {
    fn set(&self, val: bool) {
        self.state.store(val, Release);
    }

}

// State Signal
pub type StateSignal = U32Signal;
pub type CountSignal = U32Signal;

#[derive(Debug)]
pub struct U32Signal {
    pub (crate) state: AtomicU32
}

impl Eq for U32Signal {}

impl PartialEq for U32Signal {
    fn eq(&self, other: &Self) -> bool {
        self.state() == other.state()
    }
}

impl Default for U32Signal {
    
    fn default() -> Self {
        U32Signal {
            state: AtomicU32::new(0) 
        }
    }
}

impl U32Signal {
    pub fn new(val: u32) -> Self {
        U32Signal {
            state: AtomicU32::new(val)
        }
    }

    pub fn incr(&self) -> u32 {
        self.state.fetch_add(1, SeqCst)
    }

    pub fn decr(&self) -> u32 {
        self.state.fetch_sub(1, SeqCst)
    }
}

impl Signal<u32> for U32Signal {

    fn state(&self) -> u32 {
        self.state.load(Acquire)
    }

    fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl SignalEmitter<u32> for U32Signal {
    fn set(&self, val: u32) {
        self.state.store(val, Release);
    }

}
