use std::sync::{Arc, atomic::{AtomicU8}};
use atomx_macro::*;
type Signal = Arc<AtomicU8>;
use smallvec::SmallVec;

// What will implemented in the atomx crate.

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
    // pub signal: Option<Signal>,
    pub dependencies: Option<SmallVec<[Dependency; 2]>>,
}

// trait StateMachine<S:'static, E:'static> {
//     fn lookup(&self, state: &S, event: &E) -> &'static Transition<S,E>;
// }

pub trait ModifiableStateMachine<S,E> {
    fn set_state(&mut self, state: S);
    fn lookup(&self, state: &S, event: &E) -> &Transition<S,E>;
    fn mut_lookup(&mut self, state: &S, event: &E) -> &mut Transition<S,E>;
    fn last_transition_signal(&self) -> &Option<Signal>;
    fn set_last_transition_signal(&mut self, signal: Option<Signal>);
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
        S::default()
    }

    fn connect<SA,SB,SM2>(&mut self, state: SA, other: &SM2, dependency: SB, alternative: SA)
    where   SA: Into<u32>, SB: Into<u32>
    {
        // let idx = state.into() as u32 as usize;
        // self.transitions[idx].signal = Some(other.signal.clone());
        // self.transitions[idx].dependency = dependency.into();
        // self.transitions[idx].alternative = alternative.into();
    }

}


// If you are seeing a lot of "proc macro not expanded" warnings, you can add this option to the
// rust-analyzer.diagnostics.disabled list to prevent them from showing. Alternatively you can
// enable support for procedural macros (see rust-analyzer.procMacro.enable).
//
// Source: https://rust-analyzer.github.io/manual.html#unresolved-proc-macro



// What the user will have to write.


StateMachine!( SM1:
    A, G => B
    C, M => A
    B, L => C
    A, H => A
    C, N => B
);

StateMachine!( SM2:
    A, G => B
    C, M => A
    B, L => C
    A, H => A
    C, N => B
);

#[test]
fn works() {

    let sm1 = SM1::new(SM1State::A, SM1State::Undefined);
    let _sm2 = SM2::new(SM2State::A, SM2State::Undefined);
    // it should be a vector, nearly 1kb for one sm with a handful of possible connections is a lot
    // share a small vec per sm between transitions
    // >> SmallVec
    println!("size sm:    {}", std::mem::size_of::<SM1>());
    println!("size trans: {}", std::mem::size_of::<Transition<SM1State, SM1Event>>());
    println!("size state: {}", std::mem::size_of::<SM1State>());
    println!("size event: {}", std::mem::size_of::<SM1Event>());
    println!("size Opt<Signal>: {}", std::mem::size_of::<Option<Signal>>());
    println!("size Opt<u8>: {}", std::mem::size_of::<Option<u8>>());
    println!("size Opt<Vec<Dep>>: {}", std::mem::size_of::<Option<Vec<Dependency>>>());
    println!("size Opt<SmallVec<Dep>>: {}", std::mem::size_of::<Option<SmallVec<[Dependency; 2]>>>());
    println!("size Dependency<SM1State>: {}", std::mem::size_of::<Dependency>());
    println!("size Dep: {}", std::mem::size_of::<Dependency>());
    println!("{:#?}", sm1.lookup(&SM1State::A, &SM1Event::G));
}