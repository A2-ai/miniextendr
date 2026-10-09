//! Typed sidecar fields: [`Sidecar<T>`] keeps a Rust-typed value in the
//! external pointer's `prot` list.
//!
//! A Rust struct cannot root an R value: the GC doesn't trace Rust memory, so
//! a `SEXP` stored in a struct field is freed as soon as R drops its own
//! references. A `Sidecar<T>` field holds no data of its own once the struct
//! is wrapped. Its value lives in the external pointer's protection list,
//! after the type ID and the user slot, so the pointer roots it for as long as
//! the pointer is reachable, and `saveRDS()` writes it with the pointer. The
//! struct holds nothing that points back at its pointer: the value is read
//! and written through the `ExternalPtr<T>` handle, which keeps the pointer
//! alive for as long as it exists (#1856).
//!
//! The type is named in one place (this module) and re-exported from
//! [`crate::externalptr`]; its final name is #1857.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;

use super::{
    ExternalPtr, PROT_TYPE_ID_INDEX, PROT_VEC_LEN, TypedExternal, refuse_restored,
    unwrap_class_handle,
};
use crate::from_r::TryFromSexp;
use crate::into_r::IntoR;
use crate::sys::{R_ExternalPtrAddr, R_ExternalPtrProtected};
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

// region: Sidecar<T>

