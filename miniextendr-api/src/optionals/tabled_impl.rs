//! Integration with the `tabled` crate for table formatting.
//!
//! Provides helpers for formatting data as ASCII/Unicode tables.
//!
//! # Features
//!
//! Enable this module with the `tabled` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["tabled"] }
//! ```
//!
//! # Overview
//!
//! The `tabled` crate provides beautiful table formatting. This module exposes
//! helpers for converting Rust data to formatted table strings.
//!
//! # Example
//!
//! ```ignore
//! use tabled::Tabled;
//! use miniextendr_api::tabled_impl::table_to_string;
//!
//! #[derive(Tabled)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! #[miniextendr]
//! fn format_people(names: Vec<String>, ages: Vec<i32>) -> String {
//!     let people: Vec<Person> = names.into_iter().zip(ages)
//!         .map(|(name, age)| Person { name, age: age as u32 })
//!         .collect();
//!     table_to_string(&people)
//! }
//! ```
//!
//! # ANSI Policy
//!
//! By default, no ANSI styling is used. R consoles may not render ANSI codes
//! correctly, so plain ASCII/Unicode borders are preferred.

pub use tabled::builder::Builder;
pub use tabled::settings::object::Columns;
pub use tabled::settings::{Alignment, Modify, Style, Width};
pub use tabled::{Table, Tabled};

use crate::ffi::{Rf_allocVector, Rf_mkCharLenCE, SET_STRING_ELT, SEXP, SEXPTYPE, cetype_t};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;

// =============================================================================
// Helper functions
// =============================================================================

/// Format rows as a table string.
///
/// Uses default tabled formatting (ASCII box drawing).
///
/// # Example
///
/// ```ignore
/// use tabled::Tabled;
///
/// #[derive(Tabled)]
/// struct Item { name: String, count: i32 }
///
/// let items = vec![
///     Item { name: "apple".into(), count: 5 },
///     Item { name: "banana".into(), count: 3 },
/// ];
/// let table = table_to_string(&items);
/// ```
pub fn table_to_string<T: Tabled>(rows: &[T]) -> String {
    Table::new(rows).to_string()
}

/// Format rows with custom options.
///
/// # Arguments
///
/// * `rows` - Data to format
/// * `max_width` - Optional maximum column width (truncates)
/// * `align` - Alignment: "left", "right", or "center"
/// * `trim` - Whether to trim whitespace
///
/// # Example
///
/// ```ignore
/// let table = table_to_string_opts(&items, Some(20), "center", true);
/// ```
pub fn table_to_string_opts<T: Tabled>(
    rows: &[T],
    max_width: Option<usize>,
    align: &str,
    _trim: bool, // Reserved for future use
) -> String {
    let mut table = Table::new(rows);

    // Apply alignment
    let alignment = match align {
        "right" => Alignment::right(),
        "center" => Alignment::center(),
        _ => Alignment::left(),
    };
    table.with(Modify::new(Columns::new(..)).with(alignment));

    // Apply max width if specified
    if let Some(width) = max_width {
        table.with(Width::truncate(width));
    }

    table.to_string()
}

/// Build a table from a builder for dynamic schemas.
///
/// # Example
///
/// ```ignore
/// use tabled::builder::Builder;
///
/// let mut builder = Builder::new();
/// builder.push_record(["Name", "Value"]);
/// builder.push_record(["foo", "42"]);
/// builder.push_record(["bar", "99"]);
///
/// let table = builder_to_string(builder);
/// ```
pub fn builder_to_string(builder: Builder) -> String {
    builder.build().to_string()
}

/// Build a table from column headers and rows.
///
/// # Example
///
/// ```ignore
/// let headers = vec!["Name", "Age"];
/// let rows = vec![
///     vec!["Alice", "30"],
///     vec!["Bob", "25"],
/// ];
/// let table = table_from_vecs(&headers, &rows);
/// ```
pub fn table_from_vecs<S: AsRef<str>>(headers: &[S], rows: &[Vec<S>]) -> String {
    let mut builder = Builder::new();

    // Add headers
    let header_row: Vec<&str> = headers.iter().map(|s| s.as_ref()).collect();
    builder.push_record(header_row);

    // Add data rows
    for row in rows {
        let data_row: Vec<&str> = row.iter().map(|s| s.as_ref()).collect();
        builder.push_record(data_row);
    }

    builder.build().to_string()
}

