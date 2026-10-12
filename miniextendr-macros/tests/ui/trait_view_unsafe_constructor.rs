//! Test: safe code cannot build a trait View from arbitrary pointers.
//!
//! The only constructor of a View's pointer storage, `TraitViewData::new`, is
//! unsafe, so code in the View's own module cannot call a safe View method
//! on forged pointers without an `unsafe` block.

use miniextendr_api::trait_abi::TraitViewData;
use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
}

fn forge_constructor() -> i32 {
    let view = CounterView {
        __mx_view: TraitViewData::new(
            1 as *mut ::std::ffi::c_void,
            1 as *const ::std::ffi::c_void,
            None,
        ),
    };
    view.value()
}

fn main() {
    forge_constructor();
}
