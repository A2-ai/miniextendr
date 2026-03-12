//! Columnar serializer for converting `Vec<T>` directly to R data.frames.
//!
//! Instead of serializing each struct as a named list (row-oriented), this
//! module transposes the data into column-oriented R vectors — one atomic
//! vector per field. This is more memory-efficient and produces native R
//! data.frames directly.

use super::error::RSerdeError;
use crate::altrep_traits::NA_REAL;
use crate::ffi::{
    self, CE_UTF8, R_ClassSymbol, R_NaString, R_NamesSymbol, R_NilValue, R_RowNamesSymbol,
    Rf_allocVector, Rf_mkCharLenCE, Rf_protect, Rf_setAttrib, Rf_unprotect, SET_INTEGER_ELT,
    SET_LOGICAL_ELT, SET_REAL_ELT, SET_STRING_ELT, SET_VECTOR_ELT, SEXP, SEXPTYPE,
};
use serde::ser::{self, Serialize};

/// Convert a slice of serializable structs to an R data.frame in columnar layout.
///
/// Each field of `T` becomes a column (R atomic vector). This avoids the
/// row-oriented pattern of serializing each struct as a named list.
///
/// # Supported Field Types
///
/// | Rust Type | R Column Type |
/// |-----------|---------------|
/// | `bool` | `logical` |
/// | `i8/i16/i32` | `integer` |
/// | `i64/u64/f32/f64` | `numeric` |
/// | `String/&str` | `character` |
/// | `Option<T>` | Same type with NA for `None` |
/// | Other | Falls back to per-element list column |
///
/// # Example
///
/// ```ignore
/// use serde::Serialize;
/// use miniextendr_api::serde::columnar::vec_to_dataframe;
///
/// #[derive(Serialize)]
/// struct Point { x: f64, y: f64, label: String }
///
/// let points = vec![
///     Point { x: 1.0, y: 2.0, label: "a".into() },
///     Point { x: 3.0, y: 4.0, label: "b".into() },
/// ];
///
/// let df = vec_to_dataframe(&points).unwrap();
/// // Result: data.frame with columns x (numeric), y (numeric), label (character)
/// ```
pub fn vec_to_dataframe<T: Serialize>(rows: &[T]) -> Result<SEXP, RSerdeError> {
    if rows.is_empty() {
        return Ok(empty_dataframe());
    }

    // Phase 1: Discover schema from first element
    let schema = discover_schema(&rows[0])?;
    let ncol = schema.fields.len();
    let nrow = rows.len();

    if ncol == 0 {
        return Err(RSerdeError::Message(
            "vec_to_dataframe: type has no fields".into(),
        ));
    }

    // Phase 2: Allocate column buffers
    let mut columns: Vec<ColumnBuffer> = schema
        .fields
        .iter()
        .map(|f| ColumnBuffer::new(f.col_type, nrow))
        .collect();

    // Phase 3: Fill columns from all rows
    for row in rows {
        let filler = ColumnFiller {
            columns: &mut columns,
            field_idx: 0,
        };
        row.serialize(filler)?;
    }

    // Phase 4: Assemble data.frame
    Ok(unsafe { assemble_dataframe(&schema.fields, &columns, nrow) })
}

// region: Schema discovery

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Logical,
    Integer,
    Real,
    Character,
    Generic,
}

struct FieldInfo {
    name: String,
    col_type: ColumnType,
}

struct Schema {
    fields: Vec<FieldInfo>,
}

fn discover_schema<T: Serialize>(sample: &T) -> Result<Schema, RSerdeError> {
    let mut discoverer = SchemaDiscoverer { fields: Vec::new() };
    sample.serialize(&mut discoverer)?;
    Ok(Schema {
        fields: discoverer.fields,
    })
}

struct SchemaDiscoverer {
    fields: Vec<FieldInfo>,
}

impl<'a> ser::Serializer for &'a mut SchemaDiscoverer {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = SchemaStructDiscoverer<'a>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SchemaStructDiscoverer { parent: self })
    }

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "vec_to_dataframe: expected struct".into(),
        ))
    }
}

struct SchemaStructDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
}

impl ser::SerializeStruct for SchemaStructDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        let mut type_probe = TypeProbe {
            col_type: ColumnType::Generic,
        };
        // Best-effort type detection; fall back to Generic on error
        let _ = value.serialize(&mut type_probe);
        self.parent.fields.push(FieldInfo {
            name: key.to_string(),
            col_type: type_probe.col_type,
        });
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}
// endregion

