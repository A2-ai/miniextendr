//! Compile-time warnings for deprecated `#[miniextendr]` options (#2117).
//!
//! A proc macro can't emit a warning on stable Rust, but code that names a
//! `#[deprecated]` item can. The option parsers record the span of an
//! explicit `worker`, and the `#[miniextendr]` entry point appends one
//! reference to `miniextendr_api::worker::WORKER_OPTION` per recorded span,
//! so rustc warns at the option itself. `#![allow(deprecated)]` on the
//! enclosing module silences it.

use std::cell::RefCell;

thread_local! {
    static WORKER_OPTIONS: RefCell<Vec<proc_macro2::Span>> = const { RefCell::new(Vec::new()) };
}

/// Record an explicit `worker` option written at `span`.
pub(crate) fn worker_option(span: proc_macro2::Span) {
    WORKER_OPTIONS.with(|spans| spans.borrow_mut().push(span));
}

/// The references that make rustc warn at each `worker` option recorded
/// since the last call.
pub(crate) fn take() -> proc_macro2::TokenStream {
    WORKER_OPTIONS
        .with(RefCell::take)
        .into_iter()
        .map(|span| {
            quote::quote_spanned! {span=>
                const _: () = ::miniextendr_api::worker::WORKER_OPTION;
            }
        })
        .collect()
}
