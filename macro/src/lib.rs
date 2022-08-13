#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::{TokenStream};
use quote::__private::Span;
use syn::{Ident, Token, parse_macro_input};
use syn::parse::{Parse, ParseStream, Result};
use std::collections::HashSet;

#[derive(Debug)]
struct Transition {
    pub state: Ident,
    pub event: Ident,
    pub next: Ident,
}

impl Parse for Transition {
    fn parse(input: ParseStream) -> Result<Self> {
        let state: Ident  = input.parse()?;
        let _: Token![,]  = input.parse()?;
        let event: Ident  = input.parse()?;
        let _: Token![->] = input.parse()?;
        let next: Ident   = input.parse()?;

        Ok(Transition {
            state,
            event,
            next,
        })
    }
}


#[derive(Debug)]
struct Transitions{list: Vec<Transition>, name: Ident}

impl Parse for Transitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut list = Vec::new();
        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;

        loop {
            let next = input.lookahead1();
            if next.peek(Ident) {
                let transition: Transition = input.parse()?;
                list.push(transition);
            }
            // in case one will organize transitions in one line
            // ; makes it more easily readable
            else if next.peek(Token![;]) {
                let _: Token![;] = input.parse()?;
            }
            else { break }
        }

        Ok(Transitions{list, name})
    }
}

#[derive(Debug)]
struct States(Vec<Ident>);

impl Parse for States {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut vector = Vec::new();
        loop {
            let next = input.lookahead1();
            if next.peek(Ident) {
                let i: Ident = input.parse()?;
                vector.push(i);
            }
            else if next.peek(Token![,]) {
                let _:Token![,] = input.parse()?;
            }
            else { break }
        }

        Ok(States(vector))
    }
}

#[derive(Debug)]
struct Events(Vec<Ident>);

impl Parse for Events {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut vector = Vec::new();
        loop {
            let next = input.lookahead1();
            if next.peek(Ident) {
                let i: Ident = input.parse()?;
                vector.push(i);
            }
            else if next.peek(Token![,]) {
                let _:Token![,] = input.parse()?;
            }
            else { break }
        }

        Ok(Events(vector))
    }
}


/// Test documentation of StateMachine
///
#[allow(non_snake_case)]
#[proc_macro]
pub fn StateMachine(item: TokenStream) -> TokenStream {
    let mut stream = proc_macro::TokenStream::new();
    let mut state_set = HashSet::<Ident>::new();
    let mut event_set = HashSet::<Ident>::new();
    let transitions = parse_macro_input!(item as Transitions);
    let name = transitions.name;
    let machine_type = Ident::new(format!("{}Machine",name).as_str(), Span::call_site());
    let state_type = Ident::new(format!("{}State",name).as_str(), Span::call_site());
    let event_type = Ident::new(format!("{}Event",name).as_str(), Span::call_site());

    // 1. generate the static array of transitions
    let array = transitions.list.iter().map(|t| {
        let state = t.state.clone();
        let event = t.event.clone();
        let next = t.next.clone();
        state_set.insert(state.clone());
        event_set.insert(event.clone());
        quote! {Transition{
            state: #state_type::#state,
            event: #event_type::#event,
            next: #state_type::#next,
        },
    }
    }).collect::<Vec<_>>();
    let transition_count = array.len();

    // 2. generate events
    let list = event_set.iter().map(|s| {
        let event = s.clone();
        quote! {#event,}
    }).collect::<Vec<_>>();

    let event_enum = quote! {
        #[derive(Clone,Copy,Debug,PartialEq)]
        enum #event_type{ Undefined, #(#list)* }
        impl Default for #event_type {
            fn default() -> Self {
                #event_type::Undefined
            }
        }
    };
    stream.extend(proc_macro::TokenStream::from(event_enum));

    // 3. generate states
    let list = state_set.iter().map(|s| {
        let state = s.clone();
        quote! {#state,}
    }).collect::<Vec<_>>();


    let state_enum = quote! {
        #[derive(Clone,Copy,Debug,PartialEq)]
        enum #state_type{ Undefined, #(#list)* }
        impl Default for #state_type {
            fn default() -> Self {
                #state_type::Undefined
            }
        }
    };
    stream.extend(proc_macro::TokenStream::from(state_enum));

    // 4. generate transition lookup implementation
    let mut idx: usize = 0;
    let lookup_list = transitions.list.iter().map(|t| {
        let state = t.state.clone();
        let event = t.event.clone();
        let quote = quote! {(#state,#event) => #idx,};
        idx += 1;
        quote
    }).collect::<Vec<_>>();

    let undefined_name = Ident::new(format!("UNDEFINED_TRANSITION_{}",name).as_str(), Span::call_site());
    let state_machine = quote! {
        const #undefined_name: Transition<#state_type,#event_type> = Transition {
            state: #state_type::Undefined,
            event: #event_type::Undefined,
            next: #state_type::Undefined,
        };

        #[derive(Clone)]
        struct #machine_type<#state_type,#event_type> {
            state: #state_type,
            start: #state_type,
            stop: #state_type,
            transitions: [Transition<#state_type,#event_type>; #transition_count],
        }

        type #name = #machine_type<#state_type,#event_type>;

        impl StateMachine<#state_type,#event_type> for #machine_type<#state_type,#event_type>
        where #state_type: Default {

            /// This is new.
            fn new(start: #state_type, stop: #state_type) -> Self {
                #machine_type {
                    state: start,
                    start: start,
                    stop: stop,
                    transitions: [
                        #(#array)*
                    ],
                }
            }

            fn state(&self) -> #state_type {
                self.state
            }

            fn set_state(&mut self, state: #state_type) {
                self.state = state;
            }

            fn reset(&mut self) {
                self.state = self.start;
            }

            fn lookup(&self, state: &#state_type, event: &#event_type) -> &Transition<#state_type,#event_type> {
                use #state_type::*;
                use #event_type::*;
                let idx: usize = match (state, event) {
                    #(#lookup_list)*
                    (_,_) => {return &#undefined_name;}
                };
                &self.transitions[idx]
            }

        }
    };
    stream.extend(proc_macro::TokenStream::from(state_machine));

    stream
}