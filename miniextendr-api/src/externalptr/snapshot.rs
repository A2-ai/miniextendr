//! A saved value that comes back after `readRDS()` (#2186).
//!
//! R saves an external pointer without its address: `saveRDS()` writes the
//! pointer's protection list and tag, and `readRDS()` gives back a pointer
//! whose address is NULL. A type that is `Serialize + DeserializeOwned` (and
//! built with miniextendr-api's `serde` feature) keeps its value in that list
//! instead. The last element of its protection list is a carrier: a
//! zero-length ALTREP raw vector of a class registered for the type at
//! `R_init`. R itself calls the class's methods:
//!
//! - **`Serialized_state`**, during `serialize()`: the value encoded through
//!   the crate's serde-to-R path in its exact mode (`serde/exact.rs`, #2256),
//!   an R list the value comes back from unchanged. It
//!   reads the value through [`ExternalPtr::borrow`], so a `serialize()` that
//!   R code runs while a call holds the object mutably (a callback that saves
//!   `self`) raises the `miniextendr_borrow_conflict` error instead of
//!   reading a value under a `&mut`. A pointer without a value, or a value
//!   serde can't encode, gives no state: R writes the empty vector itself,
//!   and the reload is refused as before.
//! - **`Unserialize`**, during `unserialize()`, which loads the package (and
//!   so registers the class) first: decodes the state at once
//!   (the same exact mode) and keeps the value, or the decoding error, in the
//!   carrier. It never raises: a failure is kept for the first access.
//!
//! R reads the protection list before the pointer exists, so the value is
//! attached at the pointer's first access ([`attach`]): a method receiver, an
//! `ExternalPtr<T>` / `&T` / `&mut T` argument, or a sidecar accessor. A
//! decoding failure is refused there as a restored pointer is, with the
//! decoding error as `e$parent` ([`super::refuse_restored`]).
//!
//! The type opts in by being serde: `#[derive(ExternalPtr)]` emits a probe
//! ([`SnapshotProbe`]) that resolves to the serde arm ([`SnapshotSerde`])
//! when the type implements both traits and to the fallback
//! ([`SnapshotNone`]) otherwise, which keeps the restored refusal. A type
//! with a hand-written `TypedExternal` impl (the standard library types', or
//! `impl_typed_external!`'s) has no probe and keeps the refusal (#2243).

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::OnceLock;

use super::{
    ExternalPtr, NullAddress, PROT_TYPE_ID_INDEX, PROT_USER_INDEX, PROT_VEC_LEN, TypedExternal,
    classify_null_address, release_any, saved_identity, sidecar, split_type_id, type_id_str,
    type_id_symbol,
};
use crate::altrep::RBase;
use crate::altrep_traits::{AltVec, Altrep};
use crate::condition::{ConditionCall, RCondition};
use crate::gc_protect::ProtectScope;
use crate::sys::altrep::R_altrep_class_t;
use crate::sys::{
    R_ExternalPtrAddr, R_ExternalPtrProtected, R_MakeExternalPtr, R_RegisterCFinalizerEx,
    R_SetExternalPtrAddr, R_SetExternalPtrProtected, R_altrep_data1, R_altrep_data2,
    R_set_altrep_data1, R_set_altrep_data2, Rf_allocVector,
};
use crate::{R_xlen_t, Rboolean, SEXP, SEXPTYPE, SexpExt};

/// The class of the condition a failed decoding gives, the `e$parent` of the
/// restored refusal: the message is serde's.
pub const DECODE_ERROR_CLASS: &str = "miniextendr_decode_error";

// region: The probe

/// The carrier class of one type, registered at `R_init`. The derive emits
/// one per type, in its `TypedExternal::__mx_snapshot`.
#[doc(hidden)]
pub struct SnapshotClass(OnceLock<R_altrep_class_t>);

