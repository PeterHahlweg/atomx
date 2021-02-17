use super::signal::*;
use std::sync::Arc;
use std::collections::HashMap;

/* TODO:
 *      - add a test for from_map
 *      - add default stop state, for initialization
 *      - maybe turn states into an array, if if generic const feature is available
 *      
 */

struct StateMachine {
    signal: StateSignal, 
    states: Vec<(u32, Option<Arc<StateSignal>>)>
}

impl StateMachine {
    
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

    pub fn from_map(map: &[(u32, u32)]) -> StateMachine {
        let mut sm = StateMachine::new();
        for value in map {
            let max = std::cmp::max(value.0, value.1);
            if sm.states.len() < max as usize {
                sm.states.resize((max+1) as usize, (0,None));
            }
            sm.states[value.0 as usize] = (value.1, Some(Arc::new(StateSignal::new(0))));
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
