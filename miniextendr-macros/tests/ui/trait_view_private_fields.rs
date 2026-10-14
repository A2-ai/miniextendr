//! Test: safe code cannot build a trait View's pointer storage from fields.
//!
//! A View's pointers live in the runtime's `TraitViewData`, whose fields are
//! private to `miniextendr_api`. Code in the View's own module can name the
//! View's field, but cannot fill it with forged pointers.

use miniextendr_api::trait_abi::TraitViewData;
use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
}

fn forge_fields() -> i32 {
    let view = CounterView {
        __mx_view: TraitViewData {
            data: 1 as *mut ::std::ffi::c_void,
            vtable: 1 as *const CounterVTable,
            owner: None,
        },
    };
    view.value()
}

fn main() {
    forge_fields();
}
