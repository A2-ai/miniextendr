//! Regression fixtures for #1115 (r6 trait-wrapper name collision) and #1141
//! resp-4 (trait `-> Self` re-wrapping).
//!
//! # #1115 — two r6 impls of one trait
//!
//! Before #1141/#1115, `#[miniextendr(r6)]` trait *instance* methods were
//! emitted as unqualified standalone `r6_trait_<Trait>_<method>(x)` functions.
//! Two r6 impls of the *same* trait on *different* types therefore emitted
//! identical wrapper names, and the duplicate-definition guard in
//! `miniextendr-api/src/registry.rs` aborted wrapper-gen (i.e. the install).
//! They now live in the class-scoped `Type$Trait$method` namespace, which is
//! collision-free by construction — `DoublerA$Doubler$doubled` vs
//! `DoublerB$Doubler$doubled`. `DoublerA` + `DoublerB` below are the two
//! colliding r6 impls; if they install and dispatch, #1115 is fixed.
//!
//! # #1141 resp-4 — `-> Self` re-wrapping
//!
//! `Doubler::duplicate(&self) -> Self` (instance) and `Doubler::spawn(v) -> Self`
//! (static) return a bare `ExternalPtr` from the C wrapper. The R wrapper must
//! re-wrap it into a properly classed object, exactly as inherent constructors
//! do. `DoublerEnv` additionally exercises the env-class re-wrap path
//! (`class(.val) <- "DoublerEnv"`) at runtime, alongside the r6 path
//! (`DoublerA$new(.ptr = .val)`).

use miniextendr_api::miniextendr;

// region: Shared trait

/// A trait whose factory methods return `Self` (regression for #1141 resp-4)
/// and which is implemented by two distinct r6 types (regression for #1115).
#[miniextendr]
pub trait Doubler {
    /// Instance method returning a plain scalar.
    fn doubled(&self) -> i32;
    /// Instance factory returning `Self` — must be re-wrapped into a classed object.
    fn duplicate(&self) -> Self;
    /// Static factory returning `Self` — must be re-wrapped into a classed object.
    fn spawn(v: i32) -> Self;
}
// endregion

// region: First r6 impl of Doubler

#[derive(miniextendr_api::ExternalPtr)]
pub struct DoublerA {
    value: i32,
}

#[miniextendr(r6)]
impl DoublerA {
    /// Create a new DoublerA.
    /// @param v Initial value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    /// Get the stored value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(r6)]
impl Doubler for DoublerA {
    fn doubled(&self) -> i32 {
        self.value * 2
    }
    fn duplicate(&self) -> Self {
        Self { value: self.value }
    }
    fn spawn(v: i32) -> Self {
        Self { value: v }
    }
}
// endregion

// region: Second r6 impl of the SAME trait on a DIFFERENT type (#1115 collision case)

#[derive(miniextendr_api::ExternalPtr)]
pub struct DoublerB {
    value: i32,
}

#[miniextendr(r6)]
impl DoublerB {
    /// Create a new DoublerB.
    /// @param v Initial value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    /// Get the stored value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(r6)]
impl Doubler for DoublerB {
    fn doubled(&self) -> i32 {
        self.value * 10
    }
    fn duplicate(&self) -> Self {
        Self { value: self.value }
    }
    fn spawn(v: i32) -> Self {
        Self { value: v }
    }
}
// endregion

// region: Env impl of Doubler (exercises the env-class `-> Self` re-wrap path)

#[derive(miniextendr_api::ExternalPtr)]
pub struct DoublerEnv {
    value: i32,
}

#[miniextendr(env)]
impl DoublerEnv {
    /// Create a new DoublerEnv.
    /// @param v Initial value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    /// Get the stored value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(env)]
impl Doubler for DoublerEnv {
    fn doubled(&self) -> i32 {
        self.value * 2
    }
    fn duplicate(&self) -> Self {
        Self { value: self.value }
    }
    fn spawn(v: i32) -> Self {
        Self { value: v }
    }
}
// endregion
