use crate::signal::*;

// State Signal
type Signal = SignalU32;

/// TODO:
///     - improve connect api
///     - improve documentation


/// if there many dependent states, it is a problem as we need to check them all
///  - one approach could be to implement a dependency counter
///  - if it goes to 0 we can got ahead else take the detour
///  - there will be more concurrent access to it, but no heap and only a little memory
///  - implement a tracker that can be used at debugging, because a reference count removes the
///    information about which state is reached and which not
///  - in debug case it is als not important how much memory is allocated somewhere
///  - the dependency member of Transition can be reduced to Option<Signal>

#[derive(Clone, Debug)]
pub struct Transition<S,E> {
    pub state: S,
    pub event: E,
    pub next:  S,
    pub detour: S,
    pub dependencies: Option<Signal>,
    pub signal: Option<Signal>
}

impl<S,E> Default for Transition<S,E>
where   S: Into<u16> + From<u16> + Copy + Default,
        E: Into<u16> + Copy + Default
{
    fn default() -> Self {
        Transition {
            state: S::default(),
            event: E::default(),
            next: S::default(),
            detour: S::default(),
            dependencies: None,
            signal: None,
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct Transitions<S:'static + Sized, E: 'static + Sized> {
    pub list: &'static [Transition<S,E>],
    pub lookup: fn(&'static [Transition<S,E>], &S, &E) -> &'static Transition<S,E>
}

#[derive(Clone)]
pub struct StateMachine<S:'static, E:'static> {
    state: S,
    start: S,
    stop: S,
    transitions: &'static Transitions<S,E>,
    last_transition: &'static Transition<S,E>
}

impl<S,E> StateMachine<S,E>
where   S: Into<u16> + Copy + Default,
        E: Copy
{
    pub fn new(transitions: &'static Transitions<S,E>, start: S, stop: S) -> Self {
        StateMachine {
            state: start,
            start,
            stop,
            transitions,
            last_transition: &transitions.list[0], // default undefined transition of every sm
        }
    }

    pub fn state(&self) -> S {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = self.start;
        // TODO: we need a proper replacement here
        // self.signal.emit(self.start.into() as u32);
    }


    pub fn next(&mut self, event: &E) -> S {
        let lookup = self.transitions.lookup;
        let transition = lookup(self.transitions.list, &self.state, event);
        let mut next = transition.next;
        if let Some(dependencies) = &transition.dependencies {
            match dependencies.probe() {
                0 => next = transition.next,
                _ => next = transition.detour,
            }

        }
        if let Some(signal) = &transition.signal {
            signal.decr(); // fulfill someones dependency, approaching 0
        }
        if let Some(signal) = &self.last_transition.signal {
            signal.incr(); // leaving the state, the dependency of someone else is not fulfilled anymore
        }
        self.last_transition = transition;
        self.state = next;
        next
    }

    pub fn state_count(&self) -> usize {
        self.transitions.list.len()
    }


    pub fn connect<SA,SB,EB>(&mut self, state: SA, other: &StateMachine<SB,EB>, dependency: SB, alternative: SA)
    where   SA: Into<u32>, SB: Into<u32>
    {
        // let idx = state.into() as u32 as usize;
        // self.transitions[idx].signal = Some(other.signal.clone());
        // self.transitions[idx].dependency = dependency.into();
        // self.transitions[idx].alternative = alternative.into();
    }

}

/*
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
*/