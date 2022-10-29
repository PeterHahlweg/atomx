#[cfg(loom)]
pub use loom::{thread, sync::*};

#[cfg(not(loom))]
pub use std::{thread, sync::*};