impl SnapshotClass {
    /// A class not registered yet.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

/// How a value of `T` goes into a save and comes back from one: its carrier
/// class and the serde encoding. `T::__mx_snapshot()` gives it for a serde
/// type.
#[doc(hidden)]
pub struct Snapshot<T> {
    class: &'static SnapshotClass,
    encode: fn(&T) -> Result<SEXP, String>,
    decode: fn(SEXP) -> Result<T, String>,
}

impl<T> Clone for Snapshot<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Snapshot<T> {}

/// The receiver of the snapshot probe: the type's class cell. The derive
/// calls `(&SnapshotProbe::<Self>::new(&CLASS)).__mx_snapshot()` with
/// [`SnapshotSerde`] and [`SnapshotNone`] in scope, in the type's own
/// `TypedExternal` impl, where the type is concrete.
#[doc(hidden)]
pub struct SnapshotProbe<T>(
    // Read by the serde arm only.
    #[cfg_attr(not(feature = "serde"), allow(dead_code))] &'static SnapshotClass,
    PhantomData<fn() -> T>,
);

impl<T> SnapshotProbe<T> {
    /// The probe for `T`, whose carrier class is `class`.
    pub const fn new(class: &'static SnapshotClass) -> Self {
        Self(class, PhantomData)
    }
}

/// Snapshot probe, serde arm: `T: Serialize + DeserializeOwned`. This impl,
/// on the probe taking `&self`, matches the receiver `&SnapshotProbe<T>` by
/// value; [`SnapshotNone`] needs one more auto-ref and is reached only when
/// `T` is not serde.
///
/// The trait exists without the `serde` feature so the generated `use`
/// resolves; it then has no impl, and every type takes [`SnapshotNone`].
#[doc(hidden)]
pub trait SnapshotSerde {
    /// The type probed.
    type Value;
    /// How a value goes into a save and comes back.
    fn __mx_snapshot(&self) -> Option<Snapshot<Self::Value>>;
}

#[cfg(feature = "serde")]
impl<T> SnapshotSerde for SnapshotProbe<T>
where
    T: TypedExternal + ::serde::Serialize + ::serde::de::DeserializeOwned,
{
    type Value = T;

    fn __mx_snapshot(&self) -> Option<Snapshot<T>> {
        Some(Snapshot {
            class: self.0,
            encode: |value| crate::serde::exact::to_r_exact(value).map_err(|e| e.to_string()),
            decode: |state| crate::serde::exact::from_r_exact(state).map_err(|e| e.to_string()),
        })
    }
}

/// Snapshot probe, fallback arm: a type that is not serde has no snapshot,
/// and a pointer to it restored from a saved session is refused.
#[doc(hidden)]
pub trait SnapshotNone {
    /// The type probed.
    type Value;
    /// `None`.
    fn __mx_snapshot(&self) -> Option<Snapshot<Self::Value>>;
}

impl<T> SnapshotNone for &SnapshotProbe<T> {
    type Value = T;

    fn __mx_snapshot(&self) -> Option<Snapshot<T>> {
        None
    }
}

/// Whether `T`'s protection list ends in a carrier slot.
#[inline]
pub(super) fn has_carrier_slot<T: TypedExternal>() -> bool {
    T::__mx_snapshot().is_some()
}

// endregion

// region: The carrier class

/// The carrier ALTREP class of `T`: a zero-length raw vector whose
/// `Serialized_state` encodes the value and whose `Unserialize` decodes it.
///
/// `data1` is the owning pointer while the value lives in it, and a holder
/// ([`Decoded`]) from `Unserialize` until the first access attaches the
/// value; `data2` is then the state that was read, which a save before the
/// first access writes back unchanged.
struct Carrier<T>(PhantomData<T>);

impl<T: TypedExternal> Altrep for Carrier<T> {
    fn length(_x: SEXP) -> R_xlen_t {
        0
    }

    const HAS_SERIALIZED_STATE: bool = true;

    fn serialized_state(x: SEXP) -> SEXP {
        // SAFETY: an ALTREP method, on R's main thread, with a carrier of `T`.
        unsafe { serialized_state::<T>(x) }
    }

    const HAS_UNSERIALIZE: bool = true;

