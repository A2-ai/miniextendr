#![allow(dead_code)]

//! Trait ABI coverage.
//!
//! Exercises `#[miniextendr]` on trait definitions and trait impls.
//! Both paths have separate expansion logic (vtable generation).

use super::impl_matrix::CovEnv;
use crate::{miniextendr, miniextendr_module};

#[miniextendr]
pub trait CovTrait {
    fn bump(&mut self);
    fn cov_trait_value(&self) -> i32;
}

#[miniextendr]
impl CovTrait for CovEnv {
    fn bump(&mut self) {
        self.v += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.v
    }
}

// =============================================================================
// Over-aligned types for alignment regression tests
// =============================================================================

/// Type with 32-byte alignment -- exposes alignment padding bugs in data
/// pointer computation.
#[repr(align(32))]
#[derive(crate::ExternalPtr)]
pub struct Aligned32Counter {
    pub value: i32,
}

#[miniextendr]
impl CovTrait for Aligned32Counter {
    fn bump(&mut self) {
        self.value += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.value
    }
}

/// Type with 64-byte alignment (cache-line aligned).
#[repr(align(64))]
#[derive(crate::ExternalPtr)]
pub struct Aligned64Counter {
    pub value: i32,
}

#[miniextendr]
impl CovTrait for Aligned64Counter {
    fn bump(&mut self) {
        self.value += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.value
    }
}

miniextendr_module! {
    mod trait_abi_matrix;

    impl CovTrait for CovEnv;
    impl CovTrait for Aligned32Counter;
    impl CovTrait for Aligned64Counter;
}

// =============================================================================
// Tests
// =============================================================================

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
        assert_eq!(
            data_offset,
            std::mem::size_of::<crate::abi::mx_erased>()
        );

        // Verify we can read the data correctly through the offset
        let data_ptr = unsafe { (wrapped as *mut u8).add(data_offset) as *const CovEnv };
        let value = unsafe { (*data_ptr).v };
        assert_eq!(value, 42);

        // Clean up
        unsafe {
            let drop_fn = (*base).drop;
            drop_fn(wrapped);
        }
    }

    /// Verify that the generated wrapper's data_offset is correct for
    /// a 32-byte-aligned type. This catches the bug where `size_of::<mx_erased>()`
    /// was used directly (which is 8 on 64-bit, but the data field starts at 32).
    #[test]
    fn test_data_offset_align32() {
        let wrapped = __mx_wrap_aligned32counter(Aligned32Counter { value: 99 });
        let base = unsafe { (*wrapped).base };
        let data_offset = unsafe { (*base).data_offset };

        // For align(32) types on 64-bit, offset should be 32 (not 8)
        assert_eq!(data_offset, 32);
        assert_ne!(
            data_offset,
            std::mem::size_of::<crate::abi::mx_erased>(),
            "data_offset must NOT equal size_of::<mx_erased>() for over-aligned types"
        );

        // Verify we can read the data correctly
        let data_ptr = unsafe { (wrapped as *mut u8).add(data_offset) as *const Aligned32Counter };
        let value = unsafe { (*data_ptr).value };
        assert_eq!(value, 99);

        // Verify vtable query succeeds (non-null = trait is implemented)
        let vtable = unsafe {
            ((*base).query)(wrapped, TAG_COVTRAIT)
        };
        assert!(!vtable.is_null());

        unsafe {
            let drop_fn = (*base).drop;
            drop_fn(wrapped);
        }
    }

    /// Verify that the generated wrapper's data_offset is correct for
    /// a 64-byte-aligned type.
    #[test]
    fn test_data_offset_align64() {
        let wrapped = __mx_wrap_aligned64counter(Aligned64Counter { value: 77 });
        let base = unsafe { (*wrapped).base };
        let data_offset = unsafe { (*base).data_offset };

        // For align(64) types, offset should be 64
        assert_eq!(data_offset, 64);

        // Verify we can read the data correctly
        let data_ptr = unsafe { (wrapped as *mut u8).add(data_offset) as *const Aligned64Counter };
        let value = unsafe { (*data_ptr).value };
        assert_eq!(value, 77);

        unsafe {
            let drop_fn = (*base).drop;
            drop_fn(wrapped);
        }
    }
}
