//! Typed sidecar fields: [`Sidecar<T>`] keeps a Rust-typed value in the
//! external pointer's `prot` list.
//!
//! A Rust struct cannot root an R value: the GC doesn't trace Rust memory, so
//! a `SEXP` stored in a struct field is freed as soon as R drops its own
//! references. A `Sidecar<T>` field holds no data of its own once the struct
//! is wrapped. Its value lives in the external pointer's protection list,
//! after the type ID and the user slot, so the pointer roots it for as long as
//! the pointer is reachable, and `saveRDS()` writes it with the pointer. The
//! struct keeps only a back-reference to that slot.
//!
//! The type is named in one place (this module) and re-exported from
//! [`crate::externalptr`]; its final name is #1857.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::ptr;

use super::{PROT_TYPE_ID_INDEX, PROT_VEC_LEN, TypedExternal, refuse_restored};
use crate::from_r::TryFromSexp;
use crate::into_r::IntoR;
use crate::sys::{R_ExternalPtrAddr, R_ExternalPtrProtected};
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

// region: Sidecar<T>

/// A sidecar field whose value R roots: it lives in the external pointer's
/// protection list, typed as `T` on the Rust side.
///
/// Declare it as `#[r_data] pub keys: Sidecar<Vec<i32>>` on a
/// `#[derive(ExternalPtr)]` struct. The derive generates accessors on the
/// struct, with the field's visibility:
///
/// | `#[r_data(...)]` | Rust accessors |
/// |---|---|
/// | `ref` | `fn keys(&self) -> T` |
/// | `mut` | `fn set_keys(&mut self, value: T)` |
/// | `ref, mut`, or a bare `#[r_data]` | both |
///
/// Rust code sees no `SEXP`s, slot indices or other R plumbing: a plain
/// `fn new() -> Self` constructor fills the field with [`Sidecar::new`], a
/// method reads it with `self.keys()` and writes it with `self.set_keys(v)`.
/// The R side keeps its own accessors whatever the option: for a `pub` field,
/// `Type_get_keys(x)` / `Type_set_keys(x, value)`, an R6 active binding
/// (`r6(r_data_accessors)`) or an S7 property (`s7(r_data_accessors)`). The
/// R setter validates the new value with `TryFromSexp::<T>` and stores the
/// value R gave.
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// #[externalptr(r6)]
/// pub struct Engine {
///     n: i32,
///     #[r_data] _r: RSidecar,
///     #[r_data(ref, mut)] pub keys: Sidecar<Vec<i32>>,
///     #[r_data(ref)] pub labels: Sidecar<Vec<String>>,
///     #[r_data(ref, mut)] cache: Sidecar<Option<List>>,
/// }
///
/// #[miniextendr(r6(r_data_accessors))]
/// impl Engine {
///     pub fn new(n: i32) -> Self {
///         Engine {
///             n,
///             _r: RSidecar,
///             keys: Sidecar::new((1..=n).collect()),
///             labels: Sidecar::new(vec![]),
///             cache: Sidecar::default(),
///         }
///     }
///     pub fn key_count(&self) -> usize { self.keys().len() }
///     pub fn remember(&mut self, value: List) { self.set_cache(Some(value)) }
/// }
/// ```
///
/// # Values
///
/// - `T: IntoR + TryFromSexp`. The getter also needs `T: Clone`, for a field
///   that is not attached to a pointer (below), and `T::Error: Display`, for
///   its panic message.
/// - A getter converts the slot with `TryFromSexp`, so a converting type
///   (`Vec<i32>`, `String`) copies on every read. A conversion failure is a
///   panic naming the field and the type; the framework turns it into an R
///   error.
/// - `Sidecar<SEXP>`, `Sidecar<List>` and `Sidecar<DataFrame>` keep an
///   arbitrary R object unconverted. The getter returns a view that the slot
///   roots only until the next write to that field; don't hold it across one.
/// - A pending `SEXP` (one passed to [`Sidecar::new`]) is not rooted until
///   the struct is wrapped: keep it protected until `ExternalPtr::new`
///   returns.
///
/// # Lifecycle
///
/// - **Pending.** [`Sidecar::new(v)`](Sidecar::new) and [`Default`]
///   (`Sidecar::new(T::default())`) hold `v` as a pending value in the
///   struct.
/// - **Attached.** `ExternalPtr::new` moves every pending value into the
///   pointer's protection list and writes the back-reference into every
///   `Sidecar` field. From then on the value lives in R; the field is a
///   back-reference. The handle rewrites the back-reference whenever it hands
///   the struct out (`Deref`, `DerefMut`, `as_ref`, `as_mut`, the receiver
///   of a `#[miniextendr]` method, the R accessors), so a struct swapped
///   between two pointers reads its own pointer's values again at the next
///   access through the handle.
/// - **Flush on every access.** The Rust getter and setter, and the R getter
///   and setter, first move a pending value into the protection list. So
///   `*ptr = Engine::new(..)` or `ptr.keys = Sidecar::new(v)` through
///   `DerefMut` takes effect at the next access from either side. A
///   `saveRDS()` before any access writes the old slot value.
/// - **Detached.** A field whose recorded address is not its current address
///   works on its own pending value: a value on the stack, a struct moved
///   out of its pointer, a struct nested as a field of another wrapped
///   struct. `ExternalPtr::into_inner` converts each slot back into a pending
///   value first, so wrapping the value again keeps the values.
/// - **Clone** (for `T: Clone`) reads an attached slot and returns a detached
///   `Sidecar` holding that value. `#[derive(Clone)]` on the struct therefore
///   copies the sidecar values, through `ExternalPtr::clone` or directly, and
///   the two pointers are independent afterwards.
/// - **`mem::swap(&mut *a, &mut *b)`** swaps the Rust fields; the sidecar
///   values stay with each pointer, because they live in its protection list.
/// - **After `readRDS()`** there is no Rust value. The R accessors read the
///   protection list directly, checked by the type ID stored with it.
///
/// Use the accessors on R's main thread only. Off the main thread an
/// accessor on an attached field panics with a clear message.
///
/// # A struct moved out through `&mut` (#1856)
///
/// Don't move a sidecar struct out of its pointer through `&mut`
/// (`mem::replace`, `mem::take`, `mem::swap` with a local). The moved struct
/// keeps its back-reference; if the pointer is later freed and the struct
/// lands at that same address again, its next accessor call reads a freed R
/// object. Use `ExternalPtr::into_inner`, which detaches the struct and keeps
/// its values.
///
/// A plain `SEXP` field under `#[r_data]` is a compile error: nothing would
/// root it. Use `Sidecar<SEXP>`.
pub struct Sidecar<T> {
    /// The value the field holds while it is not attached to a pointer.
    pending: RefCell<Option<T>>,
    /// The owning `EXTPTRSXP`, as an integer (`0` when detached).
    owner: Cell<usize>,
    /// The field's own address when the back-reference was written.
    addr: Cell<usize>,
    /// The field's position among the type's `Sidecar` fields.
    index: Cell<usize>,
    /// The field's name, for diagnostics.
    name: Cell<&'static str>,
    /// The owning type's R-visible name, for diagnostics.
    owner_type: Cell<&'static str>,
}

