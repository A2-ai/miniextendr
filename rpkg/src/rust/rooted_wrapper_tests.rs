//! Missing and Result arguments retain the inner view's root (#2038).
use miniextendr_api::{IntoR, Missing, ProtectScope, RCow, SEXP, StrVec, miniextendr};
use std::borrow::Cow;

/// @noRd
#[miniextendr(noexport)]
pub fn rooted_missing_strings(strings: Missing<StrVec<'_>>) -> Vec<String> {
    match strings {
        Missing::Absent => vec!["absent".into()],
        Missing::Present(strings) => strings
            .iter()
            .map(|s| s.unwrap_or("NA").to_owned())
            .collect(),
    }
}

/// @noRd
#[miniextendr(noexport)]
pub fn rooted_result_strings(strings: Result<StrVec<'_>, ()>) -> Vec<String> {
    match strings {
        Err(()) => vec!["NULL".into()],
        Ok(strings) => rooted_missing_strings(Missing::Present(strings)),
    }
}

/// @noRd
#[miniextendr(noexport)]
pub fn rooted_nested_values(values: Missing<Result<RCow<'_, i32>, ()>>) -> Vec<i32> {
    match values {
        Missing::Absent => vec![-1],
        Missing::Present(Err(())) => vec![-2],
        Missing::Present(Ok(values)) => values.to_vec(),
    }
}

/// Preflight must see the inner shared borrow before creating other.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn rooted_wrapper_alias(values: Missing<Result<RCow<'_, i32>, ()>>, other: &mut [i32]) -> i32 {
    other[0] += 1;
    match values {
        Missing::Present(Ok(values)) => values.iter().sum::<i32>() + other[0],
        _ => other[0],
    }
}

/// Read uncached ALTSTRING children through two different nested converters.
/// @noRd
#[miniextendr(noexport)]
pub fn rooted_wrapper_strings_survive(
    strings: Missing<Result<StrVec<'_>, ()>>,
    text: Result<Cow<'_, str>, ()>,
) -> bool {
    let Missing::Present(Ok(strings)) = strings else {
        return false;
    };
    let text = text.unwrap();
    let first = strings.get_str(0).unwrap();
    let expected = [first.to_owned(), text.to_string()];
    unsafe { miniextendr_api::sys::R_gc() };
    for i in 0..128 {
        SEXP::charsxp(&format!("recycled-{i:08}-{}", "z".repeat(300)));
    }
    [first, &text] == expected.each_ref().map(String::as_str)
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct RootedWrapperHost;

/// @noRd
#[miniextendr(env, noexport)]
impl RootedWrapperHost {
    pub fn new() -> Self {
        Self
    }

    pub fn strings(&self, strings: Missing<StrVec<'_>>) -> Vec<String> {
        rooted_missing_strings(strings)
    }

    pub fn values(&self, values: Result<RCow<'_, i32>, ()>) -> Vec<i32> {
        rooted_nested_values(Missing::Present(values))
    }
}

#[miniextendr]
pub trait RootedWrapperTrait {
    fn read(&self, strings: Missing<StrVec<'_>>, values: Result<RCow<'_, i32>, ()>) -> Vec<String>;
}

#[miniextendr(env, noexport)]
impl RootedWrapperTrait for RootedWrapperHost {
    fn read(&self, strings: Missing<StrVec<'_>>, values: Result<RCow<'_, i32>, ()>) -> Vec<String> {
        let mut result = rooted_missing_strings(strings);
        result.extend(
            rooted_nested_values(Missing::Present(values))
                .iter()
                .map(i32::to_string),
        );
        result
    }
}

#[miniextendr]
pub trait GenericRootedWrapper<T> {
    fn read_generic(
        &self,
        strings: Missing<StrVec<'_>>,
        values: Result<RCow<'_, i32>, ()>,
        extra: T,
    ) -> Vec<String>;
}

#[miniextendr(env, noexport)]
impl GenericRootedWrapper<i32> for RootedWrapperHost {
    fn read_generic(
        &self,
        strings: Missing<StrVec<'_>>,
        values: Result<RCow<'_, i32>, ()>,
        extra: i32,
    ) -> Vec<String> {
        let mut result = self.read(strings, values);
        result.push(extra.to_string());
        result
    }
}

/// Both declaration and concrete generic shims receive their actual R inputs,
/// so conversion failures exercise the shim rather than this free wrapper.
/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn rooted_wrapper_shim(strings: SEXP, values: SEXP, generic: bool) -> SEXP {
    unsafe {
        let scope = ProtectScope::new();
        let owner = scope.protect(miniextendr_api::mx_abi::mx_wrap(
            __mx_wrap_RootedWrapperHost(RootedWrapperHost),
        ));
        let raw = miniextendr_api::mx_abi::mx_get(owner.get());
        let data = raw.cast::<u8>().add((*(*raw).base).data_offset).cast();
        if generic {
            let table = miniextendr_api::mx_abi::mx_query(owner.get(), TAG_GenericRootedWrapper);
            let table = &*table
                .cast::<__VTABLE_miniextendr_GenericRootedWrapper_FOR_RootedWrapperHost_Layout>(
            );
            let extra = scope.protect(7_i32.into_sexp());
            let args = [strings, values, extra.get()];
            (table.read_generic)(data, 3, args.as_ptr())
        } else {
            let table = miniextendr_api::mx_abi::mx_query(owner.get(), TAG_RootedWrapperTrait);
            let table = &*table.cast::<RootedWrapperTraitVTable>();
            let args = [strings, values];
            (table.read)(data, 2, args.as_ptr())
        }
    }
}

/// @noRd
#[miniextendr(noexport)]
pub fn rooted_wrapper_view(
    strings: Missing<StrVec<'_>>,
    values: Result<RCow<'_, i32>, ()>,
) -> Vec<String> {
    unsafe {
        let scope = ProtectScope::new();
        let owner = scope.protect(miniextendr_api::mx_abi::mx_wrap(
            __mx_wrap_RootedWrapperHost(RootedWrapperHost),
        ));
        RootedWrapperTraitView::from_sexp(owner.get()).read(strings, values)
    }
}
