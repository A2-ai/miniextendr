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
//!
//! # Composite keys (`group_by_multi`)
//!
//! [`DataFrame::group_by_multi`] groups on several columns at once, keying by a
//! [`GroupKey::Tuple`] of the per-column scalar keys. Non-NA groups match
//! `split(df, interaction(col1, col2, …, drop = TRUE))` (the first column varies
//! fastest, R `interaction()`'s default) — exactly, for keys whose byte order
//! coincides with the session's collation (always true in the C locale;
//! single-case ASCII in practice — the character-key byte-order choice above
//! applies per column). Extending the single-column
//! NA convention, any tuple with an NA in *any* component forms its own trailing
//! group (first-encounter order) instead of being dropped as `interaction()` +
//! `split()` would.

use std::collections::{BTreeMap, HashMap};

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

/// The key of one group produced by [`DataFrame::group_by`] or
/// [`DataFrame::group_by_multi`].
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
    /// A composite key from [`DataFrame::group_by_multi`] — one scalar element
    /// per grouping column, in column order. Elements are always scalar
    /// ([`Str`](Self::Str)/[`Int`](Self::Int)/[`Bool`](Self::Bool)/[`Na`](Self::Na));
    /// tuples never nest (enforced by a `debug_assert!` in [`label`](Self::label)).
    Tuple(Vec<GroupKey>),
}

impl GroupKey {
    /// R-facing label for this key — suitable as a name in a result list
    /// (matches how R prints the value: `TRUE`/`FALSE`, `NA`, digits). Composite
    /// [`Tuple`](Self::Tuple) keys join their element labels with `"."`, matching
    /// R `interaction()`'s default separator.
    pub fn label(&self) -> String {
        match self {
            GroupKey::Str(s) => s.clone(),
            GroupKey::Int(i) => i.to_string(),
            GroupKey::Bool(true) => "TRUE".to_string(),
            GroupKey::Bool(false) => "FALSE".to_string(),
            GroupKey::Na => "NA".to_string(),
            GroupKey::Tuple(keys) => {
                debug_assert!(
                    keys.iter().all(|k| !matches!(k, GroupKey::Tuple(_))),
                    "tuple keys never nest — elements come from scalar columns"
                );
                keys.iter()
                    .map(GroupKey::label)
                    .collect::<Vec<_>>()
                    .join(".")
            }
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
            GroupKey::Tuple(keys) => {
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    write!(f, "{}", key)?;
                }
                Ok(())
            }
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
    /// Root `source` on R's precious list and pair it with precomputed groups.
    ///
    /// `R_PreserveObject` conses onto the precious list — an allocation that can
    /// itself GC — so PROTECT `source` across the call. Released on `Drop`.
    fn new(source: DataFrame, groups: Vec<(GroupKey, Vec<usize>)>) -> Self {
        unsafe {
            let _guard = crate::OwnedProtect::new(source.sexp);
            crate::sys::R_PreserveObject(source.sexp);
        }
        GroupedDataFrame { source, groups }
    }

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
        // rooting docs).
        Ok(GroupedDataFrame::new(*self, groups))
    }

    /// Partition this frame's rows by a composite key over several columns —
    /// the multi-column analogue of [`group_by`](Self::group_by).
    ///
    /// Each supported column contributes one scalar key per row (factor level,
    /// character, integer, or logical — same rules and errors as
    /// [`group_by`](Self::group_by)); the per-row keys are zipped into a
    /// [`GroupKey::Tuple`] in column order.
    ///
    /// # Order
    ///
    /// Non-NA groups match `split(df, interaction(col1, col2, …, drop = TRUE))`:
    /// the **first** column varies fastest (R `interaction()`'s default,
    /// `lex.order = FALSE`), each column ordered as [`group_by`](Self::group_by)
    /// would order it alone (factor level order, byte-sorted characters, numeric
    /// integers, `FALSE` then `TRUE`). For character keys the match is exact for
    /// keys whose byte order coincides with the session's collation — always
    /// true in the C locale; single-case ASCII in practice (e.g. `en_US.UTF-8`
    /// collates `a A b B` where byte order gives `A B a b`). This is inherited
    /// from [`group_by`](Self::group_by)'s byte-order choice for character keys
    /// — see the group-order note in the [module docs](self).
    ///
    /// # NA
    ///
    /// `interaction()` maps any row with an NA in *any* component to NA and
    /// `split()` drops it. This method instead keeps such rows: every distinct
    /// NA-containing tuple forms its own group, all ordered **after** the non-NA
    /// groups, in first-encounter row order. This extends the single-column
    /// NA-last convention (see the [module docs](self)).
    ///
    /// # Slice
    ///
    /// An empty slice is an error. A single-column slice delegates to
    /// [`group_by`](Self::group_by) and yields **scalar** keys (not 1-tuples), so
    /// callers never have to special-case one-element tuples.
    pub fn group_by_multi(&self, cols: &[&str]) -> Result<GroupedDataFrame, DataFrameError> {
        if cols.is_empty() {
            return Err(DataFrameError::EmptyGroupColumns);
        }
        // A single column is the scalar path — identical keys/order to group_by.
        if let [col] = cols {
            return self.group_by(col);
        }

        // One pass per column: per-row keys (build the tuples) plus the
        // column's group order (assign each non-NA key an ordinal for sorting).
        let mut per_row: Vec<Vec<GroupKey>> = Vec::with_capacity(cols.len());
        let mut ordinals: Vec<HashMap<GroupKey, usize>> = Vec::with_capacity(cols.len());
        for &col in cols {
            let column = self
                .column_raw(col)
                .ok_or_else(|| DataFrameError::NoSuchColumn(col.to_string()))?;
            per_row.push(column_keys(column, col)?);
            ordinals.push(
                column_level_order(column)
                    .into_iter()
                    .enumerate()
                    .map(|(ord, key)| (key, ord))
                    .collect(),
            );
        }

        // Bucket rows into tuple keys, preserving first-encounter order.
        let mut index: HashMap<GroupKey, usize> = HashMap::new();
        let mut buckets: Vec<(GroupKey, Vec<usize>)> = Vec::new();
        for (row, first) in per_row[0].iter().enumerate() {
            let mut elems = Vec::with_capacity(cols.len());
            elems.push(first.clone());
            for col in &per_row[1..] {
                elems.push(col[row].clone());
            }
            let key = GroupKey::Tuple(elems);
            match index.get(&key) {
                Some(&pos) => buckets[pos].1.push(row),
                None => {
                    index.insert(key.clone(), buckets.len());
                    buckets.push((key, vec![row]));
                }
            }
        }

        // Non-NA tuples order by interaction() convention (first column varies
        // fastest → last column is the most-significant sort key). NA-containing
        // tuples trail, in first-encounter order (already the bucket order).
        let is_na_tuple = |k: &GroupKey| matches!(k, GroupKey::Tuple(elems) if elems.iter().any(|e| matches!(e, GroupKey::Na)));
        let (mut non_na, na_tuples): (Vec<_>, Vec<_>) =
            buckets.into_iter().partition(|(k, _)| !is_na_tuple(k));
        non_na.sort_by_key(|(k, _)| {
            let GroupKey::Tuple(elems) = k else {
                unreachable!("group_by_multi buckets are always tuples")
            };
            let mut ord: Vec<usize> = elems
                .iter()
                .enumerate()
                .map(|(c, e)| ordinals[c][e])
                .collect();
            ord.reverse();
            ord
        });
        non_na.extend(na_tuples);

        Ok(GroupedDataFrame::new(*self, non_na))
    }
}

