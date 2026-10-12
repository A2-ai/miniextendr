//! Types whose methods are split across two `#[miniextendr]` impl blocks, one
//! type per class system (#1776). The first block of each type holds the class
//! doc comment and the constructor, the second adds methods; the wrapper
//! writer defines the class once and attaches the methods of both blocks.

use miniextendr_api::{ExternalPtr, IntoR, SEXP, miniextendr};

// region: env

/// An env class whose methods are split across two impl blocks.
#[derive(ExternalPtr)]
pub struct SplitEnv {
    value: i32,
}

/// Env class with methods in two impl blocks.
#[miniextendr(env)]
impl SplitEnv {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(env)]
impl SplitEnv {
    /// Add `by` to the value.
    /// @param by Amount to add.
    pub fn add(&mut self, by: i32) {
        self.value += by;
    }

    /// Twice the value.
    pub fn doubled(&self) -> i32 {
        self.value * 2
    }

    /// The value a new object starts from by default.
    pub fn origin() -> i32 {
        0
    }
}
// endregion

// region: R6

/// An R6 class whose methods are split across two impl blocks.
#[derive(ExternalPtr)]
pub struct SplitR6 {
    value: i32,
}

/// R6 class with methods in two impl blocks.
#[miniextendr(r6)]
impl SplitR6 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(r6)]
impl SplitR6 {
    /// Add `by` to the value.
    /// @param by Amount to add.
    pub fn add(&mut self, by: i32) {
        self.value += by;
    }

    /// Twice the value, as an active binding.
    #[miniextendr(r6(active))]
    pub fn doubled(&self) -> i32 {
        self.value * 2
    }

    /// The value minus one, as a private method.
    #[miniextendr(r6(private))]
    pub fn predecessor(&self) -> i32 {
        self.value - 1
    }

    /// Reset the value when the object is garbage-collected.
    #[miniextendr(r6(finalize))]
    pub fn release(&mut self) {
        self.value = 0;
    }

    /// The value a new object starts from by default.
    pub fn origin() -> i32 {
        0
    }
}

/// A cloneable R6 class whose `deep_clone` is in its second impl block.
#[derive(ExternalPtr)]
pub struct SplitR6Clone {
    value: i32,
}

/// Cloneable R6 class with its deep-clone handler in a second impl block.
#[miniextendr(r6(cloneable))]
impl SplitR6Clone {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Replace the value.
    /// @param value New value.
    pub fn set(&mut self, value: i32) {
        self.value = value;
    }
}

#[miniextendr(r6(cloneable))]
impl SplitR6Clone {
    /// Give a deep clone its own Rust value; R6 calls this for every field,
    /// the pointer included.
    #[miniextendr(r6(deep_clone))]
    pub fn clone_field(&self, name: &str, value: SEXP) -> SEXP {
        if name == ".ptr" {
            ExternalPtr::new(Self { value: self.value }).into_sexp()
        } else {
            value
        }
    }
}
// endregion

// region: S3

/// An S3 class whose methods are split across two impl blocks.
#[derive(ExternalPtr)]
pub struct SplitS3 {
    value: i32,
}

/// S3 class with methods in two impl blocks.
#[miniextendr(s3)]
impl SplitS3 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn split3_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s3)]
impl SplitS3 {
    /// Twice the value.
    pub fn split3_doubled(&self) -> i32 {
        self.value * 2
    }

    /// The value a new object starts from by default.
    pub fn origin() -> i32 {
        0
    }
}
// endregion

// region: S4

/// An S4 class whose methods are split across two impl blocks.
#[derive(ExternalPtr)]
pub struct SplitS4 {
    value: i32,
}

/// S4 class with methods in two impl blocks.
#[miniextendr(s4)]
impl SplitS4 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn split4_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s4)]
impl SplitS4 {
    /// Twice the value.
    pub fn split4_doubled(&self) -> i32 {
        self.value * 2
    }

    /// The value a new object starts from by default.
    pub fn origin() -> i32 {
        0
    }
}
// endregion

// region: S7

/// An S7 class whose methods are split across two impl blocks.
#[derive(ExternalPtr)]
pub struct SplitS7 {
    value: i32,
}

/// S7 class with methods in two impl blocks.
#[miniextendr(s7)]
impl SplitS7 {
    /// Create an object holding `value`.
    /// @param value Starting value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The current value.
    pub fn split7_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr(s7)]
impl SplitS7 {
    /// Twice the value.
    pub fn split7_doubled(&self) -> i32 {
        self.value * 2
    }

    /// The value a new object starts from by default.
    pub fn origin() -> i32 {
        0
    }
}
// endregion

// region: vctrs

/// A vctrs class whose functions are split across two impl blocks.
pub struct SplitVctrs;

/// vctrs class with functions in two impl blocks.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "split"))]
impl SplitVctrs {
    /// Create a vector of `amounts`.
    /// @param amounts Numeric amounts.
    // The vctrs constructor returns the vector payload, not `Self`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(amounts: Vec<f64>) -> Vec<f64> {
        amounts
    }
}

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "split"))]
impl SplitVctrs {
    /// The sum of `amounts`.
    /// @param amounts Numeric amounts.
    pub fn total(amounts: Vec<f64>) -> f64 {
        amounts.iter().sum()
    }
}
// endregion
