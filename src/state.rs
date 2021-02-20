use super::signal::*;
use std::sync::Arc;
use std::collections::HashMap;

/* TODO:
 *      - add a test for from_map
 *      - add default stop state, for initialization
 *      - maybe turn states into an array, if if generic const feature is available
 *      
 */

struct StateMachine 
{
    signal: StateSignal, 
    states: Vec<(u32, Option<Arc<StateSignal>>)>
}

impl StateMachine
{
    pub fn new() -> Self {
        StateMachine {
            signal: StateSignal::default(),
            states: vec![],
        }
    }

    pub fn with_capacity(c: usize) -> Self {
        StateMachine {
            signal: StateSignal::default(),
            states: Vec::with_capacity(c),
        }
    }

    pub fn from_map<S>(map: &[(S, S)]) -> StateMachine where S: Clone + Into<u32> {
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
        sm
    }

    pub fn state(&self) -> u32 {
        self.signal.state()
    }

    pub fn next(&self, state: u32) -> u32 {
        let next = self.states[state as usize].0; 
        self.signal.set(next);
        next
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }
    
}

mod unittest {

    use super::*;

    #[test]
    fn from_map() {
        let map = [ (0,4), (4,3), (3,2), (2,1), (1,0) ];
        let sm = StateMachine::from_map(&map);
        
        for i in 0..sm.state_count() {
            let next = sm.next(map[i].0);
            //println!("i: {}, next: {}, map[i].0: {}, .1: {}", i, next, map[i].0, map[i].1);
            assert_eq!(map[i].1, next)
        }
    }
}
