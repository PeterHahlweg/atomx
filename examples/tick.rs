/// This is a really simple example to show how state machines can be connected and
/// run on some threads.
///
/// You can give it a try by running: cargo run --example tick
///
/// What you will see is that two threads counting up in alternating order.

use atomx::*;
use atomx::SignalU32 as CountSignal;
use std::{thread, thread::sleep};
use std::time::Duration;

// TODO:   - maybe it is good if every SM has its own states and events, not shared once
//         - could be combined with transition only description

states!(Tick, Wait, Stop);
events!(Init, Even, Odd, Limit);

transitions!( SM1:
    Stop, Init  => Wait
    Tick, Even  => Wait
    Wait, Odd   => Tick
    Wait, Limit => Stop
);

transitions!( SM2:
    Stop, Init  => Tick
    Tick, Even  => Wait
    Wait, Odd   => Tick
    Wait, Limit => Stop
);

// Define what happens at which state.

fn run(mut machine: StateMachine<State,Event>, counter: CountSignal, tid: u32) {
    let limit = 30;
    let threads = 2;
    let mut event = Init;

    loop {
        println!("th{:?}: {:?} {:?}", tid, machine.state(), event);
        match /* state = */ machine.next(&event) { // this sets the next state for you, based on the transitions
            Tick => {
                let c = counter.probe();
                if c >= limit {
                    event = Limit
                }
                else if c % threads == tid {
                    event = Even;
                    counter.incr();
                }
                else {
                    event = Even
                }
            },
            Wait => sleep(Duration::from_millis(100)),
            _    => panic!("th{} entered undefined state from ({:?}, {:?})", tid, machine.state(), event)
        }
    }
}


fn main() {
    // Create some state machines from the transitions, and define the stop state for each.
    // let mut sm1 = StateMachine::new(&SM1, Stop, Stop);
    let mut sm2 = StateMachine::new(&SM2, Stop, Stop);

    // This is the counter we share between the threads.
    let c1 = CountSignal::default();
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

    // Run the state machines on threads
    // let t1 = thread::spawn(|| run(sm1, c1, 0) );
    let t2 = thread::spawn(|| run(sm2, c2, 1) );

    t2.join().unwrap();
    // t1.join().unwrap();
}