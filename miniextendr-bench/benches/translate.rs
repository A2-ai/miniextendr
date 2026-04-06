use std::ffi::CStr;

use miniextendr_api::ffi::{self, SEXP};
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

// region: Helpers that mirror the two strategies

#[inline(always)]
unsafe fn charsxp_to_string_r_char_utf8_only(charsxp: SEXP) -> String {
    let ptr = unsafe { raw_ffi::R_CHAR(charsxp) };
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("non-UTF-8 data in R_CHAR (use translate)")
        .to_owned()
}

#[inline(always)]
unsafe fn charsxp_to_string_translate_utf8(charsxp: SEXP) -> String {
    let ptr = unsafe { ffi::Rf_translateCharUTF8(charsxp) };
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("translateCharUTF8 not UTF-8")
        .to_owned()
}

#[inline(always)]
unsafe fn sexp_to_string_translate_utf8(sexp: SEXP) -> String {
    <String as miniextendr_api::TryFromSexp>::try_from_sexp(sexp).expect("TryFromSexp<String>")
}
// endregion

// region: CHARSXP-level microbenchmarks (isolate R API call cost)

#[divan::bench]
fn charsxp_r_char_ptr_utf8() {
    unsafe {
        let ptr = raw_ffi::R_CHAR(fixtures().utf8_charsxp());
        divan::black_box(ptr);
    }
}

#[divan::bench]
fn charsxp_translate_ptr_utf8() {
    unsafe {
        let ptr = ffi::Rf_translateCharUTF8(fixtures().utf8_charsxp());
        divan::black_box(ptr);
    }
}

#[divan::bench]
fn charsxp_translate_ptr_latin1() {
    unsafe {
        let ptr = ffi::Rf_translateCharUTF8(fixtures().latin1_charsxp());
        divan::black_box(ptr);
    }
}
// endregion

// region: End-to-end conversions (include Rust String allocation)

#[divan::bench]
fn charsxp_r_char_to_string_utf8() {
    unsafe {
        let s = charsxp_to_string_r_char_utf8_only(fixtures().utf8_charsxp());
        divan::black_box(s);
    }
}

#[divan::bench]
fn charsxp_translate_to_string_utf8() {
    unsafe {
        let s = charsxp_to_string_translate_utf8(fixtures().utf8_charsxp());
        divan::black_box(s);
    }
}

#[divan::bench]
fn charsxp_translate_to_string_latin1() {
    unsafe {
        let s = charsxp_to_string_translate_utf8(fixtures().latin1_charsxp());
        divan::black_box(s);
    }
}

#[divan::bench]
fn sexp_tryfrom_string_translate_utf8() {
    unsafe {
        let s = sexp_to_string_translate_utf8(fixtures().utf8_strsxp());
        divan::black_box(s);
    }
}

#[divan::bench]
fn sexp_tryfrom_string_translate_latin1() {
    unsafe {
        let s = sexp_to_string_translate_utf8(fixtures().latin1_strsxp());
        divan::black_box(s);
    }
}
// endregion
