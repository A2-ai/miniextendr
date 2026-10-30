//! How a class gets its constructor, one type per class system for each case:
//!
//! - `Infer*`: the one receiver-less method returning `Self` is the
//!   constructor, whatever its name (`from_value`). `parse` returns
//!   `Result<Self, String>`, which does not make it a candidate: it stays a
//!   static method.
//! - `Marked*`: two methods return `Self`; the one marked
//!   `#[miniextendr(<system>(constructor))]` is the constructor and `zero` is
//!   a static method.
//! - `NoCtor*`: no method returns bare `Self` (`of` returns
//!   `Result<Self, String>`, `parse` `Option<Self>`), so the class has no
//!   constructor. Objects come from those static factories. R6's `T$new()`
//!   and S7's `T()` raise an error without a `.ptr`; env leaves `T$new`
//!   `NULL`, and S4 and vctrs define no `T()` or `new_t()` (#2069). S3
//!   defines neither `new_t()` nor its class environment `T`, which nothing
//!   would land in (#2264).
//! - `NoCtorTraitS3`: an S3 class without a constructor whose trait impl
//!   has a static method, `NoCtorTraitS3$MatrixCounter$default_value()`: its
//!   class environment is kept for that namespace (#2264).
//! - `SplitInfer*`: the methods are in a first impl block and the one
//!   candidate in a second; the decision is made for the type, across its
//!   blocks (#1776).
//! - `NoCtorRec`: an S3 class the package names (`class = "noctor_rec"`)
//!   and builds with its factories. Its impl docs get a page named after the
//!   class, `noctor_rec`, titled by default. (`handmade_rec` in
//!   `rdata_sidecar_tests.rs` puts them on the package's own page with
//!   `@rdname`.)
//! - `NoCtorVctrs`: a vctrs class without a constructor, whose vectors the
//!   package builds with `vctrs::new_vctr()`.
//!
//! A constructor not named `new` keeps its own name too, as a static method
//! (`InferEnv$from_value()`, `infers3_from_value()`, ...).
//!
//! The `Split*` types in `split_impl_tests.rs` cover the other cross-block
//! case: a `new` in the first block keeps the second block's `from_half` a
//! static method.

use miniextendr_api::{ExternalPtr, miniextendr};

// region: env

/// An env class whose constructor is its one method returning `Self`.
#[derive(ExternalPtr)]
pub struct InferEnv {
    value: i32,
}

/// Env class with an inferred constructor.
#[miniextendr(env)]
impl InferEnv {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }

    /// Parse an object from text.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.trim()
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("cannot parse {text:?}: {e}"))
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An env class with two methods returning `Self`, one marked as its
/// constructor.
#[derive(ExternalPtr)]
pub struct MarkedEnv {
    value: i32,
}

/// Env class with a marked constructor.
#[miniextendr(env)]
impl MarkedEnv {
    /// Create an object holding `value`.
    /// @param value Starting value.
    #[miniextendr(env(constructor))]
    pub fn with_value(value: i32) -> Self {
        Self { value }
    }

    /// An object holding zero.
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An env class without a constructor.
#[derive(ExternalPtr)]
pub struct NoCtorEnv {
    value: i32,
}

/// Env class whose objects come from its factories.
#[miniextendr(env)]
impl NoCtorEnv {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// An object parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An env class whose inferred constructor is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitInferEnv {
    value: i32,
}

#[miniextendr(env)]
impl SplitInferEnv {
    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// Env class with an inferred constructor in its second impl block.
#[miniextendr(env)]
impl SplitInferEnv {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }
}
// endregion

// region: R6

/// An R6 class whose constructor is its one method returning `Self`.
#[derive(ExternalPtr)]
pub struct InferR6 {
    value: i32,
}

/// R6 class with an inferred constructor.
#[miniextendr(r6)]
impl InferR6 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }

    /// Parse an object from text.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.trim()
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("cannot parse {text:?}: {e}"))
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An R6 class with two methods returning `Self`, one marked as its
/// constructor.
#[derive(ExternalPtr)]
pub struct MarkedR6 {
    value: i32,
}

