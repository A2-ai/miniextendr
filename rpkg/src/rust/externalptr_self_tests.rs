//! Tests for receivers that reach the R object: the views
//! `self: ExternalPtrRef<'_, Self>` / `self: ExternalPtrMut<'_, Self>`, which
//! the wrapper borrows for the call, and the handle `self: ExternalPtr<Self>`,
//! which it doesn't.

use miniextendr_api::externalptr::{ExternalPtr, ExternalPtrMut, ExternalPtrRef};
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

    /// Handle self — checks the pointer state.
    pub fn is_null_ptr(self: ExternalPtr<Self>) -> bool {
        self.is_null()
    }

    /// Shared view self — reads the value through `Deref`.
    pub fn value_via_ptr(self: ExternalPtrRef<'_, Self>) -> i32 {
        self.value
    }

    /// Exclusive view self — writes the value through `DerefMut`.
    /// @param new_val The new value.
    pub fn set_value_via_ptr(mut self: ExternalPtrMut<'_, Self>, new_val: i32) {
        self.value = new_val;
    }

    /// Handle self — reads the value through a checked `borrow()`.
    pub fn value_owned_ptr(self: ExternalPtr<Self>) -> i32 {
        self.borrow().value
    }
}