impl<T> Sidecar<T> {
    /// A sidecar field holding `value`, pending until the struct is wrapped
    /// in an `ExternalPtr`.
    pub fn new(value: T) -> Self {
        Self {
            pending: RefCell::new(Some(value)),
            owner: Cell::new(0),
            addr: Cell::new(0),
            index: Cell::new(0),
            name: Cell::new(""),
            owner_type: Cell::new(""),
        }
    }

    /// The owning pointer when the back-reference is trusted: it was written
    /// for this very address.
    #[inline]
    fn attached(&self) -> Option<SEXP> {
        let owner = self.owner.get();
        if owner == 0 || self.addr.get() != ptr::from_ref(self).addr() {
            return None;
        }
        Some(SEXP(ptr::with_exposed_provenance_mut(owner)))
    }

    /// The protection-list position of this field.
    #[inline]
    fn prot_index(&self) -> R_xlen_t {
        prot_index(self.index.get())
    }

    /// Panics unless this is R's main thread. No R API: the message uses the
    /// names recorded at attach time.
    fn assert_main_thread(&self) {
        if !crate::worker::is_r_main_thread() {
            panic!(
                "sidecar field `{}` of `{}`: its accessors run on R's main thread only; \
                 this call came from another thread (a `worker` method, or a thread the \
                 package spawned)",
                self.name.get(),
                self.owner_type.get(),
            );
        }
    }
}

