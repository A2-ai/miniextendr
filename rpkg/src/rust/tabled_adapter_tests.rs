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

miniextendr_module! {
    mod tabled_adapter_tests;
    fn tabled_from_vecs;
    fn tabled_simple;
}
