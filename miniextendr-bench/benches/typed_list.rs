//! Benchmarks for `typed_list!` validation overhead.
//!
//! Measures the cost of `validate_list()` — the runtime check that a named R list
//! conforms to a `TypedListSpec`. Compares validation times for small (3 fields),
//! medium (10 fields), and large (50 fields) specs, plus failure paths and strict
//! (`@exact`) mode.

use miniextendr_api::ffi::{self, SEXP, SEXPTYPE};
use miniextendr_api::list::List;
use miniextendr_api::typed_list::{TypeSpec, TypedEntry, TypedListSpec, validate_list};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Fixture builders — create named R lists matching various spec sizes

/// Build a named VECSXP with `n` numeric(1) entries named "f0", "f1", ..., "f{n-1}".
fn make_numeric_list(n: usize) -> SEXP {
    unsafe {
        let list = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t));
        let names = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t));

        for i in 0..n {
            let val = ffi::Rf_ScalarReal(i as f64);
            ffi::SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);

            let key = format!("f{i}");
            let chars = ffi::Rf_mkCharLenCE(
                key.as_ptr().cast::<std::os::raw::c_char>(),
                key.len() as i32,
                ffi::CE_UTF8,
            );
            ffi::SET_STRING_ELT(names, i as ffi::R_xlen_t, chars);
        }

        ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
        ffi::Rf_unprotect(2);
        list
    }
}

/// Build a spec matching `make_numeric_list(n)`: all required numeric fields.
fn make_numeric_spec(n: usize) -> TypedListSpec {
    let entries: Vec<TypedEntry> = (0..n)
        .map(|i| {
            TypedEntry::required(
                // Leak the string so we get &'static str. Fine for benchmarks.
                Box::leak(format!("f{i}").into_boxed_str()),
                TypeSpec::Numeric(None),
            )
        })
        .collect();
    TypedListSpec::new(entries)
}

/// Build a strict spec (no extra fields allowed).
fn make_strict_numeric_spec(n: usize) -> TypedListSpec {
    let entries: Vec<TypedEntry> = (0..n)
        .map(|i| {
            TypedEntry::required(
                Box::leak(format!("f{i}").into_boxed_str()),
                TypeSpec::Numeric(None),
            )
        })
        .collect();
    TypedListSpec::strict(entries)
}

/// Build a mixed-type spec: alternating numeric, integer, character, logical.
fn make_mixed_spec(n: usize) -> TypedListSpec {
    let entries: Vec<TypedEntry> = (0..n)
        .map(|i| {
            let name: &'static str = Box::leak(format!("f{i}").into_boxed_str());
            let spec = match i % 4 {
                0 => TypeSpec::Numeric(None),
                1 => TypeSpec::Integer(None),
                2 => TypeSpec::Character(None),
                _ => TypeSpec::Logical(None),
            };
            TypedEntry::required(name, spec)
        })
        .collect();
    TypedListSpec::new(entries)
}

/// Build a named list with mixed types matching `make_mixed_spec`.
fn make_mixed_list(n: usize) -> SEXP {
    unsafe {
        let list = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t));
        let names = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t));

        for i in 0..n {
            let val = match i % 4 {
                0 => ffi::Rf_ScalarReal(i as f64),
                1 => ffi::Rf_ScalarInteger(i as i32),
                2 => {
                    let s = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::STRSXP, 1));
                    let c = ffi::Rf_mkCharLenCE(b"x".as_ptr().cast(), 1, ffi::CE_UTF8);
                    ffi::SET_STRING_ELT(s, 0, c);
                    ffi::Rf_unprotect(1);
                    s
                }
                _ => ffi::Rf_ScalarLogical(1),
            };
            ffi::SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);

            let key = format!("f{i}");
            let chars = ffi::Rf_mkCharLenCE(
                key.as_ptr().cast::<std::os::raw::c_char>(),
                key.len() as i32,
                ffi::CE_UTF8,
            );
            ffi::SET_STRING_ELT(names, i as ffi::R_xlen_t, chars);
        }

        ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
        ffi::Rf_unprotect(2);
        list
    }
}

const FIELD_COUNTS: &[usize] = &[3, 10, 50];
// endregion

// region: Group 1: Homogeneous numeric validation (happy path)

mod numeric_validation {
    use super::*;

    /// Validate a list with all-numeric fields against a matching spec.
    #[divan::bench(args = FIELD_COUNTS)]
    fn validate_pass(n: usize) {
        let list = make_numeric_list(n);
        let spec = make_numeric_spec(n);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }

    /// Validate with a strict spec (no extra fields allowed).
    #[divan::bench(args = FIELD_COUNTS)]
    fn validate_strict_pass(n: usize) {
        let list = make_numeric_list(n);
        let spec = make_strict_numeric_spec(n);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }
}
// endregion

// region: Group 2: Mixed-type validation

mod mixed_validation {
    use super::*;

    /// Validate a list with alternating types against a mixed spec.
    #[divan::bench(args = FIELD_COUNTS)]
    fn validate_mixed_pass(n: usize) {
        let list = make_mixed_list(n);
        let spec = make_mixed_spec(n);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }
}
// endregion

