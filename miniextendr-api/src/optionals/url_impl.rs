//! Integration with the `url` crate.
//!
//! Provides conversions between R character vectors and `Url` types with strict validation.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `character(1)` | `Url` | Parsed and validated URL |
//! | `character` | `Vec<Url>` | Vector of URLs |
//! | `NA_character_` | `Option<Url>` | NA maps to None |
//!
//! # Features
//!
//! Enable this module with the `url` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["url"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use url::Url;
//!
//! #[miniextendr]
//! fn parse_url(s: &str) -> Result<Url, String> {
//!     Url::parse(s).map_err(|e| e.to_string())
//! }
//!
//! #[miniextendr]
//! fn get_host(url: Url) -> Option<String> {
//!     url.host_str().map(|s| s.to_string())
//! }
//! ```
//!
//! # Validation
//!
//! URLs are strictly validated on input. Invalid URLs will return an error.
//! The string representation uses the URL's canonical form via `Url::as_str()`.

pub use url::Url;

use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{
    SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp, charsxp_to_str,
};
use crate::into_r::IntoR;

// =============================================================================
// Scalar conversions
// =============================================================================

impl TryFromSexp for Url {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let charsxp = unsafe { STRING_ELT(sexp, 0) };
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::STRSXP,
            }));
        }

        let s = unsafe { charsxp_to_str(charsxp) };
        Url::parse(s).map_err(|e| SexpError::InvalidValue(format!("invalid URL: {}", e)))
    }
}

impl IntoR for Url {
    fn into_sexp(self) -> SEXP {
        self.as_str().into_sexp()
    }
}

// =============================================================================
// Option conversions (NA support)
// =============================================================================

impl TryFromSexp for Option<Url> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        // NULL -> None
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let charsxp = unsafe { STRING_ELT(sexp, 0) };
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok(None);
        }

        let s = unsafe { charsxp_to_str(charsxp) };
        match Url::parse(s) {
            Ok(url) => Ok(Some(url)),
            Err(e) => Err(SexpError::InvalidValue(format!("invalid URL: {}", e))),
        }
    }
}

impl IntoR for Option<Url> {
    fn into_sexp(self) -> SEXP {
        // Leverage Option<String>'s IntoR which handles NA correctly
        self.map(|u| u.as_str().to_string()).into_sexp()
    }
}

// =============================================================================
// Vector conversions
// =============================================================================

impl TryFromSexp for Vec<Url> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Url>",
                    i
                )));
            }
            let s = unsafe { charsxp_to_str(charsxp) };
            match Url::parse(s) {
                Ok(url) => result.push(url),
                Err(e) => {
                    return Err(SexpError::InvalidValue(format!(
                        "invalid URL at index {}: {}",
                        i, e
                    )));
                }
            }
        }

        Ok(result)
    }
}

impl IntoR for Vec<Url> {
    fn into_sexp(self) -> SEXP {
        // Convert to Vec<String> and use its IntoR
        let strings: Vec<String> = self.into_iter().map(|u| u.as_str().to_string()).collect();
        strings.into_sexp()
    }
}

// =============================================================================
// Vec<Option<Url>> conversions (NA-aware vectors)
// =============================================================================

impl TryFromSexp for Vec<Option<Url>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
                result.push(None);
            } else {
                let s = unsafe { charsxp_to_str(charsxp) };
                match Url::parse(s) {
                    Ok(url) => result.push(Some(url)),
                    Err(e) => {
                        return Err(SexpError::InvalidValue(format!(
                            "invalid URL at index {}: {}",
                            i, e
                        )));
                    }
                }
            }
        }

        Ok(result)
    }
}

impl IntoR for Vec<Option<Url>> {
    fn into_sexp(self) -> SEXP {
        // Convert to Vec<Option<String>> and use its IntoR
        let strings: Vec<Option<String>> = self
            .into_iter()
            .map(|opt| opt.map(|u| u.as_str().to_string()))
            .collect();
        strings.into_sexp()
    }
}

// =============================================================================
// RUrlOps adapter trait
// =============================================================================

/// Adapter trait for [`Url`] operations.
///
/// Provides URL inspection and manipulation methods for R.
/// Automatically implemented for `Url`.
///
/// # Example
///
/// ```rust,ignore
/// use url::Url;
/// use miniextendr_api::url_impl::RUrlOps;
///
/// #[derive(ExternalPtr)]
/// struct MyUrl(Url);
///
/// #[miniextendr]
/// impl RUrlOps for MyUrl {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RUrlOps for MyUrl;
/// }
/// ```
///
/// In R:
/// ```r
/// u <- MyUrl$new("https://example.com:8080/path?query=1#frag")
/// u$scheme()     # "https"
/// u$host()       # "example.com"
/// u$port()       # 8080 (NA if not specified)
/// u$path()       # "/path"
/// u$query()      # "query=1" (NA if none)
/// u$fragment()   # "frag" (NA if none)
/// u$as_str()     # full URL string
/// ```
pub trait RUrlOps {
    /// Get the URL scheme (e.g., "https", "http", "ftp").
    fn scheme(&self) -> String;

