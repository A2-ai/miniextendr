//! Rebuilding a pointer restored from a saved session (#2146):
//! `#[externalptr(rebuild = path)]`.
//!
//! `RebuildR6` keeps its recipe, the values, in a `Sidecar`, which a save
//! keeps; its Rust state, the total, is rebuilt from it on first use after
//! `readRDS()` / `unserialize()`. The hook refuses a recipe with a negative
//! value, with its own classed error. `RebuildPair` holds a `RebuildR6`
//! pointer in a `Sidecar<SEXP>` and rebuilds it from its own hook.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use miniextendr_api::condition::{RCondition, RError};
use miniextendr_api::expression::RCall;
use miniextendr_api::externalptr::{ExternalPtr, RSidecar, Saved, Sidecar};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::{OwnedProtect, TryFromSexp, miniextendr};

/// How many times a `RebuildR6` hook ran, and how many values were dropped.
static REBUILDS: AtomicI32 = AtomicI32::new(0);
static DROPS: AtomicI32 = AtomicI32::new(0);
/// Whether the hook queues a warning (the replayed construction's).
static WARN: AtomicBool = AtomicBool::new(false);

/// The hook's refusal of a saved recipe.
#[derive(Debug, RCondition)]
#[condition(class = "mx_rebuild_error")]
pub enum RebuildError {
    #[condition(message = "the saved values hold {n} negative value(s)")]
    Negative { n: i32 },
}

// region: RebuildR6

/// An R6 class rebuilt from its saved recipe.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6, restored(class = "mx_rebuild_saved"), rebuild = Self::from_saved)]
pub struct RebuildR6 {
    #[r_data]
    _r: RSidecar,

    /// The recipe, kept by a save.
    #[r_data]
    pub values: Sidecar<Vec<f64>>,

    /// The Rust state, rebuilt from the recipe.
    #[r_data]
    pub total: f64,
}

impl Drop for RebuildR6 {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl RebuildR6 {
    fn build(values: Vec<f64>) -> Self {
        RebuildR6 {
            _r: RSidecar,
            total: values.iter().sum(),
            values: Sidecar::new(values),
        }
    }

    /// The rebuild hook: the total from the saved values. The `Sidecar` in
    /// the struct it returns is a placeholder; the saved slot stays.
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RebuildError> {
        REBUILDS.fetch_add(1, Ordering::Relaxed);
        let values = Self::values(saved);
        let negative = values.iter().filter(|v| **v < 0.0).count();
        if negative > 0 {
            return Err(RebuildError::Negative {
                n: i32::try_from(negative).expect("count fits in i32"),
            });
        }
        if WARN.load(Ordering::Relaxed) {
            miniextendr_api::defer_warning!(class = "mx_rebuild_replayed", "replayed the build");
        }
        Ok(RebuildR6 {
            _r: RSidecar,
            total: values.iter().sum(),
            values: Sidecar::default(),
        })
    }
}

/// An R6 class whose Rust state is rebuilt from its saved recipe.
#[miniextendr(r6(r_data_accessors))]
impl RebuildR6 {
    /// Create a RebuildR6 from its values.
    /// @param values The recipe.
    pub fn new(values: Vec<f64>) -> Self {
        Self::build(values)
    }

    /// The total, from the Rust state.
    pub fn sum(&self) -> f64 {
        self.total
    }

    /// Scale the Rust state.
    /// @param k The factor.
    pub fn scale(&mut self, k: f64) {
        self.total *= k;
    }
}

/// The total of a `RebuildR6`, through an `ExternalPtr<T>` argument.
/// @param x A RebuildR6 object or its pointer.
#[miniextendr(noexport)]
pub fn rebuild_total_of(x: ExternalPtr<RebuildR6>) -> f64 {
    x.borrow().total
}

/// `c(rebuilds, drops)` so far, and turns the hook's warning on or off.
/// @param warn Whether the hook queues a warning.
#[miniextendr(noexport)]
pub fn rebuild_counts(warn: bool) -> Vec<i32> {
    WARN.store(warn, Ordering::Relaxed);
    vec![
        REBUILDS.load(Ordering::Relaxed),
        DROPS.load(Ordering::Relaxed),
    ]
}

// endregion

// region: RebuildPair: a rebuild that rebuilds another object

/// Holds a `RebuildR6` pointer; its hook converts it, which rebuilds it.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env, rebuild = Self::from_saved)]
pub struct RebuildPair {
    #[r_data]
    _r: RSidecar,

    /// The inner object's external pointer.
    #[r_data]
    pub inner: Sidecar<SEXP>,

    /// The inner total, doubled.
    pub doubled: f64,
}

impl RebuildPair {
    /// Converting the saved inner pointer rebuilds it (or raises its
    /// refusal, which becomes this rebuild's cause).
    fn from_saved(saved: &Saved<'_, Self>) -> Result<Self, RError> {
        let inner = ExternalPtr::<RebuildR6>::try_from_sexp(Self::inner(saved))
            .map_err(|e| RError::new(e.to_string()).class("mx_rebuild_pair_error"))?;
        let doubled = inner.borrow().total * 2.0;
        Ok(RebuildPair {
            _r: RSidecar,
            inner: Sidecar::new(SEXP::nil()),
            doubled,
        })
    }
}

/// A pair over a RebuildR6 object.
#[miniextendr(env)]
impl RebuildPair {
    /// Create a pair over `inner`'s pointer.
    /// @param inner A RebuildR6 object or its pointer.
    pub fn new(inner: ExternalPtr<RebuildR6>) -> Self {
        let doubled = inner.borrow().total * 2.0;
        RebuildPair {
            _r: RSidecar,
            inner: Sidecar::new(inner.as_sexp()),
            doubled,
        }
    }

    /// The inner total, doubled.
    pub fn doubled(&self) -> f64 {
        self.doubled
    }
}

// endregion

// region: gctorture

/// Allocate garbage so a GC runs between the steps.
fn churn() {
    for i in 0..64 {
        let _garbage = vec![f64::from(i); 64].into_sexp();
    }
}

/// Serialize and unserialize `RebuildR6` pointers, churn the GC, rebuild
/// them through an `ExternalPtr<T>` conversion, and read the saved slot
/// back. No arguments — picked up by the fast `gctorture(TRUE)` no-arg
/// sweep (#430).
// Stays on R's main thread under `worker-default`: the rebuild runs there.
#[miniextendr(noexport, no_worker)]
pub fn gc_stress_rebuild() {
    for n in 1..=4 {
        let values: Vec<f64> = (1..=n * 4).map(f64::from).collect();
        let ptr = ExternalPtr::new(RebuildR6::build(values.clone()));
        // SAFETY: on R's main thread; each value is rooted before the next
        // call allocates.
        let bytes = unsafe {
            OwnedProtect::new(
                RCall::new("serialize")
                    .arg(ptr.as_sexp())
                    .arg(SEXP::nil())
                    .eval_base()
                    .expect("serialize"),
            )
        };
        let back = unsafe {
            OwnedProtect::new(
                RCall::new("unserialize")
                    .arg(bytes.get())
                    .eval_base()
                    .expect("unserialize"),
            )
        };
        churn();
        let rebuilt = ExternalPtr::<RebuildR6>::try_from_sexp(back.get()).expect("rebuilt");
        churn();
        assert_eq!(rebuilt.borrow().total, values.iter().sum::<f64>());
        assert_eq!(RebuildR6::values(&rebuilt), values);
    }
}

// endregion
