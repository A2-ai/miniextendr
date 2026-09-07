//! Regression fixtures for typed NA scalar Option returns from class methods (#1415).

use miniextendr_api::{ExternalPtr, miniextendr};

/// Scalar Option return regression fixture (Env).
#[derive(ExternalPtr, Default)]
pub struct ScalarOptionEnv;

#[miniextendr(env, internal, strict)]
impl ScalarOptionEnv {
    pub fn new() -> Self {
        Self
    }
    pub fn optional_integer(&self, present: bool) -> Option<i32> {
        present.then_some(42)
    }
    pub fn optional_real(&self, present: bool) -> Option<f64> {
        present.then_some(2.5)
    }
    pub fn optional_logical(&self, present: bool) -> Option<bool> {
        present.then_some(true)
    }
    pub fn optional_string(&self, present: bool) -> Option<String> {
        present.then(|| "owned".to_owned())
    }
    pub fn optional_str(&self, present: bool) -> Option<&'static str> {
        present.then_some("borrowed")
    }
    pub fn optional_unit(&self, present: bool) -> Option<()> {
        present.then_some(())
    }
    pub fn optional_self(present: bool) -> Option<Self> {
        present.then_some(Self)
    }
    pub fn optional_i8(&self, present: bool) -> Option<i8> {
        present.then_some(42)
    }
    pub fn optional_i16(&self, present: bool) -> Option<i16> {
        present.then_some(42)
    }
    pub fn optional_u16(&self, present: bool) -> Option<u16> {
        present.then_some(42)
    }
    pub fn optional_u32(&self, present: bool) -> Option<u32> {
        present.then_some(42)
    }
    pub fn optional_f32(&self, present: bool) -> Option<f32> {
        present.then_some(2.5)
    }
    pub fn optional_i64(&self, present: bool) -> Option<i64> {
        present.then_some(42)
    }
    pub fn optional_u64(&self, present: bool) -> Option<u64> {
        present.then_some(42)
    }
    pub fn optional_isize(&self, present: bool) -> Option<isize> {
        present.then_some(42)
    }
    pub fn optional_usize(&self, present: bool) -> Option<usize> {
        present.then_some(42)
    }
    pub fn optional_rboolean(&self, present: bool) -> Option<miniextendr_api::Rboolean> {
        present.then_some(miniextendr_api::Rboolean::TRUE)
    }
    pub fn optional_rlogical(&self, present: bool) -> Option<miniextendr_api::RLogical> {
        present.then_some(miniextendr_api::RLogical::TRUE)
    }
    pub fn optional_complex(&self, present: bool) -> Option<miniextendr_api::Rcomplex> {
        present.then_some(miniextendr_api::Rcomplex { r: 2.0, i: 3.0 })
    }
    pub fn optional_path(&self, present: bool) -> Option<std::path::PathBuf> {
        present.then(|| std::path::PathBuf::from("path"))
    }
    pub fn optional_os_string(&self, present: bool) -> Option<std::ffi::OsString> {
        present.then(|| std::ffi::OsString::from("os"))
    }
    pub fn optional_strict(present: bool, large: bool) -> Option<i64> {
        present.then_some(if large { i64::MAX } else { 42 })
    }
    #[miniextendr(env(worker))]
    pub fn optional_worker(present: bool) -> Option<i32> {
        present.then_some(42)
    }
    #[miniextendr(env(worker))]
    pub fn optional_worker_strict(present: bool, large: bool) -> Option<i64> {
        present.then_some(if large { i64::MAX } else { 42 })
    }
}

/// Scalar Option return regression fixture (R6).
#[derive(ExternalPtr, Default)]
pub struct ScalarOptionR6;

#[miniextendr(r6, internal)]
impl ScalarOptionR6 {
    pub fn new() -> Self {
        Self
    }
    pub fn optional_integer(&self, present: bool) -> Option<i32> {
        present.then_some(42)
    }
    pub fn optional_real(&self, present: bool) -> Option<f64> {
        present.then_some(2.5)
    }
    pub fn optional_logical(&self, present: bool) -> Option<bool> {
        present.then_some(true)
    }
    pub fn optional_string(&self, present: bool) -> Option<String> {
        present.then(|| "owned".to_owned())
    }
    pub fn optional_str(&self, present: bool) -> Option<&'static str> {
        present.then_some("borrowed")
    }
    pub fn optional_unit(&self, present: bool) -> Option<()> {
        present.then_some(())
    }
    pub fn optional_self(present: bool) -> Option<Self> {
        present.then_some(Self)
    }
}

