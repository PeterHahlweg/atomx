/// This is a really simple example to show how state machines can be connected and
/// run on some threads.
///
/// You can give it a try by running: cargo run --example tick
///
/// What you will see is that two threads counting up in alternating order.

use atomx::*;
use std::{sync::atomic::AtomicUsize, thread, thread::sleep};
use std::sync::atomic::Ordering::SeqCst;
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
// You can also get creative with macros (FromPremitive trait could save some work)
// or map multidimensional inputs.

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

static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

// Define what happens at which state.

fn run(machine: StateMachine, counter: CountSignal) {
    let limit = 30;
    let mut state = Tick; // the state we start from, usually Init or something similar

    let thread_id = THREAD_COUNT.load(SeqCst) as u32;
    let thread_count = 2;
    THREAD_COUNT.store(thread_id as usize +1, SeqCst);
    while thread_count != THREAD_COUNT.load(SeqCst) as u32 {sleep(Duration::from_millis(1));};

    loop {
        match machine.next(&mut state) { // this sets the next state for you, based on the transitions
            Tick => {
                if counter.probe() % thread_count != thread_id {
                    println!("T{:?}: {:?}", thread_id, counter.incr())
                }
                else {
                    state = Wait
                }

            },
            Wait => {
                if thread_id == 1 {
                    sleep(Duration::from_millis(100))
                }
                if counter.probe() >= limit {
                    state = Stop // manually change state transition, if necessary
                }
            },
            Stop => return
        }
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

    // Run the state machines on threads
    let t1 = thread::spawn(|| run(sm1, c1) );
    let t2 = thread::spawn(|| run(sm2, c2) );

    t2.join().unwrap();
    t1.join().unwrap();
}