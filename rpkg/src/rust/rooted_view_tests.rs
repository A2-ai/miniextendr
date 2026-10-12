//! Borrowed `StrVec<'_>` / `RCow<'_, T>` parameters (#1427), driven by
//! `tests/testthat/test-rooted-views.R`.
//!
//! Neither view implements `TryFromSexp`: every generated path binds it with
//! `try_from_root` to a root that lives only as long as the call. Each path is
//! here: a free function, an inherent method, a trait method called from R
//! (the impl's C wrapper) and the same trait method called through the
//! trait's View (the vtable shim).

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{IntoR, RCow, StrVec, miniextendr};

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
}

/// Trait methods taking borrowed views.
#[miniextendr]
pub trait RootedViews {
    /// The first string (`NA` for `NA` or empty).
    fn view_first(&self, strings: StrVec<'_>) -> Option<String>;
    /// A copy of the values.
    fn view_values(&self, values: RCow<'_, i32>) -> Vec<i32>;
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
}

/// Call `RootedViews` through the trait's View, the vtable-shim path another
/// package takes: the View hands each view over as its SEXP and the shim binds
/// it again to a root local to the shim.
/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn rooted_views_through_view() -> Vec<String> {
    // SAFETY: main thread (`no_worker`); the scope roots the wrapped handle
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
        vec![
            first.unwrap_or_else(|| "<NA>".to_string()),
            copied
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ]
    }
}
