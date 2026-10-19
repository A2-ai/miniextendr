//! Borrowed handles over ordinary or erased external-pointer storage.

use super::{TypedExternal, borrow_value, borrow_value_mut};
use crate::SEXP;
use core::cell::{Ref, RefCell, RefMut};
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

mod sealed {
    pub trait Owner {}
}

/// A handle that reaches a concrete value together with its R object, for
/// the sidecar accessors.
///
/// Implemented by [`ExternalPtr`](super::ExternalPtr), [`ExternalPtrRef`]
/// and [`ExternalPtrMut`]. Generated sidecar getters accept any of these
/// handles: an `ExternalPtr` borrows its value for the read, a view already
/// holds the borrow. Implementations are sealed because the owner and data
/// must describe the same allocation.
pub trait SidecarOwner<T: TypedExternal + ?Sized>: sealed::Owner {
    /// The value, borrowed shared, with its R object; or, for a [`Saved`]
    /// handle, the restored pointer alone.
    #[doc(hidden)]
    fn __mx_read(&self) -> SidecarRead<'_, T>;
}

/// What a [`SidecarOwner`] gives the generated getters.
#[doc(hidden)]
pub enum SidecarRead<'a, T: TypedExternal + ?Sized> {
    /// A live value, whose pending `Sidecar` values the getter flushes first.
    Live(ExternalPtrRef<'a, T>),
    /// A pointer restored from a saved session: its slots hold the saved
    /// values, and there is no value yet.
    Saved(SEXP),
}

/// What a save of a `T` kept, handed to the type's rebuild hook
/// (`#[externalptr(rebuild = path)]`) when a pointer restored from a saved
/// session is first used.
///
/// The type's generated `Sidecar` getters read the saved values through it,
/// as through a live handle: `Self::keys(saved)`. Only `Sidecar` fields have
/// a slot in the save; a struct field or a `Computed` field is gone with the
/// Rust value, so a type whose rebuild needs one keeps it in a `Sidecar`.
///
/// The save names its slots after the fields, so a field is read by name
/// whatever fields the version that saved it had (#2187). A field the save
/// lacks (added since) reads as `None` through the generated
/// `Self::try_keys(saved)`; the plain getter raises the classed error
/// [`RESTORED_MISSING_SLOT_CLASS`](super::RESTORED_MISSING_SLOT_CLASS).
pub struct Saved<'a, T: TypedExternal + ?Sized> {
    sexp: SEXP,
    _marker: PhantomData<&'a T>,
}

impl<T: TypedExternal + ?Sized> Saved<'_, T> {
    /// The handle over `sexp`, a pointer for a `T` restored without its
    /// value.
    ///
    /// # Safety
    ///
    /// `sexp` must be a live `EXTPTRSXP` built by `ExternalPtr` for a `T` by
    /// some version of its crate, rooted for the handle's lifetime.
    pub(crate) unsafe fn new(sexp: SEXP) -> Self {
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// The restored external pointer.
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Whether the save has a slot for the `Sidecar` field `field` (its Rust
    /// name). The generated `Self::try_<field>(saved)` getters say the same
    /// with the field's type, checked by the compiler.
    pub fn has(&self, field: &str) -> bool {
        // SAFETY: on R's main thread (a hook runs there); the pointer is live.
        let prot = unsafe { crate::sys::R_ExternalPtrProtected(self.sexp) };
        T::SIDECAR_NAMES
            .iter()
            .position(|name| *name == field)
            .is_some_and(|index| super::sidecar::saved_slot(prot, index, field).is_some())
    }

    /// The version of the crate that saved the object, from the type ID the
    /// save recorded: the current version for a save this version made, an
    /// earlier (or later) one under `any_version`.
    pub fn version(&self) -> Option<&'static str> {
        // SAFETY: on R's main thread; the pointer is live.
        let sym = unsafe { super::stored_type_id_symbol(self.sexp) }?;
        super::split_type_id(super::symbol_name(sym)).1
    }
}

impl<T: TypedExternal + ?Sized> fmt::Debug for Saved<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Saved")
            .field("type", &T::TYPE_NAME)
            .finish()
    }
}

/// A handle accepted by generated sidecar setters: an
/// [`ExternalPtr`](super::ExternalPtr), which borrows its value exclusively
/// for the write, or an [`ExternalPtrMut`].
pub trait SidecarOwnerMut<T: TypedExternal + ?Sized>: SidecarOwner<T> {
    /// The value, borrowed exclusively, with its R object.
    #[doc(hidden)]
    fn __mx_write(&mut self) -> ExternalPtrMut<'_, T>;
}