    /// Get the host string, if present.
    fn host(&self) -> Option<String>;

    /// Get the port number, if explicitly specified.
    fn port(&self) -> Option<u16>;

    /// Get the port or default port for the scheme.
    fn port_or_known_default(&self) -> Option<u16>;

    /// Get the path component.
    fn path(&self) -> String;

    /// Get the query string, if present.
    fn query(&self) -> Option<String>;

    /// Get the fragment identifier, if present.
    fn fragment(&self) -> Option<String>;

    /// Get the full URL as a string.
    fn as_str(&self) -> String;

    /// Get the username, if present.
    fn username(&self) -> String;

    /// Get the password, if present.
    fn password(&self) -> Option<String>;

    /// Check if this URL cannot be a base (has opaque path).
    fn cannot_be_a_base(&self) -> bool;

    /// Get the origin as a string.
    fn origin(&self) -> String;
}

impl RUrlOps for Url {
    fn scheme(&self) -> String {
        self.scheme().to_string()
    }

    fn host(&self) -> Option<String> {
        self.host_str().map(|s| s.to_string())
    }

    fn port(&self) -> Option<u16> {
        self.port()
    }

    fn port_or_known_default(&self) -> Option<u16> {
        self.port_or_known_default()
    }

    fn path(&self) -> String {
        self.path().to_string()
    }

    fn query(&self) -> Option<String> {
        self.query().map(|s| s.to_string())
    }

    fn fragment(&self) -> Option<String> {
        self.fragment().map(|s| s.to_string())
    }

    fn as_str(&self) -> String {
        Url::as_str(self).to_string()
    }

    fn username(&self) -> String {
        self.username().to_string()
    }

    fn password(&self) -> Option<String> {
        self.password().map(|s| s.to_string())
    }

    fn cannot_be_a_base(&self) -> bool {
        self.cannot_be_a_base()
    }

    fn origin(&self) -> String {
        self.origin().ascii_serialization()
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// URL helper functions for common operations.
pub mod url_helpers {
    use super::Url;

    /// Parse a URL string, returning an error message on failure.
    pub fn parse(s: &str) -> Result<Url, String> {
        Url::parse(s).map_err(|e| e.to_string())
    }

    /// Join a base URL with a relative path.
    pub fn join(base: &Url, path: &str) -> Result<Url, String> {
        base.join(path).map_err(|e| e.to_string())
    }

    /// Check if a string is a valid URL.
    pub fn is_valid(s: &str) -> bool {
        Url::parse(s).is_ok()
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse() {
        let url = Url::parse("https://example.com/path?query=1#frag").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("query=1"));
        assert_eq!(url.fragment(), Some("frag"));
    }

    #[test]
    fn test_url_helpers_parse() {
        assert!(url_helpers::parse("https://example.com").is_ok());
        assert!(url_helpers::parse("not a url").is_err());
    }

    #[test]
    fn test_url_helpers_join() {
        let base = Url::parse("https://example.com/base/").unwrap();
        let joined = url_helpers::join(&base, "path").unwrap();
        assert_eq!(joined.as_str(), "https://example.com/base/path");
    }

    #[test]
    fn test_url_helpers_is_valid() {
        assert!(url_helpers::is_valid("https://example.com"));
        assert!(url_helpers::is_valid("http://localhost:8080"));
        assert!(!url_helpers::is_valid("not a url"));
        assert!(!url_helpers::is_valid(""));
    }

    #[test]
    fn test_rurlops_basics() {
        let url = Url::parse("https://user:pass@example.com:8080/path?q=1#frag").unwrap();

        assert_eq!(RUrlOps::scheme(&url), "https");
        assert_eq!(RUrlOps::host(&url), Some("example.com".to_string()));
        assert_eq!(RUrlOps::port(&url), Some(8080));
        assert_eq!(RUrlOps::path(&url), "/path");
        assert_eq!(RUrlOps::query(&url), Some("q=1".to_string()));
        assert_eq!(RUrlOps::fragment(&url), Some("frag".to_string()));
        assert_eq!(RUrlOps::username(&url), "user");
        assert_eq!(RUrlOps::password(&url), Some("pass".to_string()));
    }

    #[test]
    fn test_rurlops_defaults() {
        let url = Url::parse("https://example.com").unwrap();

        assert_eq!(RUrlOps::port(&url), None);
        assert_eq!(RUrlOps::port_or_known_default(&url), Some(443));
        assert_eq!(RUrlOps::query(&url), None);
        assert_eq!(RUrlOps::fragment(&url), None);
        assert_eq!(RUrlOps::password(&url), None);
    }
}
