//! Fieldwise comparisons for every pointer-backed class system (#2149).
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};
use miniextendr_api::{AllEqual, ExternalPtr, SEXP, miniextendr};

struct NotClone(f64);

macro_rules! comparison_class {
    ($name:ident, $system:ident) => {
        #[derive(ExternalPtr, AllEqual)]
        #[externalptr(restored(class = "all_equal_saved"))]
        #[all_equal(tolerance = 1e-5)]
        pub struct $name {
            #[all_equal(label = "Estimate")]
            estimate: Vec<f64>,
            #[all_equal(tolerance = 0)]
            iterations: f64,
            #[all_equal(with = "Self::auxiliary_value", label = "Auxiliary")]
            auxiliary: NotClone,
            #[all_equal(skip)]
            _cache: std::sync::Mutex<i32>,
        }

        impl $name {
            fn auxiliary_value(&self) -> f64 {
                self.auxiliary.0
            }
        }

        /// A pointer-backed object compared field by field.
        #[miniextendr($system)]
        impl $name {
            /// Create an object with numeric, fixed-tolerance, getter and skipped fields.
            pub fn new(estimate: f64, iterations: f64, auxiliary: f64, cache: i32) -> Self {
                Self {
                    estimate: vec![estimate, estimate * 2.0],
                    iterations,
                    auxiliary: NotClone(auxiliary),
                    _cache: std::sync::Mutex::new(cache),
                }
            }
        }
    };
}
comparison_class!(EqualS3, s3);
comparison_class!(EqualEnv, env);
comparison_class!(EqualR6, r6);
comparison_class!(EqualS4, s4);
comparison_class!(EqualS7, s7);

/// A vctrs class whose R value carries a pointer in its .ptr attribute.
#[cfg(feature = "vctrs")]
#[derive(ExternalPtr, AllEqual)]
#[externalptr(restored(class = "all_equal_saved"))]
#[all_equal(tolerance = 1e-5)]
pub struct EqualVctrs {
    #[all_equal(label = "Estimate")]
    estimate: Vec<f64>,
    #[all_equal(tolerance = 0)]
    iterations: f64,
    #[all_equal(with = "Self::auxiliary_value", label = "Auxiliary")]
    auxiliary: NotClone,
    #[all_equal(skip)]
    _cache: std::sync::Mutex<i32>,
}
#[cfg(feature = "vctrs")]
impl EqualVctrs {
    fn auxiliary_value(&self) -> f64 {
        self.auxiliary.0
    }
}
#[cfg(feature = "vctrs")]
#[miniextendr(vctrs, class = "equal_vctrs", internal)]
#[cfg(feature = "vctrs")]
impl EqualVctrs {}

/// Pointer storage for the AllEqual vctrs fixture.
#[cfg(feature = "vctrs")]
#[miniextendr(noexport)]
pub fn equal_vctrs_pointer(
    estimate: f64,
    iterations: f64,
    auxiliary: f64,
    cache: i32,
) -> ExternalPtr<EqualVctrs> {
    ExternalPtr::new(EqualVctrs {
        estimate: vec![estimate, estimate * 2.0],
        iterations,
        auxiliary: NotClone(auxiliary),
        _cache: std::sync::Mutex::new(cache),
    })
}

mod leaf {
    use super::*;
    #[derive(ExternalPtr, AllEqual, Clone)]
    #[all_equal(tolerance = 1e-2)]
    pub struct Leaf {
        #[all_equal(label = "Leaf value")]
        value: f64,
    }

    /// A nested class with its own comparison default.
    #[miniextendr(s3, class = "equal_leaf")]
    impl Leaf {
        pub fn new(value: f64) -> Self {
            Self { value }
        }
    }
}
type LeafHandle = ExternalPtr<leaf::Leaf>;

#[derive(ExternalPtr, AllEqual)]
pub struct EqualNested {
    #[all_equal(label = "Model")]
    model: LeafHandle,
}
/// A comparison whose unset arguments preserve the nested method's defaults.
#[miniextendr(s3)]
impl EqualNested {
    pub fn new(value: f64) -> Self {
        Self {
            model: ExternalPtr::new(leaf::Leaf::new(value)),
        }
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct EqualGetter {
    #[all_equal(with = "Self::text")]
    value: String,
    #[all_equal(with = "Self::model", label = "Computed model")]
    scalar: NotClone,
}
impl EqualGetter {
    fn text(&self) -> &str {
        &self.value
    }
    fn model(&self) -> LeafHandle {
        ExternalPtr::new(leaf::Leaf::new(self.scalar.0))
    }
}
/// Getters returning a borrow and a fresh nested pointer.
#[miniextendr(s3)]
impl EqualGetter {
    pub fn new(value: String, scalar: f64) -> Self {
        Self {
            value,
            scalar: NotClone(scalar),
        }
    }
}

#[derive(ExternalPtr, AllEqual)]
#[externalptr(s3, restored(class = "all_equal_saved"))]
pub struct EqualSidecar {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub value: Sidecar<SEXP>,
    #[r_data(get = "Self::computed_value")]
    pub computed: Computed,
    #[all_equal(skip)]
    factor: f64,
}
impl EqualSidecar {
    fn computed_value(&self) -> Vec<f64> {
        vec![self.factor, self.factor * 2.0]
    }
}
/// Compare live sidecar slots and computed fields.
#[miniextendr(s3(r_data_accessors))]
impl EqualSidecar {
    pub fn new(value: SEXP, factor: f64) -> Self {
        Self {
            _r: RSidecar,
            value: Sidecar::new(value),
            computed: Computed,
            factor,
        }
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct EqualTuple(f64, String);
/// Tuple field comparison.
#[miniextendr(s3)]
impl EqualTuple {
    pub fn new(value: f64, text: String) -> Self {
        Self(value, text)
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct EqualUnit;
/// A fieldless type still requires a live Rust value for comparison.
#[miniextendr(s3)]
impl EqualUnit {
    pub fn new() -> Self {
        Self
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct EqualNestedSystems {
    #[all_equal(with = "Self::r6_value")]
    r6: f64,
    #[all_equal(with = "Self::s4_value")]
    s4: f64,
    #[all_equal(with = "Self::s7_value")]
    s7: f64,
}
impl EqualNestedSystems {
    fn r6_value(&self) -> EqualR6 {
        EqualR6::new(self.r6, 2.0, 3.0, 0)
    }
    fn s4_value(&self) -> EqualS4 {
        EqualS4::new(self.s4, 2.0, 3.0, 0)
    }
    fn s7_value(&self) -> EqualS7 {
        EqualS7::new(self.s7, 2.0, 3.0, 0)
    }
}
/// Nested R6, S4 and S7 values retain their registered comparison methods.
#[miniextendr(s3)]
impl EqualNestedSystems {
    pub fn new(value: f64) -> Self {
        Self {
            r6: value,
            s4: value,
            s7: value,
        }
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct EqualSkipped {
    #[all_equal(skip)]
    _cache: std::sync::Mutex<i32>,
}
/// Skipping every field still checks that the pointer is live.
#[miniextendr(s3)]
impl EqualSkipped {
    pub fn new(value: i32) -> Self {
        Self {
            _cache: std::sync::Mutex::new(value),
        }
    }
}
