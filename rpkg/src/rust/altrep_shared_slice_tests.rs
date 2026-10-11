//! `Rc<[T]>` / `Arc<[T]>` ALTREP fixtures for `test-altrep-shared-slices.R`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use miniextendr_api::{Altrep, miniextendr};

// region: Shared slice ALTREP fixtures

thread_local! {
    static HELD: RefCell<Option<Rc<[i32]>>> = const { RefCell::new(None) };
}

/// An `Rc<[i32]>` ALTREP whose allocation Rust shares too: a clone stays in
/// a thread-local, read back by `shared_slice_held()`.
#[miniextendr(noexport)]
pub fn shared_slice_int() -> Altrep<Rc<[i32]>> {
    let owner: Rc<[i32]> = Rc::from([1, 2, 3]);
    HELD.with_borrow_mut(|held| *held = Some(Rc::clone(&owner)));
    Altrep(owner)
}

/// The values of the clone `shared_slice_int()` kept.
#[miniextendr(noexport)]
pub fn shared_slice_held() -> Vec<i32> {
    HELD.with_borrow(|held| held.as_deref().expect("shared_slice_int() ran").to_vec())
}

/// An `Arc<[String]>` ALTREP.
#[miniextendr(noexport)]
pub fn shared_slice_strings() -> Altrep<Arc<[String]>> {
    Altrep(Arc::from([
        String::from("a"),
        String::from("b"),
        String::from("c"),
    ]))
}

/// An `Arc<[bool]>` ALTREP.
#[miniextendr(noexport)]
pub fn shared_slice_logicals() -> Altrep<Arc<[bool]>> {
    Altrep(Arc::from([true, false]))
}
// endregion
