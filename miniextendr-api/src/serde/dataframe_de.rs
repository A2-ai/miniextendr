//! Deserializer for converting an R `data.frame` SEXP into `Vec<T>`.
//!
//! Provides two public entry points:
//!
//! - [`dataframe_to_vec`] — owned; `T: DeserializeOwned`; always materialises
//!   `String` for character cells.
//! - [`with_dataframe_rows`] — scoped callback; `T: for<'a> Deserialize<'a>`;
//!   supports zero-copy borrows (`name: &'a str`).
//!
//! # Design (Strategy A — two deserializer types)
//!
//! Two deserializer types are provided, even though both currently materialise
//! `String` for character cells (`for<'a> Deserialize<'a>` ≡ `DeserializeOwned`):
//! - `OwnedRowDeserializer` — used by `dataframe_to_vec`; `deserialize_str` calls
//!   `visit_string` (copies into `String`).
//! - `BorrowedRowDeserializer<'a>` — used by `with_dataframe_rows`; `deserialize_str`
//!   calls `visit_borrowed_str`. Prepared for the `BorrowedRows<'a, T>` RAII
//!   variant from #671b, where `T` will be able to hold `&'a str`.
//!
//! Both walk `DataFrameView`'s named columns in insertion order, yielding
//! `(col_name, CellDeserializer)` pairs to serde's `MapAccess`.
//!
//! # Limitations
//!
//! 1. **Flat structs only.** Nested struct un-flattening (e.g., reconstructing
//!    `{ address: { city, zip } }` from columns `address_city`/`address_zip`) is
//!    not implemented. See [issue #688].
//! 2. **Factor columns deserialize as integer levels.** R factor columns are
//!    `INTSXP` with a `levels` attribute; `dataframe_to_vec` treats them as `i32`.
//!    Pre-convert with `as.character()` on the R side if you need the label string.
//!    See [issue #689].
//!
//! [issue #688]: https://github.com/A2-ai/miniextendr/issues/688
//! [issue #689]: https://github.com/A2-ai/miniextendr/issues/689

use super::error::RSerdeError;
use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::dataframe::DataFrameView;
use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::charsxp_to_str;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};

// region: public API

/// Convert an R `data.frame` SEXP into a `Vec<T>`.
///
/// Each row of the data.frame is deserialized as one `T`. `T` must be a flat
/// struct (no nested `#[derive(Deserialize)]` structs as fields — see
/// "Limitations" below).
///
/// # Type mapping
///
/// | R column type | Rust field type |
/// |---|---|
/// | `logical` | `bool` / `Option<bool>` |
/// | `integer` | `i32` / `Option<i32>` (also `i8`, `i16`, `i64`, `u*` via overflow check) |
/// | `numeric` | `f64` / `Option<f64>` |
/// | `character` | `String` / `Option<String>` |
/// | NA cell | `Option<T>` → `None`; non-optional → error |
///
/// # Errors
///
/// - `sexp` does not inherit from `"data.frame"`.
/// - A column required by `T`'s schema is missing.
/// - A cell's R type does not match the Rust field type.
/// - A non-`Option<…>` field encounters an NA cell.
///
/// # Limitations
///
/// 1. **Flat structs only.** Nested struct un-flattening is not implemented
///    ([#688](https://github.com/A2-ai/miniextendr/issues/688)).
/// 2. **Factor columns** deserialize as their integer level codes (not the label
///    strings). Pre-convert with `as.character()` on the R side if needed
///    ([#689](https://github.com/A2-ai/miniextendr/issues/689)).
pub fn dataframe_to_vec<T>(sexp: SEXP) -> Result<Vec<T>, RSerdeError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if is_empty_dataframe(sexp) {
        return Ok(Vec::new());
    }
    let view = DataFrameView::from_sexp(sexp).map_err(|e| RSerdeError::Message(e.to_string()))?;
    let nrow = view.nrow();
    let mut rows = Vec::with_capacity(nrow);
    for i in 0..nrow {
        let de = OwnedRowDeserializer::new(&view, i);
        rows.push(T::deserialize(de)?);
    }
    Ok(rows)
}

