use super::signal::*;
use std::sync::Arc;

/* TODO:
 *      - integrate new state struct
 *      - create connect function
 *
 */

 struct State {
    value: u32,
    signal: Option<Arc<StateSignal>>,
    condition: u32, // compare this to signal, to decide if go next or stay at state
 }

pub struct StateMachine
{
    signal: StateSignal,
    states: Vec<(u32, Option<Arc<StateSignal>>)>,
    stop: u32,
}

pub struct Unstoppable(StateMachine);
impl Unstoppable {
    pub fn stops_at<S>(mut self, state: S) -> StateMachine where S: Clone + Into<u32> {
        self.0.stop = state.into();
        self.0
    }
}

impl StateMachine
{
    pub fn new() -> Self {
        StateMachine {
            signal: StateSignal::default(),
            states: vec![],
            stop: 0,
        }
    }

    pub fn from<S>(map: &[(S, S)]) -> Unstoppable where S: Clone + Into<u32> {
        let mut sm = StateMachine::new();
        sm.states.resize(map.len(), State::default());

        for value in map {
            let state: u32 = value.0.clone().into();
            let next: u32  = value.1.clone().into();
            let max = std::cmp::max(state, next);
            if sm.states.len() < max as usize {
                sm.states.resize((max+1) as usize, (0,None));
            }
            sm.states[state as usize] = (next, Some(Arc::new(StateSignal::new(0))));
        }
        Unstoppable(sm)
    }

    pub fn state(&self) -> u32 {
        self.signal.state()
    }

    pub fn next(&self, state: u32) -> u32 {
        if (state as usize) < self.states.len() {
            let next = &self.states[state as usize];
            match &next.1 {
                Some(_signal) => {
                    // check here at which condition we go next or stay at state
                    // this depends on a signal that is driven by another state machine
                    self.stop // workaround
                }
                None => {
                    self.signal.set(next.0);
                    next.0
                }
            }
        }
        else {
            self.stop
        }
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

}

impl Default for StateMachine {
    fn default() -> Self {
        StateMachine::new()
    }
}

mod unittest {

    use super::StateMachine;

    #[test]
    fn from() {
        let map: [(u32,u32); 5] = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let sm = StateMachine::from(&map).stops_at(0u32);

        for val in &map {
            let next = sm.next(val.0);
            println!("next: {}, val.0: {}, .1: {}", next, val.0, val.1);
            assert_eq!(val.1, next)
        }
    }

    #[test]
    fn state_out_of_bound() {
        let map: [(u32,u32); 5] = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let stop: u32 = 22;
        let sm = StateMachine::from(&map).stops_at(stop);
        assert_eq!(sm.next(99), stop);
    }
}