// region: Type probe (discovers column type from a single value)

struct TypeProbe {
    col_type: ColumnType,
}

impl ser::Serializer for &mut TypeProbe {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Logical;
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        // Keep existing type (handles Option<T> where first element is None)
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
}
// endregion

// region: Column buffers

enum ColumnBuffer {
    Logical(Vec<i32>),
    Integer(Vec<i32>),
    Real(Vec<f64>),
    Character(Vec<Option<String>>),
    Generic(Vec<Option<SEXP>>),
}

impl ColumnBuffer {
    fn new(col_type: ColumnType, capacity: usize) -> Self {
        match col_type {
            ColumnType::Logical => ColumnBuffer::Logical(Vec::with_capacity(capacity)),
            ColumnType::Integer => ColumnBuffer::Integer(Vec::with_capacity(capacity)),
            ColumnType::Real => ColumnBuffer::Real(Vec::with_capacity(capacity)),
            ColumnType::Character => ColumnBuffer::Character(Vec::with_capacity(capacity)),
            ColumnType::Generic => ColumnBuffer::Generic(Vec::with_capacity(capacity)),
        }
    }

    fn push_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        match self {
            ColumnBuffer::Logical(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Bool(b) => i32::from(b),
                    ExtractedValue::None => i32::MIN, // NA_LOGICAL
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Integer(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Int(i) => i,
                    ExtractedValue::None => i32::MIN, // NA_INTEGER
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Real(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Real(f) => f,
                    ExtractedValue::Int(i) => i as f64,
                    ExtractedValue::None => NA_REAL,
                    _ => NA_REAL,
                });
            }
            ColumnBuffer::Character(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Str(s) => Some(s),
                    ExtractedValue::None => None, // NA_character_
                    _ => None,
                });
            }
            ColumnBuffer::Generic(v) => {
                // Fall back to full serde serialization for this element
                let sexp = value.serialize(super::ser::RSerializer)?;
                v.push(Some(sexp));
            }
        }
        Ok(())
    }
}
// endregion

// region: Value extraction

#[derive(Default)]
struct ValueExtractor {
    value: ExtractedValue,
}

#[derive(Default)]
enum ExtractedValue {
    #[default]
    None,
    Bool(bool),
    Int(i32),
    Real(f64),
    Str(String),
}

impl ser::Serializer for &mut ValueExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, v: bool) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Bool(v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v as i32);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v as i32);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v as i32);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v as i32);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), RSerdeError> {
        if v <= i32::MAX as u32 {
            self.value = ExtractedValue::Int(v as i32);
        } else {
            self.value = ExtractedValue::Real(v as f64);
        }
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
}
// endregion

// region: Column filler (second pass — fills column buffers by field position)

/// Serializer that fills pre-allocated column buffers.
///
/// Consumed by value: implements both `Serializer` (accepts the struct) and
/// `SerializeStruct` (iterates over fields). This avoids the lifetime issues
/// of having a separate `StructFiller` borrow `ColumnFiller` mutably.
struct ColumnFiller<'a> {
    columns: &'a mut [ColumnBuffer],
    field_idx: usize,
}

impl<'a> ser::Serializer for ColumnFiller<'a> {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        Err(RSerdeError::Message("expected struct".into()))
    }
}

impl ser::SerializeStruct for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        if self.field_idx < self.columns.len() {
            self.columns[self.field_idx].push_value(value)?;
        }
        self.field_idx += 1;
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}
// endregion

// region: Data.frame assembly

/// Build an empty R data.frame (0 rows, 0 columns).
fn empty_dataframe() -> SEXP {
    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 0);
        Rf_protect(list);

        // Set class = "data.frame"
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        Rf_protect(class_sexp);
        let class_str = Rf_mkCharLenCE(c"data.frame".as_ptr().cast(), 10, CE_UTF8);
        SET_STRING_ELT(class_sexp, 0, class_str);
        Rf_setAttrib(list, R_ClassSymbol, class_sexp);

        // Set compact row.names: c(NA_integer_, 0)
        let row_names = Rf_allocVector(SEXPTYPE::INTSXP, 2);
        Rf_protect(row_names);
        let rn_ptr = ffi::INTEGER(row_names);
        *rn_ptr = i32::MIN; // NA_integer_
        *rn_ptr.add(1) = 0;
        Rf_setAttrib(list, R_RowNamesSymbol, row_names);

        Rf_unprotect(3);
        list
    }
}