/// Where a shared view gets its value: the `RefCell` borrow it holds, or a
/// view it was reborrowed from.
enum Shared<'a, T: ?Sized> {
    Guard(Ref<'a, T>),
    Reborrow(&'a T),
}

/// Where an exclusive view gets its value, like [`Shared`].
enum Exclusive<'a, T: ?Sized> {
    Guard(RefMut<'a, T>),
    Reborrow(&'a mut T),
}

/// A shared borrow of a Rust value together with its R object.
///
/// [`ExternalPtr::borrow`](super::ExternalPtr::borrow) makes one, and a
/// `#[miniextendr]` function or method takes one as a parameter (or as
/// `self: ExternalPtrRef<'_, Self>`) when it needs the R object as well as
/// the value: the sidecar accessors, the object's identity
/// ([`ExternalPtrRef::as_sexp`]). A body that needs only the value takes
/// `&T`.
///
/// The view holds a shared borrow of the value's `RefCell` for its whole
/// lifetime: a method of the same object that needs it mutably, a `&mut T`
/// argument holding it, or [`ExternalPtr::borrow_mut`](super::ExternalPtr::borrow_mut)
/// meanwhile raises the R error of class `miniextendr_borrow_conflict`. It
/// never owns or consumes the value, can represent ordinary `Any` storage or
/// erased trait-ABI storage, and cannot cross threads. Pass it on without
/// moving it with [`rb`](Self::rb).
///
/// A borrowed handle cannot outlive its owner:
/// ```compile_fail,E0515
/// use miniextendr_api::{ExternalPtr, ExternalPtrRef, TypedExternal};
/// fn escape<T: TypedExternal>(owner: ExternalPtr<T>) -> ExternalPtrRef<'static, T> {
///     owner.borrow()
/// }
/// ```
pub struct ExternalPtrRef<'a, T: TypedExternal + ?Sized> {
    value: Shared<'a, T>,
    owner: &'a SEXP,
    _main_thread: PhantomData<*mut ()>,
}

/// An exclusive borrow of a Rust value together with its R object.
///
/// [`ExternalPtr::borrow_mut`](super::ExternalPtr::borrow_mut) makes one, and
/// a `#[miniextendr]` function or method takes one as a parameter (or as
/// `self: ExternalPtrMut<'_, Self>`) when it needs the R object as well as
/// the value, like [`ExternalPtrRef`]. Dereferencing gives mutable access to
/// the Rust fields; the sidecar setters use the R object. There is no
/// consuming API.
///
/// The view holds an exclusive borrow of the value's `RefCell` for its whole
/// lifetime: any other borrow of the value meanwhile raises the R error of
/// class `miniextendr_borrow_conflict`. Pass it on without moving it with
/// [`rb_mut`](Self::rb_mut), or as a shared view with [`rb`](Self::rb).
///
/// A view borrows its handle for as long as it lives:
/// ```compile_fail,E0502
/// use miniextendr_api::{ExternalPtr, ExternalPtrMut, ExternalPtrRef, TypedExternal};
/// fn overlap<T: TypedExternal>(owner: &mut ExternalPtr<T>) {
///     let mut first = owner.borrow_mut();
///     let second = owner.borrow();
///     let _ = (&mut *first, second);
/// }
/// ```
///
/// A reborrow ends before the view is used again:
/// ```compile_fail,E0499
/// use miniextendr_api::{ExternalPtrMut, TypedExternal};
/// fn twice<T: TypedExternal>(view: &mut ExternalPtrMut<'_, T>) {
///     let first = view.rb_mut();
///     let second = view.rb_mut();
///     let _ = (first, second);
/// }
/// ```
pub struct ExternalPtrMut<'a, T: TypedExternal + ?Sized> {
    value: Exclusive<'a, T>,
    owner: &'a SEXP,
    _main_thread: PhantomData<*mut ()>,
}

impl<'a, T: TypedExternal + ?Sized> ExternalPtrRef<'a, T> {
    /// Hold `guard`, a borrow of the value of the R object `owner`.
    #[inline]
    pub(super) fn from_guard(owner: &'a SEXP, guard: Ref<'a, T>) -> Self {
        Self {
            value: Shared::Guard(guard),
            owner,
            _main_thread: PhantomData,
        }
    }

