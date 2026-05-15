//! Fixtures for issue #343: multi-paragraph doc comments produce `\description{}`
//! and `\details{}` in generated Rd files, for both standalone fns and impl methods.
use miniextendr_api::miniextendr;

/// Three-paragraph doc demo
///
/// This is the description paragraph (paragraph 2).
///
/// This is the details paragraph (paragraph 3). It should appear in
/// a `\details{}` block in the generated Rd file.
///
/// @return Always returns 0.
#[miniextendr]
pub fn docs_demo_three_paras() -> i32 {
    0
}

/// Demo struct for testing method-level multi-paragraph doc comments.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DocDetailsDemo {
    value: i32,
}

/// DocDetailsDemo: impl method multi-paragraph doc fixture.
///
/// Multi-paragraph doc comments on impl methods with no explicit `@param`/`@return`
/// tags previously collapsed to a single flat `@description`. This impl block
/// verifies the fix for issue #343.
#[miniextendr(r6)]
impl DocDetailsDemo {
    /// Construct a DocDetailsDemo.
    pub fn new(value: i32) -> Self {
        DocDetailsDemo { value }
    }

    /// Return the stored value.
    ///
    /// This second paragraph describes more about what returning the value means,
    /// and should appear as `\description{}` in the generated Rd.
    ///
    /// This third paragraph provides additional context and should appear as
    /// `\details{}` in the generated Rd, not merged into `\description{}`.
    pub fn get(&self) -> i32 {
        self.value
    }
}
