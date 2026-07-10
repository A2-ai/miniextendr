//! Group-level iteration over a [`DataFrame`] key column.
//!
//! Two rungs, cheapest first:
//!
//! 1. **Typed rows, grouped Rust-side** — after `Vec::<Row>::from_dataframe(&df)?`,
//!    grouping is plain Rust. [`group_rows`] makes the idiom discoverable:
//!
//!    ```ignore
//!    let rows: Vec<Obs> = Vec::from_dataframe(&df)?;
//!    let by_site = group_rows(rows, |r| r.site.clone());
//!    // by_site: BTreeMap<String, Vec<Obs>> — plain Rust data, rayon-safe.
//!    ```
//!
//! 2. **Untyped, index-based** — [`DataFrame::group_by`] computes group indices
//!    once (single pass, main thread) without extracting rows:
//!
//!    ```ignore
//!    let grouped = df.group_by("site")?;
//!    for (key, rows) in grouped.iter() { /* key: &GroupKey, rows: &[usize] */ }
//!    let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
//!    for (key, sub) in grouped.frames() {
//!        // `sub` is a rooted `BuiltDataFrame`; deref to the view for push.
//!        out = out.push(key.label(), *sub);
//!    }
//!    ```
//!
//! # Key semantics (vs R `split()`)
//!
//! - **Group order**: factor keys follow level order (empty levels kept, like
//!   `split()`); character keys sort in byte order (R sorts in locale collation
//!   order — identical for ASCII); integer keys sort numerically; logical keys
//!   order `FALSE`, `TRUE`.
//! - **`NA` keys form one group, ordered last** — a deliberate deviation from
//!   `split()`, which silently drops NA-keyed rows. A literal NA *level*
//!   (`addNA(f)`) also surfaces as [`GroupKey::Na`].
//! - **Double key columns are an error**: grouping on floating point is a
//!   footgun — `cut()` or `factor()` the column first.

use std::collections::BTreeMap;

use super::{DataFrame, DataFrameError, FromDataFrame};
use crate::{SEXP, SEXPTYPE, SexpExt};

// region: group_rows — typed-rows grouping helper (rung 1)

/// Group already-extracted rows by a key function.
///
/// Plain Rust — no SEXP contact, so the result is `Send` (given `T: Send`) and
/// safe to iterate with rayon. Keys order by `Ord`; give NA-able keys a home by
/// keying on `Option<T>` (`None` sorts first) or a custom enum.
pub fn group_rows<T, K, F>(rows: Vec<T>, key: F) -> BTreeMap<K, Vec<T>>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut groups: BTreeMap<K, Vec<T>> = BTreeMap::new();
    for row in rows {
        groups.entry(key(&row)).or_default().push(row);
    }
    groups
}
// endregion

// region: GroupKey

/// The key of one group produced by [`DataFrame::group_by`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GroupKey {
    /// A character or factor-level key.
    Str(String),
    /// An integer key.
    Int(i32),
    /// A logical key.
    Bool(bool),
    /// The NA-keyed group (always ordered last).
    Na,
}

impl GroupKey {
    /// R-facing label for this key — suitable as a name in a result list
    /// (matches how R prints the value: `TRUE`/`FALSE`, `NA`, digits).
    pub fn label(&self) -> String {
        match self {
            GroupKey::Str(s) => s.clone(),
            GroupKey::Int(i) => i.to_string(),
            GroupKey::Bool(true) => "TRUE".to_string(),
            GroupKey::Bool(false) => "FALSE".to_string(),
            GroupKey::Na => "NA".to_string(),
        }
    }
}

impl std::fmt::Display for GroupKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupKey::Str(s) => f.write_str(s),
            GroupKey::Int(i) => write!(f, "{}", i),
            GroupKey::Bool(true) => f.write_str("TRUE"),
            GroupKey::Bool(false) => f.write_str("FALSE"),
            GroupKey::Na => f.write_str("NA"),
        }
    }
}
// endregion

// region: GroupedDataFrame

/// A [`DataFrame`] partitioned by the values of one key column.
///
/// Produced by [`DataFrame::group_by`]. Holds the source frame plus one
/// `(key, row-indices)` pair per group; nothing is copied until you ask for
/// [`frames`](Self::frames) or [`extract`](Self::extract).
///
/// # GC rooting
///
/// The source frame is preserved on R's precious list
/// (`R_PreserveObject`) for this struct's lifetime and released on drop —
/// order-independent, unlike the PROTECT stack, so the struct can be held
/// across arbitrary allocations (e.g. a locally built frame from
/// [`DataFrame::builder`], which is unprotected once `build()` returns).
/// Without this, the per-group allocations in [`frames`](Self::frames) /
/// [`extract`](Self::extract) could collect the source mid-iteration.
/// Main-thread-only (holds a SEXP; `!Send`).
pub struct GroupedDataFrame {
    source: DataFrame,
    groups: Vec<(GroupKey, Vec<usize>)>,
}