    /// A shared view of the same value for a shorter lifetime, to pass on
    /// without moving this one. The value stays borrowed until this view
    /// drops.
    #[inline]
    pub fn rb(&self) -> ExternalPtrRef<'_, T> {
        ExternalPtrRef {
            value: Shared::Reborrow(&**self),
            owner: self.owner,
            _main_thread: PhantomData,
        }
    }

    /// The R object (`EXTPTRSXP`) holding the value.
    ///
    /// An associated function, `ExternalPtrRef::as_sexp(&view)`, so that it
    /// doesn't hide a method of `T` of the same name.
    #[inline]
    pub fn as_sexp(this: &Self) -> SEXP {
        *this.owner
    }
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
        Self::from_guard(owner, borrow_value(unsafe { &*cell }))
    }
}

impl<'a, T: TypedExternal + ?Sized> ExternalPtrMut<'a, T> {
    /// Hold `guard`, an exclusive borrow of the value of the R object `owner`.
    #[inline]
    pub(super) fn from_guard(owner: &'a SEXP, guard: RefMut<'a, T>) -> Self {
        Self {
            value: Exclusive::Guard(guard),
            owner,
            _main_thread: PhantomData,
        }
    }

    /// A shared view of the same value, for as long as this view isn't used
    /// mutably.
    #[inline]
    pub fn rb(&self) -> ExternalPtrRef<'_, T> {
        ExternalPtrRef {
            value: Shared::Reborrow(&**self),
            owner: self.owner,
            _main_thread: PhantomData,
        }
    }

    /// An exclusive view of the same value for a shorter lifetime, to pass on
    /// without moving this one.
    #[inline]
    pub fn rb_mut(&mut self) -> ExternalPtrMut<'_, T> {
        let owner = self.owner;
        ExternalPtrMut {
            value: Exclusive::Reborrow(&mut **self),
            owner,
            _main_thread: PhantomData,
        }
    }

    /// The R object (`EXTPTRSXP`) holding the value.
    ///
    /// An associated function, `ExternalPtrMut::as_sexp(&view)`, so that it
    /// doesn't hide a method of `T` of the same name.
    #[inline]
    pub fn as_sexp(this: &Self) -> SEXP {
        *this.owner
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
        Self::from_guard(owner, borrow_value_mut(unsafe { &*cell }))
    }
}

impl<T: TypedExternal + ?Sized> Deref for ExternalPtrRef<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match &self.value {
            Shared::Guard(guard) => guard,
            Shared::Reborrow(value) => value,
        }
    }
}
impl<T: TypedExternal + ?Sized> Deref for ExternalPtrMut<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match &self.value {
            Exclusive::Guard(guard) => guard,
            Exclusive::Reborrow(value) => value,
        }
    }
}
impl<T: TypedExternal + ?Sized> DerefMut for ExternalPtrMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.value {
            Exclusive::Guard(guard) => guard,
            Exclusive::Reborrow(value) => value,
        }
    }
}

impl<T: TypedExternal + ?Sized + fmt::Debug> fmt::Debug for ExternalPtrRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
impl<T: TypedExternal + ?Sized + fmt::Debug> fmt::Debug for ExternalPtrMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
impl<T: TypedExternal + ?Sized + fmt::Display> fmt::Display for ExternalPtrRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl<T: TypedExternal + ?Sized + fmt::Display> fmt::Display for ExternalPtrMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: TypedExternal> sealed::Owner for super::ExternalPtr<T> {}
impl<T: TypedExternal + ?Sized> sealed::Owner for ExternalPtrRef<'_, T> {}
impl<T: TypedExternal + ?Sized> sealed::Owner for ExternalPtrMut<'_, T> {}

impl<T: TypedExternal + ?Sized> sealed::Owner for Saved<'_, T> {}

impl<T: TypedExternal> SidecarOwner<T> for super::ExternalPtr<T> {
    fn __mx_read(&self) -> SidecarRead<'_, T> {
        SidecarRead::Live(self.borrow())
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwner<T> for ExternalPtrRef<'_, T> {
    fn __mx_read(&self) -> SidecarRead<'_, T> {
        SidecarRead::Live(self.rb())
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwner<T> for ExternalPtrMut<'_, T> {
    fn __mx_read(&self) -> SidecarRead<'_, T> {
        SidecarRead::Live(self.rb())
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwner<T> for Saved<'_, T> {
    fn __mx_read(&self) -> SidecarRead<'_, T> {
        SidecarRead::Saved(self.sexp)
    }
}
impl<T: TypedExternal> SidecarOwnerMut<T> for super::ExternalPtr<T> {
    fn __mx_write(&mut self) -> ExternalPtrMut<'_, T> {
        self.borrow_mut()
    }
}
impl<T: TypedExternal + ?Sized> SidecarOwnerMut<T> for ExternalPtrMut<'_, T> {
    fn __mx_write(&mut self) -> ExternalPtrMut<'_, T> {
        self.rb_mut()
    }
}
