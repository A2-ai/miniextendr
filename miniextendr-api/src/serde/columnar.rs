//! Columnar serializer for converting `Vec<T>` directly to R data.frames.
//!
//! Instead of serializing each struct as a named list (row-oriented), this
//! module transposes the data into column-oriented R vectors — one atomic
//! vector per field. This is more memory-efficient and produces native R
//! data.frames directly.
//!
//! Nested structs are recursively flattened into prefixed columns
//! (e.g., `metadata_size`). `#[serde(flatten)]` and `#[serde(skip_serializing_if)]`
//! are handled correctly.

use std::collections::HashMap;

use super::error::RSerdeError;
use crate::altrep_traits::{NA_LOGICAL, NA_REAL};
use crate::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt};
use serde::ser::{self, Serialize};

/// Generate serde `Serializer` error stubs for methods that should reject non-struct input.
/// Accepts `struct`/`map` to allow, and an error message for the rest.
macro_rules! reject_non_struct {
    ($msg:expr, allow_some_none) => {
        fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
            value.serialize(self)
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message(concat!($msg, " (got None)").into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    ($msg:expr) => {
        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    (@primitives $msg:expr) => {
        fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_char(self, _: char) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_str(self, _: &str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit(self) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
    };
}

/// Read a column name from an R STRSXP names vector at index `i`.
///
/// # Safety
/// `names_sexp` must be a valid STRSXP and `i` must be in bounds.
unsafe fn col_name(names_sexp: SEXP, i: isize) -> &'static str {
    unsafe {
        let s = names_sexp.string_elt(i);
        let p = s.r_char();
        std::ffi::CStr::from_ptr(p).to_str().unwrap_or("")
    }
}

/// Copy class and row.names attributes from one data.frame SEXP to another.
///
/// # Safety
/// Both SEXPs must be valid VECSXPs.
unsafe fn copy_df_attrs(from: SEXP, to: SEXP) {
    to.set_class(from.get_class());
    to.set_row_names(from.get_row_names());
}

/// A data.frame produced by the columnar serializer.
///
/// Supports post-assembly customization via builder-style methods:
///
/// ```ignore
/// ColumnarDataFrame::from_rows(&rows)?
///     .rename("hashes_blake3", "hash")
///     .with_column("status", status_sexp)
///     .drop("internal_id")
/// ```
pub struct ColumnarDataFrame {
    sexp: SEXP,
}

impl ColumnarDataFrame {
    /// Convert a slice of serializable structs to an R data.frame in columnar layout.
    ///
    /// Each field of `T` becomes a column (R atomic vector). Nested structs are
    /// recursively flattened into prefixed columns (`parent_child` naming).
    ///
    /// The result supports post-assembly customization:
    ///
    /// ```ignore
    /// ColumnarDataFrame::from_rows(&rows)?
    ///     .rename("hashes_blake3", "hash")
    ///     .with_column("status", status_sexp)
    ///     .drop("internal_id")
    /// ```
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
    /// | `Option<T>` (every row `None`) | `logical` NA column — R coerces to the surrounding type on first use (`c(NA, 1L)` → integer, `c(NA, "x")` → character) |
    /// | Nested struct | Recursively flattened with `parent_child` naming |
    /// | Other | Falls back to per-element list column |
    pub fn from_rows<T: Serialize>(rows: &[T]) -> Result<ColumnarDataFrame, RSerdeError> {
        if rows.is_empty() {
            return Ok(ColumnarDataFrame {
                sexp: empty_dataframe(),
            });
        }

        // Phase 1: Discover schema from ALL rows (union of fields across enum variants)
        let schema = discover_schema_union(rows)?;
        let ncol = schema.fields.len();
        let nrow = rows.len();

        if ncol == 0 {
            return Err(RSerdeError::Message(
                "ColumnarDataFrame::from_rows: type has no fields".into(),
            ));
        }

        // Phase 2: Allocate column buffers
        let mut columns: Vec<ColumnBuffer> = schema
            .fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, nrow))
            .collect();

        // Phase 3: Fill columns from all rows
        let mut filled = vec![false; ncol];
        for row in rows {
            let filler = ColumnFiller {
                columns: &mut columns,
                field_map: &schema.field_map,
                filled: &mut filled,
                col_start: 0,
                col_count: ncol,
                is_top_level: true,
                pending_key: None,
            };
            row.serialize(filler)?;
        }

        // Phase 4: Assemble data.frame
        Ok(ColumnarDataFrame {
            sexp: unsafe { assemble_dataframe(&schema.fields, &columns, nrow) },
        })
    }

    /// Rename a column. No-op if `from` doesn't match any column name.
    pub fn rename(self, from: &str, to: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = crate::ffi::Rf_xlength(names_sexp);
            for i in 0..ncol {
                if col_name(names_sexp, i) == from {
                    names_sexp.set_string_elt(i, SEXP::charsxp(to));
                    break;
                }
            }
        }
        self
    }

    /// Strip a prefix from all column names that start with it.
    /// E.g., `.strip_prefix("metadata_")` turns `metadata_size` into `size`.
    pub fn strip_prefix(self, prefix: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = crate::ffi::Rf_xlength(names_sexp);
            for i in 0..ncol {
                let name = col_name(names_sexp, i);
                if let Some(stripped) = name.strip_prefix(prefix) {
                    names_sexp.set_string_elt(i, SEXP::charsxp(stripped));
                }
            }
        }
        self
    }

    /// Remove a column by name. No-op if the column doesn't exist.
    pub fn drop(self, col: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = crate::ffi::Rf_xlength(names_sexp);
            let drop_idx = (0..ncol).find(|&i| col_name(names_sexp, i) == col);
            let Some(drop_idx) = drop_idx else {
                return self;
            };

            let new_ncol = ncol - 1;
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            let mut j: isize = 0;
            for i in 0..ncol {
                if i == drop_idx {
                    continue;
                }
                new_list.set_vector_elt(j, self.sexp.vector_elt(i));
                new_names.set_string_elt(j, names_sexp.string_elt(i));
                j += 1;
            }

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }

    /// Keep only the named columns, in the order given. Unknown names are skipped.
    pub fn select(self, cols: &[&str]) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = crate::ffi::Rf_xlength(names_sexp);

            let indices: Vec<isize> = cols
                .iter()
                .filter_map(|&want| (0..ncol).find(|&i| col_name(names_sexp, i) == want))
                .collect();

            let new_ncol: isize = indices.len().try_into().expect("ncol overflow");
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            for (j, &src_idx) in indices.iter().enumerate() {
                let j_r: isize = j.try_into().expect("index overflow");
                new_list.set_vector_elt(j_r, self.sexp.vector_elt(src_idx));
                new_names.set_string_elt(j_r, names_sexp.string_elt(src_idx));
            }

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }

    /// Upsert a column: replace the column named `name` with `column` if it
    /// already exists, otherwise append `column` at the end. Caller is
    /// responsible for matching row length and for ensuring `column` is a
    /// valid R vector; miniextendr does not validate.
    pub fn with_column(self, name: &str, column: SEXP) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();
            for i in 0..ncol {
                if col_name(names_sexp, i) == name {
                    self.sexp.set_vector_elt(i, column);
                    return self;
                }
            }

            // Not found — append at the end. Reallocate the list and names,
            // copy over existing entries, add the new column.
            let new_ncol = ncol + 1;
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            for i in 0..ncol {
                new_list.set_vector_elt(i, self.sexp.vector_elt(i));
                new_names.set_string_elt(i, names_sexp.string_elt(i));
            }
            new_list.set_vector_elt(ncol, column);
            new_names.set_string_elt(ncol, SEXP::charsxp(name));

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }
}