impl Drop for GroupedDataFrame {
    fn drop(&mut self) {
        // SAFETY: main thread (construction invariant — SEXP is !Send);
        // releases the preserve taken in `group_by`.
        unsafe { crate::sys::R_ReleaseObject(self.source.as_sexp()) };
    }
}

impl GroupedDataFrame {
    /// Number of groups (empty factor levels included).
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Whether there are no groups.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// The frame this grouping was computed from.
    pub fn source(&self) -> &DataFrame {
        &self.source
    }

    /// Iterate `(key, row-indices)` pairs in group order. Indices are 0-based
    /// rows of [`source`](Self::source).
    pub fn iter(&self) -> impl Iterator<Item = (&GroupKey, &[usize])> {
        self.groups.iter().map(|(k, idx)| (k, idx.as_slice()))
    }

    /// Iterate `(key, sub-frame)` pairs, materialising each group as its own
    /// frame via [`DataFrame::select_rows`].
    ///
    /// Main thread only. Each yielded frame is an owned, GC-rooted
    /// [`BuiltDataFrame`](crate::dataframe::BuiltDataFrame) (#1247) — safe to
    /// hold across later iterations' allocations. Deref (`*sub`) to pass the
    /// view where a [`DataFrame`] is expected, e.g. to
    /// [`NamedDataFrameListBuilder::push`](crate::dataframe::NamedDataFrameListBuilder::push)
    /// (which protects on push, so the handle may drop right after).
    pub fn frames(&self) -> impl Iterator<Item = (&GroupKey, crate::dataframe::BuiltDataFrame)> {
        self.groups
            .iter()
            .map(|(k, idx)| (k, self.source.select_rows(idx)))
    }

    /// Extract typed rows once, then partition them by group.
    ///
    /// One `Vec::<T>::from_dataframe` pass over the whole frame, then a
    /// move-partition by the stored indices — no per-group R subsetting and no
    /// `Clone` bound. The result is plain Rust data (rayon-safe afterwards).
    pub fn extract<T>(&self) -> Result<Vec<(GroupKey, Vec<T>)>, DataFrameError>
    where
        Vec<T>: FromDataFrame,
    {
        let rows = Vec::<T>::from_dataframe(&self.source)?;
        let mut slots: Vec<Option<T>> = rows.into_iter().map(Some).collect();
        Ok(self
            .groups
            .iter()
            .map(|(key, idx)| {
                let group_rows: Vec<T> = idx
                    .iter()
                    .map(|&i| slots[i].take().expect("group indices are disjoint"))
                    .collect();
                (key.clone(), group_rows)
            })
            .collect())
    }
}
// endregion

// region: DataFrame::group_by

impl DataFrame {
    /// Partition this frame's rows by the values of the named column.
    ///
    /// Computes group indices in a single pass on the main thread. Supported
    /// key columns: factor (fast path — levels are the keys, level order kept,
    /// empty levels included), character, integer, and logical. Double columns
    /// error — `cut()` or `factor()` the column in R first.
    ///
    /// NA keys form one group, ordered last (unlike R `split()`, which drops
    /// NA-keyed rows). See the [module docs](self) for the full key semantics.
    pub fn group_by(&self, col: &str) -> Result<GroupedDataFrame, DataFrameError> {
        let column = self
            .column_raw(col)
            .ok_or_else(|| DataFrameError::NoSuchColumn(col.to_string()))?;
        let groups = if column.is_factor() {
            factor_groups(column)
        } else {
            match column.type_of() {
                SEXPTYPE::STRSXP => character_groups(column),
                SEXPTYPE::INTSXP => integer_groups(column),
                SEXPTYPE::LGLSXP => logical_groups(column),
                other => {
                    return Err(DataFrameError::UnsupportedGroupColumn {
                        column: col.to_string(),
                        type_of: format!("{:?}", other),
                    });
                }
            }
        };
        // Root the source for the GroupedDataFrame's lifetime (see its GC
        // rooting docs). R_PreserveObject conses onto the precious list —
        // an allocation that can itself GC — so PROTECT across the call.
        unsafe {
            let _guard = crate::OwnedProtect::new(self.sexp);
            crate::sys::R_PreserveObject(self.sexp);
        }
        Ok(GroupedDataFrame {
            source: *self,
            groups,
        })
    }
}