/// R6 class with a marked constructor.
#[miniextendr(r6)]
impl MarkedR6 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    #[miniextendr(r6(constructor))]
    pub fn with_value(value: i32) -> Self {
        Self { value }
    }

    /// An object holding zero.
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An R6 class without a constructor.
#[derive(ExternalPtr)]
pub struct NoCtorR6 {
    value: i32,
}

/// R6 class whose objects come from its factories.
#[miniextendr(r6)]
impl NoCtorR6 {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// An object parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// An R6 class whose inferred constructor is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitInferR6 {
    value: i32,
}

#[miniextendr(r6)]
impl SplitInferR6 {
    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// R6 class with an inferred constructor in its second impl block.
#[miniextendr(r6)]
impl SplitInferR6 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }
}
// endregion

// region: S3

/// An S3 class whose constructor is its one method returning `Self`.
#[derive(ExternalPtr)]
pub struct InferS3 {
    value: i32,
}

/// S3 class with an inferred constructor.
#[miniextendr(s3)]
impl InferS3 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }

    /// Parse an object from text.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.trim()
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("cannot parse {text:?}: {e}"))
    }

    /// The current value.
    pub fn infer3_value(&self) -> i32 {
        self.value
    }
}

/// An S3 class with two methods returning `Self`, one marked as its
/// constructor.
#[derive(ExternalPtr)]
pub struct MarkedS3 {
    value: i32,
}

/// S3 class with a marked constructor.
#[miniextendr(s3)]
impl MarkedS3 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    #[miniextendr(s3(constructor))]
    pub fn with_value(value: i32) -> Self {
        Self { value }
    }

    /// An object holding zero.
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// The current value.
    pub fn marked3_value(&self) -> i32 {
        self.value
    }
}

/// An S3 class without a constructor.
#[derive(ExternalPtr)]
pub struct NoCtorS3 {
    value: i32,
}

/// S3 class whose objects come from its factories.
#[miniextendr(s3)]
impl NoCtorS3 {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// An object parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The current value.
    pub fn noctor3_value(&self) -> i32 {
        self.value
    }
}

/// An S3 class without a constructor, with a trait impl's static method.
#[derive(ExternalPtr)]
pub struct NoCtorTraitS3 {
    value: i32,
}

/// S3 class whose objects come from its factory, with a trait namespace.
#[miniextendr(s3)]
impl NoCtorTraitS3 {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }
}

// The static `default_value()` lands in `NoCtorTraitS3$MatrixCounter`, which
// keeps the class environment.
#[miniextendr(s3)]
impl crate::class_system_matrix::MatrixCounter for NoCtorTraitS3 {
    fn custom_get(&self) -> i32 {
        self.value
    }
    /// @param n Amount to add.
    fn custom_add(&mut self, n: i32) {
        self.value += n;
    }
    fn default_value() -> i32 {
        13
    }
}

/// An S3 class the package names and builds with its factories.
#[derive(ExternalPtr)]
pub struct NoCtorRec {
    value: i32,
}

/// Records of one value, built by `noctor_rec_of()` and `noctor_rec_parse()`.
#[miniextendr(s3, class = "noctor_rec")]
impl NoCtorRec {
    /// A record holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// A record parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The record's value.
    pub fn noctor_rec_value(&self) -> i32 {
        self.value
    }
}

/// An S3 class whose inferred constructor is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitInferS3 {
    value: i32,
}

#[miniextendr(s3)]
impl SplitInferS3 {
    /// The current value.
    pub fn splitinfer3_value(&self) -> i32 {
        self.value
    }
}

/// S3 class with an inferred constructor in its second impl block.
#[miniextendr(s3)]
impl SplitInferS3 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }
}
// endregion

// region: S4

/// An S4 class whose constructor is its one method returning `Self`.
#[derive(ExternalPtr)]
pub struct InferS4 {
    value: i32,
}