impl<T: IntoR + TryFromSexp> Sidecar<T>
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    /// Moves a pending value into the slot of the attached pointer `owner`.
    ///
    /// On R's main thread. `set_vector_elt` allocates nothing, so the value
    /// fresh from `into_sexp()` needs no protection.
    fn flush_into(&self, owner: SEXP) {
        let Some(value) = self.pending.borrow_mut().take() else {
            return;
        };
        // SAFETY: on R's main thread; `owner` is a live EXTPTRSXP built by
        // `ExternalPtr`, whose `prot` list has this field's slot.
        let prot = unsafe { R_ExternalPtrProtected(owner) };
        prot.set_vector_elt(self.prot_index(), value.into_sexp());
    }

    /// The slot value of the attached pointer `owner`, converted to `T`.
    fn convert_slot(&self, owner: SEXP) -> T {
        // SAFETY: on R's main thread; `owner` is a live EXTPTRSXP built by
        // `ExternalPtr`.
        let prot = unsafe { R_ExternalPtrProtected(owner) };
        let slot = prot.vector_elt(self.prot_index());
        match T::try_from_sexp(slot) {
            Ok(value) => value,
            Err(err) => panic!(
                "sidecar field `{}` of `{}` holds an R value that does not convert to `{}`: {err}",
                self.name.get(),
                self.owner_type.get(),
                std::any::type_name::<T>(),
            ),
        }
    }

    /// The field's value: the converted slot of an attached field (after
    /// flushing a pending value), or a clone of a detached field's pending
    /// value.
    ///
    /// Called by the getter `#[derive(ExternalPtr)]` generates.
    #[doc(hidden)]
    pub fn __mx_get(&self) -> T
    where
        T: Clone,
    {
        match self.attached() {
            Some(owner) => {
                self.assert_main_thread();
                self.flush_into(owner);
                self.convert_slot(owner)
            }
            None => self
                .pending
                .borrow()
                .clone()
                .unwrap_or_else(|| self.detached_without_value()),
        }
    }

    /// Stores `value`: in the slot of an attached field (discarding a pending
    /// value, which it supersedes), or as a detached field's pending value.
    ///
    /// Called by the setter `#[derive(ExternalPtr)]` generates.
    #[doc(hidden)]
    pub fn __mx_set(&mut self, value: T) {
        match self.attached() {
            Some(owner) => {
                self.assert_main_thread();
                *self.pending.get_mut() = None;
                // SAFETY: on R's main thread; `owner` is a live EXTPTRSXP built
                // by `ExternalPtr`. Storing allocates nothing.
                let prot = unsafe { R_ExternalPtrProtected(owner) };
                prot.set_vector_elt(self.prot_index(), value.into_sexp());
            }
            None => *self.pending.get_mut() = Some(value),
        }
    }

    #[cold]
    fn detached_without_value(&self) -> T {
        panic!(
            "sidecar field `{}` is detached from its external pointer and holds no value; \
             access it through its `ExternalPtr` handle, which reattaches it",
            self.name.get(),
        )
    }
}

impl<T: Clone + IntoR + TryFromSexp> Clone for Sidecar<T>
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    /// A detached `Sidecar` holding this field's value: the converted slot
    /// of an attached field, or a clone of a detached field's pending value.
    fn clone(&self) -> Self {
        Self::new(self.__mx_get())
    }
}

