//! Method gates are resolved before class analysis and R registration (#2107).
// Constant gates deliberately exercise both branches on every target.
#![allow(clippy::non_minimal_cfg)]

use miniextendr_api::{ExternalPtr, miniextendr};

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
            pub fn new(value: i32) -> Self {
                Self { value }
            }

            #[cfg(any())]
            pub fn new(_value: MissingType) -> Self { unreachable!() }

            #[cfg(all())]
            pub fn cfg_value(&self) -> i32 { self.value }

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
        pub fn new(values: Vec<f64>) -> Vec<f64> {
            values
        }

        #[cfg(all())]
        pub fn cfg_value(values: Vec<f64>) -> f64 {
            values[0]
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
