//! Class System Matrix Tests
//!
//! This module tests all combinations of class systems for trait impls:
//! - Inherent impl: Always Env (standard pattern using ExternalPtr)
//! - Trait impls: Env, S3, S4, S7, R6
//!
//! Each type is named: CounterTrait{Style}
//! For example: CounterTraitS3 has Env inherent impl and S3 trait impl.
//!
//! Note: The inherent impl class system (S3, S4, S7, R6) controls how the
//! constructor and methods are exposed. The trait impl class system controls
//! how trait methods are exposed. Not all combinations are compatible:
//! - S4/S7/R6 inherent impls don't create environments, so Env-style trait impls
//!   can't attach Type$Trait$method() patterns to them.
//! - For maximum compatibility, use Env inherent impl with any trait impl style.

use miniextendr_api::miniextendr;

// region: Shared trait for all counter types

#[miniextendr]
pub trait MatrixCounter {
    fn custom_get(&self) -> i32;
    fn custom_add(&mut self, n: i32);
    fn default_value() -> i32;
}
// endregion

// region: Env inherent impl × Env trait impl

#[derive(miniextendr_api::ExternalPtr)]
pub struct CounterTraitEnv {
    value: i32,
}

#[miniextendr]
impl CounterTraitEnv {
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl MatrixCounter for CounterTraitEnv {
    fn custom_get(&self) -> i32 {
        self.value
    }
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        1
    }
}
// endregion

// region: Env inherent impl × S3 trait impl

#[derive(miniextendr_api::ExternalPtr)]
pub struct CounterTraitS3 {
    value: i32,
}

#[miniextendr(s3)]
impl CounterTraitS3 {
    /// @param v Initial counter value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// @rdname CounterTraitS3
#[miniextendr(s3)]
impl MatrixCounter for CounterTraitS3 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    /// @param n Amount to add.
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        2
    }
}
// endregion

// region: Env inherent impl × S4 trait impl

#[derive(miniextendr_api::ExternalPtr)]
pub struct CounterTraitS4 {
    value: i32,
}

#[miniextendr(s4)]
impl CounterTraitS4 {
    /// @param v Initial counter value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// @rdname CounterTraitS4
/// @aliases s4_trait_MatrixCounter_custom_get,CounterTraitS4-method s4_trait_MatrixCounter_custom_add,CounterTraitS4-method
/// @param x A CounterTraitS4 object.
#[miniextendr(s4)]
impl MatrixCounter for CounterTraitS4 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    /// @param n Amount to add.
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        3
    }
}
// endregion

// region: Env inherent impl × S7 trait impl

#[derive(miniextendr_api::ExternalPtr)]
pub struct CounterTraitS7 {
    value: i32,
}

#[miniextendr(s7)]
impl CounterTraitS7 {
    /// @param v Initial counter value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s7)]
impl MatrixCounter for CounterTraitS7 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        4
    }
}
// endregion

// region: Env inherent impl × R6 trait impl

#[derive(miniextendr_api::ExternalPtr)]
pub struct CounterTraitR6 {
    value: i32,
}

#[miniextendr(r6)]
impl CounterTraitR6 {
    /// Create a new counter with initial value.
    /// @param v Initial counter value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    /// Get the current counter value.
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// @rdname CounterTraitR6
#[miniextendr(r6)]
impl MatrixCounter for CounterTraitR6 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    /// @param n Amount to add.
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        5
    }
}
// endregion

// region: Static method with first param named 'x' (regression test for dispatch)

/// Trait with a static method whose first param is `x`.
/// The old formals heuristic would misclassify this as an instance method.
#[miniextendr]
pub trait StaticXParam {
    fn from_value(x: i32) -> i32;
}

#[miniextendr]
impl StaticXParam for CounterTraitEnv {
    fn from_value(x: i32) -> i32 {
        x * 2
    }
}
// endregion

// region: Module registration
// endregion
