//! Compile-time coverage for the full `impl_alt*_from_data!` knob matrix.
//!
//! The public ALTREP family macros accept an optional knob list
//! (`dataptr` / `serialize` / `subset`, in canonical order) that routes
//! through the internal `__impl_alt_family!` knob matrix. Most knob
//! combinations have no production call site inside this crate (e.g. the
//! `subset` arms), so this test instantiates every supported combination to
//! guarantee the matrix keeps expanding to valid impls.
//!
//! These fixtures are compile-only: the generated `Altrep`/`AltVec`/family
//! impls are never invoked (that requires a live R runtime — covered by the
//! rpkg fixtures).
// Fixture structs exist solely to host macro-generated impls.
#![allow(dead_code)]

use miniextendr_api::SEXP;
use miniextendr_api::altrep_data::{
    AltIntegerData, AltLogicalData, AltStringData, AltrepDataptr, AltrepExtract,
    AltrepExtractSubset, AltrepLen, AltrepSerialize, Logical,
};

// region: Fixture helpers

/// Declare an integer-family fixture: `AltrepLen` + `AltIntegerData`.
macro_rules! int_fixture {
    ($name:ident) => {
        struct $name(Vec<i32>);

        impl AltrepLen for $name {
            fn len(&self) -> usize {
                self.0.len()
            }
        }

        impl AltIntegerData for $name {
            fn elt(&self, i: usize) -> i32 {
                self.0[i]
            }
        }
    };
}

/// Manual (non-TypedExternal) `AltrepExtract`. Serialize-knob fixtures must
/// NOT use this — they implement `TypedExternal` and get `AltrepExtract`
/// through its blanket impl.
macro_rules! fixture_extract {
    ($name:ident) => {
        impl AltrepExtract for $name {
            unsafe fn altrep_extract_ref(_x: SEXP) -> &'static Self {
                unreachable!("compile-time fixture — never dispatched")
            }

            unsafe fn altrep_extract_mut(_x: SEXP) -> &'static mut Self {
                unreachable!("compile-time fixture — never dispatched")
            }
        }
    };
}

macro_rules! fixture_dataptr {
    ($name:ident, $elem:ty) => {
        impl AltrepDataptr<$elem> for $name {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                None
            }
        }
    };
}

/// The `serialize` knob additionally requires `TypedExternal`: the generated
/// `Altrep::unserialize` reconstructs the value via `ExternalPtr::new_unchecked`.
macro_rules! fixture_serialize {
    ($name:ident) => {
        impl miniextendr_api::externalptr::TypedExternal for $name {
            const TYPE_NAME: &'static str = stringify!($name);
            const TYPE_NAME_CSTR: &'static [u8] = concat!(stringify!($name), "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(module_path!(), "::", stringify!($name), "\0").as_bytes();
        }

        impl AltrepSerialize for $name {
            fn serialized_state(&self) -> SEXP {
                unreachable!("compile-time fixture — never dispatched")
            }

            fn unserialize(_state: SEXP) -> Option<Self> {
                None
            }
        }
    };
}

macro_rules! fixture_subset {
    ($name:ident) => {
        impl AltrepExtractSubset for $name {
            fn extract_subset(&self, _indices: &[i32]) -> Option<SEXP> {
                None
            }
        }
    };
}
// endregion

// region: Integer family — all six knob combinations

int_fixture!(IntBasic);
fixture_extract!(IntBasic);
miniextendr_api::impl_altinteger_from_data!(IntBasic);

int_fixture!(IntDataptr);
fixture_extract!(IntDataptr);
fixture_dataptr!(IntDataptr, i32);
miniextendr_api::impl_altinteger_from_data!(IntDataptr, dataptr);

int_fixture!(IntSerialize);
fixture_serialize!(IntSerialize);
miniextendr_api::impl_altinteger_from_data!(IntSerialize, serialize);

int_fixture!(IntSubset);
fixture_extract!(IntSubset);
fixture_subset!(IntSubset);
miniextendr_api::impl_altinteger_from_data!(IntSubset, subset);

