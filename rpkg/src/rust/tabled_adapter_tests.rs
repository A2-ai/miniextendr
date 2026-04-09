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

// region: Upstream example-derived fixtures

/// Build a table row-by-row with the Builder API.
/// @param items Character vector of items to add as rows.
#[miniextendr]
pub fn tabled_builder_demo(items: Vec<String>) -> String {
    use miniextendr_api::tabled_impl::Builder;
    let mut builder = Builder::new();
    builder.push_record(["Item", "Index"]);
    for (i, item) in items.iter().enumerate() {
        builder.push_record([item.as_str(), &i.to_string()]);
    }
    builder.build().to_string()
}

/// Create a table representing language data using Builder API.
#[miniextendr]
pub fn tabled_struct_table() -> String {
    use miniextendr_api::tabled_impl::Builder;
    let mut builder = Builder::new();
    builder.push_record(["name", "year"]);
    builder.push_record(["Rust", "2010"]);
    builder.push_record(["R", "1993"]);
    builder.push_record(["C", "1972"]);
    builder.build().to_string()
}

/// Format a table with a chosen style.
/// @param style Style name: "ascii", "modern", "markdown", "rounded", "blank".
#[miniextendr]
pub fn tabled_styled(style: &str) -> String {
    use miniextendr_api::tabled_impl::{Builder, Style};
    let mut builder = Builder::new();
    builder.push_record(["key", "val"]);
    builder.push_record(["a", "1"]);
    builder.push_record(["b", "2"]);
    let mut table = builder.build();
    match style {
        "markdown" => { table.with(Style::markdown()); }
        "modern" => { table.with(Style::modern()); }
        "rounded" => { table.with(Style::rounded()); }
        "blank" => { table.with(Style::blank()); }
        _ => { table.with(Style::ascii()); }
    }
    table.to_string()
}

/// Truncate columns to a maximum width.
/// @param max Maximum column width.
#[miniextendr]
pub fn tabled_with_max_width(max: i32) -> String {
    use miniextendr_api::tabled_impl::{Builder, Width};
    let mut builder = Builder::new();
    builder.push_record(["long_name", "value"]);
    builder.push_record(["a_very_long_column_name_here", "also_long_value_text"]);
    let mut table = builder.build();
    table.with(Width::truncate(max as usize));
    table.to_string()
}

/// Center-align all columns.
#[miniextendr]
pub fn tabled_aligned() -> String {
    use miniextendr_api::tabled_impl::{Alignment, Builder, Columns, Modify};
    let mut builder = Builder::new();
    builder.push_record(["left", "right"]);
    builder.push_record(["short", "1"]);
    builder.push_record(["longer text", "200"]);
    let mut table = builder.build();
    table.with(Modify::new(Columns::new(..)).with(Alignment::center()));
    table.to_string()
}

/// Create a compact table from raw string vectors.
/// @param col1 First column values.
/// @param col2 Second column values.
#[miniextendr]
pub fn tabled_compact(col1: Vec<String>, col2: Vec<String>) -> String {
    use miniextendr_api::tabled_impl::Builder;
    let mut builder = Builder::new();
    builder.push_record(["Col1", "Col2"]);
    for (a, b) in col1.iter().zip(col2.iter()) {
        builder.push_record([a.as_str(), b.as_str()]);
    }
    builder.build().to_string()
}

/// Concatenate two tables side-by-side by merging columns.
/// @param left_keys Character vector for the left table keys.
/// @param right_vals Character vector for the right table values.
#[miniextendr]
pub fn tabled_concat_horizontal(left_keys: Vec<String>, right_vals: Vec<String>) -> String {
    use miniextendr_api::tabled_impl::Builder;
    let mut builder = Builder::new();
    builder.push_record(["Key", "Value"]);
    for (k, v) in left_keys.iter().zip(right_vals.iter()) {
        builder.push_record([k.as_str(), v.as_str()]);
    }
    builder.build().to_string()
}

// endregion
