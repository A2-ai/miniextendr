//! Trait ABI tests for cross-package trait dispatch.
//!
//! This module tests `#[miniextendr]` on traits and trait implementations,
//! plus `` wiring.

use miniextendr_api::miniextendr;

// region: Define a trait with #[miniextendr]

/// Counter trait for testing cross-type trait ABI dispatch.
#[miniextendr]
pub trait Counter {
    /// Associated constant: Maximum value for this counter type.
    const MAX_VALUE: i32;

    /// Get the current value.
    fn value(&self) -> i32;

    /// Increment the counter by 1.
    fn increment(&mut self);

    /// Add n to the counter (checked - may panic on overflow).
    fn checked_add(&mut self, n: i32);

    /// Static method: Returns the default initial value for this counter type.
    fn default_initial() -> i32;
}
// endregion

// region: Implement the trait for a concrete type

/// A simple counter implementation.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for SimpleCounter (default dispatch).
// Bare (default class system): #1115 is fixed — r6 trait wrappers are now
// class-scoped (`Type$Trait$method`), so this flipping to r6 under r6-default
// no longer collides with R6TraitCounter's explicit r6 Counter impl.
#[miniextendr(env)]
impl Counter for SimpleCounter {
    const MAX_VALUE: i32 = i32::MAX;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        0 // SimpleCounter defaults to 0
    }
}
// endregion

// region: R-exposed functions for testing (using impl block pattern)

#[miniextendr(env)]
impl SimpleCounter {
    /// Create a new counter with given initial value.
    fn new_counter(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get the counter's value.
    fn get_value(&self) -> i32 {
        self.value
    }

    /// Add to the counter using the trait method.
    fn trait_add(&mut self, n: i32) {
        // Call the trait method
        Counter::checked_add(self, n);
    }
}
// endregion

// region: Test panic handling in shims - separate struct

/// A counter that panics when you try to decrement below zero.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PanickyCounter {
    value: i32,
}

impl PanickyCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

// Bare (default class system): #1115 is fixed — r6 trait wrappers are now
// class-scoped (`Type$Trait$method`), so this flipping to r6 under r6-default
// no longer collides with R6TraitCounter's explicit r6 Counter impl.
#[miniextendr(env)]
impl Counter for PanickyCounter {
    const MAX_VALUE: i32 = 1000; // PanickyCounter has a lower max

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn checked_add(&mut self, n: i32) {
        if self.value + n < 0 {
            panic!(
                "PanickyCounter: cannot go below zero! (value={}, n={})",
                self.value, n
            );
        }
        self.value += n;
    }

    fn default_initial() -> i32 {
        100 // PanickyCounter defaults to 100 (safe margin above 0)
    }
}

#[miniextendr(env)]
impl PanickyCounter {
    /// Create a new panicky counter.
    fn new_panicky(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value.
    fn get_value(&self) -> i32 {
        self.value
    }

    /// Add (may panic).
    fn try_add(&mut self, n: i32) {
        Counter::checked_add(self, n);
    }
}
// endregion

// region: S3 trait impl test - uses S3 dispatch for trait methods

/// A counter that uses S3 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3TraitCounter {
    value: i32,
}

impl S3TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S3TraitCounter (S3 dispatch).
#[miniextendr(s3)]
impl Counter for S3TraitCounter {
    const MAX_VALUE: i32 = 500;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        50 // S3TraitCounter defaults to 50
    }
}

#[miniextendr(s3)]
impl S3TraitCounter {
    /// Create a new S3 trait counter.
    fn new_s3trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: S4 trait impl test - uses S4 dispatch for trait methods

/// A counter that uses S4 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4TraitCounter {
    value: i32,
}

impl S4TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S4TraitCounter (S4 dispatch).
#[miniextendr(s4, internal)]
impl Counter for S4TraitCounter {
    const MAX_VALUE: i32 = 400;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        40 // S4TraitCounter defaults to 40
    }
}

/// S4TraitCounter inherent methods (constructor and getters).
#[miniextendr(s4, internal)]
impl S4TraitCounter {
    /// Create a new S4 trait counter.
    fn new_s4trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: S7 trait impl test - uses S7 dispatch for trait methods

/// A counter that uses S7 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7TraitCounter {
    value: i32,
}

