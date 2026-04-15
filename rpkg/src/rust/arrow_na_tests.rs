//! Tests for NA handling in zero-copy Arrow ↔ R conversions.
//!
//! R uses sentinel values for NA (NA_integer_ = i32::MIN, NA_real_ = specific NaN,
//! NA_character_ = R_NaString, NA_logical = i32::MIN). Arrow uses a separate validity
//! bitmask. These tests verify correctness across:
//! - NA roundtrips (R → Arrow → R)
//! - NA introduced by Arrow computation
//! - R-side mutation after Arrow roundtrip
//! - Serialization (saveRDS/readRDS) with NAs
//! - ALTREP Arrow arrays with NAs
//! - Edge cases: all-NA, no-NA, single-element, alternating

use miniextendr_api::arrow_impl::{
    Array, BooleanArray, Float64Array, Int32Array, RecordBatch, StringArray,
};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;

// region: Float64Array NA patterns

/// Roundtrip f64 with NAs at specific positions.
/// Returns the null count from the Arrow side.
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_null_count(v: Float64Array) -> i32 {
    v.logical_null_count() as i32
}

/// Roundtrip f64 through Arrow, verifying NA positions survive.
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_roundtrip(v: Float64Array) -> Float64Array {
    v
}

/// Create an Arrow array from R, compute on it (NAs become nulls in result),
/// then return to R. Tests that Arrow null propagation works correctly.
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_add_one(v: Float64Array) -> Float64Array {
    // Arrow null semantics: NA + 1 = NA (null propagates)
    v.iter().map(|opt| opt.map(|x| x + 1.0)).collect()
}

/// Double roundtrip: R → Arrow → compute → R → Arrow → R.
/// The intermediate R vector should have NAs where Arrow had nulls,
/// and the second Arrow conversion should re-detect those NAs.
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_double_roundtrip(v: Float64Array) -> Float64Array {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    // First: Arrow compute (multiply by 2, preserving nulls)
    let computed: Float64Array = v.iter().map(|opt| opt.map(|x| x * 2.0)).collect();

    // Second: Arrow → R (restores NA sentinels where nulls were)
    let r_sexp = computed.into_sexp();

    // Third: R → Arrow (re-scans for NA sentinels)
    let arr2: Float64Array = TryFromSexp::try_from_sexp(r_sexp).unwrap();

    // Verify null count survived the double conversion
    arr2
}

/// Return null positions as a logical vector (TRUE = null at that index).
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_null_positions(v: Float64Array) -> Vec<bool> {
    (0..v.len()).map(|i| v.is_null(i)).collect()
}

/// Return non-null values only (filtering out NAs).
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_compact(v: Float64Array) -> Vec<f64> {
    v.iter().flatten().collect()
}

// endregion

// region: Int32Array NA patterns

/// Roundtrip i32 through Arrow, verifying NA positions survive.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_roundtrip(v: Int32Array) -> Int32Array {
    v
}

/// Return the null count for an Int32Array.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_null_count(v: Int32Array) -> i32 {
    v.logical_null_count() as i32
}

/// Compute on i32 Arrow array (add 10), preserving nulls.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_add_ten(v: Int32Array) -> Int32Array {
    v.iter().map(|opt| opt.map(|x| x + 10)).collect()
}

/// Double roundtrip for i32: R → Arrow → compute → R → Arrow → R.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_double_roundtrip(v: Int32Array) -> Int32Array {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    let computed: Int32Array = v.iter().map(|opt| opt.map(|x| x * 3)).collect();
    let r_sexp = computed.into_sexp();
    let arr2: Int32Array = TryFromSexp::try_from_sexp(r_sexp).unwrap();
    arr2
}

/// Return null positions as a logical vector.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_null_positions(v: Int32Array) -> Vec<bool> {
    (0..v.len()).map(|i| v.is_null(i)).collect()
}

// endregion

// region: BooleanArray NA patterns

/// Roundtrip logical through Arrow.
/// @param v logical vector
#[miniextendr]
pub fn arrow_na_bool_roundtrip(v: BooleanArray) -> BooleanArray {
    v
}

/// Return null count for BooleanArray.
/// @param v logical vector
#[miniextendr]
pub fn arrow_na_bool_null_count(v: BooleanArray) -> i32 {
    v.logical_null_count() as i32
}

/// Return null positions for BooleanArray.
/// @param v logical vector
#[miniextendr]
pub fn arrow_na_bool_null_positions(v: BooleanArray) -> Vec<bool> {
    (0..v.len()).map(|i| v.is_null(i)).collect()
}

