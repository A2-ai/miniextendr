//! An independently held trait view must keep its concrete receiver alive.
mod r_test_utils;

use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{ExternalPtr, miniextendr};
use std::cell::Cell;
use std::rc::Rc;

#[miniextendr]
pub trait ReadTracked {
    fn value(&self) -> i32;
}

#[derive(ExternalPtr)]
pub struct Tracked {
    value: i32,
    drops: Rc<Cell<usize>>,
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[miniextendr(env, noexport)]
impl ReadTracked for Tracked {
    fn value(&self) -> i32 {
        self.value
    }
}

unsafe fn new_view(value: i32, drops: Rc<Cell<usize>>) -> ReadTrackedView {
    unsafe {
        let owner = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(__mx_wrap_Tracked(
            Tracked { value, drops },
        )));
        ReadTrackedView::from_sexp(owner.get())
    }
}

unsafe fn collect() {
    unsafe {
        miniextendr_api::sys::R_gc();
        miniextendr_api::sys::R_RunPendingFinalizers();
    }
}

#[test]
fn views_outlive_the_creating_scope_and_release_roots_in_any_order() {
    r_test_utils::with_r_thread(|| unsafe {
        let drops = Rc::new(Cell::new(0));
        let first = new_view(11, drops.clone());
        let second = new_view(22, drops.clone());
        collect();
        // Check finalization before calling: a regression must report the
        // premature drop rather than dereference the freed concrete object.
        assert_eq!(drops.get(), 0, "a view lost its receiver root");
        assert_eq!(first.value(), 11);
        assert_eq!(second.value(), 22);

        drop(first);
        collect();
        assert_eq!(drops.get(), 1, "dropping a view must release its owner");
        assert_eq!(second.value(), 22);
        drop(second);
        collect();
        assert_eq!(drops.get(), 2);
    });
}
