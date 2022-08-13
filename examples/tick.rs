/// This is a really simple example to show how the state machine macro can be used.
///
/// You can give it a try by running: cargo run --example tick

use atomx::*;

fn main() {
    StateMachine!( M:
        Stop, Init   -> Tick
        Wait, WakeUp -> Tick
        Tick, Done   -> Wait
        Tick, Limit  -> Stop
    );

    use MState::*;
    use MEvent::*;

    // Create some state machines from the transitions, and define the stop state for each.
    let mut machine = M::new(Stop, Stop);

    let limit = 30;
    let mut event = Init;
    let mut c = 0_i32;

    // turn this into an iterator
    loop {
        println!("{:?} {:?}", machine.state(), event);
        match machine.process(&event) { // this sets the next state for you, based on the transitions
            Tick => {
                match c >= limit {
                    true  => event = Limit,
                    false => event = Done
                }
            },
            Wait => {
                c += 1;
                event = WakeUp
            },
            Stop => break,
            _    => panic!("entered undefined state from ({:?}, {:?})", machine.state(), event)
        }
    }
}