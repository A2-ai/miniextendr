//! Integration with the `indexmap` crate.
//!
//! Provides conversions between R named lists and `IndexMap<String, T>` types,
//! preserving insertion order in both directions.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | named `list()` | `IndexMap<String, T>` | Order preserved |
//!
//! **Unnamed elements:** When converting from R, unnamed list elements receive
//! auto-generated names ("V1", "V2", ...) similar to R's behavior.
//!
//! # Features
//!
//! Enable this module with the `indexmap` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["indexmap"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use indexmap::IndexMap;
//!
//! #[miniextendr]
//! fn get_config() -> IndexMap<String, i32> {
//!     let mut map = IndexMap::new();
//!     map.insert("width".to_string(), 800);
//!     map.insert("height".to_string(), 600);
//!     map.insert("depth".to_string(), 24);
//!     map
//! }
//!
//! #[miniextendr]
//! fn process_options(opts: IndexMap<String, String>) -> String {
//!     opts.into_iter()
//!         .map(|(k, v)| format!("{}={}", k, v))
//!         .collect::<Vec<_>>()
//!         .join(", ")
//! }
//! ```
//!
//! # Why IndexMap?
//!
//! Use `IndexMap` instead of `HashMap` when:
//! - Order matters (R lists preserve insertion order)
//! - You need deterministic iteration order for reproducibility
//! - You're round-tripping data through R and want stable ordering

pub use indexmap::IndexMap;

use crate::ffi::{
    CE_UTF8, R_CHAR, R_NaString, R_NamesSymbol, R_NilValue, R_xlen_t, Rf_allocVector, Rf_getAttrib,
    Rf_mkCharLenCE, Rf_protect, Rf_setAttrib, Rf_unprotect, SET_STRING_ELT, SET_VECTOR_ELT, SEXP,
    SEXPTYPE, STRING_ELT, SexpExt, VECTOR_ELT,
};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// TryFromSexp for IndexMap<String, T>
// =============================================================================

impl<T> TryFromSexp for IndexMap<String, T>
where
    T: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::VECSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::VECSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut map = IndexMap::with_capacity(len);

        // Get names attribute (may be NULL if no names)
        let names_sexp = unsafe { Rf_getAttrib(sexp, R_NamesSymbol) };
        let has_names = names_sexp != unsafe { R_NilValue };

        for i in 0..len {
            // Get name for this element
            let name = if has_names {
                let name_charsxp = unsafe { STRING_ELT(names_sexp, i as R_xlen_t) };
                if name_charsxp == unsafe { R_NaString } || name_charsxp == unsafe { R_NilValue } {
                    // NA or missing name -> generate auto name
                    format!("V{}", i + 1)
                } else {
                    let c_str = unsafe { R_CHAR(name_charsxp) };
                    if c_str.is_null() {
                        format!("V{}", i + 1)
                    } else {
                        let name_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
                        match name_str.to_str() {
                            Ok(s) if s.is_empty() => format!("V{}", i + 1),
                            Ok(s) => s.to_owned(),
                            Err(_) => format!("V{}", i + 1),
                        }
                    }
                }
            } else {
                // No names attribute -> generate auto name
                format!("V{}", i + 1)
            };

            // Get and convert element
            let elem_sexp = unsafe { VECTOR_ELT(sexp, i as R_xlen_t) };
            let value = T::try_from_sexp(elem_sexp).map_err(|e| {
                SexpError::InvalidValue(format!("failed to convert element '{}': {}", name, e))
            })?;

            map.insert(name, value);
        }

        Ok(map)
    }
}

// =============================================================================
// IntoR for IndexMap<String, T>
// =============================================================================

impl<T> IntoR for IndexMap<String, T>
where
    T: IntoR,
{
    fn into_sexp(self) -> SEXP {
        unsafe {
            let n = self.len();
            let list = Rf_allocVector(SEXPTYPE::VECSXP, n as R_xlen_t);
            Rf_protect(list);

            // Allocate names vector
            let names = Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
            Rf_protect(names);

            for (i, (key, value)) in self.into_iter().enumerate() {
                // Set list element
                SET_VECTOR_ELT(list, i as R_xlen_t, value.into_sexp());

                // Set name
                let charsxp = Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, CE_UTF8);
                SET_STRING_ELT(names, i as R_xlen_t, charsxp);
            }

            // Attach names attribute
            Rf_setAttrib(list, R_NamesSymbol, names);

            Rf_unprotect(2);
            list
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexmap_preserves_order() {
        let mut map = IndexMap::new();
        map.insert("z".to_string(), 1);
        map.insert("a".to_string(), 2);
        map.insert("m".to_string(), 3);

        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn indexmap_capacity() {
        let map: IndexMap<String, i32> = IndexMap::with_capacity(10);
        assert!(map.capacity() >= 10);
    }
}
