//! Serializer for converting Rust values to R SEXP via serde.
//!
//! Implements `serde::Serializer` that produces R SEXPs, enabling `to_sexp(&value)`
//! to serialize any serde-compatible Rust type into R data structures.

use super::error::RSerdeError;
use crate::ffi::{
    R_NaString, Rf_allocVector, Rf_protect, Rf_unprotect, SET_VECTOR_ELT, SEXP, SEXPTYPE, SexpExt,
};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use serde::ser::{self, Serialize};

/// Serializer that converts Rust values directly to R SEXP.
///
/// # Type Mappings
///
/// | Rust Type | R Type |
/// |-----------|--------|
/// | `bool` | `logical(1)` |
/// | `i8/i16/i32` | `integer(1)` |
/// | `i64/u64/f32/f64` | `numeric(1)` |
/// | `String/&str` | `character(1)` |
/// | `Option<T>::None` | NA of appropriate type |
/// | `Vec<primitive>` | atomic vector |
/// | `Vec<struct>` | list of lists |
/// | `HashMap<String, T>` | named list |
/// | `struct` | named list |
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::serde_r::RSerializer;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point { x: f64, y: f64 }
///
/// let p = Point { x: 1.0, y: 2.0 };
/// let sexp = RSerializer::to_sexp(&p).unwrap();
/// // Result: list(x = 1.0, y = 2.0)
/// ```
pub struct RSerializer;

impl RSerializer {
    /// Serialize a Rust value to an R SEXP.
    pub fn to_sexp<T: Serialize>(value: &T) -> Result<SEXP, RSerdeError> {
        value.serialize(RSerializer)
    }
}

impl ser::Serializer for RSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_sexp())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(i32::from(v).into_sexp())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(i32::from(v).into_sexp())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_sexp())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        // Use i64's IntoR which routes to INTSXP when in range, else REALSXP
        Ok(v.into_sexp())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(i32::from(v).into_sexp())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(i32::from(v).into_sexp())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        // u32 can overflow i32 — use i64's smart routing (INTSXP when fits, else REALSXP)
        Ok(i64::from(v).into_sexp())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        // Uses u64's IntoR: INTSXP when fits i32, else REALSXP (may lose precision > 2^53)
        Ok(v.into_sexp())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(f64::from(v).into_sexp())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_sexp())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        let s = v.encode_utf8(&mut buf);
        self.serialize_str(s)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_sexp())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_sexp())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // None becomes R NULL
        // For NA handling, use Option<T> which maps None -> NA in specific contexts
        Ok(SEXP::null())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(SEXP::null())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(SEXP::null())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // Unit variant becomes a character scalar: "VariantName"
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Newtype struct is transparent
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Newtype variant: list(VariantName = value)
        let inner = value.serialize(RSerializer)?;
        Ok(make_tagged_list(variant, inner))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer::new(len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SeqSerializer::new(Some(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SeqSerializer::new(Some(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TupleVariantSerializer {
            variant,
            inner: SeqSerializer::new(Some(len)),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(len))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer::new(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructVariantSerializer {
            variant,
            inner: StructSerializer::new(len),
        })
    }
}

/// Serializer for sequences (Vec, tuples).
///
/// Uses smart dispatch: if all elements are homogeneous scalars of the same
/// primitive type, coalesces into an R atomic vector. Otherwise creates a list.
pub struct SeqSerializer {
    elements: Vec<SEXP>,
    /// Track element type for potential coalescing.
    element_type: Option<SEXPTYPE>,
    /// Track whether all elements are scalar.
    all_scalar: bool,
    /// Number of Rf_protect() calls to balance on end().
    protect_count: i32,
}

impl SeqSerializer {
    fn new(len: Option<usize>) -> Self {
        SeqSerializer {
            elements: Vec::with_capacity(len.unwrap_or(0)),
            element_type: None,
            all_scalar: true,
            protect_count: 0,
        }
    }
}

impl ser::SerializeSeq for SeqSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let elem = value.serialize(RSerializer)?;

        // Track homogeneity for smart dispatch
        let elem_len = unsafe { crate::ffi::Rf_xlength(elem) };
        let elem_type = elem.type_of();

        if elem_len != 1 {
            self.all_scalar = false;
        }

        match self.element_type {
            None => self.element_type = Some(elem_type),
            Some(t) if t != elem_type => self.all_scalar = false,
            _ => {}
        }

        // Protect intermediate SEXP from GC during subsequent serializations.
        unsafe { Rf_protect(elem) };
        self.protect_count += 1;
        self.elements.push(elem);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let result = crate::list::List::from_scalars_or_list(&self.elements).as_sexp();
        // Unprotect all intermediate elements now that they're in the container.
        unsafe { Rf_unprotect(self.protect_count) };
        Ok(result)
    }
}

impl ser::SerializeTuple for SeqSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Tuples always become lists (heterogeneous by nature)
        let result = create_r_list(&self.elements);
        unsafe { Rf_unprotect(self.protect_count) };
        Ok(result)
    }
}

impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let result = create_r_list(&self.elements);
        unsafe { Rf_unprotect(self.protect_count) };
        Ok(result)
    }
}

/// Serializer for tuple variants: `Enum::Variant(a, b)` -> `list(Variant = list(a, b))`
pub struct TupleVariantSerializer {
    variant: &'static str,
    inner: SeqSerializer,
}

impl ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(&mut self.inner, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let inner_list = create_r_list(&self.inner.elements);
        unsafe { Rf_unprotect(self.inner.protect_count) };
        Ok(make_tagged_list(self.variant, inner_list))
    }
}