/// Scalar Option return regression fixture (S3).
#[derive(ExternalPtr, Default)]
pub struct ScalarOptionS3;

#[miniextendr(s3, internal)]
impl ScalarOptionS3 {
    pub fn new() -> Self {
        Self
    }
    pub fn optional_integer(&self, present: bool) -> Option<i32> {
        present.then_some(42)
    }
    pub fn optional_real(&self, present: bool) -> Option<f64> {
        present.then_some(2.5)
    }
    pub fn optional_logical(&self, present: bool) -> Option<bool> {
        present.then_some(true)
    }
    pub fn optional_string(&self, present: bool) -> Option<String> {
        present.then(|| "owned".to_owned())
    }
    pub fn optional_str(&self, present: bool) -> Option<&'static str> {
        present.then_some("borrowed")
    }
    pub fn optional_unit(&self, present: bool) -> Option<()> {
        present.then_some(())
    }
    pub fn optional_self(present: bool) -> Option<Self> {
        present.then_some(Self)
    }
}

/// Scalar Option return regression fixture (S4).
#[derive(ExternalPtr, Default)]
pub struct ScalarOptionS4;

#[miniextendr(s4, internal)]
impl ScalarOptionS4 {
    pub fn new() -> Self {
        Self
    }
    pub fn optional_integer(&self, present: bool) -> Option<i32> {
        present.then_some(42)
    }
    pub fn optional_real(&self, present: bool) -> Option<f64> {
        present.then_some(2.5)
    }
    pub fn optional_logical(&self, present: bool) -> Option<bool> {
        present.then_some(true)
    }
    pub fn optional_string(&self, present: bool) -> Option<String> {
        present.then(|| "owned".to_owned())
    }
    pub fn optional_str(&self, present: bool) -> Option<&'static str> {
        present.then_some("borrowed")
    }
    pub fn optional_unit(&self, present: bool) -> Option<()> {
        present.then_some(())
    }
    pub fn optional_self(present: bool) -> Option<Self> {
        present.then_some(Self)
    }
}

/// Scalar Option return regression fixture (S7).
#[derive(ExternalPtr, Default)]
pub struct ScalarOptionS7;

#[miniextendr(s7, internal)]
impl ScalarOptionS7 {
    pub fn new() -> Self {
        Self
    }
    pub fn optional_integer(&self, present: bool) -> Option<i32> {
        present.then_some(42)
    }
    pub fn optional_real(&self, present: bool) -> Option<f64> {
        present.then_some(2.5)
    }
    pub fn optional_logical(&self, present: bool) -> Option<bool> {
        present.then_some(true)
    }
    pub fn optional_string(&self, present: bool) -> Option<String> {
        present.then(|| "owned".to_owned())
    }
    pub fn optional_str(&self, present: bool) -> Option<&'static str> {
        present.then_some("borrowed")
    }
    pub fn optional_unit(&self, present: bool) -> Option<()> {
        present.then_some(())
    }
    pub fn optional_self(present: bool) -> Option<Self> {
        present.then_some(Self)
    }
}

#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;

    /// Scalar Option return regression fixture with vctrs storage.
    pub struct ScalarOptionVctrs;

    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "opt"), internal)]
    impl ScalarOptionVctrs {
        #[allow(clippy::new_ret_no_self)]
        pub fn new(values: Vec<f64>) -> Vec<f64> {
            values
        }
        pub fn optional_integer(_values: Vec<f64>, present: bool) -> Option<i32> {
            present.then_some(42)
        }
        pub fn optional_real(_values: Vec<f64>, present: bool) -> Option<f64> {
            present.then_some(2.5)
        }
        pub fn optional_logical(_values: Vec<f64>, present: bool) -> Option<bool> {
            present.then_some(true)
        }
        pub fn optional_string(_values: Vec<f64>, present: bool) -> Option<String> {
            present.then(|| "owned".to_owned())
        }
        pub fn optional_str(_values: Vec<f64>, present: bool) -> Option<&'static str> {
            present.then_some("borrowed")
        }
        pub fn optional_unit(_values: Vec<f64>, present: bool) -> Option<()> {
            present.then_some(())
        }
    }
}