/// A sidecar field whose value R roots: it lives in the external pointer's
/// protection list, typed as `T` on the Rust side.
///
/// Declare it as `#[r_data] pub keys: Sidecar<Vec<i32>>` on a
/// `#[derive(ExternalPtr)]` struct. The derive generates accessors as
/// associated functions of the struct, with the field's visibility. They take
/// the `ExternalPtr` handle, which keeps the pointer alive, not the struct:
///
/// | `#[r_data(...)]` | Rust accessors |
/// |---|---|
/// | `ref` | `fn keys(ptr: &ExternalPtr<Self>) -> T` |
/// | `mut` | `fn set_keys(ptr: &mut ExternalPtr<Self>, value: T)` |
/// | `ref, mut`, or a bare `#[r_data]` | both |
///
/// Rust code sees no `SEXP`s, slot indices or other R plumbing: a plain
/// `fn new() -> Self` constructor fills the field with [`Sidecar::new`], and
/// a method that needs the value takes the handle as its receiver
/// (`self: &ExternalPtr<Self>` / `self: &mut ExternalPtr<Self>`) and calls
/// `Self::keys(self)` / `Self::set_keys(self, v)`; other Rust code writes
/// `Engine::keys(&ptr)`. The R side keeps its own accessors whatever the
/// option: for a `pub` field, `Type_get_keys(x)` / `Type_set_keys(x, value)`,
/// an R6 active binding (`r6(r_data_accessors)`), an S7 property
/// (`s7(r_data_accessors)`), or `x$keys` / `x$keys <- value` on an S3, S4 or
/// env class (`s3(r_data_accessors)` & co.). The R setter validates the new
/// value with `TryFromSexp::<T>` and stores the value R gave.
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
///     pub fn key_count(self: &ExternalPtr<Self>) -> usize { Self::keys(self).len() }
///     pub fn remember(self: &mut ExternalPtr<Self>, value: List) { Self::set_cache(self, Some(value)) }
/// }
///
/// // From any other Rust code holding the handle:
/// let mut ptr = ExternalPtr::new(Engine::new(3));
/// Engine::set_keys(&mut ptr, vec![9]);
/// assert_eq!(Engine::keys(&ptr), vec![9]);
/// ```
///
/// # Values
///
/// - `T: IntoR + TryFromSexp`, with `T::Error: Display` for the getter's panic
///   message.
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
///   struct. That is the only state a `Sidecar` has: before the struct is
///   wrapped, after `*ptr = Engine::new(..)`, after `ExternalPtr::into_inner`.
/// - **Flushed when wrapped.** `ExternalPtr::new` moves every pending value
///   into the pointer's protection list. From then on the value lives in R
///   only, and the field is empty.
/// - **Flushed on every access to the field.** The handle getter and setter,
///   and the R getter and setter, first move that field's pending value into
///   its slot (the setters drop it: the write supersedes it). So
///   `*ptr = Engine::new(..)` or `ptr.keys = Sidecar::new(v)` through
///   `DerefMut` takes effect at the next access from either side. A
///   `saveRDS()` before any access writes the old slot value.
/// - **Loaded when the value leaves its pointer.** `ExternalPtr::into_inner`
///   and `into_raw` read each slot of the pointer being consumed back into its
///   field as a pending value (unless one is pending already), so wrapping
///   the value again keeps the values.
/// - **A `self -> Self` method** leaves the values in the pointer across the
///   call; the write-back flushes only what the method set with
///   `Sidecar::new`. `Self { n: .., ..self }` keeps the values,
///   `Self::new(..)` replaces them.
/// - **Clone.** `Sidecar::clone` (for `T: Clone`) copies the pending value,
///   which is none once the struct is wrapped. `ExternalPtr::clone` and
///   `clone_from` copy the slot values as R objects, so the two pointers
///   share them until one side writes the field: every write replaces the
///   slot, and R copies a shared value before modifying it in place. A struct
///   cloned outside its handle carries no sidecar values; clone the handle.
/// - **`*ptr = new` and `mem::swap(&mut *a, &mut *b)`** are plain Rust
///   moves. The sidecar values stay in each pointer's protection list; only
///   pending values travel with the struct.
/// - **After `readRDS()`** there is no Rust value. The R accessors read the
///   protection list directly, checked by the type ID stored with it.
///
/// Use the accessors on R's main thread only. Off the main thread an
/// accessor panics with a clear message before it touches R.
///
/// # A save stores the values by position
///
/// A saved object (`saveRDS()`, `serialize()`) stores its sidecar values by
/// position: one slot per `Sidecar` field, in the order the fields appear in
/// the struct, with no field names. After a reload, the R accessors and the
/// generated `$` getters read each field from its position. So reordering,
/// adding, removing or retyping `Sidecar` fields changes how an old save is
/// read, and within one version of the crate nothing detects it: a field can
/// read another field's value.
///
/// Bump the crate's version (the `version` in its `Cargo.toml`, which the
/// stored type ID records) whenever the `Sidecar` fields change. A save from
/// another version is then refused with the classed
/// `miniextendr_restored_other_version` error (#1872). `#[repr(C)]` doesn't
/// help: it fixes the struct's memory layout, which a save never stores, and
/// the slot positions follow the source order either way.
///
/// # Methods that need sidecar values take the handle
///
/// A `&self` / `&mut self` method sees the struct, which holds no reference
/// to its pointer, so it can't reach the values. Take the handle instead:
/// `self: &ExternalPtr<Self>` to read, `self: &mut ExternalPtr<Self>` to
/// write (`Deref` still gives the struct fields). The same holds for a
/// struct moved out of its pointer through `&mut` (`mem::replace`,
/// `mem::take`): it carries nothing to follow, so nothing can read a freed R
/// object (#1856). A trait-impl method (`impl Trait for Type`) and an ALTREP
/// callback receive only the struct, so they can't read sidecar values
/// (#1880).
///
/// A plain `SEXP` field under `#[r_data]` is a compile error: nothing would
/// root it. Use `Sidecar<SEXP>`.
pub struct Sidecar<T> {
    /// The value the field holds while it is not in a pointer's slot.
    pending: RefCell<Option<T>>,
}

impl<T> Sidecar<T> {
    /// A sidecar field holding `value`, pending until the struct is wrapped
    /// in an `ExternalPtr`.
    pub fn new(value: T) -> Self {
        Self {
            pending: RefCell::new(Some(value)),
        }
    }

    /// Drops a pending value, which a write to the slot supersedes.
    fn discard_pending(&self) {
        *self.pending.borrow_mut() = None;
    }
}

impl<T: IntoR> Sidecar<T> {
    /// Moves a pending value into slot `index` of `prot`, if there is one.
    ///
    /// On R's main thread. `set_vector_elt` allocates nothing, so the value
    /// fresh from `into_sexp()` needs no protection.
    fn flush_into(&self, prot: SEXP, index: usize) {
        let Some(value) = self.pending.borrow_mut().take() else {
            return;
        };
        prot.set_vector_elt(prot_index(index), value.into_sexp());
    }
}

