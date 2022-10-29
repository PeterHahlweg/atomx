#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use atomx_macro::transitions;
enum State {
    A,
    B,
    C,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for State {
    #[inline]
    fn clone(&self) -> State {
        {
            *self
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::marker::Copy for State {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for State {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&State::A,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "A");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&State::B,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "B");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&State::C,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "C");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
        }
    }
}
enum Event {
    G,
    H,
    L,
    M,
    N,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for Event {
    #[inline]
    fn clone(&self) -> Event {
        {
            *self
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::marker::Copy for Event {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for Event {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&Event::G,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "G");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&Event::H,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "H");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&Event::L,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "L");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&Event::M,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "M");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
            (&Event::N,) => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(f, "N");
                ::core::fmt::DebugTuple::finish(debug_trait_builder)
            }
        }
    }
}
use State::*;
use Event::*;
const into_pair: [(State, Event); 1usize] = [(A, G)];
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const works: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("works"),
        ignore: false,
        allow_fail: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(works())),
};
fn works() {
    ::std::io::_print(::core::fmt::Arguments::new_v1(
        &[""],
        &match (&5,) {
            (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
        },
    ));
}
#[main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&works])
}
