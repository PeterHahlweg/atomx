

#[derive(Clone, Debug)]
pub struct Transition<S,E> {
    pub state: S,
    pub event: E,
    pub next:  S,
}

impl<S,E> Default for Transition<S,E>
where   S: Copy + Default,
        E: Copy + Default
{
    fn default() -> Self {
        Transition {
            state: S::default(),
            event: E::default(),
            next: S::default(),
        }
    }
}

pub trait StateMachine<S,E>
where   S: Copy + Default + std::fmt::Debug,
        E: Copy
{
    fn new(start: S, stop: S) -> Self;

    fn state(&self) -> S;
    fn set_state(&mut self, state: S);

    fn reset(&mut self);

    fn lookup(&self, state: &S, event: &E) -> &Transition<S,E>;

    fn next_state(&mut self, event: &E) -> S
    {
        let next = self.lookup(&self.state(), event).next;
        self.set_state(next);
        next
    }
}