/// Assemble column buffers into an R data.frame SEXP.
///
/// # Safety
///
/// Must be called from the R main thread. All column buffers must have
/// exactly `nrow` elements.
unsafe fn assemble_dataframe(fields: &[FieldInfo], columns: &[ColumnBuffer], nrow: usize) -> SEXP {
    let ncol = fields.len();

    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, ncol as isize);
        Rf_protect(list);

        // Build each column and set into list
        for (i, col) in columns.iter().enumerate() {
            let col_sexp = column_to_sexp(col, nrow);
            Rf_protect(col_sexp);
            SET_VECTOR_ELT(list, i as isize, col_sexp);
            Rf_unprotect(1); // col_sexp is now held by list
        }

        // Set names
        let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, ncol as isize);
        Rf_protect(names_sexp);
        for (i, field) in fields.iter().enumerate() {
            let charsxp =
                Rf_mkCharLenCE(field.name.as_ptr().cast(), field.name.len() as i32, CE_UTF8);
            SET_STRING_ELT(names_sexp, i as isize, charsxp);
        }
        Rf_setAttrib(list, R_NamesSymbol, names_sexp);

        // Set class = "data.frame"
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        Rf_protect(class_sexp);
        let class_str = Rf_mkCharLenCE(c"data.frame".as_ptr().cast(), 10, CE_UTF8);
        SET_STRING_ELT(class_sexp, 0, class_str);
        Rf_setAttrib(list, R_ClassSymbol, class_sexp);

        // Set compact row.names: c(NA_integer_, -nrow)
        let row_names = Rf_allocVector(SEXPTYPE::INTSXP, 2);
        Rf_protect(row_names);
        let rn_ptr = ffi::INTEGER(row_names);
        *rn_ptr = i32::MIN; // NA_integer_
        *rn_ptr.add(1) = -(nrow as i32);
        Rf_setAttrib(list, R_RowNamesSymbol, row_names);

        Rf_unprotect(4); // list, names, class, row_names
        list
    }
}

/// Convert a single column buffer into an R SEXP vector.
unsafe fn column_to_sexp(col: &ColumnBuffer, nrow: usize) -> SEXP {
    unsafe {
        match col {
            ColumnBuffer::Logical(v) => {
                let sexp = Rf_allocVector(SEXPTYPE::LGLSXP, nrow as isize);
                for (i, &val) in v.iter().enumerate() {
                    SET_LOGICAL_ELT(sexp, i as isize, val);
                }
                sexp
            }
            ColumnBuffer::Integer(v) => {
                let sexp = Rf_allocVector(SEXPTYPE::INTSXP, nrow as isize);
                for (i, &val) in v.iter().enumerate() {
                    SET_INTEGER_ELT(sexp, i as isize, val);
                }
                sexp
            }
            ColumnBuffer::Real(v) => {
                let sexp = Rf_allocVector(SEXPTYPE::REALSXP, nrow as isize);
                for (i, &val) in v.iter().enumerate() {
                    SET_REAL_ELT(sexp, i as isize, val);
                }
                sexp
            }
            ColumnBuffer::Character(v) => {
                let sexp = Rf_allocVector(SEXPTYPE::STRSXP, nrow as isize);
                for (i, val) in v.iter().enumerate() {
                    match val {
                        Some(s) => {
                            let charsxp =
                                Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, CE_UTF8);
                            SET_STRING_ELT(sexp, i as isize, charsxp);
                        }
                        None => {
                            SET_STRING_ELT(sexp, i as isize, R_NaString);
                        }
                    }
                }
                sexp
            }
            ColumnBuffer::Generic(v) => {
                let sexp = Rf_allocVector(SEXPTYPE::VECSXP, nrow as isize);
                for (i, val) in v.iter().enumerate() {
                    if let Some(elem) = val {
                        SET_VECTOR_ELT(sexp, i as isize, *elem);
                    } else {
                        SET_VECTOR_ELT(sexp, i as isize, R_NilValue);
                    }
                }
                sexp
            }
        }
    }
}
// endregion
