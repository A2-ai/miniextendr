//! String extraction benchmarks: R_CHAR vs translateCharUTF8.
//!
//! Implemented groups:
//! - `charsxp_direct`: R_CHAR → CStr → String (UTF-8 only, no translation)
//! - `charsxp_translate`: Rf_translateCharUTF8 → CStr → String (handles encodings)
//! - `strsxp_direct`: STRSXP → Vec<String> via R_CHAR path
//! - `strsxp_translate`: STRSXP → Vec<String> via translateCharUTF8 path
//!
//! Compares the two strategies for extracting Rust strings from R CHARSXP
//! and STRSXP values. The translate path handles Latin-1 and native encodings
//! but has overhead; the direct path assumes UTF-8 only.
