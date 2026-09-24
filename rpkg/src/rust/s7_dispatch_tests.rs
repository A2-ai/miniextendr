//! S7 multi-dispatch and `Ops` operator methods.
//!
//! `s7(dispatch = "x, other")` builds a two-argument S7 generic: the first
//! dispatch argument names the receiver, the others are the method's leading
//! parameters, and the method is registered for `list(Class, S7::class_any)`.
//! An instance method on an `Ops` operator (`+`, `==`, ...) dispatches on
//! `(e1, e2)` the same way, with its single parameter as the right operand.
//! Tests live in `tests/testthat/test-s7-dispatch.R`.

use miniextendr_api::{ExternalPtr, miniextendr};

// region: two-argument dispatch

/// Left-hand value for the two-argument `mx_s7_pair()` generic.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxS7Left {
    label: String,
}

/// S7 class with a `mx_s7_pair()` method that dispatches on two arguments.
#[miniextendr(s7)]
impl MxS7Left {
    /// Create a left-hand value.
    /// @param label Label reported by `mx_s7_pair()`.
    pub fn new(label: String) -> Self {
        Self { label }
    }

    /// Pair this value with `other`, as `mx_s7_pair(x, other)`.
    ///
    /// The generic dispatches on `x` and `other`. This method is registered
    /// for any `other`; the Rust conversion reads it as a number.
    /// @param other Number to pair with.
    /// @param sep Separator between the two parts.
    #[miniextendr(s7(dispatch = "x, other"), defaults(sep = "\":\""))]
    pub fn mx_s7_pair(&self, other: f64, sep: String) -> String {
        format!("left {}{sep}{other}", self.label)
    }
}

/// Second class with a method on the same two-argument `mx_s7_pair()` generic.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxS7Right {
    label: String,
}

/// S7 class sharing the `mx_s7_pair()` generic with `MxS7Left`.
#[miniextendr(s7)]
impl MxS7Right {
    /// Create a right-hand value.
    /// @param label Label reported by `mx_s7_pair()`.
    pub fn new(label: String) -> Self {
        Self { label }
    }

    /// Pair this value with `other`, as `mx_s7_pair(x, other)`.
    /// @param other Number to pair with.
    /// @param sep Separator between the two parts.
    #[miniextendr(s7(dispatch = "x, other"), defaults(sep = "\":\""))]
    pub fn mx_s7_pair(&self, other: f64, sep: String) -> String {
        format!("right {}{sep}{other}", self.label)
    }
}

// endregion

// region: Ops operators

/// An amount of money whose arithmetic goes through R's operators.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxS7Money {
    amount: f64,
}

/// S7 money class with `+`, `*`, `==` and `<` methods.
#[miniextendr(s7)]
impl MxS7Money {
    /// Create an amount.
    /// @param amount Numeric amount.
    pub fn new(amount: f64) -> Self {
        Self { amount }
    }

    /// The amount as a number.
    #[miniextendr(s7(getter))]
    pub fn amount(&self) -> f64 {
        self.amount
    }

    /// Sum of two amounts, as `a + b`.
    ///
    /// Registered on the `+` operator; the Rust name keeps the
    /// `MxS7Money_add()` shortcut.
    /// @param e2 Another `MxS7Money`.
    #[miniextendr(s7(generic = "+"))]
    pub fn add(&self, e2: ExternalPtr<MxS7Money>) -> Self {
        Self {
            amount: self.amount + e2.amount,
        }
    }

    /// The amount scaled by a number, as `a * k`.
    #[miniextendr(r_name = "*")]
    pub fn times(&self, e2: f64) -> Self {
        Self {
            amount: self.amount * e2,
        }
    }

    /// Whether two amounts are equal, as `a == b`.
    /// @param e2 Another `MxS7Money`.
    #[miniextendr(s7(generic = "=="))]
    pub fn equals(&self, e2: ExternalPtr<MxS7Money>) -> bool {
        self.amount == e2.amount
    }

    /// Whether the amount is below a number, as `a < k`.
    #[miniextendr(r_name = "<")]
    pub fn below(&self, e2: f64) -> bool {
        self.amount < e2
    }
}

// endregion
