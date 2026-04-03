//! Zero-copy verification tests.
//!
//! These fixtures verify that pointer recovery works: the SEXP returned by
//! IntoR is the exact same SEXP that was passed to TryFromSexp.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;
use std::borrow::Cow;

// region: Cow<[T]> round-trip identity

/// Returns TRUE if Cow<[f64]> round-trip returns the same R object.
/// @export
#[miniextendr]
pub fn zero_copy_cow_f64_identity(x: SEXP) -> bool {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    let cow: Cow<'static, [f64]> = TryFromSexp::try_from_sexp(x).unwrap();
    let result = cow.into_sexp();
    result == x
}

/// Returns TRUE if Cow<[i32]> round-trip returns the same R object.
/// @export
#[miniextendr]
pub fn zero_copy_cow_i32_identity(x: SEXP) -> bool {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    let cow: Cow<'static, [i32]> = TryFromSexp::try_from_sexp(x).unwrap();
    let result = cow.into_sexp();
    result == x
}

// endregion

// region: Cow<str> scalar

/// Returns TRUE if Cow<str> from R is Cow::Borrowed (zero-copy).
/// @export
#[miniextendr]
pub fn zero_copy_cow_str_is_borrowed(x: Cow<'static, str>) -> bool {
    matches!(x, Cow::Borrowed(_))
}

// endregion

// region: Vec<Cow<str>>

/// Returns TRUE if all elements of Vec<Cow<str>> are Cow::Borrowed.
/// @export
#[miniextendr]
pub fn zero_copy_vec_cow_str_all_borrowed(x: Vec<Cow<'static, str>>) -> bool {
    x.iter().all(|c| matches!(c, Cow::Borrowed(_)))
}

// endregion

// region: Arrow array identity (requires arrow feature)

#[cfg(feature = "arrow")]
mod arrow {
    use miniextendr_api::ffi::SEXP;
    use miniextendr_api::miniextendr;

    /// Returns TRUE if Float64Array round-trip returns the same R object.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_f64_identity(x: SEXP) -> bool {
        use miniextendr_api::arrow_impl::Float64Array;
        use miniextendr_api::from_r::TryFromSexp;
        use miniextendr_api::into_r::IntoR;

        let arr: Float64Array = TryFromSexp::try_from_sexp(x).unwrap();
        let result = arr.into_sexp();
        result == x
    }

    /// Returns TRUE if Int32Array round-trip returns the same R object.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_i32_identity(x: SEXP) -> bool {
        use miniextendr_api::arrow_impl::Int32Array;
        use miniextendr_api::from_r::TryFromSexp;
        use miniextendr_api::into_r::IntoR;

        let arr: Int32Array = TryFromSexp::try_from_sexp(x).unwrap();
        let result = arr.into_sexp();
        result == x
    }

    /// Returns TRUE if UInt8Array round-trip returns the same R object.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_u8_identity(x: SEXP) -> bool {
        use miniextendr_api::arrow_impl::UInt8Array;
        use miniextendr_api::from_r::TryFromSexp;
        use miniextendr_api::into_r::IntoR;

        let arr: UInt8Array = TryFromSexp::try_from_sexp(x).unwrap();
        let result = arr.into_sexp();
        result == x
    }

    /// Returns FALSE — computed Arrow array has different backing memory.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_f64_computed_is_different(x: SEXP) -> bool {
        use miniextendr_api::arrow_impl::Float64Array;
        use miniextendr_api::from_r::TryFromSexp;
        use miniextendr_api::into_r::IntoR;

        let arr: Float64Array = TryFromSexp::try_from_sexp(x).unwrap();
        // Compute creates new buffer — not R-backed
        let doubled: Float64Array = arr.iter().map(|v| v.map(|f| f * 2.0)).collect();
        let result = doubled.into_sexp();
        result == x // should be FALSE
    }

    /// Verifies r_memory::sexprec_data_offset is nonzero (init ran).
    /// @export
    #[miniextendr]
    pub fn zero_copy_sexprec_offset() -> i32 {
        miniextendr_api::r_memory::sexprec_data_offset() as i32
    }
}

// endregion

// region: ProtectedStrVec

/// Count unique non-NA strings via ProtectedStrVec.
/// @export
#[miniextendr]
pub fn zero_copy_protected_strvec_unique(strings: miniextendr_api::ProtectedStrVec) -> i32 {
    use std::collections::HashSet;
    let unique: HashSet<&str> = strings.iter().filter_map(|s| s).collect();
    unique.len() as i32
}

// endregion
