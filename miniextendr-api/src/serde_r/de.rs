//! Deserializer for converting R SEXP to Rust values.

use super::error::RSerdeError;
use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::ffi::{
    R_NaString, R_NamesSymbol, R_NilValue, Rf_getAttrib, Rf_xlength, SEXP, SEXPTYPE, STRING_ELT,
    TYPEOF, VECTOR_ELT,
};
use crate::from_r::charsxp_to_str;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};

/// Deserializer that converts R SEXP to Rust values.
///
/// # Type Mappings
///
/// | R Type | Rust Type |
/// |--------|-----------|
/// | `logical(1)` | `bool` |
/// | `integer(1)` | `i32` |
/// | `numeric(1)` | `f64` |
/// | `character(1)` | `String` |
/// | NA values | `Option<T>::None` |
/// | atomic vectors | `Vec<primitive>` |
/// | lists | `Vec<T>` or struct |
/// | named lists | struct or `HashMap<String, T>` |
/// | NULL | `()` or `Option<T>::None` |
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::serde_r::RDeserializer;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Point { x: f64, y: f64 }
///
/// // Given list(x = 1.0, y = 2.0) from R:
/// let point: Point = RDeserializer::from_sexp(sexp).unwrap();
/// ```
pub struct RDeserializer {
    sexp: SEXP,
}

impl RDeserializer {
    /// Create a new deserializer from an R SEXP.
    pub fn from_sexp(sexp: SEXP) -> Self {
        RDeserializer { sexp }
    }

    /// Deserialize an R SEXP to a Rust value.
    pub fn from_sexp_to<T: for<'de> Deserialize<'de>>(sexp: SEXP) -> Result<T, RSerdeError> {
        T::deserialize(RDeserializer::from_sexp(sexp))
    }

    fn sexp_type(&self) -> SEXPTYPE {
        unsafe { TYPEOF(self.sexp) as SEXPTYPE }
    }

    fn len(&self) -> usize {
        unsafe { Rf_xlength(self.sexp) as usize }
    }

    fn is_null(&self) -> bool {
        self.sexp == unsafe { R_NilValue }
    }

    fn has_names(&self) -> bool {
        let names = unsafe { Rf_getAttrib(self.sexp, R_NamesSymbol) };
        names != unsafe { R_NilValue }
    }

    fn type_name(&self) -> String {
        let t = self.sexp_type();
        match t {
            SEXPTYPE::NILSXP => "NULL".to_string(),
            SEXPTYPE::LGLSXP => "logical".to_string(),
            SEXPTYPE::INTSXP => "integer".to_string(),
            SEXPTYPE::REALSXP => "numeric".to_string(),
            SEXPTYPE::STRSXP => "character".to_string(),
            SEXPTYPE::VECSXP => {
                if self.has_names() {
                    "named list".to_string()
                } else {
                    "list".to_string()
                }
            }
            SEXPTYPE::RAWSXP => "raw".to_string(),
            _ => format!("SEXPTYPE({})", t as i32),
        }
    }
}

impl<'de> de::Deserializer<'de> for RDeserializer {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.is_null() {
            return visitor.visit_unit();
        }

        let sexp_type = self.sexp_type();
        let len = self.len();

