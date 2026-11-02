//! Fixtures for `TypeMismatchError`: a function that reads an external
//! pointer with `ExternalPtr::wrap_sexp_with_error` and reports the variant
//! it got, as a package's own code tells a live object from one restored
//! from a saved session (`Restored`, `OtherVersion`) and from a pointer it
//! did not build.
//!
//! See `tests/testthat/test-type-mismatch.R`.

use miniextendr_api::externalptr::{ExternalPtr, TypeMismatchError};
use miniextendr_api::{NamedListBuilder, SEXP, SEXPTYPE, SexpExt, miniextendr};

/// An object of this package. It does not derive serde, so one restored from
/// a saved session comes back without its Rust value.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MismatchProbe {
    value: i32,
}

/// Another type, whose pointer is not a [`MismatchProbe`].
#[derive(miniextendr_api::ExternalPtr)]
pub struct MismatchOther;

/// A new `MismatchProbe` holding `value`, as a bare external pointer.
/// @param value An integer.
#[miniextendr(noexport)]
pub fn mismatch_probe_new(value: i32) -> MismatchProbe {
    MismatchProbe { value }
}

/// A new `MismatchOther`, as a bare external pointer.
#[miniextendr(noexport)]
pub fn mismatch_other_new() -> MismatchOther {
    MismatchOther
}

/// How `ExternalPtr::<MismatchProbe>::wrap_sexp_with_error` reads `x`:
/// `list(variant = "live", value = <value>)`, or the error's variant
/// (`"null_pointer"`, `"invalid_type_id"`, `"mismatch"`, `"restored"`,
/// `"other_version"`), its message and `restored_classes()`, and the fields
/// the variant carries.
/// @param x An external pointer.
#[miniextendr(noexport)]
pub fn mismatch_probe_read(x: SEXP) -> NamedListBuilder {
    assert_eq!(
        x.type_of(),
        SEXPTYPE::EXTPTRSXP,
        "`x` is an external pointer"
    );
    let mut out = NamedListBuilder::new();
    // SAFETY: R's main thread; `x` is the rooted `.Call()` argument and an
    // EXTPTRSXP.
    let error = match unsafe { ExternalPtr::<MismatchProbe>::wrap_sexp_with_error(x) } {
        Ok(probe) => {
            out.push("variant", "live");
            out.push("value", probe.borrow().value);
            return out;
        }
        Err(error) => error,
    };
    let variant = match &error {
        TypeMismatchError::NullPointer => "null_pointer",
        TypeMismatchError::InvalidTypeId => "invalid_type_id",
        TypeMismatchError::Mismatch { .. } => "mismatch",
        TypeMismatchError::Restored { .. } => "restored",
        TypeMismatchError::OtherVersion { .. } => "other_version",
    };
    out.push("variant", variant);
    out.push("message", error.to_string());
    out.push("classes", error.restored_classes());
    match error {
        TypeMismatchError::Mismatch { expected, found } => {
            out.push("expected", expected);
            out.push("found", found);
        }
        TypeMismatchError::Restored { type_name, .. } => {
            out.push("type_name", type_name);
        }
        TypeMismatchError::OtherVersion {
            type_name,
            crate_name,
            saved,
            current,
            ..
        } => {
            out.push("type_name", type_name);
            out.push("crate_name", crate_name);
            out.push("saved", saved);
            out.push("current", current);
        }
        TypeMismatchError::NullPointer | TypeMismatchError::InvalidTypeId => {}
    }
    out
}
