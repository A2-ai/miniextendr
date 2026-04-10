//! Tests for ExternalPtr-based self receivers:
//! `self: &ExternalPtr<Self>`, `self: &mut ExternalPtr<Self>`, and `self: ExternalPtr<Self>`.

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;

/// A test struct for ExternalPtr self-receiver methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PtrSelfTest {
    value: i32,
}

/// @name rpkg_externalptr_self
/// @aliases PtrSelfTest
#[miniextendr(env)]
impl PtrSelfTest {
    /// Create a new PtrSelfTest.
    /// @param value Integer value.
    pub fn new(value: i32) -> Self {
        PtrSelfTest { value }
    }

    /// Regular &self method — returns the stored value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// ExternalPtr self — can check pointer state.
    pub fn is_null_ptr(self: &ExternalPtr<Self>) -> bool {
        self.is_null()
    }

    /// ExternalPtr self — access inner value via Deref.
    pub fn value_via_ptr(self: &ExternalPtr<Self>) -> i32 {
        self.value
    }

    /// Mutable ExternalPtr self — modify inner value via DerefMut.
    pub fn set_value_via_ptr(self: &mut ExternalPtr<Self>, new_val: i32) {
        self.value = new_val;
    }

    // Commented out: consuming self via ExternalPtr is not supported by #[miniextendr]
    // (lint error: "takes self by value"). This test fixture was added to verify
    // the error message but blocks compilation. Tracked separately.
    // pub fn value_owned_ptr(self: ExternalPtr<Self>) -> i32 {
    //     self.value
    // }
}
