/// This is a really simple example to show how state machines can be connected and
/// run on some threads.
///
/// You can give it a try by running: cargo run --example tick
///
/// What you will see is that two threads counting up in alternating order.


// TODO: there still exists a bug, not yet in the order that was the goal


use atomx::*;
use atomx::SignalU32 as Signal;
use std::{thread, thread::sleep};
use std::time::Duration;
use smallvec::SmallVec;

StateMachine!( SM1:
    Stop,  Init   => Tick
    Wait,  WakeUp => Tick
    Pending, WakeUp => Tick
    Tick,  Done   => Wait
    Tick,  Limit  => Stop
);

StateMachine!( SM2:
    Stop,  Init   => Tock
    Wait,  WakeUp => Tock
    Pending, WakeUp => Tock
    Tock,  Done   => Wait
    Tock,  Limit  => Stop
);

// Define what happens at which state.

fn run1(mut machine: SM1, counter: Signal, tid: u32) {
    use SM1State::*;
    use SM1Event::*;
    let limit = 30;
    let mut event = Init;
    let mut c;

    // turn this into an iterator
    loop {
        // println!("th{:?}: {:?} {:?}", tid, machine.state(), event);
        match machine.next(&event) { // this sets the next state for you, based on the transitions
            Tick => {
                c = counter.incr() -1;
                match c >= limit {
                    true  => event = Limit,
                    false => event = Done
                }
                println!("th{:?}: {:?} counter {:?}", tid, event, c);
            },
            Wait => {
                sleep(Duration::from_millis(100));
                event = WakeUp
            },
            Pending => {
                sleep(Duration::from_millis(10));
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
            Tock => {
                c = counter.incr() -1;
                match c >= limit {
                    true  => event = Limit,
                    false => event = Done
                }
                println!("th{:?}: {:?} counter {:?}", tid, event, c);
            },
            Wait => {
                sleep(Duration::from_millis(100));
                event = WakeUp
            },
            Pending => {
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
    let counter1 = Signal::default();
    let counter2 = counter1.clone();

    // Connect two state machines
    // - the state machines are constant
    // - but at runtime it is allowed to add dependency's to their transitions
    // TODO: here one can abuse the transition, not good
    sm1.transition(SM1State::Wait, SM1Event::WakeUp)
       .depend_on(&sm2, SM2State::Wait)
       .next_if_pending(SM1State::Pending);

    sm2.transition(SM2State::Wait, SM2Event::WakeUp)
       .depend_on(&sm1, SM1State::Wait)
       .next_if_pending(SM2State::Pending);

    // Often we want a handshake. In a way that makes the connection reliable.
    // The Problem is, a machine can go into and out of a state without the notice
    // of other threads (maybe they are sleeping aat this moment).

    // Run the state machines on threads
    let t1 = thread::spawn(|| run1(sm1, counter1, 0) );
    let t2 = thread::spawn(|| run2(sm2, counter2, 1) );

    t2.join().unwrap();
    t1.join().unwrap();
}