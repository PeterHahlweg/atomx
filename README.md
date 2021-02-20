[![Crates.io](https://img.shields.io/crates/v/atomx)](https://crates.io/crates/atomx)
[![docs.rs](https://docs.rs/atomx/badge.svg)](https://docs.rs/atomx/)

# Atomx

A collection of concurrent data structures, build on top of atomic types.
__This is experimental and not ready for production.__

The main reason why this crate exists is the state machine type. It provides the ability to create interconnected state machines which can run on different threads. There is no guaranty that this approach is efficient, but it provides a mechanism to synchronize threads without a lot of memory overhead. State machines are also well known, easy to understand, and reliable.
This state machine implementation is based on the signal types, the base building blocks for this crate. They wrap atomic types to be practical and hiding some complexity.

## Example - Connecting State Machines
```Rust
use atomx::*;

// define some states
#[derive(Clone, Copy, Debug)]
enum TickState {Tick, Wait, Stop}

// Provide conversions from state to u32 and vice versa.
// This is important as the states are represented as u32 types internally.
// You can also be creative here with macros and/or mapping multidimensional inputs.
impl From<TickState> for u32 { ... }
impl From<u32> for TickState { ... }

// Transitions can be described as easy as this and always happens from the left to the right value.
let transitions = [(Tick, Wait), (Wait, Tick)];

// Create some state machines from the transitions, and define the stop state for each.
let mut sm1 = StateMachine::from(&transitions).stops_at(Stop);
let mut sm2 = StateMachine::from(&transitions).stops_at(Stop);

// The connection of two state machines is not complicated, but not easily readable at the moment.
// So, how to read this thing?
sm1.connect(Tick, // this is the sm1 state that depends on the other state machines (sm2) state
            &sm2, // the other state machine
            Wait, // the state we depend on
            Wait  // the state we go next if the other state machine is not in the state we depend on
                  //    (this gives us the flexibility to go ahead with something else, or try again)
);

// Please find the complete source for this in the example folder.
```

## License
Atomx is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
See [LICENSE-APACHE](LICENSE-APACHE-2.0) and [LICENSE-MIT](LICENSE-MIT) for details.

## Contribution
Opening a pull request is assumed to signal agreement with these licensing terms.