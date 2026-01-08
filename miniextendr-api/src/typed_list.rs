//! Typed list validation for structured R list arguments.
//!
//! This module provides a specification-based validation system for R lists,
//! particularly useful for validating `...` arguments passed via [`Dots`](crate::dots::Dots).
//!
//! # Overview
//!
//! R lists are heterogeneous and loosely typed. When accepting a list from R,
//! you often need to validate that specific named entries exist with expected types.
//! This module provides:
//!
//! - [`TypedListSpec`] - Specification of expected list structure
//! - [`TypeSpec`] - Individual element type expectations
//! - [`TypedList`] - A validated list with typed accessors
//! - [`validate_list`] - Validation function that checks a list against a spec
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::typed_list::{TypedListSpec, TypedEntry, TypeSpec, validate_list};
//!
//! let spec = TypedListSpec {
//!     entries: vec![
//!         TypedEntry::required("alpha", TypeSpec::Numeric(Some(4))),
//!         TypedEntry::required("beta", TypeSpec::List(None)),
//!         TypedEntry::optional("gamma", TypeSpec::Character(None)),
//!     ],
//!     allow_extra: true,
//! };
//!
//! let validated = dots.typed(spec)?;
//! let alpha: Vec<f64> = validated.get("alpha")?;
//! ```

use crate::ffi::{self, Rboolean, SEXP, SEXPTYPE};
use crate::from_r::{SexpError, TryFromSexp};
use crate::list::{List, ListFromSexpError};
use std::collections::HashSet;
use std::ffi::CStr;

// =============================================================================
// Type specification structures
// =============================================================================

/// Specification for validating a typed list.
///
/// Describes the expected structure of an R list, including required and
/// optional named entries with their type constraints.
#[derive(Clone, Debug)]
pub struct TypedListSpec {
    /// Expected entries in the list.
    pub entries: Vec<TypedEntry>,
    /// If `false`, reject lists with named entries not in the spec.
    /// Default is `true` (allow extra fields).
    pub allow_extra: bool,
}

impl TypedListSpec {
    /// Create a new spec that allows extra fields.
    pub fn new(entries: Vec<TypedEntry>) -> Self {
        Self {
            entries,
            allow_extra: true,
        }
    }

    /// Create a strict spec that rejects extra named fields.
    pub fn strict(entries: Vec<TypedEntry>) -> Self {
        Self {
            entries,
            allow_extra: false,
        }
    }
}

/// A single entry specification in a typed list.
#[derive(Clone, Debug)]
pub struct TypedEntry {
    /// The expected name of this entry.
    pub name: &'static str,
    /// The expected type of this entry.
    pub spec: TypeSpec,
    /// If `true`, the entry is optional (missing allowed).
    pub optional: bool,
}

impl TypedEntry {
    /// Create a required entry with the given name and type.
    pub const fn required(name: &'static str, spec: TypeSpec) -> Self {
        Self {
            name,
            spec,
            optional: false,
        }
    }

    /// Create an optional entry with the given name and type.
    pub const fn optional(name: &'static str, spec: TypeSpec) -> Self {
        Self {
            name,
            spec,
            optional: true,
        }
    }

    /// Create a required entry that accepts any type.
    pub const fn any(name: &'static str) -> Self {
        Self::required(name, TypeSpec::Any)
    }

    /// Create an optional entry that accepts any type.
    pub const fn any_optional(name: &'static str) -> Self {
        Self::optional(name, TypeSpec::Any)
    }
}

