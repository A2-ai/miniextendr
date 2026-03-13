// Consumer package: Works with Counter objects from other packages
//
// This demonstrates cross-package trait dispatch:
// - Has the same Counter trait definition (ABI-compatible with producer)
// - Provides generic functions that work with any Counter implementation
// - Can operate on objects created by producer package

use miniextendr_api::{miniextendr, externalptr::ErasedExternalPtr, ffi::SEXP};

// region: Shared trait definition (must match producer exactly)

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}
// endregion

// region: Generic functions working with Counter trait

/// Increment a counter twice (generic over Counter trait)
///
/// This function works with Counter objects from ANY package,
/// not just consumer.pkg. The trait dispatch happens via vtables.
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's value after incrementing twice
#[miniextendr]
fn increment_twice(counter_sexp: SEXP) -> i32 {
    unsafe {
        let mut counter_ptr = ErasedExternalPtr::from_sexp(counter_sexp);

        // Downcast to trait object via type tag matching
        let counter = counter_ptr.downcast_trait_mut::<dyn Counter>()
            .expect("Object does not implement Counter trait");

        counter.increment();
        counter.increment();
        counter.value()
    }
}

/// Add a value to a counter and return the new value
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param n Value to add
/// @return The counter's value after adding n
#[miniextendr]
fn add_and_get(counter_sexp: SEXP, n: i32) -> i32 {
    unsafe {
        let mut counter_ptr = ErasedExternalPtr::from_sexp(counter_sexp);

        let counter = counter_ptr.downcast_trait_mut::<dyn Counter>()
            .expect("Object does not implement Counter trait");

        counter.add(n);
        counter.value()
    }
}

/// Get value without modifying (uses immutable reference)
///
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's current value
#[miniextendr]
fn peek_value(counter_sexp: SEXP) -> i32 {
    unsafe {
        let counter_ptr = ErasedExternalPtr::from_sexp(counter_sexp);

        let counter = counter_ptr.downcast_trait_ref::<dyn Counter>()
            .expect("Object does not implement Counter trait");

        counter.value()
    }
}
// endregion