impl crate::IntoR for ColumnarDataFrame {
    type Error = std::convert::Infallible;

    fn into_sexp(self) -> SEXP {
        self.sexp
    }

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }
}

/// Convert a row-oriented `DataFrame<T>` into a `ColumnarDataFrame` for
/// post-assembly customization (rename, drop, select).
impl<T: crate::list::IntoList> From<crate::convert::DataFrame<T>> for ColumnarDataFrame {
    fn from(df: crate::convert::DataFrame<T>) -> Self {
        use crate::IntoR;
        use crate::convert::IntoDataFrame;
        ColumnarDataFrame {
            sexp: df.into_data_frame().into_sexp(),
        }
    }
}

impl crate::from_r::TryFromSexp for ColumnarDataFrame {
    type Error = crate::from_r::SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Validate it's a data.frame before wrapping
        crate::dataframe::DataFrameView::from_sexp(sexp)
            .map(ColumnarDataFrame::from)
            .map_err(|e| crate::from_r::SexpError::InvalidValue(e.to_string()))
    }
}

/// Convert a `DataFrameView` (received from R) into a `ColumnarDataFrame`
/// for post-hoc customization (rename, drop, select).
impl From<crate::dataframe::DataFrameView> for ColumnarDataFrame {
    fn from(view: crate::dataframe::DataFrameView) -> Self {
        use crate::IntoR;
        ColumnarDataFrame {
            sexp: view.into_sexp(),
        }
    }
}

