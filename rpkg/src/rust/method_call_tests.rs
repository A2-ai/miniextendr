//! Call suppression for class and trait methods (#1868).
use miniextendr_api::{ExternalPtr, miniextendr};

fn probe(value: i32) -> Result<i32, String> {
    match value {
        -1 => Err("method result error".to_string()),
        0 => {
            miniextendr_api::defer_warning!("method deferred warning");
            Ok(value)
        }
        1 => panic!("method panic"),
        2 => miniextendr_api::error!(class = "method_call_error", "method classed error"),
        3 => miniextendr_api::warning!("method immediate warning"),
        4 => {
            miniextendr_api::defer_message!("method deferred message");
            Ok(value)
        }
        _ => Ok(value),
    }
}

macro_rules! call_class {
    ($type:ident, $system:ident, $read:ident, $plain:ident, $quiet:ident $(, { $($extra:tt)* })?) => {
        #[derive(ExternalPtr)]
        pub struct $type {
            value: i32,
        }

        /// Call-attribution regression fixture.
        #[miniextendr($system, internal, call = none)]
        impl $type {
            /// Create a fixture, checking its initial value.
            /// @param value Integer condition selector.
            pub fn new(value: i32) -> Self {
                Self {
                    value: probe(value).unwrap_or_else(|error| panic!("{error}")),
                }
            }

            /// Exercise body conditions and argument checks.
            /// @param value Integer condition selector.
            /// @param mode One of the supported modes.
            #[miniextendr(choices(mode = "fast, safe"))]
            pub fn $read(&self, value: i32, mode: &str) -> Result<i32, String> {
                let _ = (self.value, mode);
                probe(value)
            }

            /// Exercise static calls.
            /// @param value Integer condition selector.
            pub fn static_probe(value: i32) -> Result<i32, String> {
                probe(value)
            }
            $($($extra)*)?
        }

        // A second block must keep its own default policy.
        #[miniextendr($system, internal)]
        impl $type {
            /// A method retaining default call attribution.
            /// @param value Integer condition selector.
            pub fn $plain(&self, value: i32) -> Result<i32, String> {
                probe(value)
            }

            /// Suppress calls on just this method.
            /// @param value Integer condition selector.
            #[miniextendr(call = none)]
            pub fn $quiet(&self, value: i32) -> Result<i32, String> {
                probe(value)
            }
        }
    };
}

call_class!(NoCallEnv, env, nc_env_read, nc_env_plain, nc_env_quiet);
call_class!(NoCallR6, r6, nc_r6_read, nc_r6_plain, nc_r6_quiet);
call_class!(NoCallS3, s3, nc_s3_read, nc_s3_plain, nc_s3_quiet);
call_class!(NoCallS4, s4, nc_s4_read, nc_s4_plain, nc_s4_quiet);
call_class!(NoCallS7, s7, nc_s7_read, nc_s7_plain, nc_s7_quiet, {
    /// Read the current value through an S7 property.
    #[miniextendr(s7(getter, prop = "reading"))]
    pub fn property_reading(&self) -> Result<i32, String> {
        probe(self.value)
    }

    /// Set the current value through an S7 property.
    /// @param value Integer condition selector.
    #[miniextendr(s7(setter, prop = "reading"))]
    pub fn set_reading(&mut self, value: i32) -> Result<(), String> {
        self.value = probe(value)?;
        Ok(())
    }

    /// Store a selector without invoking the property callbacks.
    /// @param value Integer condition selector.
    pub fn store_raw(&mut self, value: i32) {
        self.value = value;
    }

    /// Check the S7 property's validator callback.
    /// @param value Integer condition selector.
    #[miniextendr(s7(validate, prop = "reading"))]
    pub fn validate_reading(value: i32) -> Option<Vec<String>> {
        if value == 9 {
            panic!("validator error");
        }
        None
    }
});

/// A populated trait impl, including a constant whose conversion raises.
#[miniextendr]
pub trait NoCallTrait {
    const FAILURE: FailingConstant;
    fn trait_probe(&self, value: i32) -> i32;
    fn trait_static(value: i32) -> Result<i32, String>;
}

pub struct FailingConstant;
impl miniextendr_api::IntoR for FailingConstant {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<miniextendr_api::SEXP, Self::Error> {
        panic!("constant conversion")
    }
}

#[miniextendr(env, internal, call = none)]
impl NoCallTrait for NoCallEnv {
    const FAILURE: FailingConstant = FailingConstant;
    fn trait_probe(&self, value: i32) -> i32 {
        probe(value).unwrap_or_else(|error| panic!("{error}"))
    }
    fn trait_static(value: i32) -> Result<i32, String> {
        probe(value)
    }
}

/// A blanket implementation exposed through empty-impl metadata.
#[miniextendr]
pub trait NoCallInherited {
    fn inherited_probe(&self, value: i32) -> i32 {
        probe(value).unwrap_or_else(|error| panic!("{error}"))
    }
}
impl<T> NoCallInherited for T {}
#[miniextendr(env, internal, call = none)]
impl NoCallInherited for NoCallEnv {}

// Constructor is emitted by call_class!, which the source lint cannot expand.
// mxl::allow(MXL121)
#[miniextendr(r6, internal)]
impl NoCallR6 {
    /// Active getter keeps its default call policy.
    #[miniextendr(r6(active))]
    pub fn reading(&self) -> i32 {
        self.value
    }

    /// Active setter suppresses both R argument errors and Rust conditions.
    /// @param value Integer condition selector.
    #[miniextendr(r6(setter, prop = "reading"), call = none)]
    pub fn set_reading(&mut self, value: i32) -> Result<(), String> {
        self.value = probe(value)?;
        Ok(())
    }

    /// Exercise condition transport through the private finalizer callback.
    #[miniextendr(r6(finalize), call = none)]
    pub fn finish(&mut self) {
        if self.value == 13 {
            panic!("finalizer error");
        }
    }

    /// Exercise the private deep-clone callback's condition transport.
    /// @param name Field name.
    /// @param value Field value.
    #[miniextendr(r6(deep_clone), call = none)]
    pub fn clone_field(&self, name: &str, value: miniextendr_api::SEXP) -> miniextendr_api::SEXP {
        let _ = (name, value);
        panic!("deep clone error")
    }
}

#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;
    pub struct NoCallVctrs;

    /// Vector-payload constructor and static-method call suppression.
    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "nc"), internal, call = none)]
    impl NoCallVctrs {
        /// Build a one-element vector.
        /// @param value Integer condition selector.
        #[allow(clippy::new_ret_no_self)]
        pub fn new(value: i32) -> Result<Vec<f64>, String> {
            probe(value).map(|v| vec![f64::from(v)])
        }

        /// Exercise a vctrs static method.
        /// @param value Integer condition selector.
        pub fn static_probe(value: i32) -> Result<i32, String> {
            probe(value)
        }
    }
}
