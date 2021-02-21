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

impl StateMachine
{
    pub fn new() -> Self {
        StateMachine {
            signal: Arc::new(StateSignal::default()),
            states: vec![],
            stop: 0,
        }
    }

    pub fn from<S>(map: &[(S, S)], stop: S) -> StateMachine
    where S: Clone + Into<u32>
    {
        let mut sm = StateMachine::new();
        sm.stop = stop.into();
        let default_state = State{value: sm.stop, signal: None, condition: 0, redirect: 0};
        sm.states.resize(map.len(), default_state.clone());

        for value in map {
            let state: u32 = value.0.clone().into();
            let next: u32  = value.1.clone().into();
            let max = std::cmp::max(state, next) as usize;
            if sm.states.len() < max {
                sm.states.resize(max+1, default_state.clone());
            }
            sm.states[state as usize] = State{value: next, signal: None, condition: 0, redirect: 0};
        }
        sm
    }

    pub fn state<S>(&self) -> S
    where S: From<u32> + Clone
    {
        self.signal.state().into()
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

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestState{ A, B, C, D, E, Stop}

    impl From<TestState> for u32 {
        fn from(ts: TestState) -> Self {
            ts as u32
        }
    }

    impl From<u32> for TestState {
        fn from(x: u32) -> Self {
            use TestState::*;
            match x {
                0 => A,
                1 => B,
                2 => C,
                3 => D,
                4 => E,
                _ => Stop,
            }
        }
    }

    #[test]
    fn from() {
        use TestState::*;
        let map: [_; 5] = [(A,E), (E,D), (D,C), (C,B), (B,A)];
        let sm = super::StateMachine::from(&map, A);

        for val in &map {
            let mut state = val.0;
            sm.next(&mut state);
            println!("next: {:?}, val.0: {:?}, .1: {:?}", state, val.0, val.1);
            assert_eq!(val.1, state);
        }
    }

    #[test]
    fn state_out_of_bound() {
        let map: [(u32,u32); 5] = [(0,4), (4,3), (3,2), (2,1), (1,0)];
        let stop: u32 = 22;
        let sm = super::StateMachine::from(&map, stop);

        // off by one
        let mut state: u32 = 5;
        sm.next(&mut state);
        assert_eq!(state, stop);

        // the max state given from transitions map
        state = 4;
        sm.next(&mut state);
        assert_eq!(state, 3);

        // obviously out of bound
        state = 99;
        sm.next(&mut state);
        assert_eq!(state, stop);
    }

    #[test]
    fn stop_state() {
        use TestState::*;
        let map: [_; 2] = [(A,E), (E,D)];
        let sm = super::StateMachine::from(&map, Stop);

        // giving the stop state
        let mut state = Stop;
        sm.next(&mut state);
        assert_eq!(state, Stop);

        // given a state without a defined transition to another state
        state = D;
        sm.next(&mut state);
        assert_eq!(state, Stop);
    }

    #[test]
    fn connect() {
        use TestState::*;
        let map: [_; 5] = [(A,E), (E,D), (D,C), (C,B), (B,A)];
        let mut sma = super::StateMachine::from(&map, Stop);
        let smb = super::StateMachine::from(&map, Stop);

        sma.connect(D, &smb, B, A);
        let mut state = D;
        sma.next(&mut state); // next from state D, but smb is not in state B
        assert_eq!(state, A); // so we expect A

        // bring smb into state B
        state = C;
        smb.next(&mut state);
        assert_eq!(state, B);

        // next from state D is C as expected, because smb is in state B
        state = D;
        smb.next(&mut state);
        assert_eq!(state, C);
    }

    #[test]
    fn state() {
        use TestState::*;
        let map: [_; 5] = [(A,E), (E,D), (D,C), (C,B), (B,A)];
        let sma = super::StateMachine::from(&map, Stop);
        let mut state = C;
        sma.next(&mut state);
        state = sma.state();
        assert_eq!(state, B);
    }
}
