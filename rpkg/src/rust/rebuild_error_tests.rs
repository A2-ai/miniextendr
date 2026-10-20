//! Rebuild hooks that report a failure with fields without raising (#2217).
//!
//! `HookCondition`'s hook returns `Result<Self, RCondition>`: a recipe check
//! raises a classed error with fields, which the hook catches
//! (`catch_condition`) and returns with `?`. The refusal of the restored
//! object carries it as `e$parent`, as `hook_condition_entry()`, an entry
//! point returning the same `Err(RCondition)`, raises it.
//! `hook_condition_probe()` reads a restored pointer through
//! `ExternalPtr::wrap_sexp_with_error`, which returns a failed rebuild instead
//! of raising it. `HookPair`'s hook returns the refusal of the HookCondition
//! it holds, caught: a condition that already has a cause.
//!
//! `HookQuota`'s hook (feature `serde`) returns
//! `Err(serde_condition!(&err))` for a `QuotaError`, whose shape
//! `serde_error_shape_tests` declares for the crate: the cause is what
//! `quota_err_a()`, an entry point returning that error, raises.

use miniextendr_api::condition::{RCondition, RConditionData, catch_condition};
use miniextendr_api::externalptr::{ExternalPtr, Saved, Sidecar, TypeMismatchError};
use miniextendr_api::prelude::SEXP;
use miniextendr_api::{RValue, SEXPTYPE, SexpExt, TryFromSexp, miniextendr};

// region: HookCondition: a hook returning Err(RCondition)

/// Refuses a recipe holding a negative value: a classed error with the count
/// and the values as fields, about the data, so without a call.
fn check_recipe(values: &[f64]) {
    let negative: Vec<f64> = values.iter().copied().filter(|v| *v < 0.0).collect();
    if !negative.is_empty() {
        let n = i32::try_from(negative.len()).expect("count fits in i32");
        miniextendr_api::rust_error!(
            call = none,
            class = ["mx_hook_negative", "mx_hook_error"],
            data = { n = n, values = negative },
            "the recipe holds {n} negative value(s)"
        );
    }
}

/// A recipe and its total, rebuilt from the recipe after a reload.
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(restored(class = "mx_hook_saved"), rebuild = Self::from_saved)]
pub struct HookCondition {
    /// The recipe, kept by a save.
    #[r_data]
    pub values: Sidecar<Vec<f64>>,
    /// The Rust state, rebuilt from the recipe.
    total: f64,
}

impl HookCondition {
    /// The rebuild hook: the recipe check's condition is the `Err`.
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
        let values = Self::values(saved);
        catch_condition(|| check_recipe(&values))?;
        Ok(HookCondition {
            total: values.iter().sum(),
            values: Sidecar::default(),
        })
    }
}

/// A HookCondition over `values`, unchecked: a save of a bad recipe fails
/// its rebuild.
/// @param values The recipe.
#[miniextendr(noexport)]
pub fn hook_condition_new(values: Vec<f64>) -> HookCondition {
    HookCondition {
        total: values.iter().sum(),
        values: Sidecar::new(values),
    }
}

/// The total, through a `&T` argument: a restored pointer is rebuilt, or its
/// refusal raised.
/// @param x A HookCondition pointer.
#[miniextendr(noexport)]
pub fn hook_condition_total(x: &HookCondition) -> f64 {
    x.total
}

/// The recipe check as an entry point's `Err(RCondition)`: the condition a
/// failed rebuild carries as `e$parent`.
/// @param values The recipe.
#[miniextendr(noexport)]
pub fn hook_condition_entry(values: Vec<f64>) -> Result<f64, RCondition> {
    catch_condition(|| check_recipe(&values))?;
    Ok(values.iter().sum())
}

