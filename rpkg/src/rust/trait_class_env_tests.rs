//! Class environments that only a trait impl fills (#2289). A trait impl's
//! static methods and consts land in the `<Class>$<Trait>` namespace, so the
//! class environment `<Class>` exists for them, named by the R class, on
//! each class shape that binds nothing in it itself:
//!
//! - `TraitEnvRecS3`: an S3 class the package names (`class =
//!   "trait_env_rec"`) and builds with its factory. Its impl has no docs:
//!   the class block, which titles the page, is written for the environment.
//! - `TraitEnvVctrs`: a vctrs class with a constructor, `new_traitenvvctrs()`.
//! - `TraitEnvVecRec`: a vctrs class the package names (`class =
//!   "trait_env_vec"`) without a constructor.
//!
//! Without such a trait impl none of them would bind its class name: see
//! `NoCtorS3` and `NoCtorVctrs` in `constructor_tests.rs` and the #1918
//! readers-only classes in `rdata_sidecar_tests.rs`.

use miniextendr_api::{ExternalPtr, miniextendr};

/// A trait whose impls put a static method and a const in their class's
/// `<Class>$ClassEnvStatics` namespace.
#[miniextendr]
pub trait ClassEnvStatics {
    /// A code for the class.
    const CODE: i32;
    /// A label for the class.
    fn label() -> String;
}

// region: S3 class the package names, without a constructor

/// An S3 class the package names `trait_env_rec` and builds with its factory.
#[derive(ExternalPtr)]
pub struct TraitEnvRecS3;

#[miniextendr(s3, class = "trait_env_rec")]
impl TraitEnvRecS3 {
    /// An object for `value`, which must not be negative.
    /// @param value The value.
    pub fn of(value: i32) -> Result<Self, String> {
        if value < 0 {
            return Err(format!("negative value {value}"));
        }
        Ok(Self)
    }
}

#[miniextendr(s3)]
impl ClassEnvStatics for TraitEnvRecS3 {
    const CODE: i32 = 21;
    fn label() -> String {
        "renamed S3".to_string()
    }
}
// endregion

// region: vctrs class with a constructor

/// A vctrs class whose trait impl has a static method and a const.
pub struct TraitEnvVctrs;

/// vctrs class with a trait namespace.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "tev"))]
impl TraitEnvVctrs {
    /// Amounts as a `TraitEnvVctrs` vector.
    /// @param amounts Numeric amounts.
    // A vctrs constructor returns the vector payload, which
    // `vctrs::new_vctr()` wraps.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(amounts: Vec<f64>) -> Vec<f64> {
        amounts
    }
}

#[miniextendr(vctrs)]
impl ClassEnvStatics for TraitEnvVctrs {
    const CODE: i32 = 32;
    fn label() -> String {
        "vctrs".to_string()
    }
}
// endregion

// region: vctrs class the package names, without a constructor

/// A vctrs class the package names `trait_env_vec` and builds itself.
pub struct TraitEnvVecRec;

/// vctrs class `trait_env_vec`, whose vectors the package builds with
/// `vctrs::new_vctr()`.
#[miniextendr(
    vctrs(kind = "vctr", base = "double", abbr = "tevr"),
    class = "trait_env_vec"
)]
impl TraitEnvVecRec {
    /// The sum of `amounts`.
    /// @param amounts Numeric amounts.
    pub fn total(amounts: Vec<f64>) -> f64 {
        amounts.iter().sum()
    }
}

#[miniextendr(vctrs)]
impl ClassEnvStatics for TraitEnvVecRec {
    const CODE: i32 = 43;
    fn label() -> String {
        "renamed vctrs".to_string()
    }
}
// endregion