// endregion

// region: StringArray NA patterns

/// Roundtrip character through Arrow.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_roundtrip(v: StringArray) -> StringArray {
    v
}

/// Return null count for StringArray.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_null_count(v: StringArray) -> i32 {
    v.logical_null_count() as i32
}

/// Return null positions for StringArray.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_null_positions(v: StringArray) -> Vec<bool> {
    (0..v.len()).map(|i| v.is_null(i)).collect()
}

// endregion

// region: RecordBatch NA patterns

/// Roundtrip data.frame with various NA patterns through Arrow.
/// @param rb data.frame
#[miniextendr]
pub fn arrow_na_recordbatch_roundtrip(rb: RecordBatch) -> RecordBatch {
    rb
}

/// Return per-column null counts as integer vector.
/// @param rb data.frame
#[miniextendr]
pub fn arrow_na_recordbatch_null_counts(rb: RecordBatch) -> Vec<i32> {
    (0..rb.num_columns())
        .map(|i| rb.column(i).logical_null_count() as i32)
        .collect()
}

// endregion

// region: ALTREP Arrow with NAs

/// Create a Rust-computed Float64Array (with NAs) and return as ALTREP.
/// Multiplies input by 10; NAs become Arrow nulls which map to R NAs in ALTREP elt().
/// @param v numeric vector
#[miniextendr]
pub fn arrow_na_f64_altrep(v: Float64Array) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let computed: Float64Array = v.iter().map(|opt| opt.map(|f| f * 10.0)).collect();
    computed.into_sexp_altrep()
}

/// Create a Rust-computed Int32Array (with NAs) and return as ALTREP.
/// @param v integer vector
#[miniextendr]
pub fn arrow_na_i32_altrep(v: Int32Array) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let computed: Int32Array = v.iter().map(|opt| opt.map(|i| i + 100)).collect();
    computed.into_sexp_altrep()
}

/// Create an all-null Float64Array as ALTREP.
/// @param n length
#[miniextendr]
pub fn arrow_na_f64_all_null_altrep(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let arr: Float64Array = (0..n).map(|_| Option::<f64>::None).collect();
    arr.into_sexp_altrep()
}

/// Create an all-null Int32Array as ALTREP.
/// @param n length
#[miniextendr]
pub fn arrow_na_i32_all_null_altrep(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let arr: Int32Array = (0..n).map(|_| Option::<i32>::None).collect();
    arr.into_sexp_altrep()
}

// endregion

// region: StringArray NA computation + ALTREP

/// Uppercase non-null strings, preserving nulls.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_uppercase(v: StringArray) -> StringArray {
    let values: Vec<Option<String>> = (0..v.len())
        .map(|i| {
            if v.is_null(i) {
                None
            } else {
                Some(v.value(i).to_uppercase())
            }
        })
        .collect();
    let refs: Vec<Option<&str>> = values.iter().map(|o| o.as_deref()).collect();
    StringArray::from(refs)
}

/// Filter out null strings, returning only non-null values.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_compact(v: StringArray) -> Vec<String> {
    (0..v.len())
        .filter(|&i| !v.is_null(i))
        .map(|i| v.value(i).to_string())
        .collect()
}

/// Double roundtrip: R → Arrow → compute → R → Arrow → R.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_double_roundtrip(v: StringArray) -> StringArray {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    // First: compute (append "!")
    let values: Vec<Option<String>> = (0..v.len())
        .map(|i| {
            if v.is_null(i) {
                None
            } else {
                Some(format!("{}!", v.value(i)))
            }
        })
        .collect();
    let refs: Vec<Option<&str>> = values.iter().map(|o| o.as_deref()).collect();
    let computed = StringArray::from(refs);

    // Second: Arrow → R (restores NA_character_ where nulls were)
    let r_sexp = computed.into_sexp();

    // Third: R → Arrow (re-scans for NA_character_)
    TryFromSexp::try_from_sexp(r_sexp).unwrap()
}

/// Create a Rust-computed StringArray (with NAs) and return as ALTREP.
/// @param v character vector
#[miniextendr]
pub fn arrow_na_string_altrep(v: StringArray) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let values: Vec<Option<String>> = (0..v.len())
        .map(|i| {
            if v.is_null(i) {
                None
            } else {
                Some(format!("[{}]", v.value(i)))
            }
        })
        .collect();
    let refs: Vec<Option<&str>> = values.iter().map(|o| o.as_deref()).collect();
    let arr = StringArray::from(refs);
    arr.into_sexp_altrep()
}

