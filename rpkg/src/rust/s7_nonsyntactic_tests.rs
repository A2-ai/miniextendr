//! S7 methods on operator generics (#1475): `[` through `s7(generic = "[")`,
//! `[[` through `r_name`, `%*%` (dispatched on both operands), and the
//! package-local `%mx_scale%` (`r_name`) and `%mx_cat%` (`s7(generic = ...)`)
//! operators. Also an external generic from a package that is not imported
//! (`s7(generic = "generics::tidy")`).
//!
//! None of the operator names is syntactic R, so the generated wrappers have to
//! backtick-quote them wherever they are R symbols (`` S7::method(`[[`, ...) ``)
//! and write `export("%mx_scale%")` for a new generic, or the wrappers file
//! does not parse and the package does not install.

use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame};
use miniextendr_api::{ExternalPtr, miniextendr};

/// Handle-backed bag of numbers whose element access goes through R operators.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxS7Bag {
    values: Vec<f64>,
}

/// S7 bag with `[`, `[[`, `%*%`, `%mx_scale%`, `%mx_cat%` and `tidy()` methods.
#[miniextendr(s7)]
impl MxS7Bag {
    /// Create a bag.
    /// @param values Numeric values held by the bag.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Values at 1-based positions, as `bag[i]`.
    ///
    /// Registered on the existing `[` generic; the Rust name keeps the
    /// `MxS7Bag_subset()` fast-path shortcut.
    /// @param i Integer positions to keep.
    #[miniextendr(s7(generic = "["))]
    pub fn subset(&self, i: Vec<i32>) -> Result<Vec<f64>, String> {
        i.iter().map(|&k| self.at(k)).collect()
    }

    /// Value at a 1-based position, as `bag[[i]]`.
    /// @param i A single integer position.
    #[miniextendr(r_name = "[[")]
    pub fn at(&self, i: i32) -> Result<f64, String> {
        usize::try_from(i)
            .ok()
            .and_then(|k| k.checked_sub(1))
            .and_then(|k| self.values.get(k).copied())
            .ok_or_else(|| format!("index {i} out of range for {} values", self.values.len()))
    }

    /// Every value multiplied by `k`, as `bag %mx_scale% k`.
    /// @param k Numeric multiplier.
    #[miniextendr(r_name = "%mx_scale%")]
    pub fn scale(&self, k: f64) -> Vec<f64> {
        self.values.iter().map(|v| v * k).collect()
    }

    /// The bag's values followed by `other`, as `bag %mx_cat% other`.
    ///
    /// `s7(generic = ...)` with a name no generic has yet defines the
    /// package-local `%mx_cat%` operator; the Rust name keeps the
    /// `MxS7Bag_concat()` shortcut.
    /// @param other Numeric values to append.
    #[miniextendr(s7(generic = "%mx_cat%"))]
    pub fn concat(&self, other: Vec<f64>) -> Vec<f64> {
        self.values.iter().copied().chain(other).collect()
    }

    /// Dot product with another bag, as `bag %*% y`.
    ///
    /// S7 dispatches `%*%` on `x` and `y`; the method is registered for any
    /// `y`, which the Rust conversion reads as another `MxS7Bag`.
    /// @param y Another `MxS7Bag` with the same number of values.
    #[miniextendr(s7(generic = "%*%"))]
    pub fn dot(&self, y: ExternalPtr<MxS7Bag>) -> Result<f64, String> {
        if self.values.len() != y.values.len() {
            return Err(format!(
                "cannot multiply bags of {} and {} values",
                self.values.len(),
                y.values.len()
            ));
        }
        Ok(self.values.iter().zip(&y.values).map(|(a, b)| a * b).sum())
    }

    /// The values as a data frame of `position` and `value`, as
    /// `generics::tidy(bag)`.
    ///
    /// Registered on the `tidy()` generic of the generics package, which the
    /// package does not import: the method is attached when generics loads.
    #[miniextendr(s7(generic = "generics::tidy"))]
    pub fn tidy(&self) -> BuiltDataFrame {
        let values = self.values.clone();
        DataFrame::builder(values.len())
            .column::<i32>("position", |chunk, offset| {
                for (i, slot) in chunk.iter_mut().enumerate() {
                    *slot = i32::try_from(offset + i + 1).expect("bag position exceeds i32");
                }
            })
            .column::<f64>("value", move |chunk, offset| {
                chunk.copy_from_slice(&values[offset..offset + chunk.len()]);
            })
            .build()
    }
}