/// Convenience alias for [`ColumnarDataFrame::from_rows`].
#[inline]
pub fn vec_to_dataframe<T: Serialize>(rows: &[T]) -> Result<ColumnarDataFrame, RSerdeError> {
    ColumnarDataFrame::from_rows(rows)
}

// region: Field mapping (recursive name → column routing)

/// Maps a field name to its column location in the flat column array.
enum FieldMapping {
    /// Scalar field: writes directly to one column.
    Scalar { col_idx: usize },
    /// Compound field (flattened nested struct): spans multiple columns.
    Compound {
        col_start: usize,
        col_count: usize,
        sub_fields: FieldMap,
    },
}

/// Name-to-column mapping for one level of struct fields.
struct FieldMap {
    map: HashMap<String, FieldMapping>,
    col_start: usize,
    total_cols: usize,
}

// endregion

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
    field_map: FieldMap,
}

/// A candidate mapping for a single key, extracted from one probe row.
enum Candidate {
    Scalar(ColumnType),
    Compound {
        fields: Vec<FieldInfo>,
        sub_map: FieldMap,
    },
}

/// Resolve a slice of candidates for one key into the best single candidate.
///
/// Lattice (highest wins):
/// - `Compound` beats everything (has concrete shape).
/// - `Scalar(non-Generic)` beats `Scalar(Generic)`.
/// - `Scalar(Generic)` is the bottom (None-only probes land here).
///
/// Two `Scalar(non-Generic)` of different types: keep the first seen (no widening).
/// Two `Compound` of different shapes: keep the first seen (recursive union is out of scope).
fn resolve_candidates(candidates: &mut Vec<Candidate>) -> Candidate {
    // Walk candidates and pick the best.
    // We need to own the winner, so find its index then swap-remove.
    let mut best_idx = 0;
    for (i, c) in candidates.iter().enumerate() {
        match (&candidates[best_idx], c) {
            // Compound is always at least as good as what we have.
            (_, Candidate::Compound { .. }) => {
                best_idx = i;
                break; // Compound is the top of the lattice — no need to look further.
            }
            // Scalar(non-Generic) beats Scalar(Generic).
            (Candidate::Scalar(ColumnType::Generic), Candidate::Scalar(t))
                if *t != ColumnType::Generic =>
            {
                best_idx = i;
            }
            _ => {}
        }
    }
    candidates.swap_remove(best_idx)
}

