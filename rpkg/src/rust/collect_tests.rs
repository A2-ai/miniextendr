//! Test fixtures for Collect and CollectStrings wrappers.

use miniextendr_api::convert::{Collect, CollectStrings};
use miniextendr_api::miniextendr;

// region: Collect — ExactSizeIterator<Item = T: RNativeType> → R vector

/// @export
#[miniextendr]
pub fn test_collect_sines(n: i32) -> Collect<std::vec::IntoIter<f64>> {
    let v: Vec<f64> = (0..n).map(|i| (i as f64).sin()).collect();
    Collect(v.into_iter())
}

/// @export
#[miniextendr]
pub fn test_collect_squares(n: i32) -> Collect<std::vec::IntoIter<i32>> {
    let v: Vec<i32> = (0..n).map(|i| i * i).collect();
    Collect(v.into_iter())
}

/// @export
#[miniextendr]
pub fn test_collect_range() -> Collect<std::ops::Range<i32>> {
    Collect(1..11)
}

/// @export
#[miniextendr]
pub fn test_collect_empty() -> Collect<std::vec::IntoIter<f64>> {
    Collect(Vec::new().into_iter())
}

// endregion

// region: CollectStrings — ExactSizeIterator<Item = String> → R character vector

/// @export
#[miniextendr]
pub fn test_collect_strings_upper(words: Vec<String>) -> CollectStrings<std::vec::IntoIter<String>>
{
    CollectStrings(words.into_iter().map(|w| w.to_uppercase()).collect::<Vec<_>>().into_iter())
}

/// @export
#[miniextendr]
pub fn test_collect_strings_numbered(n: i32) -> CollectStrings<std::vec::IntoIter<String>> {
    let v: Vec<String> = (1..=n).map(|i| format!("item_{i}")).collect();
    CollectStrings(v.into_iter())
}

// endregion
