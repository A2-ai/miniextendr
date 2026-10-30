//! Borrowed `StrVec<'_>` / `RCow<'_, T>` parameters, and the `Cow` and
//! shared-reference parameters that borrow R storage the same way (#1427),
//! driven by `tests/testthat/test-rooted-views.R`.
//!
//! Every generated path binds them with `try_from_root` to a root that lives
//! only as long as the call. Each path is here: a free function, an inherent
//! method, a trait method called from R (the impl's C wrapper) and the same
//! trait method called through the trait's View (the vtable shim).

use std::borrow::Cow;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{IntoR, RCow, StrVec, miniextendr};

/// `label:values:text`, with `NA` for a missing label.
fn describe(values: &[i32], label: Option<&str>, text: &str) -> String {
    let values = values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{values}:{text}", label.unwrap_or("NA"))
}

/// The first string of a borrowed character vector (`NA` for `NA` or empty).
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_strvec_first(strings: StrVec<'_>) -> Option<String> {
    strings.get_str(0).map(str::to_owned)
}

/// A borrowed `RCow` returned while its argument root is still live: the
/// input SEXP comes back unchanged.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_rcow_identity(values: RCow<'_, i32>) -> RCow<'_, i32> {
    values
}

/// Borrowed slice, scalar, optional string and `Cow` parameters together.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_refs_describe(
    values: &[i32],
    scale: &f64,
    label: Option<&str>,
    text: Cow<'_, str>,
    words: Vec<Option<Cow<'_, str>>>,
) -> String {
    let words = words
        .iter()
        .map(|word| word.as_deref().unwrap_or("NA"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{}:{scale}:{words}", describe(values, label, &text))
}

/// Env class whose inherent and trait methods take borrowed views.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RootedViewHost;

#[miniextendr(env, noexport)]
impl RootedViewHost {
    /// Create a host.
    pub fn new() -> Self {
        RootedViewHost
    }

    /// The first string (`NA` for `NA` or empty).
    /// @param strings A character vector.
    pub fn first(&self, strings: StrVec<'_>) -> Option<String> {
        strings.get_str(0).map(str::to_owned)
    }

    /// The input integer vector, returned without a copy.
    /// @param values An integer vector.
    pub fn echo<'a>(&self, values: RCow<'a, i32>) -> RCow<'a, i32> {
        values
    }

    /// `label:values:text`.
    /// @param values An integer vector.
    /// @param label A string, or `NULL`.
    /// @param text A string.
    pub fn describe(&self, values: &[i32], label: Option<&str>, text: Cow<'_, str>) -> String {
        describe(values, label, &text)
    }
}

/// Trait methods taking borrowed views.
#[miniextendr]
pub trait RootedViews {
    /// The first string (`NA` for `NA` or empty).
    fn view_first(&self, strings: StrVec<'_>) -> Option<String>;
    /// A copy of the values.
    fn view_values(&self, values: RCow<'_, i32>) -> Vec<i32>;
    /// `label:values:text`.
    fn view_describe(&self, values: &[i32], label: Option<&str>, text: Cow<'_, str>) -> String;
}

#[miniextendr(env, noexport)]
impl RootedViews for RootedViewHost {
    /// @param strings A character vector.
    fn view_first(&self, strings: StrVec<'_>) -> Option<String> {
        strings.get_str(0).map(str::to_owned)
    }

    /// @param values An integer vector.
    fn view_values(&self, values: RCow<'_, i32>) -> Vec<i32> {
        values.to_vec()
    }

    /// @param values An integer vector.
    /// @param label A string, or `NULL`.
    /// @param text A string.
    fn view_describe(&self, values: &[i32], label: Option<&str>, text: Cow<'_, str>) -> String {
        describe(values, label, &text)
    }
}

/// Call `RootedViews` through the trait's View, the vtable-shim path another
/// package takes: the View hands each view over as its SEXP and the shim binds
/// it again to a root local to the shim.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_views_through_view() -> Vec<String> {
    // SAFETY: R's main thread (the wrapper); the scope roots the wrapped handle
    // and both inputs for every View call.
    unsafe {
        let scope = ProtectScope::new();
        let handle = scope.protect(miniextendr_api::trait_abi::ccall::mx_wrap(
            __mx_wrap_RootedViewHost(RootedViewHost),
        ));
        let view = RootedViewsView::from_sexp(handle.get());
        let strings = scope.protect(vec![Some("through view"), None].into_sexp());
        let values = scope.protect(vec![1_i32, 2, 3].into_sexp());
        let first = view.view_first(StrVec::try_from_root(strings).expect("a character vector"));
        let copied = view.view_values(RCow::try_from_root(values).expect("an integer vector"));
        let described = view.view_describe(&[4, 5], None, Cow::Borrowed("shim"));
        vec![
            first.unwrap_or_else(|| "<NA>".to_string()),
            copied
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            described,
        ]
    }
}

/// An ALTSTRING input that returns a fresh, uncached string on every read.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_uncached_strings() -> miniextendr_api::SEXP {
    crate::gc_stress_fixtures::GcTransientBorrowString.into_sexp()
}

/// Borrow uncached strings through each generated parameter path, then collect.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_uncached_strings_check(
    text: &str,
    optional: Option<&str>,
    cow: Cow<'_, str>,
    strings: StrVec<'_>,
    owned: miniextendr_api::ProtectedStrVec,
) -> bool {
    let from_view = strings.iter().next().unwrap().unwrap();
    let from_owner = owned.as_strvec().get_str(0).unwrap();
    let actual = [text, optional.unwrap(), &cow, from_view, from_owner];
    let expected = actual.map(str::to_owned);
    unsafe { miniextendr_api::sys::R_gc() };
    for i in 0..128 {
        miniextendr_api::SEXP::charsxp(&format!("recycled-{i:08}-{}", "z".repeat(300)));
    }
    actual.iter().zip(&expected).all(|(a, b)| *a == b)
}