/// Pass a slice of deserialized rows to a scoped callback.
///
/// The `T: for<'a> Deserialize<'a>` bound is equivalent to `DeserializeOwned`,
/// so character fields materialise as `String` (same as [`dataframe_to_vec`]).
/// The advantage of this surface over `dataframe_to_vec` is ergonomic: the
/// callback receives `&[T]`, the borrow is scoped to `f`, and no intermediate
/// row-list SEXPs are allocated on the R heap.
///
/// For a zero-copy variant where `T` can hold `&'a str` borrowing from the SEXP's
/// CHARSXP cache, use the `BorrowedRows<'a, T>` RAII type from #671b (ships
/// separately on top of [`Protected`](crate::Protected)).
///
/// The input SEXP remains protected by R's argument frame for the duration of
/// the call, so no extra `OwnedProtect` is needed inside this function.
///
/// # Errors
///
/// Same as [`dataframe_to_vec`].
///
/// # Limitations
///
/// Same as [`dataframe_to_vec`] — flat structs only; factor columns as integers.
pub fn with_dataframe_rows<T, F, R>(sexp: SEXP, f: F) -> Result<R, RSerdeError>
where
    T: for<'a> serde::Deserialize<'a>,
    F: FnOnce(&[T]) -> R,
{
    if is_empty_dataframe(sexp) {
        let rows: Vec<T> = Vec::new();
        return Ok(f(&rows));
    }
    let view = DataFrameView::from_sexp(sexp).map_err(|e| RSerdeError::Message(e.to_string()))?;
    let nrow = view.nrow();
    let mut rows: Vec<T> = Vec::with_capacity(nrow);
    for i in 0..nrow {
        // BorrowedRowDeserializer<'_> ties the lifetime to `view`,
        // which in turn is tied to the function-local scope.
        // The compiler enforces that `rows` (and therefore any borrows inside
        // elements of `rows`) cannot outlive this function.
        let de = BorrowedRowDeserializer::new(&view, i);
        rows.push(T::deserialize(de)?);
    }
    Ok(f(&rows))
}

// endregion

// region: empty-df short-circuit

/// Detect a 0-row 0-column data.frame for which `DataFrameView::from_sexp`
/// would error on the missing names attribute.
///
/// `vec_to_dataframe(&[])` returns such a value (a VECSXP of length 0 with
/// `class = "data.frame"` and `row.names = c(NA, 0)` but no names attribute).
/// Round-tripping that through `dataframe_to_vec` should yield `Ok(vec![])`,
/// not an error.
fn is_empty_dataframe(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::VECSXP && sexp.xlength() == 0 && sexp.is_data_frame()
}

// endregion

// region: owned deserializer

/// Row deserializer for the owned path (`dataframe_to_vec`).
///
/// No lifetime parameter — character cells are copied into `String`.
struct OwnedRowDeserializer<'v> {
    view: &'v DataFrameView,
    row: usize,
}

impl<'v> OwnedRowDeserializer<'v> {
    fn new(view: &'v DataFrameView, row: usize) -> Self {
        Self { view, row }
    }
}

impl<'de> Deserializer<'de> for OwnedRowDeserializer<'_> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(RSerdeError::Message(
            "dataframe_to_vec only supports struct deserialisation".into(),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(OwnedMapAccess::new(self.view, self.row))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(OwnedMapAccess::new(self.view, self.row))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

/// `MapAccess` that iterates columns and yields owned values.
struct OwnedMapAccess<'v> {
    view: &'v DataFrameView,
    row: usize,
    /// Column names as collected once (ordered).
    names: Vec<String>,
    /// Current column index.
    col_idx: usize,
}

impl<'v> OwnedMapAccess<'v> {
    fn new(view: &'v DataFrameView, row: usize) -> Self {
        let names: Vec<String> = view.names().map(str::to_owned).collect();
        Self {
            view,
            row,
            names,
            col_idx: 0,
        }
    }
}

impl<'de> MapAccess<'de> for OwnedMapAccess<'_> {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.col_idx >= self.names.len() {
            return Ok(None);
        }
        let name = self.names[self.col_idx].as_str();
        seed.deserialize(de::value::StrDeserializer::new(name))
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let col_name = self.names[self.col_idx].as_str();
        let col_sexp = self
            .view
            .column_raw(col_name)
            .ok_or_else(|| RSerdeError::MissingField(col_name.to_owned()))?;
        self.col_idx += 1;
        let cell_de = OwnedCellDeserializer::new(col_sexp, self.row, col_name);
        seed.deserialize(cell_de)
    }
}

// endregion

// region: owned cell deserializer

/// Deserializer for a single cell on the owned path.
///
/// Reads element `row` from `col_sexp`. Character values are copied to `String`.
struct OwnedCellDeserializer<'n> {
    col: SEXP,
    row: usize,
    col_name: &'n str,
}

impl<'n> OwnedCellDeserializer<'n> {
    fn new(col: SEXP, row: usize, col_name: &'n str) -> Self {
        Self { col, row, col_name }
    }

