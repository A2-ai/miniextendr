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
//!   order `FALSE`, `TRUE`; double keys (`Date` and `POSIXct` included) sort
//!   numerically, with `NaN` after every number.
//! - **`NA` keys form one group, ordered last** — a deliberate deviation from
//!   `split()`, which silently drops NA-keyed rows. A literal NA *level*
//!   (`addNA(f)`) also surfaces as [`GroupKey::Na`].
//! - **Double keys group by value with dplyr's (vctrs') equality**: `-0` and
//!   `0` are one key, every `NaN` is one key, and `NaN` and `NA` are separate
//!   groups. Labels are R's `as.character()` of the key — see [`RealKey`]. A
//!   bit64 `integer64` column (int64 bits stored in a double vector) is an
//!   error.
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
use crate::from_r::is_na_real;
use crate::into_r::IntoR;
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
    /// A double key: a plain number, a `Date`, a `POSIXct`, or another classed
    /// double. See [`RealKey`] for its equality and label.
    Real(RealKey),
    /// A logical key.
    Bool(bool),
    /// The NA-keyed group (always ordered last).
    Na,
    /// A composite key from [`DataFrame::group_by_multi`] — one scalar element
    /// per grouping column, in column order. Elements are always scalar
    /// ([`Str`](Self::Str)/[`Int`](Self::Int)/[`Real`](Self::Real)/[`Bool`](Self::Bool)/[`Na`](Self::Na));
    /// tuples never nest (enforced by a `debug_assert!` in [`label`](Self::label)).
    Tuple(Vec<GroupKey>),
}