        match sexp_type {
            SEXPTYPE::LGLSXP if len == 1 => {
                let val = unsafe { crate::ffi::LOGICAL_ELT(self.sexp, 0) };
                if val == NA_LOGICAL {
                    visitor.visit_none()
                } else {
                    visitor.visit_bool(val != 0)
                }
            }
            SEXPTYPE::INTSXP if len == 1 => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                if val == NA_INTEGER {
                    visitor.visit_none()
                } else {
                    visitor.visit_i32(val)
                }
            }
            SEXPTYPE::REALSXP if len == 1 => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                if val.to_bits() == NA_REAL.to_bits() {
                    visitor.visit_none()
                } else {
                    visitor.visit_f64(val)
                }
            }
            SEXPTYPE::STRSXP if len == 1 => {
                let charsxp = unsafe { STRING_ELT(self.sexp, 0) };
                if charsxp == unsafe { R_NaString } {
                    visitor.visit_none()
                } else {
                    let s = unsafe { charsxp_to_str(charsxp) };
                    visitor.visit_str(s)
                }
            }
            // Vectors
            SEXPTYPE::LGLSXP
            | SEXPTYPE::INTSXP
            | SEXPTYPE::REALSXP
            | SEXPTYPE::STRSXP
            | SEXPTYPE::RAWSXP => visitor.visit_seq(VectorSeqAccess::new(self.sexp)),
            // Named list -> map
            SEXPTYPE::VECSXP if self.has_names() => {
                visitor.visit_map(NamedListMapAccess::new(self.sexp))
            }
            // Unnamed list -> seq
            SEXPTYPE::VECSXP => visitor.visit_seq(ListSeqAccess::new(self.sexp)),
            _ => Err(RSerdeError::UnsupportedType {
                sexptype: sexp_type as i32,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let sexp_type = self.sexp_type();
        if sexp_type != SEXPTYPE::LGLSXP || self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "logical(1)",
                actual: self.type_name(),
            });
        }

        let val = unsafe { crate::ffi::LOGICAL_ELT(self.sexp, 0) };
        if val == NA_LOGICAL {
            return Err(RSerdeError::UnexpectedNa);
        }
        visitor.visit_bool(val != 0)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_i32_inner()?;
        if val < i8::MIN as i32 || val > i8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i8",
            });
        }
        visitor.visit_i8(val as i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_i32_inner()?;
        if val < i16::MIN as i32 || val > i16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i16",
            });
        }
        visitor.visit_i16(val as i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_i32_inner()?;
        visitor.visit_i32(val)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Try integer first, then numeric
        let sexp_type = self.sexp_type();
        if self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "scalar",
                actual: self.type_name(),
            });
        }

        match sexp_type {
            SEXPTYPE::INTSXP => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                if val == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                visitor.visit_i64(val as i64)
            }
            SEXPTYPE::REALSXP => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                if val.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                visitor.visit_i64(val as i64)
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "integer or numeric",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // First try raw vector
        if self.sexp_type() == SEXPTYPE::RAWSXP && self.len() == 1 {
            let val = unsafe { crate::ffi::RAW_ELT(self.sexp, 0) };
            return visitor.visit_u8(val);
        }
        // Fall back to integer
        let val = self.deserialize_i32_inner()?;
        if val < 0 || val > u8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u8",
            });
        }
        visitor.visit_u8(val as u8)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_i32_inner()?;
        if val < 0 || val > u16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u16",
            });
        }
        visitor.visit_u16(val as u16)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Try integer first, then numeric
        let sexp_type = self.sexp_type();
        if self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "scalar",
                actual: self.type_name(),
            });
        }

        match sexp_type {
            SEXPTYPE::INTSXP => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                if val == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if val < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u32",
                    });
                }
                visitor.visit_u32(val as u32)
            }
            SEXPTYPE::REALSXP => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                if val.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if val < 0.0 || val > u32::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u32",
                    });
                }
                visitor.visit_u32(val as u32)
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "integer or numeric",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let sexp_type = self.sexp_type();
        if self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "scalar",
                actual: self.type_name(),
            });
        }

        match sexp_type {
            SEXPTYPE::INTSXP => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                if val == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if val < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u64",
                    });
                }
                visitor.visit_u64(val as u64)
            }
            SEXPTYPE::REALSXP => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                if val.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if val < 0.0 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u64",
                    });
                }
                visitor.visit_u64(val as u64)
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "integer or numeric",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_f64_inner()?;
        visitor.visit_f32(val as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.deserialize_f64_inner()?;
        visitor.visit_f64(val)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.deserialize_str_inner()?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(RSerdeError::TypeMismatch {
                expected: "single character",
                actual: format!("string of length {}", s.len()),
            }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.deserialize_str_inner()?;
        visitor.visit_str(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.deserialize_str_inner()?;
        visitor.visit_string(s.to_string())
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::RAWSXP {
            return Err(RSerdeError::TypeMismatch {
                expected: "raw vector",
                actual: self.type_name(),
            });
        }

        let len = self.len();
        let ptr = unsafe { crate::ffi::RAW(self.sexp) };
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        visitor.visit_bytes(bytes)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::RAWSXP {
            return Err(RSerdeError::TypeMismatch {
                expected: "raw vector",
                actual: self.type_name(),
            });
        }

        let len = self.len();
        let ptr = unsafe { crate::ffi::RAW(self.sexp) };
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        visitor.visit_byte_buf(bytes.to_vec())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // NULL -> None
        if self.is_null() {
            return visitor.visit_none();
        }

        // NA values -> None
        let sexp_type = self.sexp_type();
        if self.len() == 1 {
            match sexp_type {
                SEXPTYPE::LGLSXP => {
                    let val = unsafe { crate::ffi::LOGICAL_ELT(self.sexp, 0) };
                    if val == NA_LOGICAL {
                        return visitor.visit_none();
                    }
                }
                SEXPTYPE::INTSXP => {
                    let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                    if val == NA_INTEGER {
                        return visitor.visit_none();
                    }
                }
                SEXPTYPE::REALSXP => {
                    let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                    if val.to_bits() == NA_REAL.to_bits() {
                        return visitor.visit_none();
                    }
                }
                SEXPTYPE::STRSXP => {
                    let charsxp = unsafe { STRING_ELT(self.sexp, 0) };
                    if charsxp == unsafe { R_NaString } {
                        return visitor.visit_none();
                    }
                }
                _ => {}
            }
        }

        // Otherwise, deserialize the inner value
        visitor.visit_some(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if !self.is_null() {
            return Err(RSerdeError::TypeMismatch {
                expected: "NULL",
                actual: self.type_name(),
            });
        }
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let sexp_type = self.sexp_type();
        match sexp_type {
            SEXPTYPE::VECSXP => visitor.visit_seq(ListSeqAccess::new(self.sexp)),
            SEXPTYPE::LGLSXP
            | SEXPTYPE::INTSXP
            | SEXPTYPE::REALSXP
            | SEXPTYPE::STRSXP
            | SEXPTYPE::RAWSXP => visitor.visit_seq(VectorSeqAccess::new(self.sexp)),
            SEXPTYPE::NILSXP => visitor.visit_seq(EmptySeqAccess),
            _ => Err(RSerdeError::TypeMismatch {
                expected: "vector or list",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if self.len() != len {
            return Err(RSerdeError::LengthMismatch {
                expected: len,
                actual: self.len(),
            });
        }
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::VECSXP {
            return Err(RSerdeError::TypeMismatch {
                expected: "named list",
                actual: self.type_name(),
            });
        }
        visitor.visit_map(NamedListMapAccess::new(self.sexp))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::VECSXP {
            return Err(RSerdeError::TypeMismatch {
                expected: "named list",
                actual: self.type_name(),
            });
        }
        visitor.visit_map(NamedListMapAccess::new(self.sexp))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let sexp_type = self.sexp_type();

        match sexp_type {
            // String -> unit variant
            SEXPTYPE::STRSXP if self.len() == 1 => {
                let charsxp = unsafe { STRING_ELT(self.sexp, 0) };
                if charsxp == unsafe { R_NaString } {
                    return Err(RSerdeError::UnexpectedNa);
                }
                let variant = unsafe { charsxp_to_str(charsxp) };
                visitor.visit_enum(UnitVariantAccess {
                    variant: variant.to_string(),
                })
            }
            // Named list with single element -> data variant
            SEXPTYPE::VECSXP if self.has_names() && self.len() == 1 => {
                let names = unsafe { Rf_getAttrib(self.sexp, R_NamesSymbol) };
                let name_charsxp = unsafe { STRING_ELT(names, 0) };
                let variant = unsafe { charsxp_to_str(name_charsxp) };
                let value = unsafe { VECTOR_ELT(self.sexp, 0) };

                visitor.visit_enum(DataVariantAccess {
                    variant: variant.to_string(),
                    value,
                })
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "character(1) or list(variant = value)",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }
}

impl RDeserializer {
    fn deserialize_i32_inner(&self) -> Result<i32, RSerdeError> {
        let sexp_type = self.sexp_type();
        if sexp_type != SEXPTYPE::INTSXP || self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "integer(1)",
                actual: self.type_name(),
            });
        }

        let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
        if val == NA_INTEGER {
            return Err(RSerdeError::UnexpectedNa);
        }
        Ok(val)
    }

    fn deserialize_f64_inner(&self) -> Result<f64, RSerdeError> {
        let sexp_type = self.sexp_type();
        if self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "scalar",
                actual: self.type_name(),
            });
        }

        match sexp_type {
            SEXPTYPE::REALSXP => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, 0) };
                if val.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                Ok(val)
            }
            SEXPTYPE::INTSXP => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, 0) };
                if val == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                Ok(val as f64)
            }
            _ => Err(RSerdeError::TypeMismatch {
                expected: "numeric(1) or integer(1)",
                actual: self.type_name(),
            }),
        }
    }

    fn deserialize_str_inner(&self) -> Result<&str, RSerdeError> {
        let sexp_type = self.sexp_type();
        if sexp_type != SEXPTYPE::STRSXP || self.len() != 1 {
            return Err(RSerdeError::TypeMismatch {
                expected: "character(1)",
                actual: self.type_name(),
            });
        }

        let charsxp = unsafe { STRING_ELT(self.sexp, 0) };
        if charsxp == unsafe { R_NaString } {
            return Err(RSerdeError::UnexpectedNa);
        }
        Ok(unsafe { charsxp_to_str(charsxp) })
    }
}