impl<T: Default> Default for Sidecar<T> {
    /// `Sidecar::new(T::default())`.
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> fmt::Debug for Sidecar<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Sidecar");
        s.field("field", &self.name.get());
        match self.attached() {
            Some(_) => s.field("slot", &self.index.get()),
            None => s.field("pending", &self.pending.borrow().is_some()),
        };
        s.finish()
    }
}

// endregion

// region: The back-reference, written by the handle

/// A `Sidecar<T>` field, as the handle sees it through the
/// `TypedExternal::__mx_visit_sidecars` hook the derive emits.
#[doc(hidden)]
pub trait SidecarField {
    /// Writes the back-reference: the owning pointer, this field's current
    /// address, its slot index, its name and the owning type's name. No R
    /// API: safe on any thread.
    fn __mx_attach(&self, owner: SEXP, index: usize, name: &'static str, owner_type: &'static str);
    /// Moves a pending value into the slot of an attached field. On R's main
    /// thread.
    fn __mx_flush(&self);
    /// Converts the slot of an attached field back into a pending value
    /// (unless one is pending already), then clears the back-reference. On
    /// R's main thread.
    fn __mx_detach(&self);
    /// Drops a pending value, which a write from R supersedes.
    fn __mx_discard_pending(&self);
}

impl<T: IntoR + TryFromSexp> SidecarField for Sidecar<T>
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    #[inline]
    fn __mx_attach(&self, owner: SEXP, index: usize, name: &'static str, owner_type: &'static str) {
        self.owner.set(owner.0.expose_provenance());
        self.addr.set(ptr::from_ref(self).addr());
        self.index.set(index);
        self.name.set(name);
        self.owner_type.set(owner_type);
    }

    fn __mx_flush(&self) {
        if let Some(owner) = self.attached() {
            self.flush_into(owner);
        }
    }

    fn __mx_detach(&self) {
        let Some(owner) = self.attached() else {
            return;
        };
        if self.pending.borrow().is_none() {
            let value = self.convert_slot(owner);
            *self.pending.borrow_mut() = Some(value);
        }
        self.owner.set(0);
    }

    fn __mx_discard_pending(&self) {
        *self.pending.borrow_mut() = None;
    }
}

/// Writes the back-reference into every `Sidecar` field of `value`, wrapped
/// in the external pointer `owner`. No R API: safe on any thread.
#[doc(hidden)]
#[inline]
pub fn attach_sidecars<T: TypedExternal>(value: &T, owner: SEXP) {
    value.__mx_visit_sidecars(&mut |index, name, field| {
        field.__mx_attach(owner, index, name, T::TYPE_NAME);
    });
}

/// Moves every pending value of `value`'s `Sidecar` fields into the
/// protection list. The fields must be attached. On R's main thread.
#[doc(hidden)]
#[inline]
pub fn flush_sidecars<T: TypedExternal>(value: &T) {
    value.__mx_visit_sidecars(&mut |_, _, field| field.__mx_flush());
}

/// Converts every slot of `value`'s attached `Sidecar` fields back into a
/// pending value and clears the back-references, before the value leaves its
/// pointer. On R's main thread.
#[doc(hidden)]
#[inline]
pub fn detach_sidecars<T: TypedExternal>(value: &T) {
    value.__mx_visit_sidecars(&mut |_, _, field| field.__mx_detach());
}

/// Attaches and flushes a freshly wrapped value: the three construction sites
/// and `restore_after_consuming` call this once the `EXTPTRSXP` exists.
///
/// # Safety
///
/// On R's main thread; `any_raw` must own a `T`, and `owner` must be the
/// live `EXTPTRSXP` holding it, with a `prot` list built by `ExternalPtr`.
pub(super) unsafe fn init_sidecars<T: TypedExternal>(any_raw: *mut Box<dyn Any>, owner: SEXP) {
    let value: &T = unsafe { &*any_raw }
        .downcast_ref::<T>()
        .expect("ExternalPtr stores a T at construction");
    attach_sidecars(value, owner);
    flush_sidecars(value);
}