    fn unserialize(_class: SEXP, state: SEXP) -> SEXP {
        // SAFETY: an ALTREP method, on R's main thread; `state` is rooted by
        // R's reader.
        unsafe { unserialize::<T>(state) }
    }
}

impl<T: TypedExternal> AltVec for Carrier<T> {
    // A zero-length vector's data is never read; R gives such vectors the
    // pointer 0x1 too. Without these, R's defaults would raise on a
    // `DATAPTR()` (a plain copy, a version-2 save).
    const HAS_DATAPTR: bool = true;

    fn dataptr(_x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        NonNull::<u8>::dangling().as_ptr().cast()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(_x: SEXP) -> *const core::ffi::c_void {
        NonNull::<u8>::dangling().as_ptr().cast_const().cast()
    }
}

/// The carrier class name of the type with ID `id`:
/// `miniextendr_saved:<crate>::<Type>`, without the crate version or the
/// module path ([`saved_identity`]), so a save by any version, or of a type
/// since moved to another module, finds the class. A hand-written ID
/// without a version is used whole.
fn class_name_of(id: &str) -> String {
    match saved_identity(id) {
        Some((crate_name, _, name)) => format!("miniextendr_saved:{crate_name}::{name}"),
        None => format!("miniextendr_saved:{id}"),
    }
}

thread_local! {
    /// The carrier class names registered so far, each with the ID of the
    /// type that registered it. Only `R_init`, on R's main thread, claims.
    static CLAIMED: RefCell<Vec<(String, &'static str)>> = const { RefCell::new(Vec::new()) };
}

/// Claims the carrier class `name` for the type with ID `id`. Two serde
/// types of one crate with the same name would share the class, and each
/// would read the other's saves, so the second one is an error naming both
/// types' paths. Claiming a name again for the same type is a no-op.
fn claim(
    claimed: &mut Vec<(String, &'static str)>,
    name: &str,
    id: &'static str,
) -> Result<(), String> {
    match claimed
        .iter()
        .find(|(claimed_name, _)| claimed_name == name)
    {
        Some(&(_, other)) if other != id => {
            let path = |id| split_type_id(id).2;
            Err(format!(
                "miniextendr: the serde `ExternalPtr` types `{}` and `{}` have the same name in \
                 one crate. A saved object is found by its crate and type name, so each would \
                 read the other's saves; rename one",
                path(other),
                path(id),
            ))
        }
        Some(_) => Ok(()),
        None => {
            claimed.push((name.to_owned(), id));
            Ok(())
        }
    }
}

/// Registers `T`'s carrier class when `T` is serde, once. The registration
/// function `#[derive(ExternalPtr)]` emits for the type calls it from
/// `R_init` (through `MX_ALTREP_REGISTRATIONS`).
///
/// # Panics
///
/// When another serde type of `T`'s crate with `T`'s name registered its
/// class first (`claim`). `R_init` turns the panic into an R error, so the
/// package does not load.
#[doc(hidden)]
pub fn register<T: TypedExternal>() {
    let Some(snapshot) = T::__mx_snapshot() else {
        return;
    };
    let id = type_id_str::<T>();
    let name = class_name_of(id);
    // Release the borrow before raising.
    let claimed = CLAIMED.with_borrow_mut(|claimed| claim(claimed, &name, id));
    if let Err(message) = claimed {
        panic!("{message}");
    }
    snapshot.class.0.get_or_init(|| {
        let name = std::ffi::CString::new(name).expect("a type ID has no NUL byte");
        // SAFETY: during `R_init`, after the package name and DLL are set.
        let class = unsafe {
            crate::altrep::make_class_by_base(
                name.as_ptr(),
                crate::AltrepPkgName::as_ptr(),
                RBase::Raw,
            )
        };
        // SAFETY: a fresh raw class.
        unsafe {
            crate::altrep_bridge::install_base::<Carrier<T>>(class);
            crate::altrep_bridge::install_vec::<Carrier<T>>(class);
        }
        class
    });
}

/// `T`'s registered carrier class, if `T` is serde and its class is
/// registered (it is not while the wrappers are generated).
fn carrier_class<T: TypedExternal>() -> Option<(Snapshot<T>, R_altrep_class_t)> {
    let snapshot = T::__mx_snapshot()?;
    let class = *snapshot.class.0.get()?;
    Some((snapshot, class))
}

/// Puts a carrier for the value `owner` holds in the last slot of its new
/// protection list. A construction site calls it after the address is set.
///
/// # Safety
///
/// On R's main thread, with `owner` a rooted pointer built for a `T` whose
/// protection list is `prot_vec_len::<T>()` long.
pub(super) unsafe fn init_carrier<T: TypedExternal>(owner: SEXP) {
    let Some((_, class)) = carrier_class::<T>() else {
        return;
    };
    // `R_new_altrep` protects its data while it allocates; the owner roots
    // the protection list.
    let carrier = unsafe { class.new_altrep(owner, SEXP::nil()) };
    let prot = unsafe { R_ExternalPtrProtected(owner) };
    prot.set_vector_elt(prot.xlength() - 1, carrier);
}

// endregion

// region: Saving and reading back

/// What `Unserialize` decoded: the value until the first access takes it, or
/// the condition decoding raised. The holder pointer in the carrier's `data1`
/// owns it, freed by the ordinary finalizer.
type Decoded<T> = RefCell<Option<Result<T, RCondition>>>;

/// The decoded value a holder pointer owns, when `holder` is one for `T`.
///
/// # Safety
///
/// On R's main thread with a valid `holder`.
unsafe fn decoded<'a, T: TypedExternal>(holder: SEXP) -> Option<&'a Decoded<T>> {
    if holder.type_of() != SEXPTYPE::EXTPTRSXP {
        return None;
    }
    let any_raw = unsafe { R_ExternalPtrAddr(holder) }.cast::<Box<dyn Any>>();
    if any_raw.is_null() {
        return None;
    }
    unsafe { &*any_raw }.downcast_ref::<Decoded<T>>()
}

/// `Serialized_state`: the state of the carrier `x`.
///
/// # Safety
///
/// On R's main thread, with `x` a carrier of `T`.
unsafe fn serialized_state<T: TypedExternal>(x: SEXP) -> SEXP {
    let data1 = unsafe { R_altrep_data1(x) };
    // Read back and not attached yet (or not decodable): save what was read.
    if unsafe { decoded::<T>(data1) }.is_some() {
        return unsafe { R_altrep_data2(x) };
    }
    let Some(snapshot) = T::__mx_snapshot() else {
        return SEXP::null();
    };
    if data1.type_of() != SEXPTYPE::EXTPTRSXP {
        return SEXP::null();
    }
    // SAFETY: on R's main thread with an EXTPTRSXP; the handle is borrowed,
    // the carrier's owner roots the pointer.
    let Some(ptr) = (unsafe { ExternalPtr::<T>::wrap_sexp(data1) }) else {
        return SEXP::null();
    };
    // A running call holding the value mutably raises
    // `miniextendr_borrow_conflict` here, which aborts the `serialize()`.
    let value = ptr.borrow();
    // A value serde can't encode: no state, so R writes the empty vector and
    // the reload is refused, as before the type was serde.
    (snapshot.encode)(&value).unwrap_or(SEXP::null())
}

/// `Unserialize`: decodes `state` into a holder and returns a carrier
/// holding it. Never raises (only an R error from the type's own
/// `Deserialize` code carries on).
///
/// # Safety
///
/// On R's main thread, during `unserialize()`, after `T`'s class was
/// registered (R loads the package first); `state` is rooted.
unsafe fn unserialize<T: TypedExternal>(state: SEXP) -> SEXP {
    let (snapshot, class) =
        carrier_class::<T>().expect("R unserializes a carrier only after its class is registered");
    let decoded: Decoded<T> = RefCell::new(Some(decode(snapshot.decode, state)));
    let inner: Box<dyn Any> = Box::new(decoded);
    let any_raw = Box::into_raw(Box::new(inner));
    // Own the allocation until the finalizer that frees it is registered.
    let allocation = unsafe { Box::from_raw(any_raw) };
    let scope = unsafe { ProtectScope::new() };
    let holder = unsafe { R_MakeExternalPtr(std::ptr::null_mut(), SEXP::nil(), SEXP::nil()) };
    unsafe { scope.protect(holder) };
    unsafe { R_RegisterCFinalizerEx(holder, Some(release_any), Rboolean::TRUE) };
    let any_raw = Box::into_raw(allocation);
    unsafe { R_SetExternalPtrAddr(holder, any_raw.cast()) };
    unsafe { class.new_altrep(holder, state) }
}

/// Decodes `state`, catching a panic: an `RCondition` as itself, any other
/// as a [`DECODE_ERROR_CLASS`] condition with its message. An R error keeps
/// unwinding: its jump belongs to R (#2244).
fn decode<T>(decode: fn(SEXP) -> Result<T, String>, state: SEXP) -> Result<T, RCondition> {
    let decoding_failed = |message: String| RCondition::Error {
        message,
        class: vec![DECODE_ERROR_CLASS.to_owned()],
        data: None,
        call: ConditionCall::None,
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode(state))) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(message)) => Err(decoding_failed(message)),
        Err(payload) if payload.is::<crate::unwind_protect::RUnwind>() => {
            std::panic::resume_unwind(payload)
        }
        Err(payload) => match payload.downcast::<RCondition>() {
            Ok(condition) => Err(*condition),
            Err(payload) => Err(decoding_failed(
                crate::unwind_protect::panic_payload_to_string(&*payload).into_owned(),
            )),
        },
    }
}

// endregion

// region: Attaching at first access

/// What [`attach`] did with a pointer without an address.
pub(crate) enum Attach {
    /// The decoded value is in the pointer now.
    Attached,
    /// The save could not be decoded; the pointer stays without a value.
    Failed,
    /// No carrier of `T`: not a serde type, a save without one (a version-2
    /// save, one read where the package was missing), or not a save of `T`.
    NoCarrier,
}

/// The carrier of `sexp`'s saved protection list, and what it decoded, when
/// `sexp` is a save of `T` with one.
///
/// # Safety
///
/// On R's main thread, with `sexp` a valid `EXTPTRSXP` without an address.
unsafe fn saved_carrier<'a, T: TypedExternal>(sexp: SEXP) -> Option<(SEXP, &'a Decoded<T>)> {
    let (_, class) = carrier_class::<T>()?;
    if !matches!(
        unsafe { classify_null_address::<T>(sexp) },
        NullAddress::Restored | NullAddress::OtherVersion { .. }
    ) {
        return None;
    }
    let saved = unsafe { R_ExternalPtrProtected(sexp) };
    let len = saved.xlength();
    if saved.type_of() != SEXPTYPE::VECSXP || len <= PROT_VEC_LEN {
        return None;
    }
    let carrier = saved.vector_elt(len - 1);
    if !unsafe { class.inherits(carrier) } {
        return None;
    }
    let decoded = unsafe { decoded::<T>(R_altrep_data1(carrier)) }?;
    Some((carrier, decoded))
}