/// Per-row group key for one supported key column, in row order. NA cells — and
/// factor NA codes / `addNA()` levels — become [`GroupKey::Na`]. Dispatches on
/// SEXPTYPE exactly as [`DataFrame::group_by`], surfacing the same
/// unsupported-type error.
fn column_keys(column: SEXP, col: &str) -> Result<Vec<GroupKey>, DataFrameError> {
    if column.is_factor() {
        // SAFETY: factor columns are INTSXP; as_slice handles empty vectors.
        let codes: &[i32] = unsafe { column.as_slice() };
        let levels = column.get_levels();
        return Ok(codes
            .iter()
            .map(|&code| {
                if code == i32::MIN {
                    GroupKey::Na
                } else {
                    match levels.string_elt_str((code - 1) as isize) {
                        Some(label) => GroupKey::Str(label.to_string()),
                        None => GroupKey::Na, // addNA() level
                    }
                }
            })
            .collect());
    }
    match column.type_of() {
        SEXPTYPE::STRSXP => {
            let n = column.len() as isize;
            Ok((0..n)
                .map(|i| match column.string_elt_str(i) {
                    Some(s) => GroupKey::Str(s.to_string()),
                    None => GroupKey::Na,
                })
                .collect())
        }
        SEXPTYPE::INTSXP => {
            // SAFETY: INTSXP column; as_slice handles empty vectors.
            let values: &[i32] = unsafe { column.as_slice() };
            Ok(values
                .iter()
                .map(|&v| {
                    if v == i32::MIN {
                        GroupKey::Na
                    } else {
                        GroupKey::Int(v)
                    }
                })
                .collect())
        }
        SEXPTYPE::LGLSXP => {
            let n = column.len() as isize;
            Ok((0..n)
                .map(|i| match column.logical_elt(i) {
                    0 => GroupKey::Bool(false),
                    v if v == i32::MIN => GroupKey::Na,
                    _ => GroupKey::Bool(true),
                })
                .collect())
        }
        other => Err(DataFrameError::UnsupportedGroupColumn {
            column: col.to_string(),
            type_of: format!("{:?}", other),
        }),
    }
}

/// Distinct non-NA keys of one column in single-column group order (factor level
/// order incl. empty levels; byte-sorted characters; numeric integers; `FALSE`
/// then `TRUE`). NA is excluded — `interaction()` drops NA rows and NA-containing
/// tuples are ordered separately. Reuses the single-column bucketers so the
/// per-column order stays byte-identical to [`DataFrame::group_by`]. Only reached
/// for supported columns ([`column_keys`] rejects the rest first).
fn column_level_order(column: SEXP) -> Vec<GroupKey> {
    let groups = if column.is_factor() {
        factor_groups(column)
    } else {
        match column.type_of() {
            SEXPTYPE::STRSXP => character_groups(column),
            SEXPTYPE::INTSXP => integer_groups(column),
            SEXPTYPE::LGLSXP => logical_groups(column),
            _ => Vec::new(),
        }
    };
    groups
        .into_iter()
        .map(|(k, _)| k)
        .filter(|k| !matches!(k, GroupKey::Na))
        .collect()
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

    #[test]
    fn tuple_key_labels_join_with_dot() {
        let key = GroupKey::Tuple(vec![
            GroupKey::Str("a".into()),
            GroupKey::Int(2),
            GroupKey::Bool(true),
            GroupKey::Na,
        ]);
        assert_eq!(key.label(), "a.2.TRUE.NA");
        assert_eq!(key.to_string(), "a.2.TRUE.NA");
    }
}
