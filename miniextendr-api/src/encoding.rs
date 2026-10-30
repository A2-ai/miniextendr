//! Encoding / locale probing utilities.
//!
//! This module exists mainly for debugging + experiments around R's string
//! encodings. R's runtime has both:
//! - per-CHARSXP encoding tags (UTF-8 / Latin-1 / bytes / native)
//! - global/locale-level settings (native encoding, UTF-8 locale flags)
//!
//! # Availability
//!
//! The global signals are **non-API** (from `Defn.h`) and require the `nonapi`
//! feature. Only the locale flags R's shared library exports are read
//! (`utf8locale`, `mbcslocale`, `known_to_be_latin1`); the hidden ones
//! (`known_to_be_utf8`, `latin1locale`, `R_nativeEncoding`) must not even be
//! *referenced* — an eager data relocation against a hidden symbol aborts
//! `dyn.load` of the whole package (see `sys::nonapi_encoding`).
//!
//! `miniextendr_encoding_init()` is not called by `package_init`; it exists
//! for debugging and for standalone Rust applications embedding R.
//!
//! A package loads in any locale. String conversions read each CHARSXP's
//! encoding mark and translate when needed; in a session whose locale isn't
//! UTF-8, the first translation of a native non-ASCII string raises one
//! warning ([`NATIVE_TRANSLATION_CLASS`]).

use std::sync::OnceLock;

/// Cached snapshot of R's encoding / locale state at init time.
#[derive(Debug, Clone)]
pub struct REncodingInfo {
    #[cfg(feature = "nonapi")]
    /// Whether R thinks the current locale is UTF-8 (non-API `utf8locale`).
    pub utf8_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether the current locale is multi-byte (non-API `mbcslocale`).
    pub mbcs_locale: Option<bool>,
    #[cfg(feature = "nonapi")]
    /// Whether R treats unknown-encoding strings as Latin-1 (non-API
    /// `known_to_be_latin1`).
    pub known_to_be_latin1: Option<bool>,
}

static ENCODING_INFO: OnceLock<REncodingInfo> = OnceLock::new();

/// Return the cached encoding info (if `miniextendr_encoding_init()` has run).
#[inline]
pub fn encoding_info() -> Option<&'static REncodingInfo> {
    ENCODING_INFO.get()
}

/// The class of the warning a package raises the first time it translates a
/// native string to UTF-8.
pub const NATIVE_TRANSLATION_CLASS: &str = "miniextendr_native_translation";

/// Queue a warning, once per session for each package, the first time a
/// native (unmarked) non-ASCII string is translated to UTF-8: the session's
/// locale isn't UTF-8, and bytes its encoding can't represent came out as
/// `<xx>` escapes.
///
/// Deferred, so it is signalled when the current call returns, never from
/// inside a conversion; class [`NATIVE_TRANSLATION_CLASS`].
pub(crate) fn warn_native_translation() {
    static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    crate::deferred_condition::defer_warning(
        crate::condition::RError::new(
            "translated a string from the session's native encoding to UTF-8: the R session's \
             locale isn't UTF-8, so characters that encoding can't represent became <xx> \
             escapes. Start R in a UTF-8 locale (for example LANG=C.UTF-8) to avoid \
             translation. Shown once per session.",
        )
        .class(NATIVE_TRANSLATION_CLASS)
        .without_call(),
    );
}

/// Initialize / snapshot R's encoding state.
///
/// Intended to be called once from `R_init_*` (package init).
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_encoding_init() {
    let _ = ENCODING_INFO.get_or_init(|| {
        #[cfg(feature = "nonapi")]
        unsafe {
            use crate::Rboolean;
            use crate::sys::nonapi_encoding;

            let utf8_locale = Some(nonapi_encoding::utf8locale != Rboolean::FALSE);
            let mbcs_locale = Some(nonapi_encoding::mbcslocale != Rboolean::FALSE);
            let known_to_be_latin1 = Some(nonapi_encoding::known_to_be_latin1 != Rboolean::FALSE);

            let info = REncodingInfo {
                utf8_locale,
                mbcs_locale,
                known_to_be_latin1,
            };

            if std::env::var_os("MINIEXTENDR_ENCODING_DEBUG").is_some() {
                let msg = format!("[miniextendr] encoding init: {info:?}\n");
                if let Ok(c_msg) = std::ffi::CString::new(msg) {
                    crate::sys::REprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
                }
            }

            info
        }

        #[cfg(not(feature = "nonapi"))]
        {
            REncodingInfo {}
        }
    });
}
