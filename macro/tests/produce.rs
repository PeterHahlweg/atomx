use atomx_macro::*;

// What will implemented in the atomx crate.

#[derive(Clone, Debug)]
pub struct Transition<S,E> {
    pub state: S,
    pub event: E,
    pub next:  S,
}

// trait StateMachine<S:'static, E:'static> {
//     fn lookup(&self, state: &S, event: &E) -> &'static Transition<S,E>;
// }

pub trait ModifiableStateMachine<S,E> {
    fn set_state(&mut self, state: S);
    fn lookup(&self, state: &S, event: &E) -> &Transition<S,E>;
    fn mut_lookup(&mut self, state: &S, event: &E) -> &mut Transition<S,E>;
}

pub trait StateMachine<S,E>
where   S: Into<u16> + Copy + Default,
        E: Copy
{
    fn new(start: S, stop: S) -> Self;

    fn state(&self) -> S;

    fn reset(&mut self);

    fn next(&mut self, event: &E) -> S
    where Self: ModifiableStateMachine<S,E>
    {
        let _ = event;
        S::default()
    }
}


// If you are seeing a lot of "proc macro not expanded" warnings, you can add this option to the
// rust-analyzer.diagnostics.disabled list to prevent them from showing. Alternatively you can
// enable support for procedural macros (see rust-analyzer.procMacro.enable).
//
// Source: https://rust-analyzer.github.io/manual.html#unresolved-proc-macro



// What the user will have to write.


StateMachine!( SM:
    A, G => B
    C, M => A
    B, L => C
    A, H => A
    C, N => B
);

#[test]
fn works() {

    let sm = SM::new(SMState::A, SMState::Undefined);

    println!("size sm:    {}", std::mem::size_of::<SM>());
    println!("size trans: {}", std::mem::size_of::<Transition<SMState, SMEvent>>());
    println!("size state: {}", std::mem::size_of::<SMState>());
    println!("size event: {}", std::mem::size_of::<SMEvent>());
    println!("size Opt<u8>: {}", std::mem::size_of::<Option<u8>>());
    println!("{:#?}", sm.lookup(&SMState::A, &SMEvent::G));
}