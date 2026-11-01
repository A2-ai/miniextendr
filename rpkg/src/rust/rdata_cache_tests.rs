//! `#[r_data(cache)]` fields (#2271): the getter keeps the R value its first
//! read converts and returns that same object until the value is borrowed
//! exclusively (a `&mut self` method, a `&mut T` argument, an R setter).
//! `CacheEnv` (methods and fields behind `$`) and `CacheS3` hold the same
//! numbers in a cached `values` and an uncached `plain`, next to a cached
//! read-only `tags` and a cached `null_if_none` `label`; `CacheR6` and
//! `CacheS7` cache through an active binding and a property; `SavedCache`
//! is a serde type, whose save never carries a kept value.

use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::{SEXP, miniextendr};

/// `1, 2, ..., n` as doubles.
fn sequence(n: i32) -> Vec<f64> {
    (1..=n).map(f64::from).collect()
}

/// `"v1", "v2", ..., "vn"`.
fn labels(n: i32) -> Vec<String> {
    (1..=n).map(|i| format!("v{i}")).collect()
}

// region: env, cached fields and the methods that change them

/// An env class whose `values`, `tags` and `label` are kept after their
/// first read, next to `plain`, the same numbers converted on every read.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct CacheEnv {
    #[r_data]
    _r: RSidecar,

    /// The values, kept after the first read.
    #[r_data(cache)]
    pub values: Vec<f64>,

    /// The same values, converted on every read.
    #[r_data]
    pub plain: Vec<f64>,

    /// One tag per value, read-only from R, kept after the first read.
    #[r_data(cache, read_only)]
    pub tags: Vec<String>,

    /// A label, `NULL` when unset, kept after the first read.
    #[r_data(cache, null_if_none)]
    pub label: Option<String>,
}

/// The env class CacheEnv: `$` reads the fields (the cached ones return the
/// object their first read made) and the methods; `$<-` / `[[<-` write
/// `values`, `plain` and `label`.
#[miniextendr(env(r_data_accessors))]
impl CacheEnv {
    /// Create a CacheEnv holding `1, ..., n` in `values` and `plain`, with
    /// one tag per value and no label.
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        CacheEnv {
            _r: RSidecar,
            values: sequence(n),
            plain: sequence(n),
            tags: labels(n),
            label: None,
        }
    }

    /// Appends `value` to `values` and a tag to `tags` (a `&mut self`
    /// method: the kept values are dropped).
    /// @param value The value to append.
    pub fn push(&mut self, value: f64) {
        self.values.push(value);
        self.tags.push(format!("v{}", self.tags.len() + 1));
    }

    /// Sets the label (a `&mut self` method).
    /// @param label The label, `NULL` to unset it.
    pub fn relabel(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// The number of values (a `&self` method: the kept values stay).
    pub fn count(&self) -> i32 {
        i32::try_from(self.values.len()).expect("fewer than 2^31 values")
    }
}

/// Appends `value` to the `values` of `target` through a `&mut CacheEnv`
/// argument, which drops its kept values.
#[miniextendr(noexport)]
pub fn rdata_cache_push(target: &mut CacheEnv, value: f64) {
    target.values.push(value);
}

/// The address of `x`, so a test can tell two reads that return the same
/// object from two equal objects.
#[miniextendr(noexport)]
pub fn rdata_cache_address(x: SEXP) -> String {
    format!("{:p}", x.as_ptr())
}
// endregion

// region: S3

/// An S3 class whose `values` is kept after its first read and whose `plain`
/// holds the same numbers, converted on every read.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct CacheS3 {
    #[r_data]
    _r: RSidecar,

    /// The values, kept after the first read.
    #[r_data(cache)]
    pub values: Vec<f64>,

    /// The same values, converted on every read.
    #[r_data]
    pub plain: Vec<f64>,
}

/// S3 class registration for CacheS3: `$` / `[[` read the fields, `$<-` /
/// `[[<-` write them.
#[miniextendr(s3(r_data_accessors))]
impl CacheS3 {
    /// Create a CacheS3 holding `1, ..., n` in `values` and `plain`.
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        CacheS3 {
            _r: RSidecar,
            values: sequence(n),
            plain: sequence(n),
        }
    }
}
// endregion

// region: R6

/// An R6 class whose `values` binding returns the object its first read
/// made until a method or the binding changes the values.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6)]
pub struct CacheR6 {
    #[r_data]
    _r: RSidecar,

    #[r_data(cache)]
    pub values: Vec<f64>,
}

/// R6 class registration for CacheR6.
/// @field values The values, kept after the first read (active binding).
#[miniextendr(r6(r_data_accessors))]
impl CacheR6 {
    /// Create a new CacheR6 holding `1, ..., n`.
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        CacheR6 {
            _r: RSidecar,
            values: sequence(n),
        }
    }

    /// Appends `value` to `values`.
    /// @param value The value to append.
    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }
}
// endregion

// region: S7

/// An S7 class whose `values` and `label` properties return the object their
/// first read made until a property write changes them.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s7)]
pub struct CacheS7 {
    #[r_data]
    _r: RSidecar,

    #[r_data(cache, prop_doc = "The values, kept after the first read.")]
    pub values: Vec<f64>,

    #[r_data(
        cache,
        null_if_none,
        prop_doc = "A label, `NULL` when unset, kept after the first read."
    )]
    pub label: Option<String>,
}

/// S7 class registration for CacheS7 with property-based sidecar accessors.
#[miniextendr(s7(r_data_accessors))]
impl CacheS7 {
    /// Create a new CacheS7 holding `1, ..., n` and no label.
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        CacheS7 {
            _r: RSidecar,
            values: sequence(n),
            label: None,
        }
    }
}
// endregion

// region: serde, a save carries no kept value

/// A serde R6 class with a cached field: a save keeps the value, never the
/// R object a read kept.
#[cfg(feature = "serde")]
#[derive(
    miniextendr_api::ExternalPtr, Debug, crate::serde::Serialize, crate::serde::Deserialize,
)]
#[serde(crate = "crate::serde")]
#[externalptr(r6)]
pub struct SavedCache {
    #[serde(skip)]
    #[r_data]
    _r: RSidecar,

    #[r_data(cache)]
    pub values: Vec<f64>,
}

/// @title A serde R6 class with a cached field
/// @description `SavedCache` comes back after `readRDS()` with its values
///   and no kept R object.
/// @field values The values, kept after the first read (active binding).
#[cfg(feature = "serde")]
#[miniextendr(r6(r_data_accessors))]
impl SavedCache {
    /// @param n The number of values.
    pub fn new(n: i32) -> Self {
        SavedCache {
            _r: RSidecar,
            values: sequence(n),
        }
    }

    /// Appends `value` to `values`.
    /// @param value The value to append.
    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }
}
// endregion
