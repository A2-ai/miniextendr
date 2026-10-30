//! Explicit borrowed conversions (#2110), exercised from R and trait Views.

use miniextendr_api::from_r::{NativeBorrow, SexpError};
use miniextendr_api::{ExternalPtr, SEXP, TryFromSexp, miniextendr};

pub type Text = str;
pub type Integers = [i32];

/// A custom unsized view, deliberately without TypedExternal.
#[repr(transparent)]
pub struct ConvertedIntegers([i32]);

impl TryFromSexp for &ConvertedIntegers {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&[i32] as TryFromSexp>::NATIVE_BORROW;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice = <&[i32]>::try_from_sexp(sexp)?;
        // SAFETY: transparent over [i32], preserving the slice's metadata.
        Ok(unsafe { &*(std::ptr::from_ref(slice) as *const ConvertedIntegers) })
    }
}

impl TryFromSexp for &mut ConvertedIntegers {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&mut [i32] as TryFromSexp>::NATIVE_BORROW;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice = <&mut [i32]>::try_from_sexp(sexp)?;
        // SAFETY: transparent over [i32]; retain mutable pointer provenance.
        Ok(unsafe { &mut *(std::ptr::from_mut(slice) as *mut ConvertedIntegers) })
    }
}

/// Read a custom converter and an optional string alias.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_describe(
    #[miniextendr(convert)] values: &ConvertedIntegers,
    #[miniextendr(convert)] label: Option<&Text>,
) -> String {
    format!(
        "{}:{}",
        label.unwrap_or("missing"),
        values.0.iter().sum::<i32>()
    )
}

/// Native metadata must reject overlap before either converter runs.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_alias_pair(
    #[miniextendr(convert)] read: &Integers,
    #[miniextendr(convert)] write: &mut Integers,
) -> i32 {
    for value in write.iter_mut() {
        *value += 1;
    }
    read.iter().chain(write.iter()).sum()
}

/// Custom converters carry exactly the same metadata as their native storage.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_custom_pair(
    #[miniextendr(convert)] read: &ConvertedIntegers,
    #[miniextendr(convert)] write: &mut ConvertedIntegers,
) -> i32 {
    converted_refs_alias_pair(&read.0, &mut write.0)
}

/// NULL remains None for mutable aliases.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_optional(#[miniextendr(convert)] values: Option<&mut Integers>) -> i32 {
    values.map_or(-1, |values| {
        for value in values.iter_mut() {
            *value += 1;
        }
        values.iter().sum()
    })
}

/// String aliases stay valid across a collection and later allocations.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_strings(
    #[miniextendr(convert)] text: &Text,
    #[miniextendr(convert)] optional: Option<&Text>,
    later: String,
) -> String {
    let before = format!("{text}:{}:{later}", optional.unwrap_or("missing"));
    unsafe { miniextendr_api::sys::R_gc() };
    for i in 0..32 {
        SEXP::charsxp(&format!("replacement-{i:08}-{}", "z".repeat(300)));
    }
    let after = format!("{text}:{}:{later}", optional.unwrap_or("missing"));
    assert_eq!(before, after);
    after
}

/// Method conversion probe.
#[derive(ExternalPtr)]
pub struct ConvertedRefHost;

#[miniextendr(env, noexport)]
impl ConvertedRefHost {
    pub fn new() -> Self {
        Self
    }

    /// Read custom and aliased references.
    /// @param values Integer vector.
    /// @param label Character scalar, or NULL.
    #[miniextendr(convert(values, label))]
    pub fn describe(&self, values: &ConvertedIntegers, label: Option<&Text>) -> String {
        converted_refs_describe(values, label)
    }

    /// Refuse aliased native arguments.
    /// @param read Integer vector to read.
    /// @param write Integer vector to update.
    #[miniextendr(convert(read, write))]
    pub fn pair(&self, read: &Integers, write: &mut Integers) -> i32 {
        converted_refs_alias_pair(read, write)
    }
}

#[miniextendr]
pub trait ConvertedReads {
    fn total(&self, values: &Integers) -> i32;
}

#[miniextendr(env, noexport)]
impl ConvertedReads for ConvertedRefHost {
    /// @param values Integer vector.
    #[miniextendr(convert(values))]
    fn total(&self, values: &Integers) -> i32 {
        values.iter().sum()
    }
}

#[miniextendr]
pub trait GenericConvertedReads<T> {
    fn generic_total(&self, values: &[T]) -> i32;
}

#[miniextendr(env, noexport)]
impl GenericConvertedReads<i32> for ConvertedRefHost {
    /// @param values Integer vector.
    #[miniextendr(convert(values))]
    fn generic_total(&self, values: &Integers) -> i32 {
        values.iter().sum()
    }
}

/// A trait View already converts non-Self references; the opt-out fixes its R wrapper.
/// @noRd
#[miniextendr(noexport)]
pub fn converted_refs_through_view() -> i32 {
    unsafe {
        let scope = miniextendr_api::ProtectScope::new();
        let handle = scope.protect(miniextendr_api::trait_abi::ccall::mx_wrap(
            __mx_wrap_ConvertedRefHost(ConvertedRefHost),
        ));
        let view = ConvertedReadsView::from_sexp(handle.get());
        view.total(&[2, 3, 5])
    }
}