/// Factor fast path: levels are the keys (level order, empty levels kept).
/// NA codes — and a literal NA level from `addNA()` — land in [`GroupKey::Na`].
fn factor_groups(column: SEXP) -> Vec<(GroupKey, Vec<usize>)> {
    // SAFETY: factor columns are INTSXP; as_slice handles empty vectors.
    let codes: &[i32] = unsafe { column.as_slice() };
    let levels = column.get_levels();
    let n_levels: usize = if levels.is_nil() { 0 } else { levels.len() };

    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); n_levels];
    let mut na_bucket: Vec<usize> = Vec::new();
    for (row, &code) in codes.iter().enumerate() {
        if code == i32::MIN {
            na_bucket.push(row);
        } else {
            buckets[(code - 1) as usize].push(row);
        }
    }

    let mut groups: Vec<(GroupKey, Vec<usize>)> = Vec::with_capacity(n_levels + 1);
    for (lvl, bucket) in buckets.into_iter().enumerate() {
        let key = match levels.string_elt_str(lvl as isize) {
            Some(label) => GroupKey::Str(label.to_string()),
            None => GroupKey::Na, // addNA() level
        };
        groups.push((key, bucket));
    }
    if !na_bucket.is_empty() {
        groups.push((GroupKey::Na, na_bucket));
    }
    groups
}

/// Character keys: byte-order sort (BTreeMap), NA last.
fn character_groups(column: SEXP) -> Vec<(GroupKey, Vec<usize>)> {
    let n = column.len() as isize;
    let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut na_bucket: Vec<usize> = Vec::new();
    for i in 0..n {
        match column.string_elt_str(i) {
            Some(s) => map.entry(s.to_string()).or_default().push(i as usize),
            None => na_bucket.push(i as usize),
        }
    }
    let mut groups: Vec<(GroupKey, Vec<usize>)> = map
        .into_iter()
        .map(|(k, idx)| (GroupKey::Str(k), idx))
        .collect();
    if !na_bucket.is_empty() {
        groups.push((GroupKey::Na, na_bucket));
    }
    groups
}

/// Integer keys: numeric sort (BTreeMap), NA (`i32::MIN`) last.
fn integer_groups(column: SEXP) -> Vec<(GroupKey, Vec<usize>)> {
    // SAFETY: INTSXP column; as_slice handles empty vectors.
    let values: &[i32] = unsafe { column.as_slice() };
    let mut map: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
    let mut na_bucket: Vec<usize> = Vec::new();
    for (row, &v) in values.iter().enumerate() {
        if v == i32::MIN {
            na_bucket.push(row);
        } else {
            map.entry(v).or_default().push(row);
        }
    }
    let mut groups: Vec<(GroupKey, Vec<usize>)> = map
        .into_iter()
        .map(|(k, idx)| (GroupKey::Int(k), idx))
        .collect();
    if !na_bucket.is_empty() {
        groups.push((GroupKey::Na, na_bucket));
    }
    groups
}

/// Logical keys: `FALSE` then `TRUE` (R's sort order), NA last.
/// Only keys present in the data appear.
fn logical_groups(column: SEXP) -> Vec<(GroupKey, Vec<usize>)> {
    let n = column.len() as isize;
    let mut false_bucket: Vec<usize> = Vec::new();
    let mut true_bucket: Vec<usize> = Vec::new();
    let mut na_bucket: Vec<usize> = Vec::new();
    for i in 0..n {
        match column.logical_elt(i) {
            0 => false_bucket.push(i as usize),
            v if v == i32::MIN => na_bucket.push(i as usize),
            _ => true_bucket.push(i as usize),
        }
    }
    let mut groups: Vec<(GroupKey, Vec<usize>)> = Vec::with_capacity(3);
    if !false_bucket.is_empty() {
        groups.push((GroupKey::Bool(false), false_bucket));
    }
    if !true_bucket.is_empty() {
        groups.push((GroupKey::Bool(true), true_bucket));
    }
    if !na_bucket.is_empty() {
        groups.push((GroupKey::Na, na_bucket));
    }
    groups
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_rows_partitions_and_orders_by_key() {
        let rows = vec![("b", 1), ("a", 2), ("b", 3), ("c", 4)];
        let grouped = group_rows(rows, |r| r.0);
        let keys: Vec<&str> = grouped.keys().copied().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
        assert_eq!(grouped["b"], vec![("b", 1), ("b", 3)]);
    }

    #[test]
    fn group_rows_option_key_gives_na_a_home() {
        let rows = vec![(Some(2), "x"), (None, "y"), (Some(1), "z")];
        let grouped = group_rows(rows, |r| r.0);
        let keys: Vec<Option<i32>> = grouped.keys().copied().collect();
        assert_eq!(keys, vec![None, Some(1), Some(2)]);
    }

    #[test]
    fn group_key_labels_match_r_printing() {
        assert_eq!(GroupKey::Str("a".into()).label(), "a");
        assert_eq!(GroupKey::Int(-3).label(), "-3");
        assert_eq!(GroupKey::Bool(true).label(), "TRUE");
        assert_eq!(GroupKey::Bool(false).label(), "FALSE");
        assert_eq!(GroupKey::Na.label(), "NA");
        assert_eq!(GroupKey::Na.to_string(), "NA");
    }
}
