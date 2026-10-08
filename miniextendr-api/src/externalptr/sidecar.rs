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
//!
//! After `readRDS()` the pointer has no address. A type with a rebuild hook
//! (`#[externalptr(revive = path)]`) is rebuilt in place from what the
//! pointer kept, through a [`StoredSidecars`] view (#1854, #1418); the
//! "Reviving a reloaded pointer" region below holds that path.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use super::{
    PROT_TYPE_ID_INDEX, PROT_USER_INDEX, PROT_VEC_LEN, TypedExternal, alloc_prot_unchecked,
    release_any, symbol_name, type_id_symbol,
};
use crate::from_r::TryFromSexp;
use crate::into_r::IntoR;
use crate::sys::{
    R_ExternalPtrAddr, R_ExternalPtrProtected, R_MakeExternalPtr, R_NamesSymbol,
    R_RegisterCFinalizerEx, R_SetExternalPtrAddr, R_SetExternalPtrProtected, Rf_allocVector,
    Rf_install, Rf_mkCharLen, Rf_protect, Rf_unprotect,
};
use crate::{R_xlen_t, Rboolean, SEXP, SEXPTYPE, SexpExt};

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
/// a `T`, and the live `T` when `x` still has an address. Allocates nothing
/// unless it revives.
///
/// A live pointer is checked by `Any::downcast`. A pointer whose address is
/// NULL (finalized, or read back by `readRDS`) is rebuilt in place first
/// when `T` has a revive hook; otherwise it still holds its sidecar values,
/// so it is accepted when its stored type ID is `T`'s.
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
    let any_raw = unsafe { live_addr::<T>(x) };
    // After `live_addr`: a revived pointer has a fresh `prot`.
    let prot = unsafe { R_ExternalPtrProtected(x) };
    let live = if any_raw.is_null() {
        if !(prot.type_of() == SEXPTYPE::VECSXP
            && prot.len() > 0
            && is_type_id_symbol::<T>(prot.vector_elt(PROT_TYPE_ID_INDEX)))
        {
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
/// without an address (after `readRDS()`) is rebuilt first when `T` has a
/// revive hook, else reads its protection list as is.
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

// region: The layout record

/// Writes the layout record on `prot`: `names()` with `""` for the type ID
/// and the user slot, then `T`'s `Sidecar` field names in slot order. A
/// rebuild hook reads a slot by name through it, whatever version of the
/// crate saved the pointer (#1854).
///
/// # Safety
///
/// On R's main thread; `prot` must be `T`'s protected `prot` list.
pub(super) unsafe fn write_layout_record<T: TypedExternal>(prot: SEXP) {
    let len = R_xlen_t::try_from(prot.len()).expect("prot length exceeds R_xlen_t::MAX");
    // `allocVector(STRSXP, ..)` fills the entries with `R_BlankString`.
    let names = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, len) };
    unsafe { Rf_protect(names) };
    for (index, name) in T::__MX_SIDECAR_NAMES.iter().enumerate() {
        let n = i32::try_from(name.len()).expect("field name length exceeds i32::MAX");
        // `mkCharLen` may allocate: `names` is protected and holds the
        // entries written so far.
        let charsxp = unsafe { Rf_mkCharLen(name.as_ptr().cast(), n) };
        names.set_string_elt(prot_index(index), charsxp);
    }
    // May allocate the attribute cell: both lists are protected.
    prot.set_attr(unsafe { R_NamesSymbol }, names);
    unsafe { Rf_unprotect(1) };
}

/// The text of a `CHARSXP`, which lives as long as R's string cache keeps it.
fn charsxp_str<'a>(charsxp: SEXP) -> &'a str {
    // SAFETY: a CHARSXP's data holds `len()` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(charsxp.r_char().cast::<u8>(), charsxp.len()) };
    std::str::from_utf8(bytes).unwrap_or("")
}

// endregion

// region: Reviving a reloaded pointer