/// Create an all-null StringArray as ALTREP.
/// @param n length
#[miniextendr]
pub fn arrow_na_string_all_null_altrep(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let arr: StringArray = (0..n).map(|_| Option::<&str>::None).collect();
    arr.into_sexp_altrep()
}

// endregion

// region: Stale null bitmap tests (R mutation after Arrow conversion)

/// Convert R vector to Arrow, return the Arrow null_count.
/// The Arrow array holds a zero-copy reference to R's buffer.
/// If R mutates the buffer after conversion, the Arrow null bitmap becomes stale.
///
/// This fixture takes the SEXP, converts to Float64Array (scanning for NAs),
/// then reads element values and null status from the Arrow side.
/// Returns: `[null_count, value_at_0, is_null_at_0, value_at_1, is_null_at_1, ...]`
/// @param x SEXP (numeric vector)
#[miniextendr]
pub fn arrow_na_f64_inspect_arrow(x: SEXP) -> Vec<f64> {
    use miniextendr_api::from_r::TryFromSexp;
    let arr: Float64Array = TryFromSexp::try_from_sexp(x).unwrap();
    let mut result = vec![arr.logical_null_count() as f64];
    for i in 0..arr.len() {
        // Raw value in the data buffer (may be NA_REAL sentinel)
        result.push(if arr.is_null(i) {
            f64::NAN // represent null as NaN for inspection
        } else {
            arr.value(i)
        });
        result.push(if arr.is_null(i) { 1.0 } else { 0.0 });
    }
    result
}

/// Allocate an R-backed Arrow buffer, fill with non-NA values, then
/// set one element to NA_REAL in the R buffer. Return the Arrow null_count
/// (which was computed BEFORE the mutation) and the raw values.
///
/// This demonstrates the stale bitmap problem: Arrow's bitmap says "all valid"
/// but the R buffer now contains an NA sentinel.
/// @param n length
#[miniextendr]
pub fn arrow_na_f64_stale_bitmap_demo(n: i32) -> Vec<f64> {
    use miniextendr_api::into_r::IntoR;
    use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;

    let n = n as usize;
    let (buffer, sexp) = unsafe { alloc_r_backed_buffer::<f64>(n) };

    // Fill with non-NA values
    for i in 0..n {
        miniextendr_api::ffi::SexpExt::set_real_elt(&sexp, i as isize, (i + 1) as f64);
    }

    // Create Arrow array — bitmap says "no nulls" (all values are valid)
    let scalar_buffer =
        miniextendr_api::optionals::arrow_impl::arrow_buffer::ScalarBuffer::<f64>::from(buffer);
    let arr = Float64Array::new(scalar_buffer, None); // None = no nulls

    // Now mutate the R buffer AFTER Arrow array was created
    miniextendr_api::ffi::SexpExt::set_real_elt(
        &sexp,
        1, // set index 1 to NA
        miniextendr_api::altrep_traits::NA_REAL,
    );

    // Arrow's view: null_count is still 0 (bitmap is stale)
    let null_count = arr.logical_null_count() as f64;

    // But if we convert back to R via IntoR, pointer recovery returns the
    // original SEXP (which now has the NA sentinel in it)
    let result_sexp = arr.into_sexp();

    // Read back from R to see what R thinks
    use miniextendr_api::ffi::SexpExt;
    let r_len = result_sexp.len();
    let mut result = vec![null_count, r_len as f64];
    for i in 0..r_len {
        result.push(result_sexp.real_elt(i as isize));
    }
    result
}

// endregion

// region: Zero-copy identity with NA edge cases

/// Returns TRUE if the zero-copy round-trip preserves identity even with NAs.
/// This tests that R's NA sentinels in the data buffer don't break pointer recovery.
/// @param x SEXP
#[miniextendr]
pub fn arrow_na_f64_zero_copy_identity(x: SEXP) -> bool {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;
    let arr: Float64Array = TryFromSexp::try_from_sexp(x).unwrap();
    let result = arr.into_sexp();
    result == x
}

/// Returns TRUE if the zero-copy round-trip preserves identity for i32 with NAs.
/// @param x SEXP
#[miniextendr]
pub fn arrow_na_i32_zero_copy_identity(x: SEXP) -> bool {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;
    let arr: Int32Array = TryFromSexp::try_from_sexp(x).unwrap();
    let result = arr.into_sexp();
    result == x
}

// endregion
