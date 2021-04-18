/// This is a really simple example to show how state machines can be connected and
/// run on some threads.
///
/// You can give it a try by running: cargo run --example tick
///
/// What you will see is that two threads counting up in alternating order.

use atomx::*;
use atomx::SignalU32 as Signal;
use std::{thread, thread::sleep};
use std::time::Duration;
use smallvec::SmallVec;

StateMachine!( SM1:
    Stop,  Init   => Sleep
    Sleep, WakeUp => Tick
    Tick,  Even   => Sleep
    Tick,  Odd    => Sleep
    Tick,  Limit  => Stop
);

StateMachine!( SM2:
    Stop,  Init   => Sleep
    Sleep, WakeUp => Tick
    Tick,  Even   => Sleep
    Tick,  Odd    => Sleep
    Tick,  Limit  => Stop
);

// Define what happens at which state.

fn run1(mut machine: SM1, counter: Signal, tid: u32) {
    use SM1State::*;
    use SM1Event::*;
    let limit = 30;
    let mut event = Init;
    let mut c;
    loop {
        // println!("th{:?}: {:?} {:?}", tid, machine.state(), event);
        match machine.next(&event) { // this sets the next state for you, based on the transitions
            Tick => {
                c = counter.probe();
                if c >= limit {
                    event = Limit
                }
                else if c % 2 == 0  {
                    event = Even;
                    println!("th{:?}: {:?} counter {:?}", tid, event, c);
                    counter.incr();
                }
                else {
                    event = Odd
                }
            },
            Sleep => {
                sleep(Duration::from_millis(100));
                event = WakeUp
            },
            Stop => break,
            _    => panic!("th{} entered undefined
             state from ({:?}, {:?})", tid, machine.state(), event)
        }
    }
}

fn run2(mut machine: SM2, counter: Signal, tid: u32) {
    use SM2State::*;
    use SM2Event::*;
    let limit = 30;
    let mut event = Init;
    let mut c;
    loop {
        // println!("th{:?}: {:?} {:?}", tid, machine.state(), event);
        match machine.next(&event) { // this sets the next state for you, based on the transitions
            Tick => {
                c = counter.probe();
                if c >= limit {
                    event = Limit
                }
                else if c % 2 == 1 {
                    event = Odd;
                    println!("th{:?}:  {:?} counter {:?}", tid, event, c);
                    counter.incr();
                }
                else {
                    event = Even
                }
            },
            Sleep => {
                sleep(Duration::from_millis(100));
                event = WakeUp
            },
            Stop => break,
            _    => panic!("th{} entered undefined
             state from ({:?}, {:?})", tid, machine.state(), event)
        }
    }
}

fn main() {
    // Create some state machines from the transitions, and define the stop state for each.
    let mut sm1 = SM1::new(SM1State::Stop, SM1State::Stop);
    let mut sm2 = SM2::new(SM2State::Stop, SM2State::Stop);

    // This is the counter we share between the threads.
    let c1 = Signal::default();
    let c2 = c1.clone();

    // The connection of two state machines is not complicated, but not easily readable at the moment.
    // So, how to read this thing?
    // sm1.connect(Tick, // this is the sm1 state that depends on the other state machines (sm2) state
    //     &sm2, // the other state machine
    //     Wait, // the state we depend on
    //     Wait  // the state we go next if the other state machine is not in the state we depend on
    //           //    (this gives us the flexibility to go ahead with something else, or try again)
    // );
    // sm2.connect(Tick, &sm1, Wait, Wait);

    // Example for better API
    // sm1.connect(state_a).with(sm2, state_b).if_pending(detour);
    //                                        .loop_if_pending();

    // Run the state machines on threads
    let t1 = thread::spawn(|| run1(sm1, c1, 0) );
    let t2 = thread::spawn(|| run2(sm2, c2, 1) );

    t2.join().unwrap();
    t1.join().unwrap();
}