impl<T: TryFromSexp> Sidecar<T>
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    /// Reads slot `index` of `prot` back into the field as its pending value,
    /// unless one is pending already. On R's main thread.
    fn load_from(&self, prot: SEXP, index: usize, name: &'static str, owner_type: &'static str) {
        if self.pending.borrow().is_some() {
            return;
        }
        let value = convert_slot::<T>(prot, index, name, owner_type);
        *self.pending.borrow_mut() = Some(value);
    }
}

impl<T: Clone> Clone for Sidecar<T> {
    /// A `Sidecar` holding a copy of this field's pending value, if any. The
    /// value in a pointer's slot is not copied: clone the `ExternalPtr`.
    fn clone(&self) -> Self {
        Self {
            pending: RefCell::new(self.pending.borrow().clone()),
        }
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
        f.debug_struct("Sidecar")
            .field("pending", &self.pending.borrow().is_some())
            .finish()
    }
}

/// Slot `index` of `prot`, converted to `T`.
fn convert_slot<T: TryFromSexp>(prot: SEXP, index: usize, name: &str, owner_type: &str) -> T
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    let slot = prot.vector_elt(prot_index(index));
    match T::try_from_sexp(slot) {
        Ok(value) => value,
        Err(err) => panic!(
            "sidecar field `{name}` of `{owner_type}` holds an R value that does not convert to `{}`: {err}",
            std::any::type_name::<T>(),
        ),
    }
}

/// Panics unless this is R's main thread. No R API: the message names the
/// field and the owning type.
fn assert_main_thread(name: &str, owner_type: &str) {
    if !crate::worker::is_r_main_thread() {
        panic!(
            "sidecar field `{name}` of `{owner_type}`: its accessors run on R's main thread only; \
             this call came from another thread (a `worker` method, or a thread the \
             package spawned)",
        );
    }
}

// endregion

// region: The handle's view of the fields

/// A `Sidecar<T>` field, as the handle sees it through the
/// `TypedExternal::__mx_visit_sidecars` hook the derive emits. Every method
/// runs on R's main thread.
#[doc(hidden)]
pub trait SidecarField {
    /// Moves a pending value into slot `index` of `prot`.
    fn __mx_flush(&self, prot: SEXP, index: usize);
    /// Reads slot `index` of `prot` back into the field as its pending value,
    /// unless one is pending already.
    fn __mx_load(&self, prot: SEXP, index: usize, name: &'static str, owner_type: &'static str);
    /// Drops a pending value, which a write to the slot supersedes.
    fn __mx_discard_pending(&self);
}

impl<T: IntoR + TryFromSexp> SidecarField for Sidecar<T>
where
    <T as TryFromSexp>::Error: fmt::Display,
{
    fn __mx_flush(&self, prot: SEXP, index: usize) {
        self.flush_into(prot, index);
    }

    fn __mx_load(&self, prot: SEXP, index: usize, name: &'static str, owner_type: &'static str) {
        self.load_from(prot, index, name, owner_type);
    }

    fn __mx_discard_pending(&self) {
        self.discard_pending();
    }
}

/// Moves every pending value of `value`'s `Sidecar` fields into its slot of
/// `prot`, the protection list of the pointer holding `value`. On R's main
/// thread.
pub(crate) fn flush_sidecars<T: TypedExternal>(value: &T, prot: SEXP) {
    value.__mx_visit_sidecars(&mut |index, _, field| field.__mx_flush(prot, index));
}

/// Reads every slot of `prot`, the protection list of the pointer `value` is
/// leaving, back into its `Sidecar` field as a pending value (unless one is
/// pending already). On R's main thread.
pub(crate) fn load_sidecars<T: TypedExternal>(value: &T, prot: SEXP) {
    value.__mx_visit_sidecars(&mut |index, name, field| {
        field.__mx_load(prot, index, name, T::TYPE_NAME);
    });
}

/// Drops every pending value of `value`'s `Sidecar` fields.
pub(crate) fn discard_pending_sidecars<T: TypedExternal>(value: &T) {
    value.__mx_visit_sidecars(&mut |_, _, field| field.__mx_discard_pending());
}

