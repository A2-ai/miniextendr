//! Fixtures for `#[derive(AllEqual)]`, the `all.equal()` method of a
//! pointer-backed class (#2149): one class per class system holding the same
//! `values`, and S3 classes for the field options, nested classes and the
//! sidecar fields.

use miniextendr_api::externalptr::{Computed, ExternalPtr, RSidecar, Sidecar};
use miniextendr_api::miniextendr;

// region: one class per class system

/// Values compared by an S3 class's derived `all.equal()`.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqS3 {
    values: Vec<f64>,
}

#[miniextendr(s3, internal)]
impl AllEqS3 {
    /// @param values The values to compare.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Values compared by an env class's derived `all.equal()`.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqEnv {
    values: Vec<f64>,
}

#[miniextendr(env, internal)]
impl AllEqEnv {
    /// @param values The values to compare.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Values compared by an R6 class's derived `all.equal()`.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqR6 {
    values: Vec<f64>,
}

#[miniextendr(r6, internal)]
impl AllEqR6 {
    /// @param values The values to compare.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Values compared by an S4 class's derived `all.equal()`.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqS4 {
    values: Vec<f64>,
}

#[miniextendr(s4, internal)]
impl AllEqS4 {
    /// @param values The values to compare.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Values compared by an S7 class's derived `all.equal()`.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqS7 {
    values: Vec<f64>,
}

#[miniextendr(s7, internal)]
impl AllEqS7 {
    /// @param values The values to compare.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

// endregion

// region: the field options, a nested class and the struct's settings

/// The model an [`AllEqFit`] holds, compared by its own derived method.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqModel {
    weights: Vec<f64>,
}

#[miniextendr(s3, internal)]
impl AllEqModel {
    /// @param weights The model's weights.
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }
}

/// A fit: labelled estimates, an iteration count compared exactly, a nested
/// model, a name compared through a getter and a scratch buffer left out.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
#[all_equal(tolerance = 1e-6)]
pub struct AllEqFit {
    #[all_equal(label = "Estimates")]
    estimates: Vec<f64>,
    #[all_equal(tolerance = 0)]
    iterations: f64,
    model: ExternalPtr<AllEqModel>,
    #[all_equal(with = "Self::folded_name")]
    name: String,
    #[all_equal(skip)]
    scratch: Vec<f64>,
}

impl AllEqFit {
    /// The name compared, without case.
    fn folded_name(&self) -> String {
        self.name.to_lowercase()
    }
}

#[miniextendr(s3, internal)]
impl AllEqFit {
    /// @param estimates The estimates.
    /// @param iterations The iteration count.
    /// @param weights The model's weights.
    /// @param name The fit's name.
    /// @param scratch A buffer the comparison leaves out.
    pub fn new(
        estimates: Vec<f64>,
        iterations: f64,
        weights: Vec<f64>,
        name: String,
        scratch: Vec<f64>,
    ) -> Self {
        Self {
            estimates,
            iterations,
            model: ExternalPtr::new(AllEqModel { weights }),
            name,
            scratch,
        }
    }

    /// The scratch buffer's length.
    pub fn scratch_len(&self) -> i32 {
        i32::try_from(self.scratch.len()).expect("a short buffer")
    }
}

/// A part [`AllEqHolder`] holds by value, as an optional pointer and in a
/// `Vec` of pointers, compared by its own derived method.
#[derive(Clone, miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqPart {
    weight: f64,
}

#[miniextendr(s3, internal)]
impl AllEqPart {
    /// @param weight The part's weight.
    pub fn new(weight: f64) -> Self {
        Self { weight }
    }
}

/// Parts of a class held by value, as an optional pointer and as a list of
/// pointers: each is given its class, so it is compared by its own method.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
pub struct AllEqHolder {
    first: AllEqPart,
    spare: Option<ExternalPtr<AllEqPart>>,
    rest: Vec<ExternalPtr<AllEqPart>>,
}

#[miniextendr(s3, internal)]
impl AllEqHolder {
    /// @param first The first part's weight.
    /// @param spare The spare part's weight, or `NULL` for none.
    /// @param rest The other parts' weights.
    pub fn new(first: f64, spare: Option<f64>, rest: Vec<f64>) -> Self {
        Self {
            first: AllEqPart { weight: first },
            spare: spare.map(|weight| ExternalPtr::new(AllEqPart { weight })),
            rest: rest
                .into_iter()
                .map(|weight| ExternalPtr::new(AllEqPart { weight }))
                .collect(),
        }
    }
}

// endregion

// region: sidecar fields

/// A `Sidecar<T>` field read from the pointer's protection list, and a
/// `Computed` field read through its getter.
#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AllEqual)]
#[externalptr(s3)]
pub struct AllEqSidecar {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub keys: Sidecar<Vec<i32>>,
    #[r_data(get = "Self::total")]
    pub total: Computed,
    #[all_equal(skip)]
    values: Vec<i32>,
}

impl AllEqSidecar {
    fn total(&self) -> i32 {
        self.values.iter().sum()
    }
}

#[miniextendr(s3, internal)]
impl AllEqSidecar {
    /// @param keys The keys, kept in the pointer's protection list.
    /// @param values The values, compared through their total only.
    pub fn new(keys: Vec<i32>, values: Vec<i32>) -> Self {
        Self {
            _r: RSidecar,
            keys: Sidecar::new(keys),
            total: Computed,
            values,
        }
    }
}

// endregion
