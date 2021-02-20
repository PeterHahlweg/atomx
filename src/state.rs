use super::signal::*;
use std::sync::Arc;

/* TODO:
 *      - improve connect api
 */

 #[derive(Clone, Debug)]
 struct State {
    value: u32,
    signal: Option<Arc<StateSignal>>,
    condition: u32, // compare this to signal, to decide if go next or stay at state
    redirect: u32, // next state if the signal is not equal to condition
 }

 impl Default for State {
    fn default() -> Self {
        State {
            value: 0,
            signal: None,
            condition: 0,
            redirect: 0, // redirect is not the correct word, for the next state if the condition fails
        }
    }
 }

#[derive(Clone)]
pub struct StateMachine
{
    signal: Arc<StateSignal>,
    states: Vec<State>,
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
            signal: Arc::new(StateSignal::default()),
            states: vec![],
            stop: 0,
        }
    }

    pub fn from<S>(map: &[(S, S)]) -> Unstoppable
    where S: Clone + Into<u32> {
        let mut sm = StateMachine::new();
        sm.states.resize(map.len(), State::default());

        for value in map {
            let state: u32 = value.0.clone().into();
            let next: u32  = value.1.clone().into();
            let max = std::cmp::max(state, next);
            if sm.states.len() < max as usize {
                sm.states.resize((max+1) as usize, State::default());
            }
            sm.states[state as usize] = State{value: next, signal:None, condition: 0, redirect: 0};
        }
        Unstoppable(sm)
    }

    pub fn state(&self) -> u32 {
        self.signal.state()
    }

    pub fn next<S>(&self, state: &mut S) -> S
    where S: Into<u32> + From<u32> + Clone
    {
        let idx = state.clone().into() as usize;
        let next = if idx < self.states.len() {
            let next = &self.states[idx];
            match &next.signal {
                Some(signal) => {
                    if signal.state() == next.condition {
                        next.value
                    }
                    else {
                        next.redirect
                    }
                }
                None => {
                    next.value
                }
            }
        }
        else {
            self.stop
        };
        self.signal.set(next);
        let next: S = next.into();
        *state = next.clone();
        next
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }


    pub fn connect<SA,SB>(&mut self, state: SA, other: &StateMachine, condition: SB, redirect: SA)
    where   SA: Into<u32>, SB: Into<u32>
    {
        let idx = state.into() as u32 as usize;
        self.states[idx].signal = Some(other.signal.clone());
        self.states[idx].condition = condition.into();
        self.states[idx].redirect = redirect.into();
    }

}

impl Default for StateMachine {
    fn default() -> Self {
        StateMachine::new()
    }
}

mod unittest {

    #[test]
    fn from() {
        let map: [(u32,u32); 5] = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let sm = super::StateMachine::from(&map).stops_at(0u32);

        for val in &map {
            let mut state = val.0;
            sm.next(&mut state);
            println!("next: {}, val.0: {}, .1: {}", state, val.0, val.1);
            assert_eq!(val.1, state)
        }
    }

    #[test]
    fn state_out_of_bound() {
        let map: [(u32,u32); 5] = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let stop: u32 = 22;
        let sm = super::StateMachine::from(&map).stops_at(stop);
        let mut state = 99_u32;
        sm.next(&mut state);
        assert_eq!(state, stop);
    }

    #[test]
    fn stop_state() {
        let map: [(u32,u32); 2] = [ (0,4), (4,3) ];
        let stop: u32 = 22;
        let sm = super::StateMachine::from(&map).stops_at(stop);
        let mut state = 99_u32;
        sm.next(&mut state);
        assert_eq!(state, stop);
        todo!("same as state out of bound")
    }

    #[test]
    fn connect() {
        let map: [(u32,u32); 5] = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let stop: u32 = 22;
        let mut sma = super::StateMachine::from(&map).stops_at(stop);
        let smb = super::StateMachine::from(&map).stops_at(stop);

        sma.connect(3_u32, &smb, 1_u32, 0_u32);
        let mut state: u32 = 3;
        sma.next(&mut state); // next from state 3, but smb is not in state 1
        assert_eq!(state, 0);
        state = 2;
        smb.next(&mut state); // bring smb into state 1
        assert_eq!(state, 1);
        state = 3;
        smb.next(&mut state); // next from state 3 is 2 as expected, because smb is in state 1
        assert_eq!(state, 2);
    }
}