// =============================================================================
// SeqAccess implementations
// =============================================================================

/// Empty sequence access (for NULL -> empty vec).
struct EmptySeqAccess;

impl<'de> SeqAccess<'de> for EmptySeqAccess {
    type Error = RSerdeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        _seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        Ok(None)
    }
}

/// Access elements of an R atomic vector as a sequence.
struct VectorSeqAccess {
    sexp: SEXP,
    sexp_type: SEXPTYPE,
    index: usize,
    len: usize,
}

impl VectorSeqAccess {
    fn new(sexp: SEXP) -> Self {
        VectorSeqAccess {
            sexp,
            sexp_type: unsafe { TYPEOF(sexp) as SEXPTYPE },
            index: 0,
            len: unsafe { Rf_xlength(sexp) as usize },
        }
    }
}

impl<'de> SeqAccess<'de> for VectorSeqAccess {
    type Error = RSerdeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.index >= self.len {
            return Ok(None);
        }

        // Create a scalar deserializer for this element
        let elem_de = VectorElementDeserializer {
            sexp: self.sexp,
            sexp_type: self.sexp_type,
            index: self.index,
        };

        self.index += 1;
        seed.deserialize(elem_de).map(Some)
    }
}

/// Deserializer for a single element of an atomic vector.
struct VectorElementDeserializer {
    sexp: SEXP,
    sexp_type: SEXPTYPE,
    index: usize,
}