/// Flushes a freshly wrapped value's pending values into its pointer's
/// protection list: the three construction sites and
/// `restore_after_consuming` call this once the `EXTPTRSXP` exists.
///
/// # Safety
///
/// On R's main thread; `any_raw` must own a `T`, and `owner` must be the
/// live `EXTPTRSXP` holding it, with a `prot` list built by `ExternalPtr`.
pub(super) unsafe fn init_sidecars<T: TypedExternal>(any_raw: *mut Box<dyn Any>, owner: SEXP) {
    let value: &T = unsafe { &*any_raw }
        .downcast_ref::<T>()
        .expect("ExternalPtr stores a T at construction");
    let prot = unsafe { R_ExternalPtrProtected(owner) };
    flush_sidecars(value, prot);
}

/// Copies every `Sidecar` slot of the pointer `source` into the pointer
/// `target`: the two share the R values until one side writes a field.
///
/// # Safety
///
/// On R's main thread; both must be live `EXTPTRSXP`s built by `ExternalPtr`
/// for a `T`. `set_vector_elt` allocates nothing.
pub(super) unsafe fn copy_sidecar_slots<T: TypedExternal>(source: SEXP, target: SEXP) {
    let from = unsafe { R_ExternalPtrProtected(source) };
    let to = unsafe { R_ExternalPtrProtected(target) };
    for index in 0..T::R_SLOT_COUNT {
        let at = prot_index(index);
        to.set_vector_elt(at, from.vector_elt(at));
    }
}

/// Reads `T`'s `Sidecar` field `index` of the object `ptr` holds, after
/// moving the field's pending value into the slot.
///
/// Called by the getters `#[derive(ExternalPtr)]` generates
/// (`T::field(&ptr)`); `field` selects the struct's `Sidecar` field.
///
/// # Panics
///
/// Off R's main thread, naming the field, before touching R; or when the
/// slot does not convert to `T`.
#[doc(hidden)]
pub fn sidecar_get<O, T>(
    ptr: &ExternalPtr<O>,
    index: usize,
    name: &'static str,
    field: fn(&O) -> &Sidecar<T>,
) -> T
where
    O: TypedExternal,
    T: IntoR + TryFromSexp,
    <T as TryFromSexp>::Error: fmt::Display,
{
    assert_main_thread(name, O::TYPE_NAME);
    // SAFETY: on R's main thread; the handle's EXTPTRSXP is live and was
    // built by `ExternalPtr` for an `O`, so its `prot` list has the slot.
    let prot = unsafe { R_ExternalPtrProtected(ptr.as_sexp()) };
    field(ptr).flush_into(prot, index);
    convert_slot::<T>(prot, index, name, O::TYPE_NAME)
}

