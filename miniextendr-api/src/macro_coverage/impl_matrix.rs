#![allow(dead_code)]

//! Class system impl block coverage.
//!
//! Exercises `#[miniextendr]` on impl blocks for each class system:
//! env, r6, s3, s4, s7 (including S7 properties and fallback).

use crate::miniextendr;

// region: Env-style class

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovEnv {
    pub(crate) v: i32,
}

#[miniextendr(env)]
impl CovEnv {
    pub fn new(v: i32) -> Self {
        Self { v }
    }

    pub fn get(&self) -> i32 {
        self.v
    }
}
// endregion

// region: R6-style class

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovR6 {
    v: i32,
}

#[miniextendr(r6)]
impl CovR6 {
    pub fn new(v: i32) -> Self {
        Self { v }
    }

    pub fn get(&self) -> i32 {
        self.v
    }

    #[miniextendr(r6(active))]
    pub fn value(&self) -> i32 {
        self.v
    }
}
// endregion

// region: S3-style class

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovS3 {
    v: i32,
}

#[miniextendr(s3)]
impl CovS3 {
    pub fn new(v: i32) -> Self {
        Self { v }
    }

    pub fn s3_get(&self) -> i32 {
        self.v
    }
}
// endregion

// region: S4-style class

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovS4 {
    v: i32,
}

#[miniextendr(s4)]
impl CovS4 {
    pub fn new(v: i32) -> Self {
        Self { v }
    }

    pub fn s4_get(&self) -> i32 {
        self.v
    }
}
// endregion

// region: S7-style class (with property patterns)

#[derive(miniextendr_api::ExternalPtr)]
pub struct CovS7 {
    v: i32,
}

#[miniextendr(s7)]
impl CovS7 {
    pub fn new(v: i32) -> Self {
        Self { v }
    }

    #[miniextendr(s7(getter))]
    pub fn value(&self) -> i32 {
        self.v
    }

    #[miniextendr(s7(no_dots))]
    pub fn strict(&self) -> i32 {
        self.v
    }

    #[miniextendr(s7(fallback))]
    pub fn describe_cov(&self) -> String {
        format!("CovS7({})", self.v)
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the generated wrapper's data_offset is correct for
    /// the default-aligned CovEnv type.
    #[test]
    fn test_data_offset_default_aligned() {
        let wrapped = __mx_wrap_covenv(CovEnv { v: 42 });
        let base = unsafe { (*wrapped).base };
        let data_offset = unsafe { (*base).data_offset };

        // For default-aligned types, offset should equal size_of::<mx_erased>()
        assert_eq!(data_offset, std::mem::size_of::<crate::abi::mx_erased>());

        // Verify we can read the data correctly through the offset
        let data_ptr = unsafe {
            wrapped
                .cast::<u8>()
                .add(data_offset)
                .cast::<CovEnv>()
                .cast_const()
        };
        let value = unsafe { (*data_ptr).v };
        assert_eq!(value, 42);

        // Clean up
        unsafe {
            let drop_fn = (*base).drop;
            drop_fn(wrapped);
        }
    }
}
// endregion