/// Discover schema by probing all rows and taking the union of field sets.
///
/// Two limitations vs. a fully-typed schema:
///
/// - **Truly-all-None nested Option<Struct>**: when every row has `None` for an
///   `Option<UserStruct>`, the probe never sees the inner struct's fields. The key lands
///   as `Scalar(Generic)` (no Compound ever seen), which the assembly-time all-None
///   downgrade converts to a single logical-NA column. Structurally unfixable without a
///   type-level hint on stable Rust.
///
/// - **Compound-vs-Compound recursive union**: when two rows produce different Compound
///   shapes for the same key (e.g., enum variants with different nested structs), the
///   first Compound wins and the second is silently discarded. Recursive union is tracked
///   as a separate follow-up.
///
/// Different rows may serialize different fields (e.g., enum variants).
/// The unified schema contains every field seen in any row. During filling,
/// fields absent from a particular row get NA via the padding mechanism.
fn discover_schema_union<T: Serialize>(rows: &[T]) -> Result<Schema, RSerdeError> {
    // Phase A — probe every row, accumulate per-key candidates.
    let mut key_order: Vec<String> = Vec::new();
    let mut per_key_candidates: HashMap<String, Vec<Candidate>> = HashMap::new();

    for row in rows {
        let mut discoverer = SchemaDiscoverer::new(0);
        if row.serialize(&mut discoverer).is_err() {
            continue; // skip rows that fail discovery (e.g., top-level None)
        }

        for key in &discoverer.key_order {
            // Register key order on first appearance.
            if !per_key_candidates.contains_key(key) {
                key_order.push(key.clone());
                per_key_candidates.insert(key.clone(), Vec::new());
            }

            let Some(mapping) = discoverer.mappings.remove(key) else {
                continue;
            };

            let candidate = match mapping {
                FieldMapping::Scalar { col_idx } => {
                    Candidate::Scalar(discoverer.fields[col_idx].col_type)
                }
                FieldMapping::Compound {
                    col_start,
                    col_count,
                    sub_fields,
                } => {
                    let fields: Vec<FieldInfo> = (col_start..col_start + col_count)
                        .map(|i| FieldInfo {
                            name: discoverer.fields[i].name.clone(),
                            col_type: discoverer.fields[i].col_type,
                        })
                        .collect();
                    Candidate::Compound {
                        fields,
                        sub_map: sub_fields,
                    }
                }
            };
            per_key_candidates.get_mut(key).unwrap().push(candidate);
        }
    }

    // Phase B — resolve each key into a single best candidate, build unified schema.
    let mut unified_fields: Vec<FieldInfo> = Vec::new();
    let mut unified_mappings: HashMap<String, FieldMapping> = HashMap::new();

    for key in &key_order {
        let candidates = per_key_candidates.get_mut(key).unwrap();
        if candidates.is_empty() {
            continue;
        }
        let new_start = unified_fields.len();
        match resolve_candidates(candidates) {
            Candidate::Scalar(col_type) => {
                unified_fields.push(FieldInfo {
                    name: key.clone(),
                    col_type,
                });
                unified_mappings.insert(key.clone(), FieldMapping::Scalar { col_idx: new_start });
            }
            Candidate::Compound { fields, sub_map } => {
                let col_count = fields.len();
                // sub_map indices were relative to col_offset=0 in the per-row probe;
                // remap them to the actual position in the unified layout.
                let old_base = sub_map.col_start;
                for field in fields {
                    unified_fields.push(field);
                }
                let remapped = remap_field_map(sub_map, old_base, new_start);
                unified_mappings.insert(
                    key.clone(),
                    FieldMapping::Compound {
                        col_start: new_start,
                        col_count,
                        sub_fields: remapped,
                    },
                );
            }
        }
    }

    if unified_fields.is_empty() {
        return Err(RSerdeError::Message(
            "ColumnarDataFrame::from_rows: no fields discovered from any row".into(),
        ));
    }

    let total = unified_fields.len();
    Ok(Schema {
        fields: unified_fields,
        field_map: FieldMap {
            map: unified_mappings,
            col_start: 0,
            total_cols: total,
        },
    })
}

/// Remap all column indices in a FieldMap from old base to new base.
fn remap_field_map(old: FieldMap, old_base: usize, new_base: usize) -> FieldMap {
    FieldMap {
        map: old
            .map
            .into_iter()
            .map(|(k, v)| (k, remap_field_mapping(v, old_base, new_base)))
            .collect(),
        col_start: new_base,
        total_cols: old.total_cols,
    }
}

fn remap_field_mapping(m: FieldMapping, old_base: usize, new_base: usize) -> FieldMapping {
    match m {
        FieldMapping::Scalar { col_idx } => FieldMapping::Scalar {
            col_idx: col_idx - old_base + new_base,
        },
        FieldMapping::Compound {
            col_start,
            col_count,
            sub_fields,
        } => {
            let new_col_start = col_start - old_base + new_base;
            FieldMapping::Compound {
                col_start: new_col_start,
                col_count,
                sub_fields: remap_field_map(sub_fields, col_start, new_col_start),
            }
        }
    }
}

/// Try to recursively discover and flatten a nested struct value.
/// Returns (flat_fields, field_map) if the value serializes as a struct.
fn try_discover_nested<T: ?Sized + Serialize>(
    value: &T,
    col_offset: usize,
) -> Option<(Vec<FieldInfo>, FieldMap)> {
    // Use single-row discovery for nested values (they're not enums at this level)
    let mut discoverer = SchemaDiscoverer::new(col_offset);
    if value.serialize(&mut discoverer).is_ok() && !discoverer.fields.is_empty() {
        let total = discoverer.fields.len();
        Some((
            discoverer.fields,
            FieldMap {
                map: discoverer.mappings,
                col_start: col_offset,
                total_cols: total,
            },
        ))
    } else {
        None
    }
}