/// Type specification for a single list element.
///
/// The optional `usize` parameter specifies an exact length constraint.
/// `None` means any length is accepted.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSpec {
    /// Accept any type.
    Any,
    /// Numeric (real/double) vector. `REALSXP` only.
    Numeric(Option<usize>),
    /// Integer vector. `INTSXP` only.
    Integer(Option<usize>),
    /// Logical vector.
    Logical(Option<usize>),
    /// Character vector.
    Character(Option<usize>),
    /// Raw vector.
    Raw(Option<usize>),
    /// Complex vector.
    Complex(Option<usize>),
    /// List (VECSXP or pairlist).
    List(Option<usize>),
    /// Object inheriting from a specific class.
    /// Uses `Rf_inherits` semantics (checks class attribute).
    Class(&'static str),
    /// Data frame (inherits `data.frame`).
    DataFrame,
    /// Factor (uses `Rf_isFactor`).
    Factor,
    /// Matrix (uses `Rf_isMatrix`).
    Matrix,
    /// Array (uses `Rf_isArray`).
    Array,
    /// Function (uses `Rf_isFunction`).
    Function,
    /// Environment (uses `Rf_isEnvironment`).
    Environment,
    /// NULL only.
    Null,
}

impl TypeSpec {
    /// Get a human-readable name for this type specification.
    pub fn type_name(&self) -> String {
        match self {
            TypeSpec::Any => "any".to_string(),
            TypeSpec::Numeric(None) => "numeric".to_string(),
            TypeSpec::Numeric(Some(n)) => format!("numeric({})", n),
            TypeSpec::Integer(None) => "integer".to_string(),
            TypeSpec::Integer(Some(n)) => format!("integer({})", n),
            TypeSpec::Logical(None) => "logical".to_string(),
            TypeSpec::Logical(Some(n)) => format!("logical({})", n),
            TypeSpec::Character(None) => "character".to_string(),
            TypeSpec::Character(Some(n)) => format!("character({})", n),
            TypeSpec::Raw(None) => "raw".to_string(),
            TypeSpec::Raw(Some(n)) => format!("raw({})", n),
            TypeSpec::Complex(None) => "complex".to_string(),
            TypeSpec::Complex(Some(n)) => format!("complex({})", n),
            TypeSpec::List(None) => "list".to_string(),
            TypeSpec::List(Some(n)) => format!("list({})", n),
            TypeSpec::Class(c) => format!("class: {}", c),
            TypeSpec::DataFrame => "data.frame".to_string(),
            TypeSpec::Factor => "factor".to_string(),
            TypeSpec::Matrix => "matrix".to_string(),
            TypeSpec::Array => "array".to_string(),
            TypeSpec::Function => "function".to_string(),
            TypeSpec::Environment => "environment".to_string(),
            TypeSpec::Null => "NULL".to_string(),
        }
    }
}

// =============================================================================
// Error types
// =============================================================================

/// Error returned when list validation fails.
#[derive(Debug, Clone)]
pub enum TypedListError {
    /// The input was not a list.
    NotList(ListFromSexpError),
    /// A required field is missing.
    Missing { name: String },
    /// A field has the wrong type.
    WrongType {
        name: String,
        expected: String,
        actual: String,
    },
    /// A field has the wrong length.
    WrongLen {
        name: String,
        expected: usize,
        actual: isize,
    },
    /// Extra named fields found when `allow_extra = false`.
    ExtraFields { names: Vec<String> },
    /// Duplicate non-empty names in the list.
    DuplicateNames { name: String },
}

impl std::fmt::Display for TypedListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedListError::NotList(e) => write!(f, "expected a list: {}", e),
            TypedListError::Missing { name } => write!(f, "missing required field: {:?}", name),
            TypedListError::WrongType {
                name,
                expected,
                actual,
            } => write!(
                f,
                "field {:?} has wrong type: expected {}, got {}",
                name, expected, actual
            ),
            TypedListError::WrongLen {
                name,
                expected,
                actual,
            } => write!(
                f,
                "field {:?} has wrong length: expected {}, got {}",
                name, expected, actual
            ),
            TypedListError::ExtraFields { names } => {
                write!(f, "unexpected extra fields: {:?}", names)
            }
            TypedListError::DuplicateNames { name } => {
                write!(f, "duplicate field name: {:?}", name)
            }
        }
    }
}

impl std::error::Error for TypedListError {}

impl From<ListFromSexpError> for TypedListError {
    fn from(e: ListFromSexpError) -> Self {
        TypedListError::NotList(e)
    }
}

// =============================================================================
// TypedList wrapper
// =============================================================================