/// What `ExternalPtr::wrap_sexp_with_error` gives for `x`, as a value: the
/// total of a live or rebuilt pointer, or the refusal of a failed rebuild as
/// `list(message, class)`, without raising it.
/// @param x A HookCondition pointer.
#[miniextendr(noexport)]
pub fn hook_condition_probe(x: SEXP) -> RValue {
    assert!(
        x.type_of() == SEXPTYPE::EXTPTRSXP,
        "expected an external pointer"
    );
    // SAFETY: on R's main thread (a `SEXP` argument keeps the call there);
    // `x` is an EXTPTRSXP the call's frame roots.
    match unsafe { ExternalPtr::<HookCondition>::wrap_sexp_with_error(x) } {
        Ok(ptr) => RValue::from(ptr.borrow().total),
        Err(TypeMismatchError::RebuildFailed { condition }) => RValue::List(vec![
            (
                Some("message".to_owned()),
                RValue::from(condition.message()),
            ),
            (Some("class".to_owned()), RValue::from(condition.class())),
        ]),
        Err(other) => panic!("{other}"),
    }
}

// endregion

// region: HookPair: a hook returning a caught refusal

/// Holds a HookCondition pointer. Its hook converts it inside
/// `catch_condition` and returns what that caught: the inner refusal, with
/// the inner hook's condition as its own cause.
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(rebuild = Self::from_saved)]
pub struct HookPair {
    /// The inner object's external pointer.
    #[r_data]
    pub inner: Sidecar<SEXP>,
    /// The Rust state: the inner total.
    total: f64,
}

impl HookPair {
    /// The rebuild hook: a failed inner rebuild, caught, is the `Err`.
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
        let inner = Self::inner(saved);
        let inner = catch_condition(|| {
            ExternalPtr::<HookCondition>::try_from_sexp(inner).unwrap_or_else(|e| {
                miniextendr_api::rust_error!(class = "mx_hook_pair_error", "{e}")
            })
        })?;
        let total = inner.borrow().total;
        Ok(HookPair {
            inner: Sidecar::new(SEXP::nil()),
            total,
        })
    }
}

/// A HookPair over `inner`'s pointer.
/// @param inner A HookCondition pointer.
#[miniextendr(noexport)]
pub fn hook_pair_new(inner: ExternalPtr<HookCondition>) -> HookPair {
    HookPair {
        total: inner.borrow().total,
        inner: Sidecar::new(inner.as_sexp()),
    }
}

/// The inner total, through a `&T` argument: a restored pointer is rebuilt,
/// or its refusal raised.
/// @param x A HookPair pointer.
#[miniextendr(noexport)]
pub fn hook_pair_total(x: &HookPair) -> f64 {
    x.total
}

// endregion

// region: HookQuota: a hook returning Err(serde_condition!(..))

/// Rows under a quota, rebuilt from the saved rows; more rows than the quota
/// fail the rebuild with a `QuotaError`.
#[cfg(feature = "serde")]
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(restored(class = "mx_hook_saved"), rebuild = Self::from_saved)]
pub struct HookQuota {
    /// The rows, kept by a save.
    #[r_data]
    pub rows: Sidecar<Vec<i32>>,
    /// The Rust state: how many rows.
    n: usize,
}

#[cfg(feature = "serde")]
impl HookQuota {
    /// The quota `quota_exceeded` reports as its limit.
    const LIMIT: usize = 10;

    /// The rebuild hook: the serde error as the `Err`, with the shape the
    /// crate declared for it.
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RCondition> {
        let rows = Self::rows(saved);
        if rows.len() > Self::LIMIT {
            let err = crate::serde_error_shape_tests::quota_exceeded(vec!["rows".to_owned()], rows);
            return Err(miniextendr_api::serde_condition!(&err));
        }
        Ok(HookQuota {
            n: rows.len(),
            rows: Sidecar::default(),
        })
    }
}

/// A HookQuota over `rows`, unchecked: a save of too many rows fails its
/// rebuild.
/// @param rows The rows.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn hook_quota_new(rows: Vec<i32>) -> HookQuota {
    HookQuota {
        n: rows.len(),
        rows: Sidecar::new(rows),
    }
}

/// The number of rows, through a `&T` argument: a restored pointer is
/// rebuilt, or its refusal raised.
/// @param x A HookQuota pointer.
#[cfg(feature = "serde")]
#[miniextendr(noexport)]
pub fn hook_quota_n(x: &HookQuota) -> i32 {
    i32::try_from(x.n).expect("count fits in i32")
}

// endregion