struct SchemaDiscoverer {
    fields: Vec<FieldInfo>,
    mappings: HashMap<String, FieldMapping>,
    key_order: Vec<String>,
    col_offset: usize,
}

impl SchemaDiscoverer {
    fn new(col_offset: usize) -> Self {
        Self {
            fields: Vec::new(),
            mappings: HashMap::new(),
            key_order: Vec::new(),
            col_offset,
        }
    }

    /// Process a field: try to flatten as nested struct, else probe as scalar.
    fn process_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.key_order.push(key.to_string());
        let abs_col = self.col_offset + self.fields.len();
        if let Some((sub_fields, sub_map)) = try_discover_nested(value, abs_col) {
            let count = sub_fields.len();
            for mut field in sub_fields {
                field.name = format!("{key}_{}", field.name);
                self.fields.push(field);
            }
            self.mappings.insert(
                key.to_string(),
                FieldMapping::Compound {
                    col_start: abs_col,
                    col_count: count,
                    sub_fields: sub_map,
                },
            );
        } else {
            let mut type_probe = TypeProbe {
                col_type: ColumnType::Generic,
            };
            let _ = value.serialize(&mut type_probe);
            self.fields.push(FieldInfo {
                name: key.to_string(),
                col_type: type_probe.col_type,
            });
            self.mappings
                .insert(key.to_string(), FieldMapping::Scalar { col_idx: abs_col });
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut SchemaDiscoverer {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = SchemaMapDiscoverer<'a>;
    type SerializeStruct = SchemaStructDiscoverer<'a>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SchemaStructDiscoverer { parent: self })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(SchemaMapDiscoverer {
            parent: self,
            pending_key: None,
        })
    }

    reject_non_struct!(
        "ColumnarDataFrame::from_rows: expected struct",
        allow_some_none
    );
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
        self.parent.process_field(key, value)
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

/// Map-based schema discoverer for structs using `#[serde(flatten)]`.
struct SchemaMapDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
    pending_key: Option<String>,
}

impl ser::SerializeMap for SchemaMapDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.parent.process_field(&key, value)
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

    fn push_na(&mut self) {
        match self {
            ColumnBuffer::Logical(v) => v.push(i32::MIN),
            ColumnBuffer::Integer(v) => v.push(i32::MIN),
            ColumnBuffer::Real(v) => v.push(NA_REAL),
            ColumnBuffer::Character(v) => v.push(None),
            ColumnBuffer::Generic(v) => v.push(None),
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
                    ExtractedValue::Int(i) => f64::from(i),
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
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
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
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), RSerdeError> {
        if let Ok(i) = i32::try_from(v) {
            self.value = ExtractedValue::Int(i);
        } else {
            self.value = ExtractedValue::Real(f64::from(v));
        }
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(f64::from(v));
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

// region: Column filling (name-based, handles skip_serializing_if)

/// Column filler. Dispatches each field by name to the correct column(s).
///
/// When `is_top_level` is true, this is the top-level row filler: `pad_unfilled`
/// resets filled flags for the next row. When false, this is a sub-filler for
/// nested struct fields: `pad_unfilled` marks columns as filled (the top-level
/// filler handles the reset), and `serialize_some`/`serialize_none` support
/// Option-wrapped nested structs with NA-fill logic.
struct ColumnFiller<'a> {
    columns: &'a mut [ColumnBuffer],
    field_map: &'a FieldMap,
    filled: &'a mut Vec<bool>,
    col_start: usize,
    col_count: usize,
    is_top_level: bool,
    pending_key: Option<String>,
}

impl ColumnFiller<'_> {
    fn fill_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        match self.field_map.map.get(key) {
            Some(FieldMapping::Scalar { col_idx }) => {
                self.columns[*col_idx].push_value(value)?;
                self.filled[*col_idx] = true;
            }
            Some(FieldMapping::Compound {
                sub_fields,
                col_start,
                col_count,
                ..
            }) => {
                let sub = ColumnFiller {
                    columns: self.columns,
                    field_map: sub_fields,
                    filled: self.filled,
                    col_start: *col_start,
                    col_count: *col_count,
                    is_top_level: false,
                    pending_key: None,
                };
                value.serialize(sub)?;
            }
            None => {
                // Field not in schema — ignore (may happen with dynamic types)
            }
        }
        Ok(())
    }

    fn pad_unfilled(&mut self) {
        let start = self.field_map.col_start;
        let end = start + self.field_map.total_cols;
        if self.is_top_level {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                self.filled[i] = false; // reset for next row
            }
        } else {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                // Don't reset — the top-level filler handles reset
                self.filled[i] = true;
            }
        }
    }
}

