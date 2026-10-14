//! Borrowed receiver handles over ordinary or erased external-pointer storage.

use super::{ExternalPtr, TypedExternal, borrow_value, borrow_value_mut};
use crate::SEXP;
use core::cell::{Ref, RefCell, RefMut};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

mod sealed {
    pub trait Owner {}
}

/// A handle that borrows a concrete value and keeps its sidecar owner reachable.
///
/// Implemented by [`ExternalPtr`], [`ExternalPtrRef`] and [`ExternalPtrMut`].
/// Generated sidecar getters accept any of these handles. Implementations are
/// sealed because the owner and data must describe the same allocation.
pub trait SidecarOwner<T: TypedExternal + ?Sized>: sealed::Owner + Deref<Target = T> {
    #[doc(hidden)]
    fn __mx_owner(&self) -> SEXP;
}

/// An exclusive receiver handle accepted by generated sidecar setters.
pub trait SidecarOwnerMut<T: TypedExternal + ?Sized>: SidecarOwner<T> + DerefMut {}

/// A shared borrow of a Rust value together with its R sidecar owner.
///
/// Unlike [`ExternalPtr`], this handle never owns or consumes the Rust
/// allocation. It can represent ordinary `Any` storage or erased trait-ABI
/// storage. Its lifetime comes from a live owner, and it cannot cross threads.
/// Use `ExternalPtrRef::from(&ptr)` when calling a handle-receiver trait from
/// Rust. The generated trait ABI constructs the same borrow from its receiver.
///
/// The handle holds a shared borrow of the value's `RefCell` for its whole
/// lifetime. Making one while a running call holds the value mutably raises
/// the `miniextendr_borrow_conflict` R error.
///
/// A borrowed handle cannot outlive its owner:
/// ```compile_fail,E0597
/// use miniextendr_api::{ExternalPtr, ExternalPtrRef, TypedExternal};
/// fn escape<T: TypedExternal>(owner: ExternalPtr<T>) -> ExternalPtrRef<'static, T> {
///     ExternalPtrRef::from(&owner)
/// }
/// ```
pub struct ExternalPtrRef<'a, T: TypedExternal + ?Sized> {
    value: Ref<'a, T>,
    owner: &'a SEXP,
    _main_thread: PhantomData<*mut ()>,
}

/// An exclusive borrow of a Rust value together with its R sidecar owner.
///
/// Use `ExternalPtrMut::from(&mut ptr)` when calling a handle-receiver trait
/// from Rust. Dereferencing gives mutable access to the Rust fields; generated
/// sidecar accessors use the accompanying owner. There is no consuming API.
///
/// The handle holds an exclusive borrow of the value's `RefCell` for its whole
/// lifetime. Making one while a running call holds the value raises the
/// `miniextendr_borrow_conflict` R error.
///
/// A mutable borrow excludes a second borrow of the same handle:
/// ```compile_fail,E0502
/// use miniextendr_api::{ExternalPtr, ExternalPtrMut, ExternalPtrRef, TypedExternal};
/// fn overlap<T: TypedExternal>(owner: &mut ExternalPtr<T>) {
///     let mut first = ExternalPtrMut::from(&mut *owner);
///     let second = ExternalPtrRef::from(&*owner);
///     let _ = (&mut *first, second);
/// }
/// ```
pub struct ExternalPtrMut<'a, T: TypedExternal + ?Sized> {
    value: RefMut<'a, T>,
    owner: &'a SEXP,
    _main_thread: PhantomData<*mut ()>,
}

impl<'a, T: TypedExternal> ExternalPtrRef<'a, T> {
    /// Borrow a receiver after its type and argument aliases have been checked.
    ///
    /// # Safety
    /// R's main thread; `cell` must point to the live `RefCell<T>` owned by
    /// `owner`, whose initialized sidecar slots use `T`'s layout. Its SEXP must
    /// stay rooted for `'a`; the slot reference alone does not root it. The cell
    /// must remain valid for `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw_parts(owner: &'a SEXP, cell: *const RefCell<T>) -> Self {
        Self {
            value: borrow_value(unsafe { &*cell }),
            owner,
            _main_thread: PhantomData,
        }
    }
}

impl<'a, T: TypedExternal> ExternalPtrMut<'a, T> {
    /// Borrow a receiver after its type and argument aliases have been checked.
    ///
    /// # Safety
    /// R's main thread; `cell` must point to the live `RefCell<T>` owned by
    /// `owner`, whose initialized sidecar slots use `T`'s layout. Its SEXP must
    /// stay rooted for `'a`; the slot reference alone does not root it. The cell
    /// must remain valid for `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw_parts(owner: &'a SEXP, cell: *const RefCell<T>) -> Self {
        Self {
            value: borrow_value_mut(unsafe { &*cell }),
            owner,
            _main_thread: PhantomData,
        }
    }
}

impl<'a, T: TypedExternal> From<&'a ExternalPtr<T>> for ExternalPtrRef<'a, T> {
    fn from(ptr: &'a ExternalPtr<T>) -> Self {
        assert!(
            crate::worker::is_r_main_thread(),
            "ExternalPtrRef requires R's main thread"
        );
        // The original handle retains the owner throughout this shared borrow.
        Self {
            value: borrow_value(ptr.cell()),
            owner: &ptr.sexp,
            _main_thread: PhantomData,
        }
    }
}

impl<'a, T: TypedExternal> From<&'a mut ExternalPtr<T>> for ExternalPtrMut<'a, T> {
    fn from(ptr: &'a mut ExternalPtr<T>) -> Self {
        assert!(
            crate::worker::is_r_main_thread(),
            "ExternalPtrMut requires R's main thread"
        );
        // The original handle retains the owner throughout this borrow.
        Self {
            value: borrow_value_mut(ptr.cell()),
            owner: &ptr.sexp,
            _main_thread: PhantomData,
        }
    }
}

impl<T: TypedExternal + ?Sized> Deref for ExternalPtrRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T: TypedExternal + ?Sized> Deref for ExternalPtrMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T: TypedExternal + ?Sized> DerefMut for ExternalPtrMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: TypedExternal> sealed::Owner for ExternalPtr<T> {}
impl<T: TypedExternal + ?Sized> sealed::Owner for ExternalPtrRef<'_, T> {}
impl<T: TypedExternal + ?Sized> sealed::Owner for ExternalPtrMut<'_, T> {}

impl<T: TypedExternal> SidecarOwner<T> for ExternalPtr<T> {
    fn __mx_owner(&self) -> SEXP {
        self.as_sexp()
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwner<T> for ExternalPtrRef<'_, T> {
    fn __mx_owner(&self) -> SEXP {
        *self.owner
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwner<T> for ExternalPtrMut<'_, T> {
    fn __mx_owner(&self) -> SEXP {
        *self.owner
    }
}
impl<T: TypedExternal> SidecarOwnerMut<T> for ExternalPtr<T> {}
impl<T: TypedExternal + ?Sized> SidecarOwnerMut<T> for ExternalPtrMut<'_, T> {}
