//! Tests for `#[miniextendr(doc = "...")]` attribute.

use miniextendr_api::{miniextendr, miniextendr_module};

/// This doc comment should be replaced by the custom doc.
#[miniextendr(
    doc = "@title Custom title from doc attr\n@description This description was set via the doc attribute.\n@param x A numeric value.\n@return The value doubled."
)]
pub fn doc_attr_basic(x: f64) -> f64 {
    x * 2.0
}

#[miniextendr(doc = "@title No-param doc\n@description A function with custom doc and no params.")]
pub fn doc_attr_no_params() -> &'static str {
    "hello from doc_attr"
}

miniextendr_module! {
    mod doc_attr_tests;

    fn doc_attr_basic;
    fn doc_attr_no_params;
}