/// The condition decoding `sexp`'s save raised, when `T` is serde and the
/// save carries a value it could not decode.
///
/// # Safety
///
/// As [`saved_carrier`].
pub(crate) unsafe fn decode_failure<T: TypedExternal>(sexp: SEXP) -> Option<RCondition> {
    let (_, decoded) = unsafe { saved_carrier::<T>(sexp) }?;
    match &*decoded.borrow() {
        Some(Err(cause)) => Some(cause.clone()),
        _ => None,
    }
}

/// Gives `sexp`, a pointer without an address, the value its save decoded,
/// when `T` is serde and the save carries one (#2186): the first access of a
/// pointer `readRDS()` gave back. Raises nothing: a failure stays in the
/// carrier, for the caller's refusal ([`super::refuse_restored`]) to name as
/// `e$parent`.
///
/// The pointer gets a protection list in this version's layout, as a save by
/// any version of the crate is attached: its type ID and slot names, the
/// saved user slot, for each `Sidecar` field the saved slot of its R name
/// (#2187) or else the decoded value's pending value (`#[serde(skip)]` gives
/// the field's `Default`), and the carrier, which holds the pointer from now
/// on.
///
/// # Safety
///
/// On R's main thread, with `sexp` a valid `EXTPTRSXP` without an address,
/// rooted for the call.
#[cold]
pub(crate) unsafe fn attach<T: TypedExternal>(sexp: SEXP) -> Attach {
    let Some((carrier, decoded)) = (unsafe { saved_carrier::<T>(sexp) }) else {
        return Attach::NoCarrier;
    };
    let value = {
        let mut slot = decoded.borrow_mut();
        match slot.take() {
            Some(Ok(value)) => value,
            failed @ Some(Err(_)) => {
                *slot = failed;
                return Attach::Failed;
            }
            None => return Attach::NoCarrier,
        }
    };
    unsafe { install::<T>(sexp, carrier, value) };
    Attach::Attached
}