/// Format a table with a specific style.
///
/// Available styles: "ascii", "modern", "markdown", "rounded", "blank"
pub fn table_to_string_styled<T: Tabled>(rows: &[T], style: &str) -> String {
    let mut table = Table::new(rows);

    match style {
        "markdown" => {
            table.with(Style::markdown());
        }
        "modern" => {
            table.with(Style::modern());
        }
        "rounded" => {
            table.with(Style::rounded());
        }
        "blank" => {
            table.with(Style::blank());
        }
        "ascii" => {
            table.with(Style::ascii());
        }
        _ => {
            table.with(Style::ascii());
        }
    }

    table.to_string()
}

// =============================================================================
// IntoR for Table
// =============================================================================

impl IntoR for Table {
    /// Convert a Table to an R character scalar.
    fn into_sexp(self) -> SEXP {
        let s = self.to_string();
        unsafe {
            // Protect sexp before Rf_mkCharLenCE which can trigger GC
            let sexp = OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, 1));
            let charsxp = Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, cetype_t::CE_UTF8);
            SET_STRING_ELT(sexp.get(), 0, charsxp);
            // Return the SEXP - guard drops and unprotects
            sexp.get()
        }
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Tabled)]
    struct TestRow {
        name: String,
        value: i32,
    }

    #[test]
    fn test_table_to_string() {
        let rows = vec![
            TestRow {
                name: "foo".into(),
                value: 1,
            },
            TestRow {
                name: "bar".into(),
                value: 2,
            },
        ];
        let table = table_to_string(&rows);
        assert!(table.contains("foo"));
        assert!(table.contains("bar"));
        assert!(table.contains("name"));
        assert!(table.contains("value"));
    }

    #[test]
    fn test_table_to_string_opts_left() {
        let rows = vec![TestRow {
            name: "test".into(),
            value: 42,
        }];
        let table = table_to_string_opts(&rows, None, "left", true);
        assert!(table.contains("test"));
    }

    #[test]
    fn test_table_to_string_opts_center() {
        let rows = vec![TestRow {
            name: "test".into(),
            value: 42,
        }];
        let table = table_to_string_opts(&rows, None, "center", true);
        assert!(table.contains("test"));
    }

    #[test]
    fn test_table_to_string_opts_max_width() {
        let rows = vec![TestRow {
            name: "verylongname".into(),
            value: 42,
        }];
        let table = table_to_string_opts(&rows, Some(5), "left", true);
        // Width truncation should apply
        assert!(!table.is_empty());
    }

    #[test]
    fn test_builder_to_string() {
        let mut builder = Builder::new();
        builder.push_record(["Col1", "Col2"]);
        builder.push_record(["a", "b"]);
        let table = builder_to_string(builder);
        assert!(table.contains("Col1"));
        assert!(table.contains("a"));
    }

    #[test]
    fn test_table_from_vecs() {
        let headers = vec!["H1", "H2"];
        let rows = vec![vec!["r1c1", "r1c2"], vec!["r2c1", "r2c2"]];
        let table = table_from_vecs(&headers, &rows);
        assert!(table.contains("H1"));
        assert!(table.contains("r1c1"));
        assert!(table.contains("r2c2"));
    }

    #[test]
    fn test_table_to_string_styled() {
        let rows = vec![TestRow {
            name: "x".into(),
            value: 1,
        }];

        let ascii = table_to_string_styled(&rows, "ascii");
        assert!(ascii.contains("x"));

        let markdown = table_to_string_styled(&rows, "markdown");
        assert!(markdown.contains("|"));
    }

    #[test]
    fn test_empty_table() {
        let rows: Vec<TestRow> = vec![];
        let table = table_to_string(&rows);
        // Empty table should still have headers
        assert!(table.contains("name"));
    }
}
