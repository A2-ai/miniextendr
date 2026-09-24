//! Functions documented on a shared R help page (#1590).
//!
//! `R/doc_range_summaries.R` holds the `range_summaries` topic, whose block
//! documents every argument once (`lower,upper` as one grouped entry). The
//! functions here join that page with `@rdname` / `@describeIn`, or take the
//! descriptions with `@inheritParams`, so their generated wrappers add no
//! `@param` placeholder that would replace the shared descriptions.

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
