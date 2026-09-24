//! S7 methods on operator generics (#1475): `[` through `s7(generic = "[")`,
//! `[[` through `r_name`, and a package-local `%mx_scale%` operator.
//!
//! None of these names is syntactic R, so the generated wrappers have to
//! backtick-quote them wherever they are R symbols (`` S7::method(`[[`, ...) ``)
//! and write `export("%mx_scale%")` for the new generic, or the wrappers file
//! does not parse and the package does not install.

use miniextendr_api::miniextendr;

/// Handle-backed bag of numbers whose element access goes through R operators.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxS7Bag {
    values: Vec<f64>,
}

/// S7 bag with `[`, `[[` and `%mx_scale%` methods.
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
}