/// Serializer for maps (HashMap, BTreeMap).
pub struct MapSerializer {
    keys: Vec<String>,
    values: Vec<SEXP>,
    protect_count: i32,
}

impl MapSerializer {
    fn new(len: Option<usize>) -> Self {
        let cap = len.unwrap_or(0);
        MapSerializer {
            keys: Vec::with_capacity(cap),
            values: Vec::with_capacity(cap),
            protect_count: 0,
        }
    }
}

impl ser::SerializeMap for MapSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        // Keys must be strings for R named lists
        let key_sexp = key.serialize(RSerializer)?;
        let key_str = sexp_to_string(key_sexp)?;
        self.keys.push(key_str);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let val = value.serialize(RSerializer)?;
        unsafe { Rf_protect(val) };
        self.protect_count += 1;
        self.values.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let result = create_named_list(&self.keys, &self.values);
        unsafe { Rf_unprotect(self.protect_count) };
        Ok(result)
    }
}

/// Serializer for structs.
pub struct StructSerializer {
    fields: Vec<(&'static str, SEXP)>,
    protect_count: i32,
}

impl StructSerializer {
    fn new(len: usize) -> Self {
        StructSerializer {
            fields: Vec::with_capacity(len),
            protect_count: 0,
        }
    }
}

impl ser::SerializeStruct for StructSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let val = value.serialize(RSerializer)?;
        unsafe { Rf_protect(val) };
        self.protect_count += 1;
        self.fields.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let names: Vec<&str> = self.fields.iter().map(|(k, _)| *k).collect();
        let values: Vec<SEXP> = self.fields.into_iter().map(|(_, v)| v).collect();
        let result = create_named_list_static(&names, &values);
        unsafe { Rf_unprotect(self.protect_count) };
        Ok(result)
    }
}

/// Serializer for struct variants: `Enum::Variant { a, b }` -> `list(Variant = list(a=..., b=...))`
pub struct StructVariantSerializer {
    variant: &'static str,
    inner: StructSerializer,
}

impl ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = SEXP;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        ser::SerializeStruct::serialize_field(&mut self.inner, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let inner = ser::SerializeStruct::end(self.inner)?;
        // SerializeStruct::end already called Rf_unprotect for struct fields.
        Ok(make_tagged_list(self.variant, inner))
    }
}

// region: Helper functions

/// Create an unnamed R list from SEXPs.
fn create_r_list(elements: &[SEXP]) -> SEXP {
    let n: isize = elements
        .len()
        .try_into()
        .expect("list length exceeds isize::MAX");
    let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n)) };

    for (i, &elem) in elements.iter().enumerate() {
        let idx: isize = i.try_into().expect("index exceeds isize::MAX");
        unsafe { SET_VECTOR_ELT(sexp.get(), idx, elem) };
    }

    sexp.get()
}

/// Create a named R list from string keys and SEXP values.
fn create_named_list(keys: &[String], values: &[SEXP]) -> SEXP {
    debug_assert_eq!(keys.len(), values.len());
    let n: isize = keys
        .len()
        .try_into()
        .expect("list length exceeds isize::MAX");

    let list = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n)) };
    let names = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n)) };

    for (i, (key, &value)) in keys.iter().zip(values.iter()).enumerate() {
        let idx: isize = i.try_into().expect("index exceeds isize::MAX");
        unsafe { SET_VECTOR_ELT(list.get(), idx, value) };
        names.get().set_string_elt(idx, SEXP::charsxp(key));
    }

    list.get().set_names(names.get());
    list.get()
}

/// Create a named R list from static string keys and SEXP values.
fn create_named_list_static(keys: &[&str], values: &[SEXP]) -> SEXP {
    debug_assert_eq!(keys.len(), values.len());
    let n: isize = keys
        .len()
        .try_into()
        .expect("list length exceeds isize::MAX");

    let list = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n)) };
    let names = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n)) };

    for (i, (&key, &value)) in keys.iter().zip(values.iter()).enumerate() {
        let idx: isize = i.try_into().expect("index exceeds isize::MAX");
        unsafe { SET_VECTOR_ELT(list.get(), idx, value) };
        names.get().set_string_elt(idx, SEXP::charsxp(key));
    }

    list.get().set_names(names.get());
    list.get()
}

/// Create a tagged list: `list(tag = value)`
fn make_tagged_list(tag: &str, value: SEXP) -> SEXP {
    let list = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, 1)) };
    let names = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, 1)) };

    unsafe { SET_VECTOR_ELT(list.get(), 0, value) };
    names.get().set_string_elt(0, SEXP::charsxp(tag));
    list.get().set_names(names.get());

    list.get()
}

/// Extract a string from a SEXP (must be STRSXP of length 1).
fn sexp_to_string(sexp: SEXP) -> Result<String, RSerdeError> {
    let sexp_type = sexp.type_of();
    if sexp_type != SEXPTYPE::STRSXP {
        return Err(RSerdeError::NonStringKey);
    }

    let len = unsafe { crate::ffi::Rf_xlength(sexp) };
    if len != 1 {
        return Err(RSerdeError::NonStringKey);
    }

    let charsxp = unsafe { crate::ffi::STRING_ELT(sexp, 0) };
    if charsxp == unsafe { R_NaString } {
        return Err(RSerdeError::NonStringKey);
    }

    let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
    Ok(s.to_string())
}
// endregion
