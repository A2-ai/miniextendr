//! Fixture for issue #343: 3-paragraph doc comments produce `\details{}` in Rd.
use miniextendr_api::miniextendr;

/// Three-paragraph doc demo
///
/// This is the description paragraph (paragraph 2).
///
/// This is the details paragraph (paragraph 3). It should appear in
/// a separate details section in the generated Rd file.
///
/// @return Always returns 0.
#[miniextendr]
pub fn docs_demo_three_paras() -> i32 {
    0
}