// endregion

// region: The R accessors

/// The protection-list position of `Sidecar` field `index`.
fn prot_index(index: usize) -> R_xlen_t {
    let index = R_xlen_t::try_from(index).expect("sidecar index exceeds R_xlen_t::MAX");
    PROT_VEC_LEN + index
}

/// Whether `sym` is the symbol of `T`'s type ID.
///
/// Compares names rather than installing `T`'s symbol, so it allocates nothing.
fn is_type_id_symbol<T: TypedExternal>(sym: SEXP) -> bool {
    if sym.type_of() != SEXPTYPE::SYMSXP {
        return false;
    }
    let id = T::TYPE_ID_CSTR
        .strip_suffix(b"\0")
        .unwrap_or(T::TYPE_ID_CSTR);
    let printname = sym.printname();
    // SAFETY: a CHARSXP's data holds `len` bytes.
    let name =
        unsafe { std::slice::from_raw_parts(printname.r_char().cast::<u8>(), printname.len()) };
    name == id
}

/// The `prot` list of `x`, after checking that `x` is an external pointer to
/// a `T`, and the live `T` when `x` still has an address. Allocates nothing.
///
/// A live pointer is checked by `Any::downcast`. A pointer whose address is
/// NULL (restored from a saved session by `readRDS()` / `unserialize()`)
/// still holds its sidecar values, so it is accepted when its stored type ID
/// is `T`'s; one another version of the crate saved is refused with the
/// classed [`RESTORED_OTHER_VERSION_CLASS`](super::RESTORED_OTHER_VERSION_CLASS)
/// error.
///
/// # Panics
///
/// When `x` is not an external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
unsafe fn checked_prot<'a, T: TypedExternal>(x: SEXP) -> (SEXP, Option<&'a T>) {
    if x.type_of() != SEXPTYPE::EXTPTRSXP {
        panic!(
            "expected ExternalPtr<{}>, got a non-external-pointer object",
            T::TYPE_NAME
        );
    }
    let prot = unsafe { R_ExternalPtrProtected(x) };
    let any_raw = unsafe { R_ExternalPtrAddr(x) }.cast::<Box<dyn Any>>();
    let live = if any_raw.is_null() {
        if !(prot.type_of() == SEXPTYPE::VECSXP
            && prot.len() > 0
            && is_type_id_symbol::<T>(prot.vector_elt(PROT_TYPE_ID_INDEX)))
        {
            // Another version's save raises its own error here.
            unsafe { refuse_restored::<T>(x) };
            panic!("expected ExternalPtr<{}>", T::TYPE_NAME);
        }
        None
    } else {
        match unsafe { &*any_raw }.downcast_ref::<T>() {
            Some(value) => Some(value),
            None => panic!("expected ExternalPtr<{}>", T::TYPE_NAME),
        }
    };
    if !usize::try_from(prot_index(T::R_SLOT_COUNT)).is_ok_and(|len| len <= prot.len()) {
        panic!(
            "ExternalPtr<{}> has no sidecar slots: its protection list was not built by `ExternalPtr`",
            T::TYPE_NAME
        );
    }
    (prot, live)
}

/// The live `T` of the external pointer `x`, for the R getters of a struct
/// field (a field that is not a `Sidecar`). A pointer without an address is
/// refused: with the classed restored-object errors when `T` built it
/// ([`refuse_restored`]), else with `expected ExternalPtr<T>, got a null
/// external pointer`.
///
/// # Panics
///
/// When `x` is not a live external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`; the reference is
/// valid while `x` keeps its value.
#[doc(hidden)]
pub unsafe fn sidecar_r_struct<'a, T: TypedExternal>(x: SEXP) -> &'a T {
    // SAFETY: the caller's contract; `downcast_ref` reads through a shared
    // reference.
    unsafe { &*sidecar_r_struct_ptr::<T>(x) }
}

