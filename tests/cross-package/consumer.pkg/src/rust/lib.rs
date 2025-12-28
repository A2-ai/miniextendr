// Consumer package: Works with Counter objects from other packages
//
// This demonstrates cross-package trait dispatch:
// - Has the same Counter trait definition (ABI-compatible with producer)
// - Provides generic functions that work with any Counter implementation
// - Can operate on objects created by producer package
// - Defines its OWN Counter implementation (DoubleCounter) that producer can use

use miniextendr_api::{miniextendr, miniextendr_module, ffi::SEXP, trait_abi::ccall, ExternalPtr};

// ============================================================================
// Shared trait definition (must match producer exactly)
// ============================================================================

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}

// ============================================================================
// Cross-package ExternalPtr pass-through test
// ============================================================================

/// Pass an ExternalPtr through consumer without knowing its type.
///
/// This tests that ExternalPtr can cross package boundaries as opaque SEXP.
/// Consumer has NO knowledge of what type the ExternalPtr holds.
///
/// @param ptr An ExternalPtr from any package
/// @return The same ExternalPtr (pass-through)
/// @export
#[miniextendr]
fn passthrough_ptr(ptr: SEXP) -> SEXP {
    // Consumer doesn't know or care what type this is - just passes it through
    ptr
}

/// Check if a SEXP is an ExternalPtr (type-agnostic check)
///
/// @param sexp Any R object
/// @return TRUE if it's an ExternalPtr
/// @export
#[miniextendr]
fn is_external_ptr(sexp: SEXP) -> bool {
    use miniextendr_api::ffi::{TYPEOF, SEXPTYPE};
    unsafe { TYPEOF(sexp) == SEXPTYPE::EXTPTRSXP }
}

// ============================================================================
// Consumer's own Counter implementation: DoubleCounter
// Increments by 2 instead of 1, to prove it's a different implementation
// ============================================================================

/// A counter that increments by 2 (consumer's own implementation)
#[derive(ExternalPtr)]
pub struct DoubleCounter {
    value: i32,
}

#[miniextendr]
impl DoubleCounter {
    fn create(initial: i32) -> Self {
        Self { value: initial }
    }

    /// Get the current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl Counter for DoubleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 2; // Increments by 2, not 1!
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }
}

/// Create a new DoubleCounter (consumer's own Counter implementation)
///
/// DoubleCounter increments by 2, demonstrating a different implementation
/// that can still be used by producer.pkg's trait-generic functions.
///
/// @param initial Initial counter value
/// @return An external pointer to the wrapped DoubleCounter
/// @export
#[miniextendr]
fn new_double_counter(initial: i32) -> SEXP {
    let counter = DoubleCounter::create(initial);
    let erased = __mx_wrap_doublecounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}

// ============================================================================
// Generic functions working with Counter trait
// ============================================================================

/// Increment a counter twice (generic over Counter trait)
///
/// This function works with Counter objects from ANY package,
/// not just consumer.pkg. The trait dispatch happens via vtables.
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's value after incrementing twice
/// @export
#[miniextendr]
fn increment_twice(counter_sexp: SEXP) -> i32 {
    let mut counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.increment();
    counter.increment();
    counter.value()
}

/// Add a value to a counter and return the new value
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param n Value to add
/// @return The counter's value after adding n
/// @export
#[miniextendr]
fn add_and_get(counter_sexp: SEXP, n: i32) -> i32 {
    let mut counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.add(n);
    counter.value()
}

/// Get value without modifying (uses immutable reference)
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's current value
/// @export
#[miniextendr]
fn peek_value(counter_sexp: SEXP) -> i32 {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.value()
}

/// Check if an object implements the Counter trait
///
/// @param sexp Any R object
/// @return TRUE if the object implements Counter trait
/// @export
#[miniextendr]
fn is_counter(sexp: SEXP) -> bool {
    unsafe { CounterView::try_from_sexp(sexp).is_some() }
}

/// Greet with a message demonstrating consumer package is working
///
/// @param name Name to greet
/// @return A greeting string
/// @export
#[miniextendr]
fn consumer_greet(name: String) -> String {
    format!("Hello {} from consumer.pkg!", name)
}

/// Return a constant to verify the package is loaded
///
/// @return The number 42
/// @export
#[miniextendr]
fn consumer_magic_number() -> i32 {
    42
}

/// Debug: Get TAG_COUNTER as hex string
/// @export
#[miniextendr]
fn debug_consumer_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_COUNTER.hi, TAG_COUNTER.lo)
}

miniextendr_module! {
    mod consumer_pkg;
    fn passthrough_ptr;
    fn is_external_ptr;
    impl DoubleCounter;
    impl Counter for DoubleCounter;
    fn new_double_counter;
    fn increment_twice;
    fn add_and_get;
    fn peek_value;
    fn is_counter;
    fn consumer_greet;
    fn consumer_magic_number;
    fn debug_consumer_tag_counter;
}