impl<'a> ser::Serializer for ColumnFiller<'a> {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(self)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        // Fill all columns owned by this sub-filler with NA
        for i in self.col_start..self.col_start + self.col_count {
            self.columns[i].push_na();
            self.filled[i] = true;
        }
        Ok(())
    }

    reject_non_struct!(@primitives "expected struct");
}

impl ser::SerializeStruct for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fill_field(key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}

impl ser::SerializeMap for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.fill_field(&key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
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
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, 0)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = 0;
        list.set_row_names(row_names);

        Rf_unprotect(2);
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
    let ncol: isize = fields.len().try_into().expect("ncol exceeds isize::MAX");

    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, ncol);
        Rf_protect(list);

        // Build each column and set into list
        for (i, col) in columns.iter().enumerate() {
            let idx: isize = i.try_into().expect("column index exceeds isize::MAX");
            let col_sexp = column_to_sexp(col, nrow);
            Rf_protect(col_sexp);
            list.set_vector_elt(idx, col_sexp);
            Rf_unprotect(1); // col_sexp is now held by list
        }

        // Set names
        let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, ncol);
        Rf_protect(names_sexp);
        for (i, field) in fields.iter().enumerate() {
            let idx: isize = i.try_into().expect("field index exceeds isize::MAX");
            names_sexp.set_string_elt(idx, SEXP::charsxp(&field.name));
        }
        list.set_names(names_sexp);

        // Set class = "data.frame"
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, -nrow)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = -i32::try_from(nrow).expect("data.frame row count exceeds i32::MAX");
        list.set_row_names(row_names);

        Rf_unprotect(3); // list, names, row_names
        list
    }
}

/// Convert a single column buffer into an R SEXP vector.
unsafe fn column_to_sexp(col: &ColumnBuffer, nrow: usize) -> SEXP {
    use crate::into_r::alloc_r_vector;

    unsafe {
        match col {
            ColumnBuffer::Logical(v) => {
                let (sexp, dst) = alloc_r_vector::<crate::ffi::RLogical>(nrow);
                let dst_i32: &mut [i32] =
                    std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                dst_i32.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Integer(v) => {
                let (sexp, dst) = alloc_r_vector::<i32>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Real(v) => {
                let (sexp, dst) = alloc_r_vector::<f64>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Character(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                let sexp = Rf_allocVector(SEXPTYPE::STRSXP, nrow_r);
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    match val {
                        Some(s) => {
                            sexp.set_string_elt(idx, SEXP::charsxp(s));
                        }
                        None => {
                            sexp.set_string_elt(idx, SEXP::na_string());
                        }
                    }
                }
                sexp
            }
            ColumnBuffer::Generic(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                // If every entry is None or Some(NULL) — meaning all rows had
                // `Option<T> = None` (which serializes as NULL) or the column
                // was always NA-padded — emit a logical NA vector instead of
                // list(NULL, …).  R coerces logical NA to the surrounding type
                // on first use, so this is invisible downstream:
                //   c(NA, 1L) → integer,  c(NA, "x") → character, etc.
                //
                // `push_na` (pad for missing rows) stores `None`.
                // `push_value(&None::<T>)` stores `Some(SEXP::nil())` via
                // RSerializer::serialize_none.
                // Both are "NA-like" in the generic-list context.
                let all_null = v.iter().all(|e| match e {
                    None => true,
                    Some(s) => s.is_nil(),
                });
                if all_null {
                    let (sexp, dst) = alloc_r_vector::<crate::ffi::RLogical>(nrow);
                    let dst_i32: &mut [i32] =
                        std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                    dst_i32.fill(NA_LOGICAL);
                    return sexp;
                }
                let sexp = Rf_allocVector(SEXPTYPE::VECSXP, nrow_r);
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    if let Some(elem) = val {
                        sexp.set_vector_elt(idx, *elem);
                    } else {
                        sexp.set_vector_elt(idx, SEXP::nil());
                    }
                }
                sexp
            }
        }
    }
}
// endregion

// region: Enum split (vec_to_dataframe_split)

struct VariantInfo {
    name: String,
    is_unit: bool,
    tag_field: Option<String>,
}

fn extract_variant_info<T: Serialize>(row: &T) -> Option<VariantInfo> {
    let mut ext = VariantNameExtractor::default();
    let _ = row.serialize(&mut ext);
    ext.name.map(|name| VariantInfo {
        name,
        is_unit: ext.is_unit,
        tag_field: ext.tag_field,
    })
}

// ── VariantNameExtractor ──────────────────────────────────────────────────────

#[derive(Default)]
struct VariantNameExtractor {
    name: Option<String>,
    is_unit: bool,
    tag_field: Option<String>,
}

struct NoopStructVariant;
impl ser::SerializeStructVariant for NoopStructVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct NoopTupleVariant;
impl ser::SerializeTupleVariant for NoopTupleVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct TagStructCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    first_done: bool,
}