/// The live `T` of the external pointer `x`, mutably, for the R setters of a
/// struct field; see [`sidecar_r_struct`].
///
/// # Panics
///
/// When `x` is not a live external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`; the reference is
/// valid while `x` keeps its value, and nothing else may read or write the
/// struct meanwhile.
#[doc(hidden)]
#[allow(clippy::mut_from_ref)]
pub unsafe fn sidecar_r_struct_mut<'a, T: TypedExternal>(x: SEXP) -> &'a mut T {
    // SAFETY: the caller's contract; the pointer has mutable provenance
    // (`downcast_mut`).
    unsafe { &mut *sidecar_r_struct_ptr::<T>(x) }
}

/// The body of [`sidecar_r_struct`] and [`sidecar_r_struct_mut`]: the
/// struct's address with mutable provenance.
unsafe fn sidecar_r_struct_ptr<T: TypedExternal>(x: SEXP) -> *mut T {
    if x.type_of() != SEXPTYPE::EXTPTRSXP {
        panic!(
            "expected ExternalPtr<{}>, got a non-external-pointer object",
            T::TYPE_NAME
        );
    }
    let any_raw = unsafe { R_ExternalPtrAddr(x) }.cast::<Box<dyn Any>>();
    if any_raw.is_null() {
        // A save by this or another version raises its own error here.
        unsafe { refuse_restored::<T>(x) };
        panic!(
            "expected ExternalPtr<{}>, got a null external pointer",
            T::TYPE_NAME
        );
    }
    let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
    match any_box.downcast_mut::<T>() {
        Some(value) => std::ptr::from_mut(value),
        None => panic!("expected ExternalPtr<{}>", T::TYPE_NAME),
    }
}

/// Checks that `x` is an external pointer to a `T`, live or read back by
/// `readRDS()`, before a setter validates its value.
///
/// # Panics
///
/// When `x` is not an external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
#[doc(hidden)]
pub unsafe fn sidecar_r_check<T: TypedExternal>(x: SEXP) {
    let _ = unsafe { checked_prot::<T>(x) };
}

/// Reads `T`'s `Sidecar` field `index` from the external pointer `x`.
///
/// Called by the R getters `#[derive(ExternalPtr)]` generates. A live struct
/// is reattached and its pending value for the field flushed first; a pointer
/// without an address (after `readRDS()`) reads its protection list as is.
///
/// # Panics
///
/// When `x` is not an external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
#[doc(hidden)]
pub unsafe fn sidecar_r_get<T: TypedExternal>(x: SEXP, index: usize) -> SEXP {
    let (prot, live) = unsafe { checked_prot::<T>(x) };
    if let Some(value) = live {
        value.__mx_visit_sidecars(&mut |i, name, field| {
            field.__mx_attach(x, i, name, T::TYPE_NAME);
            if i == index {
                field.__mx_flush();
            }
        });
    }
    prot.vector_elt(prot_index(index))
}

/// Stores `value`, which the caller validated with `TryFromSexp::<T>`, in
/// `T`'s `Sidecar` field `index` of the external pointer `x`.
///
/// Called by the R setters `#[derive(ExternalPtr)]` generates. A live struct
/// is reattached and its pending value for the field dropped, since this
/// write supersedes it. Allocates nothing, so `value` needs no protection
/// across the call.
///
/// # Panics
///
/// When `x` is not an external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with valid `x` and `value`.
#[doc(hidden)]
pub unsafe fn sidecar_r_set<T: TypedExternal>(x: SEXP, index: usize, value: SEXP) {
    let (prot, live) = unsafe { checked_prot::<T>(x) };
    if let Some(data) = live {
        data.__mx_visit_sidecars(&mut |i, name, field| {
            field.__mx_attach(x, i, name, T::TYPE_NAME);
            if i == index {
                field.__mx_discard_pending();
            }
        });
    }
    prot.set_vector_elt(prot_index(index), value);
}

// endregion
