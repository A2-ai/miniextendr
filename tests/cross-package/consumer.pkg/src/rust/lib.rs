// Consumer package: Works with objects from producer package
//
// This demonstrates cross-package interoperability:
// - Receives objects with various class systems from producer
// - Tests that objects can pass through without type knowledge
// - Verifies trait dispatch works across package boundaries
// - Implements its own Counter (DoubleCounter) for bidirectional testing

use miniextendr_api::{ExternalPtr, ffi::SEXP, miniextendr, trait_abi::ccall};

miniextendr_api::miniextendr_init!();

// Import the shared Counter trait and its generated ABI types
pub use shared_traits::{__counter_build_vtable, Counter, CounterVTable, CounterView, TAG_COUNTER};

// Import the shared Resettable trait and its generated ABI types
pub use shared_traits::{
    __resettable_build_vtable, Resettable, ResettableVTable, ResettableView, TAG_RESETTABLE,
};

// region: Cross-package ExternalPtr pass-through utilities

/// Pass an ExternalPtr through consumer without knowing its type.
/// This tests that ExternalPtr can cross package boundaries as opaque SEXP.
/// @param ptr An ExternalPtr from any package
/// @return The same ExternalPtr (pass-through)
/// @export
#[miniextendr]
pub fn passthrough_ptr(ptr: SEXP) -> SEXP {
    ptr
}

/// Check if a SEXP is an ExternalPtr (type-agnostic check)
/// @param sexp Any R object
/// @return TRUE if it's an ExternalPtr
/// @export
#[miniextendr]
pub fn is_external_ptr(sexp: SEXP) -> bool {
    use miniextendr_api::ffi::{SEXPTYPE, TYPEOF};
    unsafe { TYPEOF(sexp) == SEXPTYPE::EXTPTRSXP }
}

/// Get class of any R object (for cross-package testing)
/// @param x Any R object
/// @return Character vector of class names
/// @export
#[miniextendr]
pub fn consumer_get_class(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::ffi::Rf_getAttrib(x, miniextendr_api::ffi::R_ClassSymbol) }
}

/// Check if an object has a specific class
/// @param x Any R object
/// @param class_name Class name to check for
/// @return TRUE if object has the specified class
/// @export
#[miniextendr]
pub fn has_class(x: SEXP, class_name: String) -> bool {
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
// endregion

// region: Consumer's own Counter implementation: DoubleCounter

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
pub fn new_double_counter(initial: i32) -> SEXP {
    let counter = DoubleCounter::create(initial);
    let erased = __mx_wrap_doublecounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}
// endregion

// region: Generic functions working with Counter trait

/// Increment a counter twice (generic over Counter trait)
/// This function works with Counter objects from ANY package.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's value after incrementing twice
/// @export
#[miniextendr]
pub fn increment_twice(counter_sexp: SEXP) -> i32 {
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
pub fn add_and_get(counter_sexp: SEXP, n: i32) -> i32 {
    let mut counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.add(n);
    counter.value()
}

/// Get value without modifying (uses immutable reference)
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's current value
/// @export
#[miniextendr]
pub fn peek_value(counter_sexp: SEXP) -> i32 {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.value()
}

/// Check if an object implements the Counter trait
/// @param sexp Any R object
/// @return TRUE if the object implements Counter trait
/// @export
#[miniextendr]
pub fn is_counter(sexp: SEXP) -> bool {
    unsafe { CounterView::try_from_sexp(sexp).is_some() }
}
// endregion

// region: Generic functions working with Resettable trait

/// Reset an object and check if it's in default state
/// This function works with Resettable objects from ANY package.
/// @param sexp An ExternalPtr to any type implementing Resettable
/// @return TRUE if the object is in default state after reset
/// @export
#[miniextendr]
pub fn reset_and_check(sexp: SEXP) -> bool {
    let mut view = unsafe { ResettableView::from_sexp(sexp) };
    view.reset();
    view.is_default()
}

/// Check if an object is in its default state (without resetting)
/// @param sexp An ExternalPtr to any type implementing Resettable
/// @return TRUE if the object is in default state
/// @export
#[miniextendr]
pub fn check_is_default(sexp: SEXP) -> bool {
    let view = unsafe { ResettableView::from_sexp(sexp) };
    view.is_default()
}

/// Check if an object implements the Resettable trait
/// @param sexp Any R object
/// @return TRUE if the object implements Resettable trait
/// @export
#[miniextendr]
pub fn is_resettable(sexp: SEXP) -> bool {
    unsafe { ResettableView::try_from_sexp(sexp).is_some() }
}
// endregion

// region: Combined trait usage (Counter + Resettable on same object)

/// Increment a Counter twice, then reset it via Resettable, return is_default
/// Tests combined trait usage on the same object across packages.
/// @param sexp An ExternalPtr to a type implementing BOTH Counter and Resettable
/// @return TRUE if the object is in default state after increment+reset
/// @export
#[miniextendr]
pub fn increment_then_reset(sexp: SEXP) -> bool {
    // First use it as a Counter
    let mut counter = unsafe { CounterView::from_sexp(sexp) };
    counter.increment();
    counter.increment();

    // Then use it as a Resettable (same underlying object)
    let mut resettable = unsafe { ResettableView::from_sexp(sexp) };
    resettable.reset();
    resettable.is_default()
}

/// Get the value of a Counter, reset it, then get the value again
/// Returns the value after reset.
/// @param sexp An ExternalPtr to a type implementing BOTH Counter and Resettable
/// @return The counter value after reset
/// @export
#[miniextendr]
pub fn get_reset_get(sexp: SEXP) -> i32 {
    // Reset via Resettable
    let mut resettable = unsafe { ResettableView::from_sexp(sexp) };
    resettable.reset();

    // Read via Counter
    let counter = unsafe { CounterView::from_sexp(sexp) };
    counter.value()
}

/// Debug: Get TAG_RESETTABLE as hex string
/// @export
#[miniextendr]
pub fn debug_consumer_tag_resettable() -> String {
    format!("{:016x}{:016x}", TAG_RESETTABLE.hi, TAG_RESETTABLE.lo)
}
// endregion

// region: Simple utility functions for testing

/// Greet with a message demonstrating consumer package is working
/// @param name Name to greet
/// @return A greeting string
/// @export
#[miniextendr]
pub fn consumer_greet(name: String) -> String {
    format!("Hello {} from consumer.pkg!", name)
}

/// Return a constant to verify the package is loaded
/// @return The number 42
/// @export
#[miniextendr]
pub fn consumer_magic_number() -> i32 {
    42
}

/// Debug: Get TAG_COUNTER as hex string
/// @export
#[miniextendr]
pub fn debug_consumer_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_COUNTER.hi, TAG_COUNTER.lo)
}
// endregion
