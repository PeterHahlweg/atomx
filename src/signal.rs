use std::{ops::Deref, sync::Arc};
use std::{sync::atomic::{ AtomicBool, Ordering::*, AtomicU32 }};

pub trait SignalAccess<T> {
    fn probe(&self) -> T;
    fn emit(&self, val: T);
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
        self.state.load(SeqCst)
    }

    pub fn is_false(&self) -> bool {
        ! self.state.load(SeqCst)
    }
}

impl SignalAccess<bool> for BooleanSignal {
    fn probe(&self) -> bool {
        self.state.load(SeqCst)
    }

    fn emit(&self, val: bool) {
        self.state.store(val, SeqCst);
    }
}

#[derive(Debug)]
pub struct RawSignalU32 {
    pub (crate) state: AtomicU32
}

impl Eq for RawSignalU32 {}

impl PartialEq for RawSignalU32 {
    fn eq(&self, other: &Self) -> bool {
        self.probe() == other.probe()
    }
}

impl Default for RawSignalU32 {
    fn default() -> Self {
        RawSignalU32 {
            state: AtomicU32::new(0)
        }
    }
}

impl RawSignalU32 {
    pub const fn new(val: u32) -> Self {
        RawSignalU32 {
            state: AtomicU32::new(val)
        }
    }

    pub fn incr(&self) -> u32 {
        self.state.fetch_add(1, SeqCst) +1
    }

    pub fn decr(&self) -> u32 {
        self.state.fetch_sub(1, SeqCst) -1
    }
}

impl SignalAccess<u32> for RawSignalU32 {

    fn probe(&self) -> u32 {
        self.state.load(SeqCst)
    }

    fn emit(&self, val: u32) {
        self.state.store(val, SeqCst);
    }

}

#[derive(Clone, Debug)]
pub struct SignalU32 { arc: Arc<RawSignalU32> }

impl SignalU32 {
    fn new(value: u32) -> Self {
        SignalU32 {
            arc: Arc::new(RawSignalU32::new(value))
        }
    }
}

impl Default for SignalU32 {
    fn default() -> Self {
        SignalU32 {
            arc: Arc::new(RawSignalU32::default())
        }
    }
}

impl Deref for SignalU32 {
    type Target = RawSignalU32;

    fn deref(&self) -> &Self::Target {
        &*self.arc
    }
}