/// What a pointer read back by `readRDS()` kept, as a revive hook sees it.
///
/// After `readRDS()` an external pointer has no address: the Rust value is
/// gone, and only the protection list survives, with the type ID of the
/// crate version that saved it, the user slot, the [`Sidecar<T>`] values,
/// and the layout record (the field names) a type writes since #1854. A
/// type opts into reading such a pointer, from any version of its crate,
/// with a rebuild hook:
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// #[externalptr(r6, revive = Self::revive)]
/// pub struct Engine {
///     #[r_data] _r: RSidecar,
///     #[r_data] pub keys: Sidecar<Vec<i32>>,
///     #[r_data] pub label: Sidecar<String>,
///     n: usize,
/// }
///
/// impl Engine {
///     fn revive(stored: StoredSidecars<'_>) -> Result<Self, EngineError> {
///         let keys: Vec<i32> = match stored.slot("keys") {
///             Ok(keys) => keys,
///             Err(SlotError::Missing { .. }) => Vec::new(),
///             Err(SlotError::Conversion { name, error }) => {
///                 defer_warning!(class = "engine_rebuilt", "`{name}` was rebuilt: {error}");
///                 Vec::new()
///             }
///         };
///         if stored.version() != Some(env!("CARGO_PKG_VERSION")) {
///             defer_warning!("saved by version {}", stored.version().unwrap_or("?"));
///         }
///         Ok(Engine {
///             _r: RSidecar,
///             n: keys.len(),
///             keys: Sidecar::new(keys),
///             label: Sidecar::new(stored.slot("label").unwrap_or_default()),
///         })
///     }
/// }
/// ```
///
/// The framework calls the hook, on R's main thread, the first time it meets
/// a pointer of the type without an address: a method receiver, an
/// `ExternalPtr<T>` argument, an R accessor of a `Sidecar` field. The stored
/// type ID has to name the same crate and type name (the module path and the
/// version may differ); a pointer of another type is refused as before. The
/// rebuilt value is installed in the same `EXTPTRSXP`, so every R binding
/// sharing the object sees it, with a fresh protection list: the current
/// type ID, the stored user slot, the rebuilt struct's `Sidecar` values and
/// the current layout record. Every later access sees a live, current
/// object. An `Err` from the hook is raised at that call, with the error's
/// own classes when it implements `RConditionError` (through
/// `#[derive(RConditionError)]`), as the `Err` of a `#[miniextendr]` fn
/// would be. A type without a hook is unchanged: a pointer from another
/// version is refused, and reading its slots by index stays safe.
///
/// What a hook can read:
///
/// - [`type_id`](Self::type_id) and [`version`](Self::version): which crate
///   version saved the object (a later one, say);
/// - [`field_names`](Self::field_names): the layout record, `None` for a
///   save from before the record existed; then only the raw
///   [`prot`](Self::prot) and the [`user_slot`](Self::user_slot) are there to
///   read, positionally;
/// - [`slot`](Self::slot): a slot by name, converted; the error says whether
///   the slot is missing (a field this version added, or no record) or its
///   value no longer converts (a field this version retyped), so a hook can
///   keep the field's default, rebuild it from its other state, and say so
///   with its own warning. A stored slot nobody asks for (a field this
///   version dropped) is ignored.
///
/// The view borrows the pointer's stored protection list for the hook's
/// call; it is replaced once the hook returns.
pub struct StoredSidecars<'a> {
    /// The stored `prot` list.
    prot: SEXP,
    /// The name of the `prot[0]` symbol; symbols live for the session.
    type_id: &'static str,
    /// The field names of the layout record, in slot order; `None` without
    /// one.
    names: Option<Vec<&'a str>>,
    _marker: PhantomData<&'a SEXP>,
}

impl<'a> StoredSidecars<'a> {
    /// A view of `prot`, or `None` when it is not a list `ExternalPtr`
    /// built (a `VECSXP` of at least two entries with a symbol first).
    fn new(prot: SEXP) -> Option<Self> {
        if prot.is_null_or_nil()
            || prot.type_of() != SEXPTYPE::VECSXP
            || prot.len() < PROT_VEC_LEN as usize
        {
            return None;
        }
        let sym = prot.vector_elt(PROT_TYPE_ID_INDEX);
        if sym.type_of() != SEXPTYPE::SYMSXP {
            return None;
        }
        let names_sexp = prot.get_names();
        let names = (names_sexp.type_of() == SEXPTYPE::STRSXP && names_sexp.len() == prot.len())
            .then(|| {
                (PROT_VEC_LEN as usize..prot.len())
                    .map(|i| {
                        let i = isize::try_from(i).expect("prot length exceeds isize::MAX");
                        charsxp_str(names_sexp.string_elt(i))
                    })
                    .collect()
            });
        Some(Self {
            prot,
            type_id: symbol_name(sym),
            names,
            _marker: PhantomData,
        })
    }

    /// The stored type ID, `crate@version::module::Type` for a
    /// `#[derive(ExternalPtr)]` type: the crate version that saved the
    /// object.
    pub fn type_id(&self) -> &str {
        self.type_id
    }

