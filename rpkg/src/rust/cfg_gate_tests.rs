//! Gates inside `#[miniextendr]` items are resolved before anything is
//! generated (#2107): methods, trait items, parameters and enum variants.
// Constant gates deliberately exercise both branches on every target.
#![allow(clippy::non_minimal_cfg)]

use miniextendr_api::{ExternalPtr, miniextendr};

// region: Inherent methods

macro_rules! gated_class {
    ($name:ident, $system:ident, $visibility:ident $(, $extra:item)*) => {
        #[derive(ExternalPtr)]
        pub struct $name {
            value: i32,
        }

        /// @noRd
        #[miniextendr($system, $visibility)]
        impl $name {
            #[cfg(all())]
            pub fn new(#[cfg(any())] _hidden: MissingType, value: i32) -> Self {
                Self { value }
            }

            #[cfg(any())]
            pub fn new(_value: MissingType) -> Self { unreachable!() }

            #[cfg(all())]
            pub fn cfg_value(&self) -> i32 { self.value }

            #[cfg_attr(any(), miniextendr(no_na(hidden)))]
            pub fn cfg_sum(
                &self,
                #[cfg(any())] #[miniextendr(no_na)] hidden: MissingType,
                #[cfg_attr(all(), miniextendr(no_na))] value: i32,
                #[cfg_attr(all(), cfg(any()))] _nested: MissingType,
                #[cfg_attr(any(), cfg(any()))] kept: i32,
            ) -> i32 { self.value + value + kept }

            #[cfg(any())]
            #[miniextendr($system(worker))]
            pub fn cfg_disabled(_value: MissingType) -> MissingType { unreachable!() }

            #[cfg_attr(all(), cfg_attr(all(), miniextendr(r_name = "cfg_renamed")))]
            pub fn original_name(&self) -> i32 { self.value + 1 }

            #[cfg(all())]
            #[cfg_attr(all(), cfg(any()))]
            pub fn cfg_nested_disabled(&self) -> MissingType { unreachable!() }

            #[cfg(feature = "vctrs")]
            pub fn cfg_feature(&self) -> bool { true }

            #[cfg(not(feature = "vctrs"))]
            pub fn cfg_feature(&self) -> bool { false }

            $($extra)*
        }
    };
}

gated_class!(GatedMethodEnv, env, noexport);
gated_class!(GatedMethodR6, r6, noexport);
gated_class!(GatedMethodS3, s3, internal);
gated_class!(GatedMethodS4, s4, noexport);
gated_class!(
    GatedMethodS7,
    s7,
    noexport,
    #[cfg(all())]
    #[miniextendr(s7(getter, prop = "current"))]
    pub fn current(&self) -> i32 {
        self.value
    },
    #[cfg_attr(all(), miniextendr(s7(setter, prop = "current")))]
    pub fn set_current(&mut self, value: i32) {
        self.value = value;
    }
);

#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;
    pub struct GatedMethodVctrs;

    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "cfg"), noexport)]
    impl GatedMethodVctrs {
        #[cfg(all())]
        #[allow(clippy::new_ret_no_self)]
        pub fn new(#[cfg(any())] _hidden: MissingType, values: Vec<f64>) -> Vec<f64> {
            values
        }

        #[cfg(all())]
        pub fn cfg_value(values: Vec<f64>) -> f64 {
            values[0]
        }

        pub fn cfg_sum(
            values: Vec<f64>,
            #[cfg(any())] _hidden: MissingType,
            #[cfg_attr(all(), miniextendr(no_na))] value: i32,
            #[cfg_attr(all(), cfg(any()))] _nested: MissingType,
            #[cfg_attr(any(), cfg(any()))] kept: i32,
        ) -> f64 {
            values[0] + f64::from(value + kept)
        }

        #[cfg_attr(all(), miniextendr(r_name = "cfg_renamed"))]
        pub fn original_name(values: Vec<f64>) -> f64 {
            values[0] + 1.0
        }

        #[cfg(any())]
        pub fn cfg_disabled(_values: MissingType) -> MissingType {
            unreachable!()
        }

        #[cfg(feature = "vctrs")]
        pub fn cfg_feature(_values: Vec<f64>) -> bool {
            true
        }
    }
}
// endregion