/// S4 class with an inferred constructor.
#[miniextendr(s4)]
impl InferS4 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }

    /// Parse an object from text.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.trim()
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("cannot parse {text:?}: {e}"))
    }

    /// The current value.
    pub fn infer4_value(&self) -> i32 {
        self.value
    }
}

/// An S4 class with two methods returning `Self`, one marked as its
/// constructor.
#[derive(ExternalPtr)]
pub struct MarkedS4 {
    value: i32,
}

/// S4 class with a marked constructor.
#[miniextendr(s4)]
impl MarkedS4 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    #[miniextendr(s4(constructor))]
    pub fn with_value(value: i32) -> Self {
        Self { value }
    }

    /// An object holding zero.
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// The current value.
    pub fn marked4_value(&self) -> i32 {
        self.value
    }
}

/// An S4 class without a constructor.
#[derive(ExternalPtr)]
pub struct NoCtorS4 {
    value: i32,
}

/// S4 class whose objects come from its factories.
#[miniextendr(s4)]
impl NoCtorS4 {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// An object parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The current value.
    pub fn noctor4_value(&self) -> i32 {
        self.value
    }
}

/// An S4 class whose inferred constructor is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitInferS4 {
    value: i32,
}

#[miniextendr(s4)]
impl SplitInferS4 {
    /// The current value.
    pub fn splitinfer4_value(&self) -> i32 {
        self.value
    }
}

/// S4 class with an inferred constructor in its second impl block.
#[miniextendr(s4)]
impl SplitInferS4 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }
}
// endregion

// region: S7

/// An S7 class whose constructor is its one method returning `Self`.
#[derive(ExternalPtr)]
pub struct InferS7 {
    value: i32,
}

/// S7 class with an inferred constructor.
#[miniextendr(s7)]
impl InferS7 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }

    /// Parse an object from text.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.trim()
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("cannot parse {text:?}: {e}"))
    }

    /// The current value.
    pub fn infer7_value(&self) -> i32 {
        self.value
    }
}

/// An S7 class with two methods returning `Self`, one marked as its
/// constructor.
#[derive(ExternalPtr)]
pub struct MarkedS7 {
    value: i32,
}

/// S7 class with a marked constructor.
#[miniextendr(s7)]
impl MarkedS7 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    #[miniextendr(s7(constructor))]
    pub fn with_value(value: i32) -> Self {
        Self { value }
    }

    /// An object holding zero.
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// The current value.
    pub fn marked7_value(&self) -> i32 {
        self.value
    }
}

/// An S7 class without a constructor.
#[derive(ExternalPtr)]
pub struct NoCtorS7 {
    value: i32,
}

/// S7 class whose objects come from its factories.
#[miniextendr(s7)]
impl NoCtorS7 {
    /// An object holding `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self { value })
    }

    /// An object parsed from `text`; an error when it is no integer.
    /// @param text An integer, as text.
    pub fn parse(text: &str) -> Option<Self> {
        text.trim().parse().ok().map(|value| Self { value })
    }

    /// The current value.
    pub fn noctor7_value(&self) -> i32 {
        self.value
    }
}

/// An S7 class whose inferred constructor is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitInferS7 {
    value: i32,
}

#[miniextendr(s7)]
impl SplitInferS7 {
    /// The current value.
    pub fn splitinfer7_value(&self) -> i32 {
        self.value
    }
}

/// S7 class with an inferred constructor in its second impl block.
#[miniextendr(s7)]
impl SplitInferS7 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn from_value(value: i32) -> Self {
        Self { value }
    }
}
// endregion

// region: vctrs

/// A vctrs class without a constructor.
pub struct NoCtorVctrs;

/// vctrs class whose vectors the package builds with `vctrs::new_vctr()`.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "noctor"))]
impl NoCtorVctrs {
    /// The sum of `amounts`.
    /// @param amounts Numeric amounts.
    pub fn total(amounts: Vec<f64>) -> f64 {
        amounts.iter().sum()
    }
}
// endregion