    /// The version part of the stored type ID (between `@` and the first
    /// `::`), `None` for a hand-written `TYPE_ID_CSTR` without one.
    pub fn version(&self) -> Option<&str> {
        let (_, rest) = self.type_id.split_once('@')?;
        Some(rest.split_once("::").map_or(rest, |(version, _)| version))
    }

    /// The layout record: the names of the `Sidecar` fields the saving
    /// version had, in slot order. `None` for a save from before the record
    /// existed; the hook then has only [`prot`](Self::prot) and
    /// [`user_slot`](Self::user_slot).
    pub fn field_names(&self) -> Option<&[&'a str]> {
        self.names.as_deref()
    }

    /// The stored value of the slot named `name`, as stored, or `None`
    /// without a record or a slot by that name.
    pub fn slot_raw(&self, name: &str) -> Option<SEXP> {
        let index = self.names.as_ref()?.iter().position(|n| *n == name)?;
        Some(self.prot.vector_elt(prot_index(index)))
    }

    /// The stored value of the slot named `name`, converted to `T`.
    ///
    /// [`SlotError::Missing`] when there is no slot by that name (a field
    /// this version added, or a save without a record);
    /// [`SlotError::Conversion`] when the stored value does not convert (a
    /// field this version retyped), with the conversion error.
    pub fn slot<T: TryFromSexp>(&self, name: &str) -> Result<T, SlotError<T::Error>> {
        let Some(value) = self.slot_raw(name) else {
            return Err(SlotError::Missing {
                name: name.to_owned(),
            });
        };
        T::try_from_sexp(value).map_err(|error| SlotError::Conversion {
            name: name.to_owned(),
            error,
        })
    }

    /// The stored user slot (`ExternalPtr::protected()`), `NULL` when none
    /// was set.
    pub fn user_slot(&self) -> SEXP {
        self.prot.vector_elt(PROT_USER_INDEX)
    }

    /// The raw stored protection list: the type-ID symbol, the user slot,
    /// then the slot values in the saving version's order.
    pub fn prot(&self) -> SEXP {
        self.prot
    }
}

impl fmt::Debug for StoredSidecars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredSidecars")
            .field("type_id", &self.type_id)
            .field("field_names", &self.names)
            .finish_non_exhaustive()
    }
}

/// Why [`StoredSidecars::slot`] could not give a slot's value.
#[derive(Debug)]
pub enum SlotError<E> {
    /// No slot by that name: a field this version added, or a save without
    /// a layout record.
    Missing {
        /// The name asked for.
        name: String,
    },
    /// The stored value does not convert to the type asked for: a field this
    /// version retyped.
    Conversion {
        /// The name asked for.
        name: String,
        /// The conversion error.
        error: E,
    },
}

impl<E> SlotError<E> {
    /// The slot name asked for.
    pub fn name(&self) -> &str {
        match self {
            Self::Missing { name } | Self::Conversion { name, .. } => name,
        }
    }

    /// Whether the slot is absent, as opposed to present but of another
    /// shape.
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing { .. })
    }
}

impl<E: fmt::Display> fmt::Display for SlotError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name } => write!(f, "no stored slot `{name}`"),
            Self::Conversion { name, error } => {
                write!(f, "stored slot `{name}` does not convert: {error}")
            }
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for SlotError<E> {}

/// The parts of a type ID that persist across versions: the crate name
/// (before `@`, else before the first `::`) and the type name (after the
/// last `::`).
fn type_identity(id: &str) -> (&str, &str) {
    let crate_name = match id.split_once('@') {
        Some((crate_name, _)) => crate_name,
        None => id.split("::").next().unwrap_or(id),
    };
    let type_name = id.rsplit("::").next().unwrap_or(id);
    (crate_name, type_name)
}

/// Whether a stored type ID names the type whose `TYPE_ID_CSTR` is `own`:
/// the same crate and type name, at any version and module path.
pub(super) fn same_type_identity(stored: &str, own: &[u8]) -> bool {
    let own = own.strip_suffix(b"\0").unwrap_or(own);
    let Ok(own) = std::str::from_utf8(own) else {
        return false;
    };
    type_identity(stored) == type_identity(own)
}

/// The address of `sexp`'s value, reviving a pointer without one when `T`
/// has a rebuild hook and the stored type ID names `T` (see
/// [`StoredSidecars`]). Null when there is no value and no hook applies; the
/// caller then refuses the pointer as before.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
pub(super) unsafe fn live_addr<T: TypedExternal>(sexp: SEXP) -> *mut Box<dyn Any> {
    let any_raw = unsafe { R_ExternalPtrAddr(sexp) }.cast::<Box<dyn Any>>();
    if !any_raw.is_null() || !T::__MX_REVIVES {
        return any_raw;
    }
    unsafe { revive_in_place::<T>(sexp) }
}

