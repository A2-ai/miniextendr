// Producer package: Creates Counter objects and exposes them to R
//
// This demonstrates cross-package trait dispatch:
// - Defines the Counter trait with #[miniextendr]
// - Implements Counter for SimpleCounter
// - Objects can be passed to consumer package

use miniextendr_api::{miniextendr, miniextendr_module, ffi::SEXP, trait_abi::ccall, ExternalPtr};

// ============================================================================
// Shared trait definition
// ============================================================================

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}

// ============================================================================
// SimpleCounter implementation
// ============================================================================

#[derive(ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn create(initial: i32) -> Self {
        Self { value: initial }
    }
}

#[miniextendr]
impl SimpleCounter {
    /// Get current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }
}

/// Create a new counter with cross-package trait dispatch support
///
/// This wraps the SimpleCounter in an mx_erased wrapper so it can be
/// used with consumer.pkg's trait-generic functions.
///
/// @param initial Initial counter value
/// @return An external pointer to the wrapped counter
/// @export
#[miniextendr]
fn new_counter(initial: i32) -> SEXP {
    let counter = SimpleCounter::create(initial);
    // Use the generated mx_wrap function to wrap with trait dispatch support
    let erased = __mx_wrap_simplecounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}

/// Get the value from a trait-wrapped counter
///
/// @param counter_sexp An external pointer from new_counter()
/// @return The counter's current value
/// @export
#[miniextendr]
fn counter_get_value(counter_sexp: SEXP) -> i32 {
    let view = unsafe { CounterView::try_from_sexp(counter_sexp) }
        .expect("Object does not implement Counter trait");
    view.value()
}

/// Debug: Get TAG_COUNTER as hex string
/// @export
#[miniextendr]
fn debug_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_COUNTER.hi, TAG_COUNTER.lo)
}

miniextendr_module! {
    mod producer_pkg;
    impl SimpleCounter;
    impl Counter for SimpleCounter;
    fn new_counter;
    fn counter_get_value;
    fn debug_tag_counter;
}