impl ser::SerializeStruct for TagStructCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key.to_string());
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

// Defends against custom `Serialize` impls that emit internally-tagged enums via
// `serialize_map` rather than `serialize_struct`. `#[derive(Serialize)]` always
// uses `serialize_struct` for internally-tagged enums, so this path doesn't fire
// for derive-generated impls — but hand-written serializers may use a map.
struct TagMapCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    pending_key: Option<String>,
    first_done: bool,
}

impl ser::SerializeMap for TagMapCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            let mut ve = ValueExtractor::default();
            let _ = key.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.pending_key = Some(s);
            }
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let key = self.pending_key.take().unwrap_or_default();
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key);
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut VariantNameExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = NoopTupleVariant;
    type SerializeMap = TagMapCapture<'a>;
    type SerializeStruct = TagStructCapture<'a>;
    type SerializeStructVariant = NoopStructVariant;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        self.is_unit = true;
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
        variant: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message("seq in variant extractor".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message("tuple in variant extractor".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "tuple_struct in variant extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopTupleVariant)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(TagMapCapture {
            parent: self,
            pending_key: None,
            first_done: false,
        })
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Ok(TagStructCapture {
            parent: self,
            first_done: false,
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopStructVariant)
    }
}

// ── VariantStrippingSerializer ────────────────────────────────────────────────

struct VariantPayload<T>(T);

impl<T: Serialize> Serialize for VariantPayload<T> {
    fn serialize<S: ser::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(VariantStrippingSerializer { inner: s })
    }
}

struct VariantStrippingSerializer<S: ser::Serializer> {
    inner: S,
}

struct VariantAsStruct<S: ser::SerializeStruct>(S);

impl<S: ser::SerializeStruct> ser::SerializeStructVariant for VariantAsStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

struct VariantAsTupleStruct<S: ser::SerializeTupleStruct>(S);

