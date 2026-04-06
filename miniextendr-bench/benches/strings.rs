//! String conversion benchmarks: comparing approaches for Rust <-> R string conversion.
//!
//! Compares:
//! - Into R (empty strings): `R_BlankString` vs `Rf_mkCharLenCE(ptr, 0, CE_UTF8)`
//! - From R: `CStr::from_ptr` (strlen scan) vs LENGTH-based slice construction

use miniextendr_api::ffi::{self, SEXP};
use miniextendr_bench::SIZES;
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

// region: Into R: Empty string handling

/// Current approach: Rf_mkCharLenCE with length 0
#[divan::bench]
fn into_r_empty_mkcharlen() -> SEXP {
    unsafe { raw_ffi::Rf_mkCharLenCE(b"".as_ptr().cast(), 0, ffi::CE_UTF8) }
}

/// Alternative: R_BlankString static (no FFI call, just static access)
#[divan::bench]
fn into_r_empty_blankstring() -> SEXP {
    unsafe { raw_ffi::R_BlankString }
}
// endregion

// region: Into R: Non-empty strings (baseline showing mkCharLenCE cost scales with size)

/// Rf_mkCharLenCE with pre-allocated string (avoids allocation in benchmark)
#[divan::bench(args = SIZES)]
fn into_r_str_mkcharlen(bencher: divan::Bencher, n: usize) {
    let s = "x".repeat(n);
    bencher.bench(|| unsafe {
        divan::black_box(raw_ffi::Rf_mkCharLenCE(
            s.as_ptr().cast(),
            n as i32,
            ffi::CE_UTF8,
        ))
    });
}
// endregion

// region: From R: String extraction approaches

/// Current: CStr::from_ptr (O(n) strlen scan to find null terminator)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn from_r_cstr(size_idx: usize) {
    let strsxp = fixtures().str_vec(size_idx);
    let charsxp = unsafe { raw_ffi::STRING_ELT(strsxp, 0) };
    let ptr = unsafe { raw_ffi::R_CHAR(charsxp) };
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    divan::black_box(cstr.to_str().unwrap());
}

/// Alternative: LENGTH + from_utf8_unchecked (O(1) length lookup, no strlen)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn from_r_length_slice(size_idx: usize) {
    let strsxp = fixtures().str_vec(size_idx);
    let charsxp = unsafe { raw_ffi::STRING_ELT(strsxp, 0) };
    let ptr = unsafe { raw_ffi::R_CHAR(charsxp) };
    let len = unsafe { raw_ffi::LENGTH(charsxp) } as usize;
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    // Use from_utf8_unchecked since R already guarantees UTF-8 for CE_UTF8 strings
    let s = unsafe { std::str::from_utf8_unchecked(bytes) };
    divan::black_box(s);
}

/// Variant: LENGTH + from_utf8 (with validation, for comparison)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn from_r_length_slice_validated(size_idx: usize) {
    let strsxp = fixtures().str_vec(size_idx);
    let charsxp = unsafe { raw_ffi::STRING_ELT(strsxp, 0) };
    let ptr = unsafe { raw_ffi::R_CHAR(charsxp) };
    let len = unsafe { raw_ffi::LENGTH(charsxp) } as usize;
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    // With UTF-8 validation
    let s = std::str::from_utf8(bytes).unwrap();
    divan::black_box(s);
}
// endregion

// region: CStr::from_ptr internals breakdown

/// Just the strlen part (CStr::from_ptr without to_str)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn from_r_cstr_only(size_idx: usize) {
    let strsxp = fixtures().str_vec(size_idx);
    let charsxp = unsafe { raw_ffi::STRING_ELT(strsxp, 0) };
    let ptr = unsafe { raw_ffi::R_CHAR(charsxp) };
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    divan::black_box(cstr);
}

/// Just the LENGTH call
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn from_r_length_only(size_idx: usize) {
    let strsxp = fixtures().str_vec(size_idx);
    let charsxp = unsafe { raw_ffi::STRING_ELT(strsxp, 0) };
    let len = unsafe { raw_ffi::LENGTH(charsxp) };
    divan::black_box(len);
}
// endregion

// region: Full TryFromSexp comparison (what users actually call)

/// Current TryFromSexp implementation path
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn tryfromsexp_str_current(size_idx: usize) {
    use miniextendr_api::TryFromSexp;
    let strsxp = fixtures().str_vec(size_idx);
    let s: &str = TryFromSexp::try_from_sexp(strsxp).unwrap();
    divan::black_box(s);
}
// endregion