impl S7TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S7TraitCounter (S7 dispatch).
#[miniextendr(s7, internal)]
impl Counter for S7TraitCounter {
    const MAX_VALUE: i32 = 300;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        30 // S7TraitCounter defaults to 30
    }
}

/// S7TraitCounter inherent methods (constructor and getters).
#[miniextendr(s7, internal)]
impl S7TraitCounter {
    /// Create a new S7 trait counter.
    fn new_s7trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: R6 trait impl test - uses R6 dispatch for trait methods

/// A counter that uses R6 dispatch (standalone functions) for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6TraitCounter {
    value: i32,
}

impl R6TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for R6TraitCounter (R6 dispatch).
#[miniextendr(r6)]
impl Counter for R6TraitCounter {
    const MAX_VALUE: i32 = 200;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        20 // R6TraitCounter defaults to 20
    }
}

/// R6TraitCounter inherent methods (constructor and getters).
#[miniextendr(r6)]
impl R6TraitCounter {
    /// Create a new R6 trait counter.
    pub fn new_r6trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    pub fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: Self-reference arguments (#1990)

/// Trait whose methods take another value of the implementing type.
#[miniextendr]
pub trait SelfArgs {
    /// Copy this value into `other`.
    fn copy_to(&self, other: &mut Self);

    /// Swap values with `other`.
    fn swap_with(&mut self, other: &mut Self);

    /// This value minus `type`'s.
    fn difference(&self, r#type: &Self) -> i32;

    /// Swap the values of `left` and `right`.
    fn exchange(left: &mut Self, right: &mut Self);
}

/// A counter whose trait methods take other counters.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SelfArgCounter {
    value: i32,
}

/// SelfArgs for SelfArgCounter: `&Self` / `&mut Self` arguments, one of
/// them raw-named, through the generated C wrappers.
#[miniextendr(env)]
impl SelfArgs for SelfArgCounter {
    /// @param other The counter to overwrite.
    fn copy_to(&self, other: &mut Self) {
        other.value = self.value;
    }

    /// @param other The counter to swap with.
    fn swap_with(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.value, &mut other.value);
    }

    /// @param type The counter to subtract.
    fn difference(&self, r#type: &Self) -> i32 {
        self.value - r#type.value
    }

    /// @param left,right The counters to swap.
    fn exchange(left: &mut Self, right: &mut Self) {
        std::mem::swap(&mut left.value, &mut right.value);
    }
}

/// SelfArgCounter inherent methods (constructor and getter).
#[miniextendr(env)]
impl SelfArgCounter {
    /// Create a counter holding `value`.
    /// @param value The initial value.
    fn new_self_arg(value: i32) -> Self {
        Self { value }
    }

    /// The counter's value.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: Module registration
// endregion

// region: Trait names that differ only in case (#1778)

/// A gauge read through two traits whose names differ only in case.
#[derive(miniextendr_api::ExternalPtr)]
pub struct CaseGauge {
    value: i32,
}

/// CaseGauge inherent methods (constructor).
#[miniextendr(env)]
impl CaseGauge {
    /// Create a gauge holding `value`.
    /// @param value The reading.
    fn new_case_gauge(value: i32) -> Self {
        Self { value }
    }
}

// `Gauge` and `gauge` sit in sibling modules, as two crates' traits would.
// Their tags, vtable builders, `#[no_mangle]` vtable statics and
// registrations keep the identifiers' case, so both impls link and dispatch.
mod case_upper {
    use super::CaseGauge;
    use miniextendr_api::miniextendr;

    /// Read a gauge.
    #[miniextendr]
    pub trait Gauge {
        /// The reading.
        fn reading(&self) -> i32;
    }

    /// Gauge for CaseGauge: the stored reading.
    #[miniextendr(env)]
    impl Gauge for CaseGauge {
        fn reading(&self) -> i32 {
            self.value
        }
    }
}

mod case_lower {
    use super::CaseGauge;
    use miniextendr_api::miniextendr;

    /// Read a gauge, negated.
    #[allow(non_camel_case_types)]
    #[miniextendr]
    pub trait gauge {
        /// The negated reading.
        fn reading(&self) -> i32;
    }

    /// gauge for CaseGauge: the reading negated.
    #[miniextendr(env)]
    impl gauge for CaseGauge {
        fn reading(&self) -> i32 {
            -self.value
        }
    }
}
// endregion