/// Runs `T`'s rebuild hook on the pointer `sexp`, which has no address, and
/// installs the value and a fresh protection list in it. Returns the new
/// address, or null when the stored type ID does not name `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP` whose
/// address is NULL; `T::__MX_REVIVES` must hold.
#[cold]
unsafe fn revive_in_place<T: TypedExternal>(sexp: SEXP) -> *mut Box<dyn Any> {
    let stored_prot = unsafe { R_ExternalPtrProtected(sexp) };
    let Some(stored) = StoredSidecars::new(stored_prot) else {
        return ptr::null_mut();
    };
    if !same_type_identity(stored.type_id(), T::TYPE_ID_CSTR) {
        return ptr::null_mut();
    }

    // The hook may allocate: `stored_prot` is reachable from `sexp`, which
    // the caller roots, until it is replaced below.
    let value = match T::__mx_revive(stored) {
        Ok(value) => value,
        Err(condition) => std::panic::panic_any(condition),
    };

    // A fresh `prot`: the current type ID, the stored user slot (still
    // rooted by the stored list), empty slots and the current record.
    let user_slot = stored_prot.vector_elt(PROT_USER_INDEX);
    let type_id_sym = unsafe { type_id_symbol::<T>() };
    let prot = unsafe { alloc_prot_unchecked::<T>(type_id_sym, user_slot) };
    unsafe { R_SetExternalPtrProtected(sexp, prot) };
    unsafe { Rf_unprotect(1) };

    // The value, in the same `EXTPTRSXP`. A pointer read back by `readRDS()`
    // has no finalizer; one cleared by Rust has one already, which then runs
    // twice and returns at once the second time (a NULL address).
    let inner: Box<dyn Any> = Box::new(value);
    let any_raw: *mut Box<dyn Any> = Box::into_raw(Box::new(inner));
    unsafe { R_SetExternalPtrAddr(sexp, any_raw.cast()) };
    unsafe { R_RegisterCFinalizerEx(sexp, Some(release_any), Rboolean::TRUE) };

    // The rebuilt struct's pending sidecar values, into the new slots.
    if T::R_SLOT_COUNT > 0 {
        unsafe { init_sidecars::<T>(any_raw, sexp) };
    }
    any_raw
}

/// Builds the `EXTPTRSXP` that `readRDS()` gives for a save by another
/// version of a crate: no address, `type_id` as the stored type ID, the user
/// slot, the slot values, and the layout record when `names` is given (one
/// name per slot). For tests of revive hooks.
///
/// Returns an unprotected `EXTPTRSXP`: protect it or return it to R at once.
///
/// # Safety
///
/// Must be called from R's main thread; `user_slot` and `slots` must be
/// rooted by the caller.
#[doc(hidden)]
pub unsafe fn stored_pointer_for_tests(
    type_id: &str,
    user_slot: SEXP,
    slots: &[SEXP],
    names: Option<&[&str]>,
) -> SEXP {
    let type_id_c = std::ffi::CString::new(type_id).expect("a type ID has no NUL");
    let tag_c = std::ffi::CString::new(type_identity(type_id).1).expect("a type name has no NUL");
    let len = R_xlen_t::try_from(PROT_VEC_LEN as usize + slots.len()).expect("slot count");
    let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, len) };
    unsafe { Rf_protect(prot) };
    prot.set_vector_elt(PROT_TYPE_ID_INDEX, unsafe {
        Rf_install(type_id_c.as_ptr())
    });
    prot.set_vector_elt(PROT_USER_INDEX, user_slot);
    for (index, &slot) in slots.iter().enumerate() {
        prot.set_vector_elt(prot_index(index), slot);
    }
    if let Some(names) = names {
        assert_eq!(names.len(), slots.len(), "one name per slot");
        let record = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, len) };
        unsafe { Rf_protect(record) };
        for (index, name) in names.iter().enumerate() {
            let n = i32::try_from(name.len()).expect("name length");
            let charsxp = unsafe { Rf_mkCharLen(name.as_ptr().cast(), n) };
            record.set_string_elt(prot_index(index), charsxp);
        }
        prot.set_attr(unsafe { R_NamesSymbol }, record);
        unsafe { Rf_unprotect(1) };
    }
    let tag = unsafe { Rf_install(tag_c.as_ptr()) };
    let sexp = unsafe { R_MakeExternalPtr(ptr::null_mut(), tag, prot) };
    unsafe { Rf_unprotect(1) };
    sexp
}

// endregion