    fn sexp_type(&self) -> SEXPTYPE {
        self.col.type_of()
    }

    fn col_type_name(&self) -> String {
        self.sexp_type().type_name().to_string()
    }

    fn type_mismatch(&self, expected: &'static str) -> RSerdeError {
        RSerdeError::Message(format!(
            "column {:?}: type mismatch: expected {}, got {}",
            self.col_name,
            expected,
            self.col_type_name()
        ))
    }

    fn row_isize(&self) -> isize {
        self.row as isize
    }
}

impl<'de> Deserializer<'de> for OwnedCellDeserializer<'_> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        match self.sexp_type() {
            SEXPTYPE::LGLSXP => {
                let v = self.col.logical_elt(i);
                if v == NA_LOGICAL {
                    visitor.visit_none()
                } else {
                    visitor.visit_bool(v != 0)
                }
            }
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(i);
                if v == NA_INTEGER {
                    visitor.visit_none()
                } else {
                    visitor.visit_i32(v)
                }
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(i);
                if v.to_bits() == NA_REAL.to_bits() {
                    visitor.visit_none()
                } else {
                    visitor.visit_f64(v)
                }
            }
            SEXPTYPE::STRSXP => {
                let charsxp = self.col.string_elt(i);
                if charsxp == SEXP::na_string() {
                    visitor.visit_none()
                } else {
                    let s = unsafe { charsxp_to_str(charsxp) };
                    visitor.visit_string(s.to_owned())
                }
            }
            _ => Err(RSerdeError::UnsupportedType {
                sexptype: self.sexp_type() as i32,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::LGLSXP {
            return Err(self.type_mismatch("logical"));
        }
        let v = self.col.logical_elt(self.row_isize());
        if v == NA_LOGICAL {
            return Err(RSerdeError::UnexpectedNa);
        }
        visitor.visit_bool(v != 0)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < i8::MIN as i32 || v > i8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i8",
            });
        }
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < i16::MIN as i32 || v > i16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i16",
            });
        }
        visitor.visit_i16(v as i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                visitor.visit_i64(v as i64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < i64::MIN as f64 || v > i64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "i64",
                    });
                }
                visitor.visit_i64(v as i64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < 0 || v > u8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u8",
            });
        }
        visitor.visit_u8(v as u8)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < 0 || v > u16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u16",
            });
        }
        visitor.visit_u16(v as u16)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u32::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell(&self)?;
        visitor.visit_f32(v as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell(&self)?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::STRSXP {
            return Err(self.type_mismatch("character"));
        }
        let charsxp = self.col.string_elt(self.row_isize());
        if charsxp == SEXP::na_string() {
            return Err(RSerdeError::UnexpectedNa);
        }
        let s = unsafe { charsxp_to_str(charsxp) };
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
        if self.sexp_type() != SEXPTYPE::STRSXP {
            return Err(self.type_mismatch("character"));
        }
        let charsxp = self.col.string_elt(self.row_isize());
        if charsxp == SEXP::na_string() {
            return Err(RSerdeError::UnexpectedNa);
        }
        let s = unsafe { charsxp_to_str(charsxp) };
        // owned path: copy to String
        visitor.visit_string(s.to_owned())
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        let is_na = match self.sexp_type() {
            SEXPTYPE::LGLSXP => self.col.logical_elt(i) == NA_LOGICAL,
            SEXPTYPE::INTSXP => self.col.integer_elt(i) == NA_INTEGER,
            SEXPTYPE::REALSXP => self.col.real_elt(i).to_bits() == NA_REAL.to_bits(),
            SEXPTYPE::STRSXP => self.col.string_elt(i) == SEXP::na_string(),
            _ => false,
        };
        if is_na {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum
    }
}

// endregion

// region: borrowed deserializer

/// Row deserializer for the borrowed path (`with_dataframe_rows`).
///
/// Lifetime `'sexp` is tied to the SEXP (via `DataFrameView`). Character cells
/// are handed out as `&'sexp str` — zero-copy from CHARSXP.
struct BorrowedRowDeserializer<'sexp> {
    view: &'sexp DataFrameView,
    row: usize,
}

impl<'sexp> BorrowedRowDeserializer<'sexp> {
    fn new(view: &'sexp DataFrameView, row: usize) -> Self {
        Self { view, row }
    }
}

impl<'de> Deserializer<'de> for BorrowedRowDeserializer<'de> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(RSerdeError::Message(
            "with_dataframe_rows only supports struct deserialisation".into(),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(BorrowedMapAccess::new(self.view, self.row))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(BorrowedMapAccess::new(self.view, self.row))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

