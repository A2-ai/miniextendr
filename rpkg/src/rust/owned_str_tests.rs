//! `Box<str>` / `Rc<str>` / `Arc<str>` fixtures for `test-owned-str.R`.

use std::rc::Rc;
use std::sync::Arc;

use miniextendr_api::{Altrep, miniextendr};

// region: Owned str conversions

/// A character scalar read as `Box<str>` (`NA` reads as `""`) and returned.
/// @param x A character scalar.
#[miniextendr(noexport)]
pub fn owned_str_box_scalar(x: Box<str>) -> Box<str> {
    x
}

/// A character vector read as `Vec<Option<Rc<str>>>` and returned.
/// @param x A character vector.
// `Rc` is `!Send`: stay on R's main thread under `worker-default`.
#[miniextendr(noexport, no_worker)]
pub fn owned_str_rc_vec(x: Vec<Option<Rc<str>>>) -> Vec<Option<Rc<str>>> {
    x
}

/// Whether a character scalar read as `Option<Arc<str>>` is `None` (`NA`).
/// @param x A character scalar.
#[miniextendr(noexport)]
pub fn owned_str_arc_is_na(x: Option<Arc<str>>) -> bool {
    x.is_none()
}

/// A string ALTREP over `Vec<Option<Arc<str>>>`.
#[miniextendr(noexport)]
pub fn owned_str_arc_altrep() -> Altrep<Vec<Option<Arc<str>>>> {
    Altrep(vec![Some(Arc::from("one")), None, Some(Arc::from("three"))])
}

/// A string ALTREP over `Vec<Rc<str>>` whose elements share one allocation.
// `Rc` is `!Send`: stay on R's main thread under `worker-default`.
#[miniextendr(noexport, no_worker)]
pub fn owned_str_rc_altrep() -> Altrep<Vec<Rc<str>>> {
    let shared: Rc<str> = Rc::from("same");
    Altrep(vec![Rc::clone(&shared), Rc::from("other"), shared])
}
// endregion
