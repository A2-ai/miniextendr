//! Zero-copy verification tests.
//!
//! These fixtures verify that pointer recovery works: the SEXP returned by
//! IntoR is the exact same SEXP that was passed to TryFromSexp.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;
use std::borrow::Cow;

// region: Cow<[T]> round-trip identity

/// Returns TRUE if `Cow<[f64]>` round-trip returns the same R object.
/// @export
#[miniextendr]
pub fn zero_copy_cow_f64_identity(x: SEXP) -> bool {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    let cow: Cow<'static, [f64]> = TryFromSexp::try_from_sexp(x).unwrap();
    let result = cow.into_sexp();
    result == x
}

/// Returns TRUE if `Cow<[i32]>` round-trip returns the same R object.
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

    /// Round-trip Float64Array through Arrow and return the result.
    /// The zero-copy path returns the original R SEXP — serialization is trivial.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_f64_roundtrip(
        x: miniextendr_api::arrow_impl::Float64Array,
    ) -> miniextendr_api::arrow_impl::Float64Array {
        x
    }

    /// Round-trip Int32Array through Arrow and return the result.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_i32_roundtrip(
        x: miniextendr_api::arrow_impl::Int32Array,
    ) -> miniextendr_api::arrow_impl::Int32Array {
        x
    }

    /// Create a Rust-allocated Float64Array (NOT R-backed) and return as ALTREP.
    /// The data lives in Rust memory — this is the interesting serialization case.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_f64_altrep(
        x: miniextendr_api::arrow_impl::Float64Array,
    ) -> miniextendr_api::ffi::SEXP {
        use miniextendr_api::IntoRAltrep;
        // Compute creates a Rust-owned buffer (not R-backed)
        let computed: miniextendr_api::arrow_impl::Float64Array =
            x.iter().map(|v| v.map(|f| f * 10.0)).collect();
        computed.into_sexp_altrep()
    }

    /// Create a Rust-allocated Int32Array and return as ALTREP.
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_i32_altrep(
        x: miniextendr_api::arrow_impl::Int32Array,
    ) -> miniextendr_api::ffi::SEXP {
        use miniextendr_api::IntoRAltrep;
        let computed: miniextendr_api::arrow_impl::Int32Array =
            x.iter().map(|v| v.map(|i| i + 100)).collect();
        computed.into_sexp_altrep()
    }

    /// Create a Vec<f64> ALTREP (not Arrow, just plain Rust vec).
    /// @export
    #[miniextendr]
    pub fn zero_copy_vec_f64_altrep(n: i32) -> miniextendr_api::ffi::SEXP {
        use miniextendr_api::IntoRAltrep;
        let data: Vec<f64> = (0..n).map(|i| i as f64 * 1.5).collect();
        data.into_sexp_altrep()
    }

    /// Allocate an R-backed Arrow buffer, fill it, return as Float64Array.
    /// Tests the alloc_r_backed_buffer → pointer recovery round-trip.
    /// @export
    #[miniextendr]
    pub fn zero_copy_alloc_r_backed(n: i32) -> miniextendr_api::ffi::SEXP {
        use miniextendr_api::into_r::IntoR;
        use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;
        let n = n as usize;
        let (buffer, sexp) = unsafe { alloc_r_backed_buffer::<f64>(n) };
        // Fill via SexpExt element setters
        for i in 0..n {
            miniextendr_api::ffi::SexpExt::set_real_elt(&sexp, i as isize, (i + 1) as f64 * 100.0);
        }
        let values =
            miniextendr_api::optionals::arrow_impl::arrow_buffer::ScalarBuffer::<f64>::from(buffer);
        let array = miniextendr_api::optionals::arrow_impl::Float64Array::new(values, None);
        array.into_sexp()
    }

    /// Slice a Float64Array and return it — recovery should fail (different pointer).
    /// @export
    #[miniextendr]
    pub fn zero_copy_arrow_f64_sliced(x: miniextendr_api::ffi::SEXP) -> bool {
        use miniextendr_api::arrow_impl::Float64Array;
        use miniextendr_api::from_r::TryFromSexp;
        use miniextendr_api::into_r::IntoR;

        let arr: Float64Array = TryFromSexp::try_from_sexp(x).unwrap();
        let sliced = arr.slice(1, arr.len() - 1);
        let result = sliced.into_sexp();
        // Sliced pointer is shifted — recovery fails, result is a NEW SEXP
        result != x
    }
}

// endregion

// region: Cow<[f64]> roundtrip for serialization test

/// Round-trip `Cow<[f64]>` and return the result.
/// @export
#[miniextendr]
pub fn zero_copy_cow_f64_roundtrip(
    x: std::borrow::Cow<'static, [f64]>,
) -> std::borrow::Cow<'static, [f64]> {
    x
}

// endregion

// region: ProtectedStrVec

/// Count unique non-NA strings via ProtectedStrVec.
/// @export
#[miniextendr]
pub fn zero_copy_protected_strvec_unique(strings: miniextendr_api::ProtectedStrVec) -> i32 {
    use std::collections::HashSet;
    let unique: HashSet<&str> = strings.iter().flatten().collect();
    unique.len() as i32
}

// endregion