int_fixture!(IntDataptrSerialize);
fixture_dataptr!(IntDataptrSerialize, i32);
fixture_serialize!(IntDataptrSerialize);
miniextendr_api::impl_altinteger_from_data!(IntDataptrSerialize, dataptr, serialize);

int_fixture!(IntSubsetSerialize);
fixture_subset!(IntSubsetSerialize);
fixture_serialize!(IntSubsetSerialize);
miniextendr_api::impl_altinteger_from_data!(IntSubsetSerialize, subset, serialize);
// endregion

// region: Generic form — `impl_alt*_from_data_generic!` (audit D1)
//
// Coverage for the generic sibling that the ~19 hand-rolled iterator/stream
// ALTREP adaptors (`altrep_data::iter::*`, `altrep_data::stream::*`) route
// through: a type with a generic parameter and a `where` clause, mirroring
// the shape those adaptors need (e.g. `IterIntCoerceData<I, T>`).

/// Generic fixture: wraps any `T: Copy + Into<i32> + 'static` in a `Vec`.
struct GenericIntFixture<T>(Vec<T>)
where
    T: Copy + Into<i32> + 'static;

impl<T> AltrepLen for GenericIntFixture<T>
where
    T: Copy + Into<i32> + 'static,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> AltIntegerData for GenericIntFixture<T>
where
    T: Copy + Into<i32> + 'static,
{
    fn elt(&self, i: usize) -> i32 {
        self.0[i].into()
    }
}

// Manual (non-TypedExternal) `AltrepExtract`, generic over `T` — the
// `fixture_extract!` helper above only takes a bare ident, not a generic type.
impl<T> AltrepExtract for GenericIntFixture<T>
where
    T: Copy + Into<i32> + 'static,
{
    unsafe fn altrep_extract_ref(_x: SEXP) -> &'static Self {
        unreachable!("compile-time fixture — never dispatched")
    }

    unsafe fn altrep_extract_mut(_x: SEXP) -> &'static mut Self {
        unreachable!("compile-time fixture — never dispatched")
    }
}

miniextendr_api::impl_altinteger_from_data_generic!(
    {T} GenericIntFixture<T> {T: Copy + Into<i32> + 'static}
);
// endregion

// region: Logical family — asymmetric element types (dataptr: i32, materializing: RLogical)

struct LglBasic(Vec<bool>);

impl AltrepLen for LglBasic {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AltLogicalData for LglBasic {
    fn elt(&self, i: usize) -> Logical {
        Logical::from(self.0[i])
    }
}

fixture_extract!(LglBasic);
miniextendr_api::impl_altlogical_from_data!(LglBasic);
// endregion

// region: String family — default and `dataptr` both route to string_dataptr

struct StrBasic(Vec<Option<String>>);

impl AltrepLen for StrBasic {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AltStringData for StrBasic {
    fn elt(&self, i: usize) -> Option<&str> {
        self.0[i].as_deref()
    }
}

fixture_extract!(StrBasic);
miniextendr_api::impl_altstring_from_data!(StrBasic, dataptr);
// endregion

#[test]
fn knob_matrix_compiles() {
    // The test is the successful expansion above; spot-check a few of the
    // generated associated consts so the impls are demonstrably present.
    use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep};

    const {
        assert!(!<IntBasic as Altrep>::HAS_SERIALIZED_STATE);
        assert!(<IntSerialize as Altrep>::HAS_SERIALIZED_STATE);
        assert!(<IntDataptrSerialize as Altrep>::HAS_SERIALIZED_STATE);
        assert!(<IntBasic as AltVec>::HAS_DATAPTR);
        assert!(<IntSubset as AltVec>::HAS_EXTRACT_SUBSET);
        assert!(<IntSubsetSerialize as AltVec>::HAS_EXTRACT_SUBSET);
        assert!(<IntBasic as AltInteger>::HAS_ELT);

        // Generic form (audit D1): same bridge stack, instantiated for a
        // type with a generic parameter + where clause.
        assert!(<GenericIntFixture<u8> as AltVec>::HAS_DATAPTR);
        assert!(<GenericIntFixture<u8> as AltInteger>::HAS_ELT);
    }
}
