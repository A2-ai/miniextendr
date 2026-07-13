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

// env pinned: the env-style trait impl below attaches via `Type$Trait <- ...`,
// which needs an environment-creating inherent impl (breaks under s7-default).
#[miniextendr(env)]
impl CounterTraitEnv {
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

// env pinned: this is the deliberate Env cell of the class-system matrix, so it
// must stay env regardless of the default-class-system feature (it pairs with the
// env inherent impl above). (#1115's r6 wrapper-name collision is fixed —
// r6 trait wrappers are class-scoped now — so a flip would no longer collide.)
#[miniextendr(env)]
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

#[miniextendr(s4, internal)]
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
#[miniextendr(s4, internal)]
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

#[miniextendr(s7, internal)]
impl CounterTraitS7 {
    /// @param v Initial counter value.
    pub fn new(v: i32) -> Self {
        Self { value: v }
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s7, internal)]
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

// env pinned: matches CounterTraitEnv's env inherent impl (the matrix's Env
// cell), so `CounterTraitEnv$StaticXParam$from_value` attaches to the env object.
#[miniextendr(env)]
impl StaticXParam for CounterTraitEnv {
    fn from_value(x: i32) -> i32 {
        x * 2
    }
}
// endregion

// region: trait-method-emitter regression (audit/2026-07-03-dogfooding-macros-codegen.md #1)
//
// `scale`'s `x_factor` param used to be corrupted by the S4/S7/R6 trait
// generators' receiver-ptr extraction: they built the `.Call()` invocation
// assuming self="x", then ran `call.replace(", x", ", .ptr")`. `str::replace`
// rewrites *every* match of that substring, so a parameter whose R name
// starts with `x` was silently rewritten too (`x_factor` -> `.ptr_factor`),
// producing a runtime "object '.ptr_factor' not found" error. `bump`
// exercises the `stopifnot()` precondition-check prelude step trait methods
// used to skip entirely; `set_mode`'s `choices(...)` param exercises
// `match.arg()` support, which trait methods had none of at all before this
// refactor. See `TraitMethodContext` (miniextendr_impl_trait/method_context.rs).

#[miniextendr]
pub trait Scaler {
    /// Regression for the `x`-prefixed substring-corruption bug (BUG1).
    fn scale(&mut self, x_factor: f64) -> f64;
    /// Regression for the missing `stopifnot()` precondition prelude (BUG2).
    fn bump(&mut self, amount: i32) -> f64;
    /// Regression for trait methods having no `match_arg`/`choices` support
    /// at all (BUG2). `choices(...)` is set per-impl (see below), matching
    /// how `#[miniextendr(...)]` attributes work on `impl Trait for Type`
    /// method bodies rather than the trait declaration.
    fn set_mode(&mut self, mode: String) -> String;
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct ScalerS4 {
    value: f64,
    mode: String,
}

#[miniextendr(s4, internal)]
impl ScalerS4 {
    pub fn new(v: f64) -> Self {
        Self {
            value: v,
            mode: "fast".to_string(),
        }
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[miniextendr(s4, internal)]
impl Scaler for ScalerS4 {
    fn scale(&mut self, x_factor: f64) -> f64 {
        self.value *= x_factor;
        self.value
    }
    fn bump(&mut self, amount: i32) -> f64 {
        self.value += f64::from(amount);
        self.value
    }
    #[miniextendr(choices(mode = "fast, slow"))]
    fn set_mode(&mut self, mode: String) -> String {
        self.mode = mode.clone();
        mode
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct ScalerS7 {
    value: f64,
    mode: String,
}

#[miniextendr(s7, internal)]
impl ScalerS7 {
    pub fn new(v: f64) -> Self {
        Self {
            value: v,
            mode: "fast".to_string(),
        }
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[miniextendr(s7, internal)]
impl Scaler for ScalerS7 {
    fn scale(&mut self, x_factor: f64) -> f64 {
        self.value *= x_factor;
        self.value
    }
    fn bump(&mut self, amount: i32) -> f64 {
        self.value += f64::from(amount);
        self.value
    }
    #[miniextendr(choices(mode = "fast, slow"))]
    fn set_mode(&mut self, mode: String) -> String {
        self.mode = mode.clone();
        mode
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct ScalerR6 {
    value: f64,
    mode: String,
}

#[miniextendr(r6)]
impl ScalerR6 {
    pub fn new(v: f64) -> Self {
        Self {
            value: v,
            mode: "fast".to_string(),
        }
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[miniextendr(r6)]
impl Scaler for ScalerR6 {
    fn scale(&mut self, x_factor: f64) -> f64 {
        self.value *= x_factor;
        self.value
    }
    fn bump(&mut self, amount: i32) -> f64 {
        self.value += f64::from(amount);
        self.value
    }
    #[miniextendr(choices(mode = "fast, slow"))]
    fn set_mode(&mut self, mode: String) -> String {
        self.mode = mode.clone();
        mode
    }
}
// endregion

// region: Module registration
// endregion