impl<'de> de::Deserializer<'de> for VectorElementDeserializer {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type {
            SEXPTYPE::LGLSXP => {
                let val = unsafe { crate::ffi::LOGICAL_ELT(self.sexp, self.index as isize) };
                if val == NA_LOGICAL {
                    visitor.visit_none()
                } else {
                    visitor.visit_bool(val != 0)
                }
            }
            SEXPTYPE::INTSXP => {
                let val = unsafe { crate::ffi::INTEGER_ELT(self.sexp, self.index as isize) };
                if val == NA_INTEGER {
                    visitor.visit_none()
                } else {
                    visitor.visit_i32(val)
                }
            }
            SEXPTYPE::REALSXP => {
                let val = unsafe { crate::ffi::REAL_ELT(self.sexp, self.index as isize) };
                if val.to_bits() == NA_REAL.to_bits() {
                    visitor.visit_none()
                } else {
                    visitor.visit_f64(val)
                }
            }
            SEXPTYPE::STRSXP => {
                let charsxp = unsafe { STRING_ELT(self.sexp, self.index as isize) };
                if charsxp == unsafe { R_NaString } {
                    visitor.visit_none()
                } else {
                    let s = unsafe { charsxp_to_str(charsxp) };
                    visitor.visit_str(s)
                }
            }
            SEXPTYPE::RAWSXP => {
                let val = unsafe { crate::ffi::RAW_ELT(self.sexp, self.index as isize) };
                visitor.visit_u8(val)
            }
            _ => Err(RSerdeError::UnsupportedType {
                sexptype: self.sexp_type as i32,
            }),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

/// Access elements of an R list as a sequence.
struct ListSeqAccess {
    sexp: SEXP,
    index: usize,
    len: usize,
}

impl ListSeqAccess {
    fn new(sexp: SEXP) -> Self {
        ListSeqAccess {
            sexp,
            index: 0,
            len: unsafe { Rf_xlength(sexp) as usize },
        }
    }
}

impl<'de> SeqAccess<'de> for ListSeqAccess {
    type Error = RSerdeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.index >= self.len {
            return Ok(None);
        }