// region: Group 3: Failure paths (validation rejects the list)

mod failure_paths {
    use super::*;

    /// Validation fails immediately: first field has wrong type.
    #[divan::bench(args = FIELD_COUNTS)]
    fn wrong_type_first_field(n: usize) {
        // List has all numeric, but spec expects integer for first field.
        let list = make_numeric_list(n);
        let mut entries: Vec<TypedEntry> = (0..n)
            .map(|i| {
                TypedEntry::required(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(None),
                )
            })
            .collect();
        // Override first entry to expect integer (will fail on numeric input).
        entries[0] = TypedEntry::required("f0", TypeSpec::Integer(None));
        let spec = TypedListSpec::new(entries);

        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_err());
        let _ = divan::black_box(result);
    }

    /// Validation fails: required field is missing.
    #[divan::bench(args = FIELD_COUNTS)]
    fn missing_required_field(n: usize) {
        let list = make_numeric_list(n);
        let mut entries: Vec<TypedEntry> = (0..n)
            .map(|i| {
                TypedEntry::required(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(None),
                )
            })
            .collect();
        // Add a required field that doesn't exist in the list.
        entries.push(TypedEntry::required("missing_field", TypeSpec::Any));
        let spec = TypedListSpec::new(entries);

        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_err());
        let _ = divan::black_box(result);
    }

    /// Strict validation fails: list has extra fields not in spec.
    #[divan::bench]
    fn extra_fields_rejected() {
        // List has 10 fields but spec only expects 3 (strict mode).
        let list = make_numeric_list(10);
        let spec = make_strict_numeric_spec(3);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_err());
        let _ = divan::black_box(result);
    }
}
// endregion

// region: Group 4: Optional fields

mod optional_fields {
    use super::*;

    /// Spec has optional fields that are present in the list.
    #[divan::bench(args = FIELD_COUNTS)]
    fn all_optional_present(n: usize) {
        let list = make_numeric_list(n);
        let entries: Vec<TypedEntry> = (0..n)
            .map(|i| {
                TypedEntry::optional(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(None),
                )
            })
            .collect();
        let spec = TypedListSpec::new(entries);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }

    /// Spec has optional fields that are absent (list is smaller).
    #[divan::bench]
    fn optional_fields_missing() {
        // List has 3 fields, spec expects 3 required + 7 optional.
        let list = make_numeric_list(3);
        let mut entries: Vec<TypedEntry> = (0..3)
            .map(|i| {
                TypedEntry::required(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(None),
                )
            })
            .collect();
        for i in 3..10 {
            entries.push(TypedEntry::optional(
                Box::leak(format!("f{i}").into_boxed_str()),
                TypeSpec::Numeric(None),
            ));
        }
        let spec = TypedListSpec::new(entries);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }
}
// endregion

// region: Group 5: Length-constrained validation

mod length_check {
    use super::*;

    /// Validate numeric fields with exact length constraint (length=1 scalar).
    #[divan::bench(args = FIELD_COUNTS)]
    fn validate_with_length(n: usize) {
        let list = make_numeric_list(n);
        let entries: Vec<TypedEntry> = (0..n)
            .map(|i| {
                TypedEntry::required(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(Some(1)), // Exact length = 1
                )
            })
            .collect();
        let spec = TypedListSpec::new(entries);
        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_ok());
        let _ = divan::black_box(result);
    }

    /// Length constraint fails on first field.
    #[divan::bench]
    fn length_mismatch() {
        let list = make_numeric_list(5);
        let mut entries: Vec<TypedEntry> = (0..5)
            .map(|i| {
                TypedEntry::required(
                    Box::leak(format!("f{i}").into_boxed_str()),
                    TypeSpec::Numeric(None),
                )
            })
            .collect();
        // First field: expect length=100 but actual is length=1.
        entries[0] = TypedEntry::required("f0", TypeSpec::Numeric(Some(100)));
        let spec = TypedListSpec::new(entries);

        let list = unsafe { List::from_raw(list) };
        let result = validate_list(list, &spec);
        assert!(result.is_err());
        let _ = divan::black_box(result);
    }
}
// endregion

// region: Group 6: Spec construction cost (baseline)

mod spec_construction {
    use super::*;

    /// Cost of building a TypedListSpec with n required numeric entries.
    #[divan::bench(args = FIELD_COUNTS)]
    fn build_spec(n: usize) {
        let spec = make_numeric_spec(n);
        divan::black_box(spec);
    }

    /// Cost of building a strict TypedListSpec.
    #[divan::bench(args = FIELD_COUNTS)]
    fn build_strict_spec(n: usize) {
        let spec = make_strict_numeric_spec(n);
        divan::black_box(spec);
    }

    /// Cost of building a mixed-type spec.
    #[divan::bench(args = FIELD_COUNTS)]
    fn build_mixed_spec(n: usize) {
        let spec = make_mixed_spec(n);
        divan::black_box(spec);
    }
}
// endregion