/// `MapAccess` that iterates columns and yields borrowed values.
struct BorrowedMapAccess<'sexp> {
    view: &'sexp DataFrameView,
    row: usize,
    /// Column names as collected once (ordered).
    names: Vec<String>,
    col_idx: usize,
}

impl<'sexp> BorrowedMapAccess<'sexp> {
    fn new(view: &'sexp DataFrameView, row: usize) -> Self {
        let names: Vec<String> = view.names().map(str::to_owned).collect();
        Self {
            view,
            row,
            names,
            col_idx: 0,
        }
    }
}

impl<'de> MapAccess<'de> for BorrowedMapAccess<'de> {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.col_idx >= self.names.len() {
            return Ok(None);
        }
        let name = self.names[self.col_idx].as_str();
        seed.deserialize(de::value::StrDeserializer::new(name))
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let col_name = self.names[self.col_idx].as_str();
        let col_sexp = self
            .view
            .column_raw(col_name)
            .ok_or_else(|| RSerdeError::MissingField(col_name.to_owned()))?;
        self.col_idx += 1;
        let cell_de = BorrowedCellDeserializer::new(col_sexp, self.row, col_name);
        seed.deserialize(cell_de)
    }
}

// endregion

// region: borrowed cell deserializer

/// Deserializer for a single cell on the borrowed path.
///
/// The lifetime `'sexp` on `BorrowedCellDeserializer<'sexp>` is the lifetime of
/// the CHARSXP data pointers. When serde calls `deserialize_str`, we hand out a
/// `&'sexp str` — zero-copy from R's string cache.
struct BorrowedCellDeserializer<'sexp, 'n> {
    col: SEXP,
    row: usize,
    col_name: &'n str,
    _marker: std::marker::PhantomData<&'sexp ()>,
}

impl<'sexp, 'n> BorrowedCellDeserializer<'sexp, 'n> {
    fn new(col: SEXP, row: usize, col_name: &'n str) -> Self {
        Self {
            col,
            row,
            col_name,
            _marker: std::marker::PhantomData,
        }
    }

    fn sexp_type(&self) -> SEXPTYPE {
        self.col.type_of()
    }

    fn col_type_name(&self) -> String {
        self.sexp_type().type_name().to_string()
    }

    fn type_mismatch(&self, expected: &'static str) -> RSerdeError {
        RSerdeError::Message(format!(
            "column {:?}: type mismatch: expected {}, got {}",
            self.col_name,
            expected,
            self.col_type_name()
        ))
    }

    fn row_isize(&self) -> isize {
        self.row as isize
    }
}

// Helper: build an OwnedCellDeserializer-like view for shared integer/real helpers.
// Since BorrowedCellDeserializer is almost identical to OwnedCellDeserializer
// for non-string types, we factor the integer/real reads out as free functions
// below to avoid duplication.

