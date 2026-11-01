//! Test fixtures for the encoding snapshot (`miniextendr-api/src/encoding.rs`).
//!
//! String conversions in any locale are exercised through the conversion
//! fixtures (`conv_string_arg`, `zero_copy_cow_str_is_borrowed`, ...) by
//! `tests/testthat/test-encoding.R`.

use miniextendr_api::encoding;
use miniextendr_api::prelude::*;

/// Whether `miniextendr_encoding_init()` populated the cached snapshot.
///
/// On standard R-package builds this returns `FALSE` (encoding init is
/// disabled because the underlying symbols aren't exported from libR).
/// Embedded R builds with the `nonapi` feature can flip this to `TRUE`.
#[miniextendr]
pub fn encoding_info_available() -> bool {
    encoding::encoding_info().is_some()
}

// region: per-string encodings in argument conversions and string reads

use std::borrow::Cow;

use miniextendr_api::sys::{Rf_allocVector, Rf_mkCharLenCE};
use miniextendr_api::{SEXPTYPE, cetype_t};

/// Echo a `String` argument. A string marked as bytes is refused with an
/// argument error; a Latin-1 one is translated.
/// @param x A string.
#[miniextendr]
pub fn encoding_echo_string(x: String) -> String {
    x
}

/// Echo a `&str` argument, as [`encoding_echo_string`] does.
/// @param x A string.
#[miniextendr]
pub fn encoding_echo_str(x: &str) -> String {
    x.to_owned()
}

/// Echo an `Option<String>` argument, as [`encoding_echo_string`] does.
/// @param x A string, `NA` or `NULL`.
#[miniextendr]
pub fn encoding_echo_option_string(x: Option<String>) -> Option<String> {
    x
}

/// Echo a `Cow<str>` argument, as [`encoding_echo_string`] does.
/// @param x A string.
#[miniextendr]
pub fn encoding_echo_cow(x: Cow<'_, str>) -> String {
    x.into_owned()
}

/// Echo a `Vec<String>` argument: every element marked as bytes is refused,
/// all of them in one argument error.
/// @param x A character vector.
#[miniextendr]
pub fn encoding_echo_vec_string(x: Vec<String>) -> Vec<String> {
    x
}

/// Echo a `Vec<Option<String>>` argument, as [`encoding_echo_vec_string`]
/// does; `NA` stays `NA`.
/// @param x A character vector.
#[miniextendr]
pub fn encoding_echo_vec_option_string(x: Vec<Option<String>>) -> Vec<Option<String>> {
    x
}

/// Echo a `Vec<&str>` argument (the rooted borrow path), as
/// [`encoding_echo_vec_string`] does.
/// @param x A character vector.
#[miniextendr]
pub fn encoding_echo_vec_str(x: Vec<&str>) -> Vec<String> {
    x.into_iter().map(str::to_owned).collect()
}

/// `DataFrame::names()`, read with each name's encoding.
/// @param df A data.frame.
#[miniextendr]
pub fn encoding_df_names(df: DataFrame) -> Vec<String> {
    df.names()
}

/// Whether `df` has a column named `name`: a Latin-1 `name` argument is
/// translated, and so are the frame's Latin-1 names.
/// @param df A data.frame.
/// @param name A column name.
#[miniextendr]
pub fn encoding_df_has_column(df: DataFrame, name: String) -> bool {
    df.contains_column(&name)
}

/// Every element of a character vector through `SexpExt::string_elt_str`.
/// @param x A character vector.
#[miniextendr]
pub fn encoding_string_elt_str(x: SEXP) -> Vec<Option<String>> {
    (0..x.xlength())
        .map(|i| x.string_elt_str(i).map(str::to_owned))
        .collect()
}

/// A Latin-1 CHARSXP holding `text`, which must be Latin-1 representable.
fn latin1_charsxp(text: &str) -> SEXP {
    let bytes: Vec<u8> = text
        .chars()
        .map(|c| u8::try_from(u32::from(c)).expect("a Latin-1 character"))
        .collect();
    let len = i32::try_from(bytes.len()).expect("a short string");
    // SAFETY: R's main thread; R copies the bytes into the CHARSXP.
    unsafe { Rf_mkCharLenCE(bytes.as_ptr().cast(), len, cetype_t::CE_LATIN1) }
}

/// Latin-1 strings read through every encoding-aware path while R allocates
/// in between (run under `gctorture`): `string_elt_str` (its translations
/// live on R's memory stack, which the GC keeps), a `Vec<String>`
/// conversion, and `DataFrame::names()` / `contains_column` on a frame whose
/// names are Latin-1. Returns the frame's names.
#[miniextendr]
pub fn gc_stress_latin1_string_reads() -> Vec<String> {
    let expected: Vec<String> = (0..6).map(|i| format!("DéV{i}")).collect();
    // SAFETY: R's main thread; every allocation is rooted in `scope` before
    // the next one.
    unsafe {
        let scope = ProtectScope::new();
        let n = isize::try_from(expected.len()).expect("a short vector");
        let strings = scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, n));
        for (i, text) in (0..n).zip(&expected) {
            strings.set_string_elt(i, latin1_charsxp(text));
        }

        // Hold every translated `&str` across allocations.
        let reads: Vec<&str> = (0..n)
            .map(|i| strings.string_elt_str(i).expect("a non-NA element"))
            .collect();
        for _ in 0..4 {
            scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, 16));
        }
        assert_eq!(reads, expected, "string_elt_str");

        let owned: Vec<String> = TryFromSexp::try_from_sexp(strings).expect("Latin-1 is text");
        assert_eq!(owned, expected, "Vec<String>");

        let frame = scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n));
        for i in 0..n {
            frame.set_vector_elt(i, SEXP::scalar_integer(i32::try_from(i).expect("small")));
        }
        frame.set_names(strings);
        frame.set_class(scope.protect_raw(SEXP::scalar_string_from_str("data.frame")));
        let row_names = scope.protect_raw(Rf_allocVector(SEXPTYPE::INTSXP, 2));
        row_names.set_integer_elt(0, i32::MIN);
        row_names.set_integer_elt(1, -i32::try_from(n).expect("small"));
        frame.set_row_names(row_names);

        let df = DataFrame::from_sexp(frame).expect("a data.frame");
        let names = df.names();
        assert_eq!(names, expected, "DataFrame::names");
        assert!(df.contains_column("DéV3"), "contains_column");
        names
    }
}

// endregion