/// A validated list that matches a [`TypedListSpec`].
///
/// Provides typed accessors for list elements with good error messages.
#[derive(Clone, Debug)]
pub struct TypedList {
    inner: List,
    spec: TypedListSpec,
}

impl TypedList {
    /// Get the underlying [`List`].
    #[inline]
    pub fn as_list(&self) -> List {
        self.inner
    }

    /// Get the specification this list was validated against.
    #[inline]
    pub fn spec(&self) -> &TypedListSpec {
        &self.spec
    }

    /// Get an element by name and convert to type `T`.
    ///
    /// Returns [`TypedListError::Missing`] if the field doesn't exist.
    /// Returns [`TypedListError::WrongType`] if conversion fails.
    pub fn get<T>(&self, name: &str) -> Result<T, TypedListError>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let sexp = self.get_raw(name)?;
        T::try_from_sexp(sexp).map_err(|e| TypedListError::WrongType {
            name: name.to_string(),
            expected: self.expected_type_for(name),
            actual: format!("{}", e),
        })
    }

    /// Get an optional element by name and convert to type `T`.
    ///
    /// Returns `Ok(None)` if the field doesn't exist.
    /// Returns [`TypedListError::WrongType`] if the field exists but conversion fails.
    pub fn get_opt<T>(&self, name: &str) -> Result<Option<T>, TypedListError>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        match self.get_raw(name) {
            Ok(sexp) => {
                let value = T::try_from_sexp(sexp).map_err(|e| TypedListError::WrongType {
                    name: name.to_string(),
                    expected: self.expected_type_for(name),
                    actual: format!("{}", e),
                })?;
                Ok(Some(value))
            }
            Err(TypedListError::Missing { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get the raw SEXP for a named element.
    pub fn get_raw(&self, name: &str) -> Result<SEXP, TypedListError> {
        let names_sexp = self.inner.names();
        let n = self.inner.len();

        if let Some(names) = names_sexp {
            for i in 0..n {
                let name_sexp = unsafe { ffi::STRING_ELT(names, i) };
                if name_sexp == unsafe { ffi::R_NaString } {
                    continue;
                }
                let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
                let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
                if let Ok(s) = name_cstr.to_str() {
                    if s == name {
                        return Ok(unsafe { ffi::VECTOR_ELT(self.inner.as_sexp(), i) });
                    }
                }
            }
        }

        Err(TypedListError::Missing {
            name: name.to_string(),
        })
    }

    /// Get the expected type string for a field from the spec.
    fn expected_type_for(&self, name: &str) -> String {
        for entry in &self.spec.entries {
            if entry.name == name {
                return entry.spec.type_name();
            }
        }
        "any".to_string()
    }
}

// =============================================================================
// Validation
// =============================================================================

/// Validate a list against a specification.
///
/// # Errors
///
/// - [`TypedListError::Missing`] if a required field is absent
/// - [`TypedListError::WrongType`] if a field has the wrong SEXP type
/// - [`TypedListError::WrongLen`] if a field has the wrong length
/// - [`TypedListError::ExtraFields`] if `allow_extra = false` and extra named fields exist
/// - [`TypedListError::DuplicateNames`] if the list has duplicate non-empty names
pub fn validate_list(list: List, spec: &TypedListSpec) -> Result<TypedList, TypedListError> {
    // Build name index and check for duplicates
    let names_sexp = list.names();
    let n = list.len();
    let mut name_to_idx: std::collections::HashMap<String, isize> =
        std::collections::HashMap::new();

    if let Some(names) = names_sexp {
        for i in 0..n {
            let name_sexp = unsafe { ffi::STRING_ELT(names, i) };
            if name_sexp == unsafe { ffi::R_NaString } {
                continue;
            }
            let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
            let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
            if let Ok(s) = name_cstr.to_str() {
                if s.is_empty() {
                    continue;
                }
                if name_to_idx.contains_key(s) {
                    return Err(TypedListError::DuplicateNames {
                        name: s.to_string(),
                    });
                }
                name_to_idx.insert(s.to_string(), i);
            }
        }
    }

    // Validate each spec entry
    let mut validated_names: HashSet<&str> = HashSet::new();

    for entry in &spec.entries {
        validated_names.insert(entry.name);

        match name_to_idx.get(entry.name) {
            None => {
                if !entry.optional {
                    return Err(TypedListError::Missing {
                        name: entry.name.to_string(),
                    });
                }
            }
            Some(&idx) => {
                let elem = unsafe { ffi::VECTOR_ELT(list.as_sexp(), idx) };
                validate_element(elem, entry)?;
            }
        }
    }

    // Check for extra fields if strict mode
    if !spec.allow_extra {
        let extra: Vec<String> = name_to_idx
            .keys()
            .filter(|k| !validated_names.contains(k.as_str()))
            .cloned()
            .collect();

        if !extra.is_empty() {
            return Err(TypedListError::ExtraFields { names: extra });
        }
    }

    Ok(TypedList {
        inner: list,
        spec: spec.clone(),
    })
}

/// Validate a single element against its type spec.
fn validate_element(elem: SEXP, entry: &TypedEntry) -> Result<(), TypedListError> {
    let actual_type = unsafe { ffi::TYPEOF(elem) };
    let actual_len = unsafe { ffi::Rf_xlength(elem) };

    match &entry.spec {
        TypeSpec::Any => Ok(()),

        TypeSpec::Numeric(len) => {
            if actual_type != SEXPTYPE::REALSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Integer(len) => {
            if actual_type != SEXPTYPE::INTSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Logical(len) => {
            if actual_type != SEXPTYPE::LGLSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Character(len) => {
            if actual_type != SEXPTYPE::STRSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Raw(len) => {
            if actual_type != SEXPTYPE::RAWSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Complex(len) => {
            if actual_type != SEXPTYPE::CPLXSXP {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::List(len) => {
            let is_list = unsafe { ffi::Rf_isList(elem) } != Rboolean::FALSE
                || actual_type == SEXPTYPE::VECSXP;
            if !is_list {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            check_length(entry.name, *len, actual_len)
        }

        TypeSpec::Class(class_name) => {
            let c_str = std::ffi::CString::new(*class_name).unwrap();
            let inherits = unsafe { ffi::Rf_inherits(elem, c_str.as_ptr()) } != Rboolean::FALSE;
            if !inherits {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::DataFrame => {
            let inherits =
                unsafe { ffi::Rf_inherits(elem, c"data.frame".as_ptr()) } != Rboolean::FALSE;
            if !inherits {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Factor => {
            if unsafe { ffi::Rf_isFactor(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Matrix => {
            if unsafe { ffi::Rf_isMatrix(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Array => {
            if unsafe { ffi::Rf_isArray(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Function => {
            if unsafe { ffi::Rf_isFunction(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Environment => {
            if unsafe { ffi::Rf_isEnvironment(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: actual_type_string(elem),
                });
            }
            Ok(())
        }

        TypeSpec::Null => {
            if unsafe { ffi::Rf_isNull(elem) } == Rboolean::FALSE {
                return Err(TypedListError::WrongType {
                    name: entry.name.to_string(),
                    expected: entry.spec.type_name(),
                    actual: sexptype_name(actual_type),
                });
            }
            Ok(())
        }
    }
}

/// Check length constraint.
fn check_length(name: &str, expected: Option<usize>, actual: isize) -> Result<(), TypedListError> {
    if let Some(exp) = expected {
        if actual != exp as isize {
            return Err(TypedListError::WrongLen {
                name: name.to_string(),
                expected: exp,
                actual,
            });
        }
    }
    Ok(())
}

// =============================================================================
// Helper functions for type detection and diagnostics
// =============================================================================

/// Get a human-readable name for a SEXPTYPE.
pub fn sexptype_name(stype: SEXPTYPE) -> String {
    match stype {
        SEXPTYPE::NILSXP => "NULL".to_string(),
        SEXPTYPE::SYMSXP => "symbol".to_string(),
        SEXPTYPE::LISTSXP => "pairlist".to_string(),
        SEXPTYPE::CLOSXP => "closure".to_string(),
        SEXPTYPE::ENVSXP => "environment".to_string(),
        SEXPTYPE::PROMSXP => "promise".to_string(),
        SEXPTYPE::LANGSXP => "language".to_string(),
        SEXPTYPE::SPECIALSXP => "special".to_string(),
        SEXPTYPE::BUILTINSXP => "builtin".to_string(),
        SEXPTYPE::CHARSXP => "char".to_string(),
        SEXPTYPE::LGLSXP => "logical".to_string(),
        SEXPTYPE::INTSXP => "integer".to_string(),
        SEXPTYPE::REALSXP => "numeric".to_string(),
        SEXPTYPE::CPLXSXP => "complex".to_string(),
        SEXPTYPE::STRSXP => "character".to_string(),
        SEXPTYPE::DOTSXP => "...".to_string(),
        SEXPTYPE::ANYSXP => "any".to_string(),
        SEXPTYPE::VECSXP => "list".to_string(),
        SEXPTYPE::EXPRSXP => "expression".to_string(),
        SEXPTYPE::BCODESXP => "bytecode".to_string(),
        SEXPTYPE::EXTPTRSXP => "externalptr".to_string(),
        SEXPTYPE::WEAKREFSXP => "weakref".to_string(),
        SEXPTYPE::RAWSXP => "raw".to_string(),
        SEXPTYPE::S4SXP => "S4".to_string(),
        SEXPTYPE::NEWSXP => "new".to_string(),
        SEXPTYPE::FREESXP => "free".to_string(),
        _ => format!("SEXPTYPE({})", stype as i32),
    }
}

/// Get a human-readable string for the actual type of a SEXP.
///
/// Includes class attribute if present.
pub fn actual_type_string(sexp: SEXP) -> String {
    let stype = unsafe { ffi::TYPEOF(sexp) };
    let base_type = sexptype_name(stype);

    // Check if it has a class attribute
    let class_attr = unsafe { ffi::Rf_getAttrib(sexp, ffi::R_ClassSymbol) };
    if class_attr != unsafe { ffi::R_NilValue } {
        let class_len = unsafe { ffi::Rf_xlength(class_attr) };
        if class_len > 0 {
            let first_class = unsafe { ffi::STRING_ELT(class_attr, 0) };
            if first_class != unsafe { ffi::R_NaString } {
                let class_ptr = unsafe { ffi::R_CHAR(first_class) };
                let class_cstr = unsafe { CStr::from_ptr(class_ptr) };
                if let Ok(s) = class_cstr.to_str() {
                    return format!("{} (class: {})", base_type, s);
                }
            }
        }
    }

    base_type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_spec_names() {
        assert_eq!(TypeSpec::Any.type_name(), "any");
        assert_eq!(TypeSpec::Numeric(None).type_name(), "numeric");
        assert_eq!(TypeSpec::Numeric(Some(4)).type_name(), "numeric(4)");
        assert_eq!(
            TypeSpec::Class("data.frame").type_name(),
            "class: data.frame"
        );
    }

    #[test]
    fn test_typed_entry_constructors() {
        let req = TypedEntry::required("foo", TypeSpec::Integer(None));
        assert_eq!(req.name, "foo");
        assert!(!req.optional);

        let opt = TypedEntry::optional("bar", TypeSpec::Character(None));
        assert_eq!(opt.name, "bar");
        assert!(opt.optional);

        let any = TypedEntry::any("baz");
        assert_eq!(any.spec, TypeSpec::Any);
        assert!(!any.optional);
    }

    #[test]
    fn test_sexptype_name() {
        assert_eq!(sexptype_name(SEXPTYPE::REALSXP), "numeric");
        assert_eq!(sexptype_name(SEXPTYPE::INTSXP), "integer");
        assert_eq!(sexptype_name(SEXPTYPE::STRSXP), "character");
        assert_eq!(sexptype_name(SEXPTYPE::VECSXP), "list");
    }
}