/// Stores `value` in `T`'s `Sidecar` field `index` of the object `ptr` holds,
/// dropping the field's pending value, which this write supersedes.
///
/// Called by the setters `#[derive(ExternalPtr)]` generates
/// (`T::set_field(&mut ptr, value)`); `field` selects the struct's `Sidecar`
/// field.
///
/// # Panics
///
/// Off R's main thread, naming the field, before touching R.
#[doc(hidden)]
pub fn sidecar_set<O, T>(
    ptr: &mut ExternalPtr<O>,
    index: usize,
    name: &'static str,
    field: fn(&O) -> &Sidecar<T>,
    value: T,
) where
    O: TypedExternal,
    T: IntoR,
{
    assert_main_thread(name, O::TYPE_NAME);
    // SAFETY: as in `sidecar_get`. Storing allocates nothing.
    let prot = unsafe { R_ExternalPtrProtected(ptr.as_sexp()) };
    field(ptr).discard_pending();
    prot.set_vector_elt(prot_index(index), value.into_sexp());
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

/// The external pointer an R accessor's receiver `x` carries: `x` itself, or
/// the pointer an R6 / S4 / S7 handle, an environment or a list holds in
/// `.ptr`, the shapes an instance method's receiver takes
/// ([`unwrap_class_handle`]). So `Type_get_f(x)` and the `$` / `[[` field
/// methods of `r_data_accessors` read a list that carries the handle (#1848).
/// A bare pointer costs one `TYPEOF` compare; an S4 receiver reads its `ptr`
/// slot through `methods::slot()`, which allocates. The pointer is reachable
/// from `x`, which the caller's `.Call()` frame roots.
///
/// # Panics
///
/// When `x` carries no external pointer.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
unsafe fn sidecar_receiver<T: TypedExternal>(x: SEXP) -> SEXP {
    if x.type_of() == SEXPTYPE::EXTPTRSXP {
        return x;
    }
    match unsafe { unwrap_class_handle(x) } {
        Some(ptr) => ptr,
        None => panic!(
            "expected ExternalPtr<{}>, got a non-external-pointer object",
            T::TYPE_NAME
        ),
    }
}

/// The `prot` list of the external pointer `x` carries
/// ([`sidecar_receiver`]), after checking that it points to a `T`, and the
/// live `T` when it still has an address. Allocates nothing for a pointer, a
/// list or an environment receiver.
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
/// When `x` carries no external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
unsafe fn checked_prot<'a, T: TypedExternal>(x: SEXP) -> (SEXP, Option<&'a T>) {
    let x = unsafe { sidecar_receiver::<T>(x) };
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

/// The live `T` of the external pointer `x` carries ([`sidecar_receiver`]),
/// for the R getters of a struct field (a field that is not a `Sidecar`). A pointer without an address is
/// refused: with the classed restored-object errors when `T` built it
/// ([`refuse_restored`]), else with `expected ExternalPtr<T>, got a null
/// external pointer`.
///
/// # Panics
///
/// When `x` carries no live external pointer to a `T`.
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

/// The live `T` of the external pointer `x` carries, mutably, for the R
/// setters of a struct field; see [`sidecar_r_struct`].
///
/// # Panics
///
/// When `x` carries no live external pointer to a `T`.
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
/// address of the struct behind the pointer `x` carries
/// ([`sidecar_receiver`]), with mutable provenance.
unsafe fn sidecar_r_struct_ptr<T: TypedExternal>(x: SEXP) -> *mut T {
    let x = unsafe { sidecar_receiver::<T>(x) };
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

/// Checks that `x` carries an external pointer to a `T` ([`sidecar_receiver`]),
/// live or read back by `readRDS()`, before a setter validates its value.
///
/// # Panics
///
/// When `x` carries no external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
#[doc(hidden)]
pub unsafe fn sidecar_r_check<T: TypedExternal>(x: SEXP) {
    let _ = unsafe { checked_prot::<T>(x) };
}

/// Reads `T`'s `Sidecar` field `index` from the external pointer `x` carries
/// ([`sidecar_receiver`]).
///
/// Called by the R getters `#[derive(ExternalPtr)]` generates. A live
/// struct's pending value for the field is flushed into the slot first; a
/// pointer without an address (after `readRDS()`) reads its protection list
/// as is.
///
/// # Panics
///
/// When `x` carries no external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `x`.
#[doc(hidden)]
pub unsafe fn sidecar_r_get<T: TypedExternal>(x: SEXP, index: usize) -> SEXP {
    let (prot, live) = unsafe { checked_prot::<T>(x) };
    if let Some(value) = live {
        value.__mx_visit_sidecars(&mut |i, _, field| {
            if i == index {
                field.__mx_flush(prot, i);
            }
        });
    }
    prot.vector_elt(prot_index(index))
}

/// Stores `value`, which the caller validated with `TryFromSexp::<T>`, in
/// `T`'s `Sidecar` field `index` of the external pointer `x` carries
/// ([`sidecar_receiver`]).
///
/// Called by the R setters `#[derive(ExternalPtr)]` generates. A live
/// struct's pending value for the field is dropped, since this write
/// supersedes it. The store allocates nothing; reading an S4 receiver's slot
/// does, and `value`, an argument of the setter's `.Call()`, is rooted by
/// that call.
///
/// # Panics
///
/// When `x` carries no external pointer to a `T`.
///
/// # Safety
///
/// Must be called from R's main thread with valid `x` and `value`.
#[doc(hidden)]
pub unsafe fn sidecar_r_set<T: TypedExternal>(x: SEXP, index: usize, value: SEXP) {
    let (prot, live) = unsafe { checked_prot::<T>(x) };
    if let Some(data) = live {
        data.__mx_visit_sidecars(&mut |i, _, field| {
            if i == index {
                field.__mx_discard_pending();
            }
        });
    }
    prot.set_vector_elt(prot_index(index), value);
}

// endregion