impl<'de> Deserializer<'de> for BorrowedCellDeserializer<'de, '_> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        match self.sexp_type() {
            SEXPTYPE::LGLSXP => {
                let v = self.col.logical_elt(i);
                if v == NA_LOGICAL {
                    visitor.visit_none()
                } else {
                    visitor.visit_bool(v != 0)
                }
            }
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(i);
                if v == NA_INTEGER {
                    visitor.visit_none()
                } else {
                    visitor.visit_i32(v)
                }
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(i);
                if v.to_bits() == NA_REAL.to_bits() {
                    visitor.visit_none()
                } else {
                    visitor.visit_f64(v)
                }
            }
            SEXPTYPE::STRSXP => {
                let charsxp = self.col.string_elt(i);
                if charsxp == SEXP::na_string() {
                    visitor.visit_none()
                } else {
                    // SAFETY: The SEXP is protected by R's argument frame for the
                    // duration of the enclosing `with_dataframe_rows` call, so the
                    // CHARSXP data pointer is valid for `'de` (= `'sexp`).
                    let s: &'de str = unsafe { charsxp_to_str(charsxp) };
                    visitor.visit_borrowed_str(s)
                }
            }
            _ => Err(RSerdeError::UnsupportedType {
                sexptype: self.sexp_type() as i32,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::LGLSXP {
            return Err(self.type_mismatch("logical"));
        }
        let v = self.col.logical_elt(self.row_isize());
        if v == NA_LOGICAL {
            return Err(RSerdeError::UnexpectedNa);
        }
        visitor.visit_bool(v != 0)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        if v < i8::MIN as i32 || v > i8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i8",
            });
        }
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        if v < i16::MIN as i32 || v > i16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i16",
            });
        }
        visitor.visit_i16(v as i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                visitor.visit_i64(v as i64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < i64::MIN as f64 || v > i64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "i64",
                    });
                }
                visitor.visit_i64(v as i64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        if v < 0 || v > u8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u8",
            });
        }
        visitor.visit_u8(v as u8)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        if v < 0 || v > u16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u16",
            });
        }
        visitor.visit_u16(v as u16)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u32::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        visitor.visit_f32(v as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell_raw(
            &self.col,
            self.row_isize(),
            self.sexp_type(),
            self.col_name,
        )?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::STRSXP {
            return Err(self.type_mismatch("character"));
        }
        let charsxp = self.col.string_elt(self.row_isize());
        if charsxp == SEXP::na_string() {
            return Err(RSerdeError::UnexpectedNa);
        }
        let s: &'de str = unsafe { charsxp_to_str(charsxp) };
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
        if self.sexp_type() != SEXPTYPE::STRSXP {
            return Err(self.type_mismatch("character"));
        }
        let charsxp = self.col.string_elt(self.row_isize());
        if charsxp == SEXP::na_string() {
            return Err(RSerdeError::UnexpectedNa);
        }
        // SAFETY: The SEXP is rooted by R's argument frame for the duration of
        // `with_dataframe_rows`; `'de` equals `'sexp` here.
        let s: &'de str = unsafe { charsxp_to_str(charsxp) };
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        let is_na = match self.sexp_type() {
            SEXPTYPE::LGLSXP => self.col.logical_elt(i) == NA_LOGICAL,
            SEXPTYPE::INTSXP => self.col.integer_elt(i) == NA_INTEGER,
            SEXPTYPE::REALSXP => self.col.real_elt(i).to_bits() == NA_REAL.to_bits(),
            SEXPTYPE::STRSXP => self.col.string_elt(i) == SEXP::na_string(),
            _ => false,
        };
        if is_na {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum
    }
}

// endregion

// region: shared cell helpers

/// Read an integer cell, failing on NA and type mismatch.
fn deserialize_integer_cell(de: &OwnedCellDeserializer<'_>) -> Result<i32, RSerdeError> {
    if de.sexp_type() != SEXPTYPE::INTSXP {
        return Err(de.type_mismatch("integer"));
    }
    let v = de.col.integer_elt(de.row_isize());
    if v == NA_INTEGER {
        return Err(RSerdeError::UnexpectedNa);
    }
    Ok(v)
}

/// Read a real cell, accepting integer columns via widening.
fn deserialize_real_cell(de: &OwnedCellDeserializer<'_>) -> Result<f64, RSerdeError> {
    match de.sexp_type() {
        SEXPTYPE::REALSXP => {
            let v = de.col.real_elt(de.row_isize());
            if v.to_bits() == NA_REAL.to_bits() {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v)
        }
        SEXPTYPE::INTSXP => {
            let v = de.col.integer_elt(de.row_isize());
            if v == NA_INTEGER {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v as f64)
        }
        _ => Err(de.type_mismatch("numeric or integer")),
    }
}

/// Read an integer cell (free-fn variant for borrowed path).
fn deserialize_integer_cell_raw(
    col: &SEXP,
    i: isize,
    stype: SEXPTYPE,
    col_name: &str,
) -> Result<i32, RSerdeError> {
    if stype != SEXPTYPE::INTSXP {
        return Err(RSerdeError::Message(format!(
            "column {:?}: type mismatch: expected integer, got {}",
            col_name,
            stype.type_name()
        )));
    }
    let v = col.integer_elt(i);
    if v == NA_INTEGER {
        return Err(RSerdeError::UnexpectedNa);
    }
    Ok(v)
}

/// Read a real cell, accepting integer columns via widening (free-fn variant for borrowed path).
fn deserialize_real_cell_raw(
    col: &SEXP,
    i: isize,
    stype: SEXPTYPE,
    col_name: &str,
) -> Result<f64, RSerdeError> {
    match stype {
        SEXPTYPE::REALSXP => {
            let v = col.real_elt(i);
            if v.to_bits() == NA_REAL.to_bits() {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v)
        }
        SEXPTYPE::INTSXP => {
            let v = col.integer_elt(i);
            if v == NA_INTEGER {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v as f64)
        }
        _ => Err(RSerdeError::Message(format!(
            "column {:?}: type mismatch: expected numeric or integer, got {}",
            col_name,
            stype.type_name()
        ))),
    }
}

// endregion