// region: Parameters, enum variants and traits

/// A gated-off parameter has no R formal.
/// @param kept Returned unchanged.
/// @noRd
#[miniextendr(noexport)]
pub fn cfg_gate_params(
    #[cfg(any())] _hidden: MissingType,
    kept: i32,
    #[cfg_attr(all(), cfg(any()))] _nested: MissingType,
) -> i32 {
    kept
}

/// Factor whose gated-off variant is not a level.
#[miniextendr]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CfgGateLevel {
    Low,
    #[cfg(any())]
    Hidden,
    High,
}

/// @noRd
#[miniextendr(noexport)]
pub fn cfg_gate_level_high() -> CfgGateLevel {
    CfgGateLevel::High
}

/// Choices whose gated-off variant is not offered.
#[miniextendr(match_arg)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CfgGateMode {
    Fast,
    #[cfg_attr(all(), cfg(any()))]
    Hidden,
    Slow,
}

/// @param mode A mode.
/// @noRd
#[miniextendr(noexport)]
pub fn cfg_gate_mode(#[miniextendr(match_arg)] mode: CfgGateMode) -> String {
    format!("{mode:?}")
}

/// Trait with a gated-off method.
#[miniextendr]
pub trait CfgGateShape {
    /// The shape's area.
    fn area(&self, #[cfg(any())] _hidden: MissingType) -> f64;

    /// A default whose ABI signature contains only its enabled parameter.
    fn shifted(
        &self,
        #[cfg_attr(all(), cfg_attr(all(), cfg(any())))] _hidden: MissingType,
        #[cfg(all())] value: f64,
    ) -> f64 {
        self.area() + value
    }

    /// Gated off.
    #[cfg(any())]
    fn hidden(&self) -> MissingType;
}

#[derive(ExternalPtr)]
pub struct CfgGateSquare {
    side: f64,
}

/// @noRd
#[miniextendr(env, noexport)]
impl CfgGateSquare {
    pub fn new(side: f64) -> Self {
        Self { side }
    }
}

/// One of two gated alternative implementations is compiled.
#[miniextendr(env, noexport)]
impl CfgGateShape for CfgGateSquare {
    #[cfg(all())]
    fn area(&self, #[cfg(any())] _hidden: MissingType) -> f64 {
        self.side * self.side
    }

    #[cfg(any())]
    fn area(&self) -> f64 {
        unreachable!()
    }
}
// endregion

// region: Gated parameters in generic and empty trait impls

#[miniextendr]
pub trait CfgGateGeneric<T> {
    fn seed(&self) -> T;
    fn echo(&self, #[cfg(any())] _hidden: MissingType, value: T) -> T {
        value
    }
}

#[miniextendr(env, noexport)]
impl CfgGateGeneric<i32> for CfgGateSquare {
    fn seed(&self) -> i32 {
        1
    }
}

#[miniextendr]
pub trait CfgGateDefaults {
    fn echo(&self, #[cfg(any())] _hidden: MissingType, value: i32) -> i32 {
        value
    }
}

impl CfgGateDefaults for CfgGateSquare {}

#[miniextendr(env, noexport)]
impl CfgGateDefaults for CfgGateSquare {}

/// Dispatch through the trait ABI, including its gated default signature.
/// @noRd
#[miniextendr(noexport)]
pub fn cfg_gate_shape_view(side: f64, value: f64) -> f64 {
    unsafe {
        let erased = __mx_wrap_CfgGateSquare(CfgGateSquare { side });
        let sexp = miniextendr_api::gc_protect::OwnedProtect::new(
            miniextendr_api::trait_abi::ccall::mx_wrap(erased),
        );
        let view = CfgGateShapeView::from_sexp(sexp.get());
        view.shifted(value)
    }
}
// endregion
