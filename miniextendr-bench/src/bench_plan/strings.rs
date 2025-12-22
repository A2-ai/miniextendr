//! String and encoding benchmarks.
//!
//! Expand on the existing translate benchmark with a full matrix:
//!
//! 1) `charsxp_access`
//!    - R_CHAR (UTF-8/ASCII)
//!    - Rf_translateCharUTF8 (UTF-8, Latin-1, bytes)
//!
//! 2) `strsxp_to_string`
//!    - TryFromSexp<String> from STRSXP
//!    - Vec<String> from STRSXP
//!
//! 3) `option_strings`
//!    - Option<String> from NA vs empty
//!    - Vec<Option<String>> across NA densities
//!
//! 4) `roundtrip`
//!    - Rust String -> SEXP -> String
//!    - ASCII vs UTF-8 vs Latin-1 payloads
//!
//! Measure:
//! - ns/op for scalar extraction
//! - throughput for vector conversions
//! - impact of encoding on translation path
