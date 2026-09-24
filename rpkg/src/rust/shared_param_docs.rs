//! Functions documented on a shared R help page (#1590).
//!
//! `R/range_summaries.R` holds the `range_summaries` topic, whose block
//! documents every argument once (`lower,upper` as one grouped entry, and
//! `x` / `...` for the `RangeBox` S3 methods). The functions and methods here
//! join that page with `@rdname` / `@describeIn`, or take the descriptions
//! with `@inheritParams`, so their generated wrappers add no `@param` line
//! that would replace the shared descriptions. The R file sorts after
//! `R/miniextendr-wrappers.R`: the joining blocks sort after the page's own
//! block (`@order NaN`), so it still names and titles the page.

use miniextendr_api::miniextendr;

/// Smallest and largest value (`Inf` / `-Inf` for an empty vector).
fn min_max(values: &[f64]) -> (f64, f64) {
    values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &value| {
            (lo.min(value), hi.max(value))
        })
}

/// Width of the values' range: the largest value minus the smallest.
/// @rdname range_summaries
#[miniextendr]
pub fn range_width(values: Vec<f64>) -> f64 {
    let (lo, hi) = min_max(&values);
    hi - lo
}

/// Which values lie inside the range.
/// @rdname range_summaries
/// @param inclusive Whether a value equal to a bound counts as inside.
#[miniextendr]
pub fn range_within(values: Vec<f64>, lower: f64, upper: f64, inclusive: bool) -> Vec<bool> {
    values
        .iter()
        .map(|&value| {
            if inclusive {
                lower <= value && value <= upper
            } else {
                lower < value && value < upper
            }
        })
        .collect()
}

/// Keep the values inside the range, handling the others as `outside` says.
/// @rdname range_summaries
#[miniextendr]
pub fn range_clamp(
    values: Vec<f64>,
    lower: f64,
    upper: f64,
    #[miniextendr(choices("clamp", "drop"))] outside: &str,
) -> Vec<f64> {
    match outside {
        "drop" => values
            .into_iter()
            .filter(|&value| lower <= value && value <= upper)
            .collect(),
        _ => values
            .into_iter()
            .map(|value| value.clamp(lower, upper))
            .collect(),
    }
}

/// @describeIn range_summaries Midpoint between the smallest and the largest
/// value.
#[miniextendr]
pub fn range_midpoint(values: Vec<f64>) -> f64 {
    let (lo, hi) = min_max(&values);
    (lo + hi) / 2.0
}

/// @title Share of values inside a range
/// @description Share of the values that lie inside the range, bounds
/// included.
/// @inheritParams range_summaries
/// @returns A number between 0 and 1 (`NaN` for an empty vector).
#[miniextendr]
pub fn range_share_within(values: Vec<f64>, lower: f64, upper: f64) -> f64 {
    let (inside, total) = values.iter().fold((0.0, 0.0), |(inside, total), &value| {
        let hit = if lower <= value && value <= upper {
            1.0
        } else {
            0.0
        };
        (inside + hit, total + 1.0)
    });
    inside / total
}

/// A closed range `[lower, upper]`, whose methods are documented on the
/// shared `range_summaries` page.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RangeBox {
    lower: f64,
    upper: f64,
}

/// A closed range as an S3 object. The constructor is documented here; the
/// methods join the shared `range_summaries` page, whose R block documents
/// `x` and `...` for them.
#[miniextendr(s3)]
impl RangeBox {
    /// Create a range box.
    /// @param lower Lower bound.
    /// @param upper Upper bound.
    pub fn new(lower: f64, upper: f64) -> Self {
        RangeBox { lower, upper }
    }

    /// @describeIn range_summaries Width of a range box.
    pub fn box_width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Which values a range box covers, bounds included.
    /// @rdname range_summaries
    pub fn box_covers(&self, values: Vec<f64>) -> Vec<bool> {
        values
            .iter()
            .map(|&value| self.lower <= value && value <= self.upper)
            .collect()
    }

    /// @describeIn range_summaries The smallest range box holding every
    /// value.
    pub fn from_values(values: Vec<f64>) -> Self {
        let (lower, upper) = min_max(&values);
        RangeBox { lower, upper }
    }
}
