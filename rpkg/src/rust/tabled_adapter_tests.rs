//! Tabled adapter tests
use miniextendr_api::tabled_impl::table_from_vecs;
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn tabled_from_vecs(headers: Vec<String>, col1: Vec<String>, col2: Vec<String>) -> String {
    let rows: Vec<Vec<&str>> = col1
        .iter()
        .zip(col2.iter())
        .map(|(a, b)| vec![a.as_str(), b.as_str()])
        .collect();
    let hdrs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    table_from_vecs(&hdrs, &rows)
}

/// @noRd
#[miniextendr]
pub fn tabled_simple() -> String {
    table_from_vecs(&["Name", "Value"], &[vec!["pi", "3.14"], vec!["e", "2.72"]])
}

/// Empty input: no rows, just headers
/// @noRd
#[miniextendr]
pub fn tabled_empty_rows() -> String {
    table_from_vecs(&["Name", "Value"], &[])
}

/// Many columns table
/// @noRd
#[miniextendr]
pub fn tabled_many_columns(headers: Vec<String>) -> String {
    let hdrs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    // One row with same values as headers (echoed back)
    let row: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    table_from_vecs(&hdrs, &[row])
}

/// Special characters in cells (pipes, newlines, unicode)
/// @noRd
#[miniextendr]
pub fn tabled_special_chars() -> String {
    table_from_vecs(
        &["Key", "Value"],
        &[
            vec!["pipe|char", "back\\slash"],
            vec!["tab\there", "emoji\u{2764}"],
        ],
    )
}

/// Single cell table
/// @noRd
#[miniextendr]
pub fn tabled_single_cell() -> String {
    table_from_vecs(&["Only"], &[vec!["cell"]])
}

miniextendr_module! {
    mod tabled_adapter_tests;
    fn tabled_from_vecs;
    fn tabled_simple;
    fn tabled_empty_rows;
    fn tabled_many_columns;
    fn tabled_special_chars;
    fn tabled_single_cell;
}
