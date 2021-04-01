/// This is a really simple example to show how state machines can be connected and
/// run on some threads.
///
/// You can give it a try by running: cargo run --example tick
///
/// What you will see is that two threads counting up in alternating order.

use atomx::*;
use std::thread;
use std::time::Duration;
use TickState::*;

type CountSignal = SignalU32;

// First define your states.

#[derive(Clone, Copy, Debug)]
enum TickState {
    Tick, Wait, Stop
}

// Provide conversions from state to u32 and vice versa.
// This is important as the states are represented as u32 types internally.
// You can also be creative here with macros and/or mapping multidimensional inputs.

impl From<TickState> for u32 {
    fn from(ts: TickState) -> Self {
        ts as u32
    }
}

impl From<u32> for TickState {
    fn from(x: u32) -> Self {
        match x {
            0 => Tick,
            1 => Wait,
            _ => Stop,
        }
    }
}

// Define what happens at which state.

fn run(machine: StateMachine, counter: CountSignal) {
    let limit = 30;
    let mut state = Tick; // the state we start from, usually Init or something similar

    loop {
        match machine.next(&mut state) { // this sets the next state for you, based on the transitions
            Tick => {
                let c = counter.incr()+1;
                println!("{:?}: tick, {:?}", std::thread::current().id(), c);
            },
            Wait => {
                thread::sleep(Duration::from_secs(1));
                if counter.probe() >= limit {
                    state = Stop // manually change state transition, if necessary
                }
            },
            Stop => return
        };
    }
}


fn main() {
    // Transitions can be described as easy as this and always happens from the left to the right value.
    let transitions = [
        (Tick, Wait),
        (Wait, Tick),
    ];

    // Create some state machines from the transitions, and define the stop state for each.
    let mut sm1 = StateMachine::from(&transitions, Stop);
    let mut sm2 = StateMachine::from(&transitions, Stop);

    // This is the counter we share between the threads.
    let c1 = CountSignal::default();
    let c2 = c1.clone();

    // The connection of two state machines is not complicated, but not easily readable at the moment.
    // So, how to read this thing?
    sm1.connect(Tick, // this is the sm1 state that depends on the other state machines (sm2) state
        &sm2, // the other state machine
        Wait, // the state we depend on
        Wait  // the state we go next if the other state machine is not in the state we depend on
              //    (this gives us the flexibility to go ahead with something else, or try again)
    );
    sm2.connect(Tick, &sm1, Wait, Wait);

    // Put the state machines on threads
    let t1 = thread::spawn(|| run(sm1, c1) );
    let t2 = thread::spawn(|| run(sm2, c2) );

    t2.join().unwrap();
    t1.join().unwrap();
}