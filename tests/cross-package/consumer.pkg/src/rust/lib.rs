// Consumer package: Works with objects from producer package
//
// This demonstrates cross-package interoperability:
// - Receives objects with various class systems from producer
// - Tests that objects can pass through without type knowledge
// - Verifies trait dispatch works across package boundaries
// - Implements its own Counter (DoubleCounter) for bidirectional testing

use miniextendr_api::{ExternalPtr, ffi::SEXP, miniextendr, miniextendr_module, trait_abi::ccall};

// Import the shared Counter trait and its generated ABI types
pub use shared_traits::{__counter_build_vtable, Counter, CounterVTable, CounterView, TAG_COUNTER};

// ============================================================================
// Cross-package ExternalPtr pass-through utilities
// ============================================================================

/// Pass an ExternalPtr through consumer without knowing its type.
/// This tests that ExternalPtr can cross package boundaries as opaque SEXP.
/// @param ptr An ExternalPtr from any package
/// @return The same ExternalPtr (pass-through)
/// @export
#[miniextendr]
fn passthrough_ptr(ptr: SEXP) -> SEXP {
    ptr
}

/// Check if a SEXP is an ExternalPtr (type-agnostic check)
/// @param sexp Any R object
/// @return TRUE if it's an ExternalPtr
/// @export
#[miniextendr]
fn is_external_ptr(sexp: SEXP) -> bool {
    use miniextendr_api::ffi::{SEXPTYPE, TYPEOF};
    unsafe { TYPEOF(sexp) == SEXPTYPE::EXTPTRSXP }
}

/// Get class of any R object (for cross-package testing)
/// @param x Any R object
/// @return Character vector of class names
/// @export
#[miniextendr]
fn consumer_get_class(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::ffi::Rf_getAttrib(x, miniextendr_api::ffi::R_ClassSymbol) }
}

/// Check if an object has a specific class
/// @param x Any R object
/// @param class_name Class name to check for
/// @return TRUE if object has the specified class
/// @export
#[miniextendr]
fn has_class(x: SEXP, class_name: String) -> bool {
    use miniextendr_api::ffi::{
        R_CHAR, R_ClassSymbol, Rf_getAttrib, Rf_xlength, SEXPTYPE, STRING_ELT, TYPEOF,
    };
    unsafe {
        let class_attr = Rf_getAttrib(x, R_ClassSymbol);
        if TYPEOF(class_attr) != SEXPTYPE::STRSXP {
            return false;
        }
        let len = Rf_xlength(class_attr);
        for i in 0..len {
            let s = STRING_ELT(class_attr, i);
            let cstr = std::ffi::CStr::from_ptr(R_CHAR(s));
            if let Ok(name) = cstr.to_str()
                && name == class_name
            {
                return true;
            }
        }
        false
    }
}

// ============================================================================
// Consumer's own Counter implementation: DoubleCounter
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
/// DoubleCounter increments by 2, demonstrating a different implementation.
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
/// This function works with Counter objects from ANY package.
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
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's current value
/// @export
#[miniextendr]
fn peek_value(counter_sexp: SEXP) -> i32 {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.value()
}

/// Check if an object implements the Counter trait
/// @param sexp Any R object
/// @return TRUE if the object implements Counter trait
/// @export
#[miniextendr]
fn is_counter(sexp: SEXP) -> bool {
    unsafe { CounterView::try_from_sexp(sexp).is_some() }
}

// ============================================================================
// Simple utility functions for testing
// ============================================================================

/// Greet with a message demonstrating consumer package is working
/// @param name Name to greet
/// @return A greeting string
/// @export
#[miniextendr]
fn consumer_greet(name: String) -> String {
    format!("Hello {} from consumer.pkg!", name)
}

/// Return a constant to verify the package is loaded
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

    // Cross-package utilities
    fn passthrough_ptr;
    fn is_external_ptr;
    fn consumer_get_class;
    fn has_class;

    // DoubleCounter (consumer's own Counter implementation)
    impl DoubleCounter;
    impl Counter for DoubleCounter;
    fn new_double_counter;

    // Counter trait generic functions
    fn increment_twice;
    fn add_and_get;
    fn peek_value;
    fn is_counter;

    // Utility functions
    fn consumer_greet;
    fn consumer_magic_number;
    fn debug_consumer_tag_counter;
}