impl<S: ser::SerializeTupleStruct> ser::SerializeTupleVariant for VariantAsTupleStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_field(value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::Serializer> ser::Serializer for VariantStrippingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = VariantAsTupleStruct<S::SerializeTupleStruct>;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = VariantAsStruct<S::SerializeStruct>;

    fn serialize_bool(self, v: bool) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bool(v)
    }
    fn serialize_i8(self, v: i8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i64(v)
    }
    fn serialize_u8(self, v: u8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u64(v)
    }
    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f64(v)
    }
    fn serialize_char(self, v: char) -> Result<S::Ok, S::Error> {
        self.inner.serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bytes(v)
    }
    fn serialize_none(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_none()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<S::Ok, S::Error> {
        self.inner.serialize_some(v)
    }
    fn serialize_unit(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_newtype_struct(name, v)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        v.serialize(self.inner)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<S::SerializeSeq, S::Error> {
        self.inner.serialize_seq(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<S::SerializeTuple, S::Error> {
        self.inner.serialize_tuple(len)
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeTupleStruct, S::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        let ts = self.inner.serialize_tuple_struct(variant, len)?;
        Ok(VariantAsTupleStruct(ts))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<S::SerializeMap, S::Error> {
        self.inner.serialize_map(len)
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeStruct, S::Error> {
        self.inner.serialize_struct(name, len)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        let s = self.inner.serialize_struct(variant, len)?;
        Ok(VariantAsStruct(s))
    }
}

// ── 0-column data.frame for unit variants ────────────────────────────────────

fn unit_variant_dataframe(nrow: usize) -> SEXP {
    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 0);
        Rf_protect(list);
        list.set_class(crate::cached_class::data_frame_class_sexp());
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN;
        rn[1] = -i32::try_from(nrow).expect("nrow overflow");
        list.set_row_names(row_names);
        Rf_unprotect(2);
        list
    }
}

// ── vec_to_dataframe_split ────────────────────────────────────────────────────

/// Partition a slice of serializable enum rows into a named list of data.frames,
/// one per variant.
///
/// Each variant's data.frame contains only that variant's fields — no NA-filled
/// columns from other variants. For internally-tagged enums (`#[serde(tag = "...")]`),
/// the tag column is automatically dropped from each partition.
///
/// Returns:
/// - **Single variant**: the bare data.frame as a [`List`](crate::list::List)
/// - **Multiple variants**: a named `List` of per-variant data.frames, keyed by
///   the variant name as serialized by serde
/// - **Empty input**: an empty unnamed `list()` — the variant set is unknowable
///   from zero rows, so no data.frame structure can be inferred
///
/// Supports externally-tagged (default) and internally-tagged (`#[serde(tag)]`)
/// enums. Unit variants produce 0-column data.frames with the correct row count.
///
/// # Errors
///
/// Returns an error if any row serializes without a variant name (not an enum),
/// or if column building fails.
pub fn vec_to_dataframe_split<T: Serialize>(rows: &[T]) -> Result<crate::list::List, RSerdeError> {
    use crate::IntoR as _;
    use crate::OwnedProtect;
    use crate::list::List;

    if rows.is_empty() {
        return Ok(List::from_raw_pairs(Vec::<(String, SEXP)>::new()));
    }

    // Phase 1: extract variant info for each row
    let infos: Vec<VariantInfo> = {
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            match extract_variant_info(row) {
                Some(info) => out.push(info),
                None => {
                    return Err(RSerdeError::Message(
                        "vec_to_dataframe_split: row has no variant — use vec_to_dataframe for plain structs".into(),
                    ));
                }
            }
        }
        out
    };

    // Phase 2: group indices by variant name (preserve first-seen order)
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, info) in infos.iter().enumerate() {
        if let Some(grp) = groups.iter_mut().find(|(n, _)| n == &info.name) {
            grp.1.push(i);
        } else {
            groups.push((info.name.clone(), vec![i]));
        }
    }

    // Detect enum style from the first row that has a tag field
    let tag_field: Option<&str> = infos.iter().find_map(|i| i.tag_field.as_deref());

    // Phase 3: build per-partition data.frames, protecting each immediately
    let mut protected: Vec<(String, OwnedProtect)> = Vec::with_capacity(groups.len());

    for (name, indices) in &groups {
        let is_unit = infos[indices[0]].is_unit;

        let prot = if is_unit {
            unsafe { OwnedProtect::new(unit_variant_dataframe(indices.len())) }
        } else if tag_field.is_some() {
            // Internally-tagged: call from_rows directly, then drop the tag column
            let refs: Vec<&T> = indices.iter().map(|&i| &rows[i]).collect();
            let df = ColumnarDataFrame::from_rows(&refs)?;
            let df = if let Some(tf) = tag_field {
                df.drop(tf)
            } else {
                df
            };
            unsafe { OwnedProtect::new(df.into_sexp()) }
        } else {
            // Externally-tagged: wrap each row so serialize_struct_variant is
            // redirected to serialize_struct (strips the outer variant wrapper)
            let wrapped: Vec<VariantPayload<&T>> =
                indices.iter().map(|&i| VariantPayload(&rows[i])).collect();
            let df = ColumnarDataFrame::from_rows(&wrapped)?;
            unsafe { OwnedProtect::new(df.into_sexp()) }
        };

        protected.push((name.clone(), prot));
    }

    // Phase 4: assemble the result
    if protected.len() == 1 {
        let (_, prot) = protected.into_iter().next().unwrap();
        Ok(unsafe { List::from_raw(prot.get()) })
    } else {
        // All partition SEXPs are protected via `protected` throughout from_raw_pairs
        let pairs: Vec<(String, SEXP)> = protected
            .iter()
            .map(|(n, p)| (n.clone(), p.get()))
            .collect();
        Ok(List::from_raw_pairs(pairs))
    }
}

// endregion