        let elem = unsafe { VECTOR_ELT(self.sexp, self.index as isize) };
        self.index += 1;
        seed.deserialize(RDeserializer::from_sexp(elem)).map(Some)
    }
}

// =============================================================================
// MapAccess implementation
// =============================================================================

/// Access named list as a map/struct.
struct NamedListMapAccess {
    sexp: SEXP,
    names: SEXP,
    index: usize,
    len: usize,
}

impl NamedListMapAccess {
    fn new(sexp: SEXP) -> Self {
        let names = unsafe { Rf_getAttrib(sexp, R_NamesSymbol) };
        NamedListMapAccess {
            sexp,
            names,
            index: 0,
            len: unsafe { Rf_xlength(sexp) as usize },
        }
    }
}

impl<'de> MapAccess<'de> for NamedListMapAccess {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.index >= self.len {
            return Ok(None);
        }

        let name_charsxp = unsafe { STRING_ELT(self.names, self.index as isize) };
        let name = unsafe { charsxp_to_str(name_charsxp) };

        // Create a string deserializer for the key
        seed.deserialize(StrDeserializer { s: name }).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let elem = unsafe { VECTOR_ELT(self.sexp, self.index as isize) };
        self.index += 1;
        seed.deserialize(RDeserializer::from_sexp(elem))
    }
}

/// Simple string deserializer for map keys.
struct StrDeserializer<'a> {
    s: &'a str,
}

impl<'de, 'a> de::Deserializer<'de> for StrDeserializer<'a> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_str(self.s)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

// =============================================================================
// EnumAccess implementation
// =============================================================================

/// Access for unit enum variants (from character scalar).
struct UnitVariantAccess {
    variant: String,
}

impl<'de> de::EnumAccess<'de> for UnitVariantAccess {
    type Error = RSerdeError;
    type Variant = UnitVariantDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(StrDeserializer { s: &self.variant })?;
        Ok((variant, UnitVariantDeserializer))
    }
}

struct UnitVariantDeserializer;

impl<'de> de::VariantAccess<'de> for UnitVariantDeserializer {
    type Error = RSerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        _seed: T,
    ) -> Result<T::Value, Self::Error> {
        Err(RSerdeError::Message(
            "expected unit variant, found newtype".to_string(),
        ))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(RSerdeError::Message(
            "expected unit variant, found tuple".to_string(),
        ))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Err(RSerdeError::Message(
            "expected unit variant, found struct".to_string(),
        ))
    }
}

/// Access for data enum variants (from tagged list).
struct DataVariantAccess {
    variant: String,
    value: SEXP,
}

impl<'de> de::EnumAccess<'de> for DataVariantAccess {
    type Error = RSerdeError;
    type Variant = DataVariantDeserializer;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(StrDeserializer { s: &self.variant })?;
        Ok((variant, DataVariantDeserializer { value: self.value }))
    }
}

struct DataVariantDeserializer {
    value: SEXP,
}

impl<'de> de::VariantAccess<'de> for DataVariantDeserializer {
    type Error = RSerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(RSerdeError::Message(
            "expected data variant, found unit".to_string(),
        ))
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        seed.deserialize(RDeserializer::from_sexp(self.value))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let de = RDeserializer::from_sexp(self.value);
        de.deserialize_seq(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let de = RDeserializer::from_sexp(self.value);
        de.deserialize_map(visitor)
    }
}