impl GroupKey {
    /// R-facing label for this key — suitable as a name in a result list
    /// (matches how R prints the value: `TRUE`/`FALSE`, `NA`, digits, and R's
    /// `as.character()` text for a [`Real`](Self::Real) key). Composite
    /// [`Tuple`](Self::Tuple) keys join their element labels with `"."`, matching
    /// R `interaction()`'s default separator.
    pub fn label(&self) -> String {
        match self {
            GroupKey::Str(s) => s.clone(),
            GroupKey::Int(i) => i.to_string(),
            GroupKey::Real(r) => r.label.clone(),
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
            GroupKey::Real(r) => f.write_str(&r.label),
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

/// The key of one double-valued group ([`GroupKey::Real`]): the value plus
/// the text R prints for it.
///
/// # Equality
///
/// Keys compare and hash by value alone, with the rules vctrs uses and
/// therefore dplyr's `group_by()`: `-0.0` equals `0.0`, and every `NaN` is the
/// same key. `NA_real_` is never a `RealKey`: an `NA` cell keys as
/// [`GroupKey::Na`], so `NA` and `NaN` rows form separate groups, as in dplyr.
/// The label takes no part in equality.
///
/// # Label
///
/// The grouping verbs label a key with R's `as.character()` of its value,
/// called with the key column's attributes (class, `tzone`, …): the same text
/// `factor()` and `split()` use for level names. So a plain double prints
/// with up to 15 significant digits (`"0.1"`, `"1e+05"`, `"NaN"`), a `Date`
/// prints as `"2024-01-02"`, and a `POSIXct` prints in the column's time zone
/// (`"2024-01-02 10:30:00"`, or just the date at midnight).
#[derive(Debug, Clone)]
pub struct RealKey {
    value: f64,
    label: String,
}

impl RealKey {
    /// A key for `value`, printed as `label`. `-0.0` is stored as `0.0` and
    /// every `NaN` as one canonical `NaN`, so equal keys hold identical bits.
    pub fn new(value: f64, label: impl Into<String>) -> Self {
        RealKey {
            value: normalize_real(value),
            label: label.into(),
        }
    }

    /// The key's value: never `-0.0`, and `NaN` is the canonical positive
    /// quiet `NaN`. For a `Date` key this is days since 1970-01-01; for a
    /// `POSIXct` key, seconds since the epoch.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The text R prints for the key (see the [type docs](Self)).
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl PartialEq for RealKey {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}

impl Eq for RealKey {}

impl std::hash::Hash for RealKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

/// Bits of the one `NaN` every `NaN` key is stored as: positive and quiet, so
/// [`f64::total_cmp`] orders it after `+Inf` on every platform (x86's `0/0` has
/// the sign bit set), and its low word is not R's NA payload (1954).
const CANONICAL_NAN_BITS: u64 = 0x7FF8_0000_0000_0000;

/// Map a non-NA double to its key representative: `-0.0` → `0.0`, any `NaN`
/// → the canonical `NaN`. Equal keys (by vctrs' rules) then share their bits.
fn normalize_real(x: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x.is_nan() {
        f64::from_bits(CANONICAL_NAN_BITS)
    } else {
        x
    }
}
// endregion

// region: GroupDeclaration

/// How a dplyr-grouped frame declares its grouping, returned by
/// [`DataFrame::group_declaration`]: the grouping variables and the `.drop`
/// policy, without the cached group rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDeclaration {
    /// The grouping variable names, in declaration order — dplyr's
    /// `group_vars()`, `setdiff(names(groups), ".rows")`: the `groups`
    /// frame's column names other than `.rows`, each once. Empty for a
    /// `rowwise_df` without id columns.
    pub vars: Vec<String>,
    /// Whether grouping drops empty groups — dplyr's
    /// `group_by_drop_default()`. For a `grouped_df`, `true` unless the
    /// `groups` frame's `.drop` attribute is exactly `FALSE`
    /// (`identical(attr(groups, ".drop"), FALSE)`: a length-1 logical `FALSE`
    /// with no attributes). Always `true` for a frame that is a `rowwise_df`
    /// but not a `grouped_df`, whose `group_by_drop_default()` is the default
    /// method's `TRUE`.
    pub drop: bool,
}

/// A grouped frame's `groups` attribute after the structural checks of
/// [`DataFrame::groups_attr`].
struct GroupsAttr {
    /// The `groups` data frame: a list inheriting from `data.frame`, with at
    /// least one column.
    frame: SEXP,
    /// Its last column, `.rows`: a list of integer vectors, one per group.
    rows: SEXP,
}

impl GroupsAttr {
    /// The grouping-variable columns, by position, with their names: every
    /// column not named `.rows`, keeping only the first column of a repeated
    /// name. Their names are dplyr's `group_vars()`,
    /// `setdiff(names(groups), ".rows")`. An `NA` name reads as `""`, as in
    /// [`DataFrame::names`].
    fn key_columns(&self) -> Vec<(String, SEXP)> {
        let names = self.frame.get_names();
        let mut columns: Vec<(String, SEXP)> = Vec::new();
        for i in 0..self.frame.len() as isize {
            let name = names.string_elt_str(i).unwrap_or("");
            if name == ".rows" || columns.iter().any(|(seen, _)| seen == name) {
                continue;
            }
            columns.push((name.to_string(), self.frame.vector_elt(i)));
        }
        columns
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
    /// empty levels included), character, integer, logical, and double
    /// (`Date` and `POSIXct` included; keyed as [`GroupKey::Real`], labelled by
    /// R's `as.character()`). List columns and bit64 `integer64` columns are an
    /// error.
    ///
    /// NA keys form one group, ordered last (unlike R `split()`, which drops
    /// NA-keyed rows). See the [module docs](self) for the full key semantics.
    pub fn group_by(&self, col: &str) -> Result<GroupedDataFrame, DataFrameError> {
        let column = self
            .column_raw(col)
            .ok_or_else(|| DataFrameError::NoSuchColumn(col.to_string()))?;
        let groups = column_groups(column, col)?;
        // Root the source for the GroupedDataFrame's lifetime (see its GC
        // rooting docs).
        Ok(GroupedDataFrame::new(*self, groups))
    }

    /// Partition this frame's rows by a composite key over several columns —
    /// the multi-column analogue of [`group_by`](Self::group_by).
    ///
    /// Each supported column contributes one scalar key per row (factor level,
    /// character, integer, logical, or double — same rules and errors as
    /// [`group_by`](Self::group_by)); the per-row keys are zipped into a
    /// [`GroupKey::Tuple`] in column order.
    ///
    /// # Order
    ///
    /// Non-NA groups match `split(df, interaction(col1, col2, …, drop = TRUE))`:
    /// the **first** column varies fastest (R `interaction()`'s default,
    /// `lex.order = FALSE`), each column ordered as [`group_by`](Self::group_by)
    /// would order it alone (factor level order, byte-sorted characters, numeric
    /// integers and doubles — `NaN` after every number — and `FALSE` then
    /// `TRUE`). For character keys the match is exact for
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

        // One grouping pass per column gives both the per-row keys (to build
        // the tuples) and the column's group order (an ordinal per non-NA key,
        // for sorting). NA is excluded from the ordinals: NA-containing tuples
        // are ordered separately below.
        let mut per_row: Vec<Vec<GroupKey>> = Vec::with_capacity(cols.len());
        let mut ordinals: Vec<HashMap<GroupKey, usize>> = Vec::with_capacity(cols.len());
        for &col in cols {
            let column = self
                .column_raw(col)
                .ok_or_else(|| DataFrameError::NoSuchColumn(col.to_string()))?;
            let groups = column_groups(column, col)?;
            per_row.push(row_keys(&groups, column.len()));
            ordinals.push(
                groups
                    .into_iter()
                    .map(|(key, _)| key)
                    .filter(|key| !matches!(key, GroupKey::Na))
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

    /// Read a dplyr-grouped frame's grouping **declaration** — its grouping
    /// variables and `.drop` policy — without its cached group rows.
    ///
    /// Returns what dplyr's `group_vars()` and `group_by_drop_default()`
    /// report: `Ok(None)` when the frame is not grouped, and an error when it
    /// is grouped but its `groups` attribute is corrupt, where `group_vars()`
    /// errors too (see below). Nothing is partitioned and no row index is
    /// read, so the key-column types and whether the cached `.rows` still fit
    /// the frame never matter. A function that groups the frame's **current**
    /// rows itself (e.g. with [`group_by_multi`](Self::group_by_multi) over
    /// [`GroupDeclaration::vars`]) reads this instead of
    /// [`group_by_metadata`](Self::group_by_metadata), which trusts the cached
    /// rows.
    ///
    /// # Grouped frames
    ///
    /// A frame is grouped when it inherits from `grouped_df` or `rowwise_df`,
    /// whatever its attributes: `dplyr::is_grouped_df()` is
    /// `inherits(x, "grouped_df")`. A plain `data.frame` with a stray `groups`
    /// attribute is not grouped (`Ok(None)`).
    ///
    /// # Errors
    ///
    /// A grouped frame's `groups` attribute must pass the structural checks
    /// of dplyr's `validate_grouped_df()`, which `group_vars()` runs on a
    /// `grouped_df`: it is a data frame ([`GroupsNotDataFrame`]), its last
    /// column is called `.rows` ([`MissingGroupRows`]), and that column is a
    /// list of integer vectors ([`BadGroupRows`]; a list of doubles is
    /// rejected, as in dplyr). The row indices themselves are not checked,
    /// as `group_vars()` does not check them either, so stale `.rows` after a
    /// subset still yield the declaration.
    ///
    /// A `rowwise_df` gets the same checks. dplyr is looser here:
    /// `group_vars()` reads a `rowwise_df`'s `groups` attribute without
    /// validating it. `validate_rowwise_df()` applies the same checks, plus
    /// row-shape checks that this method skips (one `.rows` element per row,
    /// each holding its own row number).
    ///
    /// [`GroupsNotDataFrame`]: DataFrameError::GroupsNotDataFrame
    /// [`MissingGroupRows`]: DataFrameError::MissingGroupRows
    /// [`BadGroupRows`]: DataFrameError::BadGroupRows
    ///
    /// # Example
    ///
    /// ```ignore
    /// // df arrived as dplyr::group_by(data, site, .drop = FALSE)
    /// let decl = df.group_declaration()?.expect("a grouped_df");
    /// assert_eq!(decl.vars, ["site"]);
    /// assert!(!decl.drop);
    /// ```
    pub fn group_declaration(&self) -> Result<Option<GroupDeclaration>, DataFrameError> {
        let Some(groups) = self.groups_attr()? else {
            return Ok(None);
        };
        // group_by_drop_default(): only its grouped_df method reads `.drop`,
        // as `!identical(attr(groups, ".drop"), FALSE)` with identical()'s
        // default flags. A frame that is only a rowwise_df gets the default
        // method's TRUE. `Rf_ScalarLogical(0)` returns R's shared FALSE
        // without allocating.
        let drop = !self.sexp.inherits_class(c"grouped_df") || {
            let drop_attr = groups
                .frame
                .get_attr(unsafe { crate::sys::Rf_install(c".drop".as_ptr()) });
            unsafe {
                crate::sys::R_compute_identical(
                    drop_attr,
                    crate::sys::Rf_ScalarLogical(0),
                    crate::sys::IDENT_USE_CLOENV,
                ) == crate::sexp_types::Rboolean::FALSE
            }
        };
        Ok(Some(GroupDeclaration {
            vars: groups
                .key_columns()
                .into_iter()
                .map(|(name, _)| name)
                .collect(),
            drop,
        }))
    }

    /// Ingest a dplyr `grouped_df`'s existing grouping from its `groups`
    /// attribute — **honoring the caller's grouping without recomputing it**.
    ///
    /// dplyr stores a `grouped_df`'s grouping in `attr(df, "groups")`: a
    /// `data.frame` whose leading columns are the group-key columns (one row
    /// per group, in dplyr's group order) and whose trailing `.rows`
    /// list-column holds, per group, the 1-based row indices into `df`. This
    /// method reads that metadata verbatim into a [`GroupedDataFrame`] — the
    /// same type [`group_by`](Self::group_by) / [`group_by_multi`](Self::group_by_multi)
    /// produce — so a `#[miniextendr]` function handed a dplyr-grouped frame can
    /// respect the caller's grouping, including multi-column groupings.
    ///
    /// Unlike [`group_by`](Self::group_by), this does **no** recomputation: a
    /// frame that is not grouped (see [`group_declaration`](Self::group_declaration)
    /// for the test) is an error ([`NotGroupedDataFrame`]). Callers who want the
    /// framework to compute grouping should use [`group_by`](Self::group_by) /
    /// [`group_by_multi`](Self::group_by_multi).
    ///
    /// # Keys
    ///
    /// A single key column yields scalar [`GroupKey`]s; multiple key columns
    /// yield [`GroupKey::Tuple`]s (labels `.`-joined), consistent with
    /// [`group_by_multi`](Self::group_by_multi). Supported key-column types are
    /// the same as [`group_by`](Self::group_by) (factor, character, integer,
    /// logical, double — `Date` and `POSIXct` included); a list / `integer64`
    /// key column is an error.
    ///
    /// # Order & empty groups
    ///
    /// The `groups`-frame row order is preserved verbatim — dplyr's order is
    /// authoritative, with no re-sorting and no NA reordering. `.drop = FALSE`
    /// empty groups (zero-length `.rows`) are **kept** as groups with empty
    /// index vectors, mirroring the empty-factor-level convention of
    /// [`group_by`](Self::group_by).
    ///
    /// # Trusts the cached rows
    ///
    /// The `.rows` are dplyr's cache of the grouping, and this method uses them
    /// as they are. It checks that they still fit the frame — every index in
    /// `1..=nrow`, every row in exactly one group — and reports stale metadata
    /// otherwise ([`GroupIndexOutOfRange`], [`GroupRowUncovered`],
    /// [`GroupRowDuplicated`]). The usual cause is subsetting a grouped frame
    /// without dplyr loaded: `[` then keeps the old `groups` attribute. It does
    /// **not** compare the rows' key values with their group's key, so a pure
    /// reorder done the same way (`df[c(3, 4, 1, 2), ]`) still fits and yields
    /// the old row positions (#1687). To group the current rows instead, read
    /// the declaration with [`group_declaration`](Self::group_declaration) and
    /// group by its variables.
    ///
    /// # Errors
    ///
    /// [`NotGroupedDataFrame`] (not grouped); for a corrupt `groups`
    /// attribute, the errors [`group_declaration`](Self::group_declaration)
    /// reports ([`GroupsNotDataFrame`], [`MissingGroupRows`],
    /// [`BadGroupRows`]: `.rows` must be a list of integer vectors, so a
    /// list of doubles is rejected, as in dplyr); [`UnequalLengths`] (a key
    /// column's length differs from that of `.rows`);
    /// [`UnsupportedGroupColumn`] (a key column of an unsupported type); or
    /// one of the stale-metadata errors above. Every `.rows` index is
    /// converted from R's 1-based to 0-based.
    ///
    /// [`NotGroupedDataFrame`]: DataFrameError::NotGroupedDataFrame
    /// [`GroupsNotDataFrame`]: DataFrameError::GroupsNotDataFrame
    /// [`MissingGroupRows`]: DataFrameError::MissingGroupRows
    /// [`BadGroupRows`]: DataFrameError::BadGroupRows
    /// [`UnequalLengths`]: DataFrameError::UnequalLengths
    /// [`UnsupportedGroupColumn`]: DataFrameError::UnsupportedGroupColumn
    /// [`GroupIndexOutOfRange`]: DataFrameError::GroupIndexOutOfRange
    /// [`GroupRowUncovered`]: DataFrameError::GroupRowUncovered
    /// [`GroupRowDuplicated`]: DataFrameError::GroupRowDuplicated
    ///
    /// # GC
    ///
    /// The `groups` attribute frame (and its columns) is only read here, during
    /// construction; it stays reachable via `self`'s attribute pairlist — which
    /// R protects as a `.Call` argument frame for the duration of the call — so
    /// it needs no separate root. Only the returned [`GroupedDataFrame`] roots
    /// the **source** frame for its own lifetime (see its GC-rooting docs).
    pub fn group_by_metadata(&self) -> Result<GroupedDataFrame, DataFrameError> {
        let attr = self
            .groups_attr()?
            .ok_or(DataFrameError::NotGroupedDataFrame)?;
        let n_groups = attr.rows.len();
        let key_columns = attr.key_columns();

        // Per-key-column keys: one Vec<GroupKey> of length n_groups per key
        // column (column_keys yields one scalar key per row of the groups frame).
        let mut per_col_keys: Vec<Vec<GroupKey>> = Vec::with_capacity(key_columns.len());
        for (name, column) in &key_columns {
            let keys = column_keys(*column, name)?;
            if keys.len() != n_groups {
                return Err(DataFrameError::UnequalLengths {
                    expected: n_groups,
                    column: name.clone(),
                    actual: keys.len(),
                });
            }
            per_col_keys.push(keys);
        }

        let nrow = self.nrow();
        let single_key = key_columns.len() == 1;

        let mut groups: Vec<(GroupKey, Vec<usize>)> = Vec::with_capacity(n_groups);
        for g in 0..n_groups {
            // Build this group's key: scalar for a single key column, otherwise
            // a Tuple in column order (`.`-joined labels, like group_by_multi).
            let key = if single_key {
                per_col_keys[0][g].clone()
            } else {
                GroupKey::Tuple(per_col_keys.iter().map(|col| col[g].clone()).collect())
            };
            // Convert `.rows[[g]]` (1-based indices) to a validated 0-based Vec.
            let elt = attr.rows.vector_elt(g as isize);
            let indices = group_rows_indices(elt, g, nrow)?;
            groups.push((key, indices));
        }
        // Every index is in range; now the rows must partition the frame.
        check_rows_partition(&groups, nrow)?;

        Ok(GroupedDataFrame::new(*self, groups))
    }

    /// The `groups` attribute of a dplyr-grouped frame, checked the way
    /// dplyr's `validate_grouped_df()` checks it (with its default
    /// `check_bounds = FALSE`).
    ///
    /// `Ok(None)` when the frame inherits from neither `grouped_df` nor
    /// `rowwise_df`. Otherwise the attribute must be a data frame with at
    /// least one column ([`DataFrameError::GroupsNotDataFrame`]), whose last
    /// column is called `.rows` ([`DataFrameError::MissingGroupRows`]) and is
    /// a list of integer vectors ([`DataFrameError::BadGroupRows`]). These are
    /// dplyr's tests: `inherits(groups, "data.frame")` with at least one
    /// column, the last name, `typeof(.rows) == "list"`, and every element's
    /// `typeof() == "integer"`. `.rows` and the key columns are read by
    /// position, so a column named `.rows` elsewhere in the frame is not
    /// mistaken for it.
    fn groups_attr(&self) -> Result<Option<GroupsAttr>, DataFrameError> {
        if !(self.sexp.inherits_class(c"grouped_df") || self.sexp.inherits_class(c"rowwise_df")) {
            return Ok(None);
        }
        let groups_sym = unsafe { crate::sys::Rf_install(c"groups".as_ptr()) };
        let frame = self.sexp.get_attr(groups_sym);
        // dplyr reads the columns with VECTOR_ELT, so the frame must be a list.
        if frame.type_of() != SEXPTYPE::VECSXP || !frame.is_data_frame() || frame.len() == 0 {
            return Err(DataFrameError::GroupsNotDataFrame);
        }
        let names = frame.get_names();
        let last_is_rows = names.type_of() == SEXPTYPE::STRSXP
            && names.len() == frame.len()
            && names.string_elt_str(names.len() as isize - 1) == Some(".rows");
        if !last_is_rows {
            return Err(DataFrameError::MissingGroupRows);
        }
        let rows = frame.vector_elt(frame.len() as isize - 1);
        if rows.type_of() != SEXPTYPE::VECSXP {
            return Err(DataFrameError::BadGroupRows {
                group: None,
                type_of: r_typeof(rows),
            });
        }
        for group in 0..rows.len() {
            let elt = rows.vector_elt(group as isize);
            if elt.type_of() != SEXPTYPE::INTSXP {
                return Err(DataFrameError::BadGroupRows {
                    group: Some(group),
                    type_of: r_typeof(elt),
                });
            }
        }
        Ok(Some(GroupsAttr { frame, rows }))
    }
}

/// R's `typeof()` of `x`: `"double"`, `"NULL"`, `"list"`, ….
fn r_typeof(x: SEXP) -> String {
    // SAFETY: Rf_type2char returns a static C string for every SEXPTYPE.
    unsafe { std::ffi::CStr::from_ptr(crate::sys::Rf_type2char(x.type_of())) }
        .to_string_lossy()
        .into_owned()
}

/// Convert one `.rows` list element — an integer vector of 1-based row
/// indices ([`DataFrame::groups_attr`] checked its type) — into 0-based
/// indices, rejecting any index outside `1..=nrow` (`NA` included).
fn group_rows_indices(elt: SEXP, group: usize, nrow: usize) -> Result<Vec<usize>, DataFrameError> {
    // SAFETY: element is INTSXP (checked by groups_attr); as_slice handles
    // empty vectors.
    let values: &[i32] = unsafe { elt.as_slice() };
    values
        .iter()
        .map(|&value| match usize::try_from(value) {
            Ok(row) if (1..=nrow).contains(&row) => Ok(row - 1),
            _ => Err(DataFrameError::GroupIndexOutOfRange {
                group,
                value: i64::from(value),
                nrow,
            }),
        })
        .collect()
}

/// Check that in-range `.rows` cover each of the frame's `nrow` rows exactly
/// once — the shape dplyr always maintains. The first row claimed twice, or
/// else the first row no group claims, is reported.
fn check_rows_partition(
    groups: &[(GroupKey, Vec<usize>)],
    nrow: usize,
) -> Result<(), DataFrameError> {
    let mut owner: Vec<Option<usize>> = vec![None; nrow];
    for (group, (_, rows)) in groups.iter().enumerate() {
        for &row in rows {
            if let Some(first_group) = owner[row] {
                return Err(DataFrameError::GroupRowDuplicated {
                    row,
                    first_group,
                    second_group: group,
                });
            }
            owner[row] = Some(group);
        }
    }
    match owner.iter().position(Option::is_none) {
        Some(row) => Err(DataFrameError::GroupRowUncovered { row, nrow }),
        None => Ok(()),
    }
}

/// One supported key column's groups, in single-column group order (NA last)
/// — the per-type dispatch behind [`DataFrame::group_by`], and through
/// [`column_keys`] / [`row_keys`] behind `group_by_multi` and
/// `group_by_metadata`.
fn column_groups(column: SEXP, col: &str) -> Result<Vec<(GroupKey, Vec<usize>)>, DataFrameError> {
    if column.is_factor() {
        return Ok(factor_groups(column));
    }
    let unsupported = |type_of: String| DataFrameError::UnsupportedGroupColumn {
        column: col.to_string(),
        type_of,
    };
    match column.type_of() {
        SEXPTYPE::STRSXP => Ok(character_groups(column)),
        SEXPTYPE::INTSXP => Ok(integer_groups(column)),
        SEXPTYPE::LGLSXP => Ok(logical_groups(column)),
        // bit64 stores int64 bits in a double vector: read as doubles, its
        // values would order wrongly, and its NA (the bits of -0.0) would
        // join the 0 group.
        SEXPTYPE::REALSXP if column.inherits_class(c"integer64") => {
            Err(unsupported("integer64".to_string()))
        }
        SEXPTYPE::REALSXP => real_groups(column, col),
        other => Err(unsupported(format!("{:?}", other))),
    }
}

/// Invert groups into one key per row (`nrow` rows, row order). Each row of a
/// key column lands in exactly one group, so every slot is filled.
fn row_keys(groups: &[(GroupKey, Vec<usize>)], nrow: usize) -> Vec<GroupKey> {
    let mut keys: Vec<Option<GroupKey>> = vec![None; nrow];
    for (key, rows) in groups {
        for &row in rows {
            keys[row] = Some(key.clone());
        }
    }
    keys.into_iter()
        .map(|key| key.expect("each row of a key column is in exactly one group"))
        .collect()
}

/// Per-row group key for one supported key column, in row order. NA cells — and
/// factor NA codes / `addNA()` levels — become [`GroupKey::Na`]. Same type
/// dispatch and errors as [`DataFrame::group_by`].
fn column_keys(column: SEXP, col: &str) -> Result<Vec<GroupKey>, DataFrameError> {
    Ok(row_keys(&column_groups(column, col)?, column.len()))
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

/// Double keys (plain, `Date`, `POSIXct`, other classed doubles): numeric
/// sort with `NaN` after every number, `NA_real_` last. Equality follows
/// [`RealKey`] (`-0 == 0`, one `NaN` key); labels come from one
/// `as.character()` call over the distinct keys ([`real_labels`]).
fn real_groups(column: SEXP, col: &str) -> Result<Vec<(GroupKey, Vec<usize>)>, DataFrameError> {
    // SAFETY: REALSXP column; as_slice handles empty vectors.
    let values: &[f64] = unsafe { column.as_slice() };
    let mut buckets: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut na_bucket: Vec<usize> = Vec::new();
    for (row, &v) in values.iter().enumerate() {
        if is_na_real(v) {
            na_bucket.push(row);
        } else {
            buckets
                .entry(normalize_real(v).to_bits())
                .or_default()
                .push(row);
        }
    }
    // Normalised values hold no -0.0 and one positive NaN, so `total_cmp`
    // gives -Inf < … < Inf < NaN, dplyr's order (NA trails separately).
    let mut distinct: Vec<(f64, Vec<usize>)> = buckets
        .into_iter()
        .map(|(bits, rows)| (f64::from_bits(bits), rows))
        .collect();
    distinct.sort_by(|a, b| a.0.total_cmp(&b.0));

    let key_values: Vec<f64> = distinct.iter().map(|(value, _)| *value).collect();
    let labels = real_labels(column, &key_values, col)?;
    let mut groups: Vec<(GroupKey, Vec<usize>)> = distinct
        .into_iter()
        .zip(labels)
        .map(|((value, rows), label)| (GroupKey::Real(RealKey { value, label }), rows))
        .collect();
    if !na_bucket.is_empty() {
        groups.push((GroupKey::Na, na_bucket));
    }
    Ok(groups)
}

/// R's `as.character()` of `values` carrying `column`'s attributes (class,
/// `tzone`, `units`, …): the text R prints for each key, with S3 dispatch for
/// a classed column (`Date`, `POSIXct`, …) and C-level coercion otherwise.
///
/// The temporary key vector is protected while `as.character()` allocates; a
/// failing method surfaces as [`DataFrameError::Conversion`] (it is evaluated
/// with `RCall::eval`, so it never unwinds through Rust frames).
fn real_labels(column: SEXP, values: &[f64], col: &str) -> Result<Vec<String>, DataFrameError> {
    let label_error = |detail: String| {
        DataFrameError::Conversion(format!("cannot label the keys of column {col:?}: {detail}"))
    };
    // SAFETY: the grouping verbs run on R's main thread. `keys` is a fresh
    // REALSXP rooted by `OwnedProtect` until the labels are copied out.
    let labels = unsafe {
        let keys = crate::OwnedProtect::new(values.into_sexp());
        crate::sys::Rf_copyMostAttrib(column, keys.get());
        crate::convert::read_character(keys.get())
    }
    .map_err(|e| label_error(e.to_string()))?;
    if labels.len() != values.len() {
        return Err(label_error(format!(
            "as.character() returned {} values for {} keys",
            labels.len(),
            values.len()
        )));
    }
    Ok(labels
        .into_iter()
        .map(|label| label.unwrap_or_else(|| "NA".to_string()))
        .collect())
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
            GroupKey::Real(RealKey::new(2.5, "2.5")),
            GroupKey::Bool(true),
            GroupKey::Na,
        ]);
        assert_eq!(key.label(), "a.2.2.5.TRUE.NA");
        assert_eq!(key.to_string(), "a.2.2.5.TRUE.NA");
    }

    fn hash_of(key: &RealKey) -> u64 {
        use std::hash::{BuildHasher, BuildHasherDefault};
        BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().hash_one(key)
    }

    #[test]
    fn real_key_equality_follows_vctrs() {
        // -0 and 0 are one key (and hash alike).
        let neg_zero = RealKey::new(-0.0, "0");
        let zero = RealKey::new(0.0, "0");
        assert_eq!(neg_zero, zero);
        assert_eq!(hash_of(&neg_zero), hash_of(&zero));
        assert!(neg_zero.value().is_sign_positive());

        // Every NaN is one key, stored as the canonical positive NaN —
        // including x86's negative `0/0` NaN.
        let nan = RealKey::new(f64::NAN, "NaN");
        let neg_nan = RealKey::new(f64::from_bits(0xFFF8_0000_0000_0000), "NaN");
        assert_eq!(nan, neg_nan);
        assert_eq!(hash_of(&nan), hash_of(&neg_nan));
        assert_eq!(neg_nan.value().to_bits(), CANONICAL_NAN_BITS);
        assert!(!is_na_real(nan.value()));

        // The label plays no part in equality; distinct values differ.
        assert_eq!(RealKey::new(1.5, "x"), RealKey::new(1.5, "y"));
        assert_ne!(RealKey::new(1.5, "1.5"), RealKey::new(2.5, "2.5"));
    }

    #[test]
    fn real_key_label_and_order() {
        let key = GroupKey::Real(RealKey::new(19724.0, "2024-01-02"));
        assert_eq!(key.label(), "2024-01-02");
        assert_eq!(key.to_string(), "2024-01-02");

        // Normalised values sort -Inf < numbers < Inf < NaN under total_cmp.
        let mut values: Vec<f64> = [f64::NAN, 1.0, f64::INFINITY, -0.0, f64::NEG_INFINITY]
            .into_iter()
            .map(normalize_real)
            .collect();
        values.sort_by(f64::total_cmp);
        assert_eq!(values[0], f64::NEG_INFINITY);
        assert_eq!(values[1].to_bits(), 0.0f64.to_bits());
        assert_eq!(values[2], 1.0);
        assert_eq!(values[3], f64::INFINITY);
        assert!(values[4].is_nan());
    }
}
