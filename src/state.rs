use crate::signal::*;
use smallvec::SmallVec;

// State Signal
type Signal = SignalU32;

/// TODO:
///     - improve documentation
///     - implement a tracker that can be used at debugging


#[derive(Clone, Debug)]
pub struct Dependency {
    pub(crate) signal: Signal,
    pub(crate) state: u16
}

impl Default for Dependency
{
    fn default() -> Self {
        Dependency {
            signal: Signal::default(),
            state: 0
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transition<S,E> {
    pub state: S,
    pub event: E,
    pub next:  S,
    pub detour: S,

    // This dependencies field is not really optional, but using Option is a workaround, as the Signals can not be copied. Which would be needed at array initialization.
    pub dependencies: Option<SmallVec<[Dependency; 2]>>,
    // pub signal: Option<Signal>
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
            // signal: None,
        }
    }
}

impl<S,E> Transition<S,E>
{
    pub fn depend_on<'a, SMB, SB, EB>(&'a mut self, state_machine: &SMB, state: SB) -> DetourConnection<S,E>
    where SB: Into<u16> + Copy + Default, EB: Copy, SMB: StateMachine<SB,EB> {
        let transition = self;
        let dependency = Dependency {
            signal: state_machine.signal().clone(),
            state:  state.into()
        };

        if transition.dependencies.is_none() {
            transition.dependencies = Some(SmallVec::<[Dependency; 2]>::with_capacity(2))
        }

        if let Some(dep) = &mut transition.dependencies {
            dep.push(dependency);
        }

        DetourConnection{transition}
    }
}

#[allow(clippy::type_complexity)]
pub struct Transitions<S:'static + Sized, E: 'static + Sized> {
    pub list: &'static [Transition<S,E>],
    pub lookup: fn(&'static [Transition<S,E>], &S, &E) -> &'static Transition<S,E>
}

pub trait ModifiableStateMachine<S,E> {
    fn last_transition_signal(&self) -> &Option<Signal>;
    fn lookup(&self, state: &S, event: &E) -> &Transition<S,E>;
    fn mut_lookup(&mut self, state: &S, event: &E) -> &mut Transition<S,E>;
    fn set_state(&mut self, state: S);
    fn set_last_transition_signal(&mut self, signal: Option<Signal>);
}

pub struct DetourConnection<'c, SA, EA> {
    transition: &'c mut Transition<SA,EA>,
}

impl<'c, SA, EA> DetourConnection<'c, SA, EA> where SA: Copy {
    pub fn next_if_pending(self, state: SA) {
        self.transition.detour = state;
    }
    pub fn loop_if_pending(self) {
        self.transition.detour = self.transition.state;
    }
}

pub trait StateMachine<S,E>
where   S: Into<u16> + Copy + Default,
        E: Copy
{
    fn new(start: S, stop: S) -> Self;

    fn state(&self) -> S;

    fn signal(&self) -> &Signal;

    fn reset(&mut self);

    fn next(&mut self, event: &E) -> S
    where Self: ModifiableStateMachine<S,E>
    {
        let next;

        // TODO: is the last transition really needed without ref counting?
        // let last_signal;

        {   // here only a immutable reference to self is needed,
            // and it has to be dropped before mutating self
            let transition = self.lookup(&self.state(), event);
            // last_signal = transition.signal.clone();
            next = match &transition.dependencies {
                Some(dependencies) => {
                    let count = dependencies.iter()
                        .map(|d| (d.signal.probe() as u16 == d.state) as usize )
                        .sum();
                        match count {
                            0 => transition.next,
                            _ => transition.detour
                    }
                }
                None => transition.next
            };
            // if let Some(signal) = &transition.signal {
            //     signal.decr(); // fulfill someones dependency, approaching 0
            // }
            // if let Some(signal) = self.last_transition_signal() {
            //     signal.incr(); // leaving the state, the dependency of someone else is not fulfilled anymore
            // }
        }

        // self.set_last_transition_signal(last_signal);
        self.set_state(next);
        next
    }

    // sm.connect(state_x).with(sm2, state_b).loop_if_pending()

    fn transition(&mut self, state:S, event:E) -> &mut Transition<S,E>
    where Self: ModifiableStateMachine<S,E> {
        self.mut_lookup(&state, &event)
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