/// Puts `value` in `sexp`, as construction puts a new value in a new pointer
/// (the finalizer first, then the address), with the protection list
/// [`attach`] describes.
///
/// # Safety
///
/// As [`attach`]; `carrier` is the last element of `sexp`'s saved list.
unsafe fn install<T: TypedExternal>(sexp: SEXP, carrier: SEXP, value: T) {
    let saved = unsafe { R_ExternalPtrProtected(sexp) };
    let scope = unsafe { ProtectScope::new() };
    // Detached from `sexp` below, the saved list stays rooted by the scope
    // while its slots are read.
    unsafe { scope.protect(saved) };
    let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, super::prot_vec_len::<T>()) };
    unsafe { scope.protect(prot) };
    prot.set_vector_elt(PROT_TYPE_ID_INDEX, unsafe { type_id_symbol::<T>() });
    prot.set_vector_elt(PROT_USER_INDEX, saved.vector_elt(PROT_USER_INDEX));
    if T::R_SLOT_COUNT > 0 {
        unsafe { sidecar::name_slots::<T>(prot) };
    }
    unsafe { R_SetExternalPtrProtected(sexp, prot) };
    // A field the save has a slot for keeps it; the others take the decoded
    // value's pending value, rooted until the slot holds it (#2216).
    value.__mx_visit_sidecars(
        &mut |index, name, field| match saved_field_slot(saved, index, name) {
            Some(at) => prot.set_vector_elt(sidecar::prot_index(index), saved.vector_elt(at)),
            None => field.__mx_flush(prot, index),
        },
    );
    sidecar::discard_pending_sidecars(&value);
    prot.set_vector_elt(prot.xlength() - 1, carrier);
    unsafe { R_set_altrep_data1(carrier, sexp) };
    unsafe { R_set_altrep_data2(carrier, SEXP::nil()) };

    let (_, any_raw) = ExternalPtr::<T>::erase(value);
    // Own the allocation until the finalizer that frees it is registered.
    let allocation = unsafe { Box::from_raw(any_raw) };
    unsafe { R_RegisterCFinalizerEx(sexp, super::finalizer::<T>(), Rboolean::TRUE) };
    let any_raw = Box::into_raw(allocation);
    unsafe { R_SetExternalPtrAddr(sexp, any_raw.cast()) };
}

/// The position in the saved list `saved` of `Sidecar` field `index`, named
/// `name`: by name ([`sidecar::saved_slot`]), or, in a list without slot
/// names (a version whose type had no `Sidecar` field), by position, short
/// of the carrier in the last slot.
fn saved_field_slot(saved: SEXP, index: usize, name: &str) -> Option<R_xlen_t> {
    if sidecar::has_slot_names(saved) {
        sidecar::saved_slot(saved, index, name)
    } else {
        Some(sidecar::prot_index(index)).filter(|&at| at < saved.xlength() - 1)
    }
}

// endregion

#[cfg(test)]
mod tests;
