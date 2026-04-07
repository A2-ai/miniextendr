//! Tabled adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::tabled_impl::table_from_vecs;

/// Test creating a table from header and column vectors.
/// @param headers Character vector of column headers.
/// @param col1 Character vector for the first column.
/// @param col2 Character vector for the second column.
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

/// Test creating a simple two-column table with known data.
#[miniextendr]
pub fn tabled_simple() -> String {
    table_from_vecs(&["Name", "Value"], &[vec!["pi", "3.14"], vec!["e", "2.72"]])
}

/// Test creating a table with no rows, only headers.
#[miniextendr]
pub fn tabled_empty_rows() -> String {
    table_from_vecs(&["Name", "Value"], &[])
}

/// Test creating a table with many columns.
/// @param headers Character vector of column headers.
#[miniextendr]
pub fn tabled_many_columns(headers: Vec<String>) -> String {
    let hdrs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    // One row with same values as headers (echoed back)
    let row: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    table_from_vecs(&hdrs, &[row])
}

/// Test creating a table with special characters in cells (pipes, unicode).
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

/// Test creating a table with a single cell.
#[miniextendr]
pub fn tabled_single_cell() -> String {
    table_from_vecs(&["Only"], &[vec!["cell"]])
}
