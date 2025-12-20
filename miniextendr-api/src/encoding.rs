//! Encoding / locale probing utilities.
//!
//! This module exists mainly for debugging + experiments around R's string
//! encodings. R's runtime has both:
//! - per-CHARSXP encoding tags (UTF-8 / Latin-1 / bytes / native)
//! - global/locale-level settings (native encoding, UTF-8 locale flags)
//!
//! # Availability
//!
//! The global signals are **non-API** (from `Defn.h`) and require the `nonapi` feature.
//! Additionally, these symbols are **not exported from R's shared library**, so
//! `miniextendr_encoding_init()` only works when:
//! - Embedding R via `miniextendr-engine` (which links directly to R internals)
//! - Running on platforms where these symbols happen to be exported
//!
//! For R packages (loaded via `.Call`), these symbols are typically unavailable,
//! so `miniextendr_encoding_init()` is **disabled by default** in the entrypoint.
//! The module is still useful for standalone Rust applications embedding R.

use std::sync::OnceLock;

/// Cached snapshot of R's encoding / locale state at init time.
#[derive(Debug, Clone)]
pub struct REncodingInfo {
    #[cfg(feature = "nonapi")]
    /// R's reported native encoding (non-API).
    pub native_encoding: Option<String>,
    #[cfg(feature = "nonapi")]
    /// Whether R thinks the current locale is UTF-8 (non-API).
    pub utf8_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether R thinks the current locale is Latin-1 (non-API).
    pub latin1_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether R has determined it's "known to be UTF-8" (non-API).
    pub known_to_be_utf8: Option<bool>,
}

static ENCODING_INFO: OnceLock<REncodingInfo> = OnceLock::new();

/// Return the cached encoding info (if `miniextendr_encoding_init()` has run).
#[inline]
pub fn encoding_info() -> Option<&'static REncodingInfo> {
    ENCODING_INFO.get()
}

/// Initialize / snapshot R's encoding state.
///
/// Intended to be called once from `R_init_*` (package init).
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_encoding_init() {
    let _ = ENCODING_INFO.get_or_init(|| {
        #[cfg(feature = "nonapi")]
        unsafe {
            use crate::ffi::{Rboolean, nonapi_encoding};

            let native_encoding = {
                let ptr = nonapi_encoding::R_nativeEncoding();
                if ptr.is_null() {
                    None
                } else {
                    Some(std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
                }
            };

            let utf8_locale = Some(nonapi_encoding::utf8locale != Rboolean::FALSE);
            let latin1_locale = Some(nonapi_encoding::latin1locale != Rboolean::FALSE);
            let known_to_be_utf8 = Some(nonapi_encoding::known_to_be_utf8 != Rboolean::FALSE);

            let info = REncodingInfo {
                native_encoding,
                utf8_locale,
                latin1_locale,
                known_to_be_utf8,
            };

            if std::env::var_os("MINIEXTENDR_ENCODING_DEBUG").is_some() {
                let msg = format!("[miniextendr] encoding init: {info:?}\n");
                if let Ok(c_msg) = std::ffi::CString::new(msg) {
                    crate::ffi::REprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
                }
            }

            return info;
        }

        #[cfg(not(feature = "nonapi"))]
        {
            REncodingInfo {}
        }
    });
}
