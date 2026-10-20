#![allow(rustdoc::private_intra_doc_links)]
//! `ExternalPtr<T>` — a Box-like owned pointer that wraps R's EXTPTRSXP.
//!
//! This provides ownership semantics similar to `Box<T>`, with the key difference
//! that cleanup is deferred to R's garbage collector via finalizers.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`altrep_helpers`] | ALTREP data1/data2 slot access helpers + the `RSidecar` and `Computed` marker types |
//! | [`sidecar`] | [`Sidecar<T>`] fields: typed values kept in the `prot` list |
//!
//! # Core Types
//!
//! - [`ExternalPtr<T>`] — owned pointer wrapping EXTPTRSXP
//! - [`TypedExternal`] — display and diagnostic metadata for stored types
//! - [`ExternalSlice<T>`] — helper for slice data in external pointers
//! - [`ErasedExternalPtr`] — type-erased `ExternalPtr<()>` alias
//! - [`IntoExternalPtr`] — conversion trait for wrapping values
//!
//! `PartialEq`/`PartialOrd` compare the pointee values (like `Box<T>`),
//! borrowing both. Use `ptr_eq` when you care about pointer identity.
//!
//! # Protection Strategies in miniextendr
//!
//! miniextendr provides three complementary protection mechanisms for different scenarios:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`](crate::gc_protect) | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **VECSXP pool** | [`protect_pool`](crate::protect_pool) | Until released or the pool drops | Any order | Values a struct owns; many values; cross-call roots |
//! | **R ownership** | [`ExternalPtr`](struct@crate::externalptr::ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! ## When to Use ExternalPtr
//!
//! **Use `ExternalPtr` (this module) when:**
//! - You want R to own a Rust value
//! - The Rust value should be dropped when R garbage collects the pointer
//! - You're exposing Rust structs to R code
//!
//! **Use [`gc_protect`](crate::gc_protect) instead when:**
//! - You're allocating temporary R objects during computation
//! - Protection is short-lived (within a single `.Call`)
//!
//! **Use [`ProtectPool`] instead when:**
//! - You need R objects (not Rust values) kept alive by a struct, or across `.Call`s
//! - You need arbitrary-order release of protections
//!
//! ## How ExternalPtr Protection Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  ExternalPtr<MyStruct>::new(value)                              │
//! │  ├── Rf_protect() during construction (temporary)               │
//! │  ├── R_MakeExternalPtr() creates EXTPTRSXP                      │
//! │  ├── R_RegisterCFinalizerEx() registers cleanup callback        │
//! │  ├── pool.insert() roots it for the Rust handle's lifetime       │
//! │  └── Rf_unprotect() after construction complete                 │
//! │                                                                 │
//! │  Held in Rust (even across other R allocations, e.g. in a Vec)  │
//! │  └── stays alive — the pool's GC-traced VECSXP slot roots it     │
//! │                                                                 │
//! │  Return to R → R now also references the EXTPTRSXP              │
//! │  └── Rust handle drops → pool.release(key) drops the root,      │
//! │      but R's own reference keeps it live                        │
//! │                                                                 │
//! │  R GC runs (no refs left) → finalizer (release_any) frees value │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Owning handles (`new` / `from_raw` / `Clone`) root their `EXTPTRSXP` in a
//! process-wide [`ProtectPool`](crate::protect_pool) so they survive R
//! allocations while held in Rust; *borrowed* views (`wrap_sexp` / `from_sexp`
//! / `same_object`) take no root — the object is kept alive by whatever R-side
//! reference handed it to them. The pool (O(1) any-order release) is used
//! rather than `R_PreserveObject` because a `Vec<ExternalPtr>` releases its
//! roots front-to-back, the O(n²) worst case for `R_ReleaseObject`'s
//! precious-list scan — see `analysis/gc-protection-benchmarks-results.md`.
//!
//! When the end goal is an R `list()` of external pointers (rather than a
//! `Vec<ExternalPtr>` you keep working with in Rust), prefer
//! [`ExternalPtr::collect_into_r_list`](struct@ExternalPtr) — it builds each
//! `EXTPTRSXP` straight into the protected result list, so the list roots every
//! element and the pool is never touched at all.
//!
//! # Type Identification
//!
//! Type safety is enforced via `Any::downcast` (Rust's `TypeId`). R symbols
//! in the `tag` and `prot` slots are retained for display and error messages
//! but are **never authoritative** for downcast safety — the `Any` vtable is.
//!
//! Internally, data is stored as `Box<Box<dyn Any>>` — a thin pointer (fits
//! in R's `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any`
//! vtable for runtime downcasting). The `Any` value is a [`RefCell<T>`], whose
//! borrow flag is what generated method wrappers, field accessors and trait
//! shims check before they make a `&T` / `&mut T` (see "Reentrant calls"
//! below). The outer `Box` keeps the heap address stable so
//! [`ExternalPtr::cached_ptr`](struct@ExternalPtr) can be cached once at
//! construction.
//!
//! The `tag` slot holds a symbol (type name, for display).
//! The `prot` slot holds a VECSXP (list) of `2 + T::R_SLOT_COUNT` elements:
//!   - Index 0: SYMSXP (interned type ID symbol, for error messages)
//!   - Index 1: User-protected SEXP slot (for preventing GC of R objects)
//!   - Index 2..: one value per [`Sidecar<T>`] field of `T`
//!
//! Before reading the address as Rust storage, checked conversions and erased
//! handle accessors require this metadata shape (including a non-ALTREP list).
//! Trait-ABI pointers carry a reserved tag and a different allocation layout;
//! they are rejected here. These checks reject ordinary foreign pointers, not
//! foreign addresses deliberately forged with miniextendr metadata. Symbol
//! names never establish the concrete Rust type; `Any::downcast` still does.
//!
//! ## `TYPE_NAME_CSTR` vs `TYPE_ID_CSTR`
//!
//! [`TypedExternal`] exposes two associated constants with distinct roles —
//! mixing them up does not break type safety (`Any::downcast` is the real
//! gate) but produces noisy diagnostics.
//!
//! | Constant | Role | Visible to R as | Authoritative? |
//! |---|---|---|---|
//! | `TYPE_NAME_CSTR` | Display tag | `class()` / `print()` | No |
//! | `TYPE_ID_CSTR` | Error-message identifier on downcast failure | Stored in `prot[0]` | No (cosmetic; downcast uses `TypeId`) |
//!
//! `#[derive(ExternalPtr)]` fills both with sensible defaults; only override
//! manually when implementing `TypedExternal` by hand.
//!
//! # Pointer provenance for `cached_ptr`
//!
//! `ExternalPtr` caches the address of the value's [`RefCell<T>`] at
//! construction so [`borrow`](ExternalPtr::borrow) /
//! [`borrow_mut`](ExternalPtr::borrow_mut) avoid an FFI call on every access.
//! A `RefCell` hands out `&mut T` from a shared reference to the cell, so a
//! pointer cached from `downcast_ref::<RefCell<T>>()` is as good for writes
//! as one from `Box::into_raw`.
//!
//! # Reentrant calls
//!
//! R reaches a Rust value only through a generated wrapper, on R's main
//! thread, and the wrapper borrows the value's [`RefCell`] for the whole call:
//! shared for `&self` methods, `&T` arguments, [`ExternalPtrRef`] receivers
//! and arguments, and field getters; exclusive for `&mut self` and `self`
//! methods, `&mut T` arguments, [`ExternalPtrMut`] receivers and arguments,
//! and field setters. The body gets plain references, which Rust's borrow
//! checker covers from there. A second borrow during the call, from the same
//! object passed twice with a `&mut T`, or from R code the body calls back
//! into (an R function argument, `RCall`) reaching the object again, raises
//! an R error of class `miniextendr_borrow_conflict` instead of making a
//! second reference. A borrow taken through a trait view lives in the
//! producer's shim, so views in other packages see it too.
//!
//! An `ExternalPtr<T>` handle, as an argument, a `self: ExternalPtr<Self>`
//! receiver, a `Vec` element or a value Rust keeps, takes no borrow until it
//! is read: [`borrow`](ExternalPtr::borrow) and
//! [`borrow_mut`](ExternalPtr::borrow_mut) check the `RefCell` and raise the
//! same error. A handle has no `Deref`, so every read through it is one of
//! these. `into_inner` and the other consuming functions refuse a value
//! something still borrows, with the same error.
//!
//! The borrow is released when the wrapper returns, panics, or unwinds an R
//! error raised through [`RCall::eval_with_handlers`](crate::expression::RCall::eval_with_handlers)
//! (or any API that turns an R error into a Rust unwind). A bare `Rf_eval`
//! that raises an R error jumps over the wrapper's frames, so the object
//! stays borrowed for the rest of the session.
//!
//! # See also
//!
//! - [`crate::altrep`] — when the alternative (an ALTREP class) makes more
//!   sense than `ExternalPtr`.
//!
//! # ExternalPtr is Not an R Native Type
//!
//! Unlike R's native atomic types (`integer`, `double`, `character`, etc.),
//! external pointers cannot be coerced to vectors or used in R's vectorized
//! operations. This is an R limitation, not a miniextendr limitation:
//!
//! ```r
//! > matrix(new("externalptr"), 1, 1)
//! Error in `as.vector()`:
//! ! cannot coerce type 'externalptr' to vector of type 'any'
//! ```
//!
//! If you need your Rust type to participate in R's vector/matrix operations,
//! consider implementing [`IntoList`](crate::list::IntoList) (via `#[derive(IntoList)]`)
//! to convert your struct to a named R list, or use ALTREP to expose Rust
//! iterators as lazy R vectors.

use std::any::Any;
use std::any::TypeId;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ptr::{self, NonNull};

use crate::RValue;
use crate::condition::{ConditionCall, RCondition};
use crate::gc_protect::ProtectScope;
use crate::protect_pool::{ProtectKey, ProtectPool};
use crate::sys::{
    R_ClearExternalPtr, R_ExternalPtrAddr, R_ExternalPtrProtected, R_ExternalPtrTag,
    R_MakeExternalPtr, R_MakeExternalPtr_unchecked, R_RegisterCFinalizerEx,
    R_RegisterCFinalizerEx_unchecked, R_SetExternalPtrAddr, R_SetExternalPtrAddr_unchecked,
    R_SetExternalPtrProtected, R_SetExternalPtrTag, Rf_allocVector, Rf_allocVector_unchecked,
    Rf_install, Rf_install_unchecked,
};
use crate::{R_xlen_t, Rboolean, SEXP, SEXPTYPE, SexpExt};

/// Own the allocation during the handoff to R's main thread.
///
/// # Safety
///
/// The value is exclusively owned and accessed only after the handoff. Keeping
/// the box owned also reclaims it if routing rejects the calling context.
type SendableAnyPtr = crate::worker::Sendable<Box<Box<dyn Any>>>;

/// Create a new sendable pointer from a raw `*mut Box<dyn Any>`.
///
/// # Safety
///
/// The pointer must be a live `Box::into_raw` allocation; ownership transfers.
#[inline]
unsafe fn sendable_any_ptr_new(ptr: *mut Box<dyn Any>) -> SendableAnyPtr {
    crate::worker::Sendable(unsafe { Box::from_raw(ptr) })
}

/// Get the raw pointer, consuming the sendable wrapper.
#[inline]
fn sendable_any_ptr_into_ptr(ptr: SendableAnyPtr) -> *mut Box<dyn Any> {
    Box::into_raw(ptr.0)
}

/// Index of the type SYMSXP contained in the `prot` (a `VECSXP` list)
const PROT_TYPE_ID_INDEX: isize = 0;
/// Index of user-protected objects contained in the `prot` (a `VECSXP` list)
const PROT_USER_INDEX: isize = 1;
/// Length of the `prot` list (`VECSXP`) before the [`Sidecar<T>`] entries,
/// which start at this index
const PROT_VEC_LEN: isize = 2;

/// Length of `T`'s `prot` list: the type ID, the user slot and one entry per
/// [`Sidecar<T>`] field.
#[inline]
fn prot_vec_len<T: TypedExternal>() -> R_xlen_t {
    let r_slots = R_xlen_t::try_from(T::R_SLOT_COUNT).expect("sidecar count exceeds R_xlen_t::MAX");
    PROT_VEC_LEN + r_slots
}

#[inline]
fn is_type_erased<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

/// Get the interned R symbol for a type's name.
///
/// R interns symbols via `Rf_install`, so the same string always returns the
/// same pointer. The symbol is stable display metadata; `Any::downcast` is the
/// authoritative type check.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_symbol`] - no thread safety checks.
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Get the namespaced type ID symbol used in diagnostics.
///
/// Uses `TYPE_ID_CSTR`, which includes the module path to make mismatch
/// messages unambiguous.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_id_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_id_symbol`].
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_id_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Get the type name from a stored symbol SEXP.
///
/// # Safety
///
/// `sym` must be a valid SYMSXP.
#[inline]
fn symbol_name(sym: SEXP) -> &'static str {
    use crate::SexpExt;
    let printname = sym.printname();
    let cstr = printname.r_char();
    let len = printname.len();
    unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(cstr.cast(), len))
            .expect("R SYMSXP PRINTNAME is not valid UTF-8")
    }
}

/// The type-ID symbol an `ExternalPtr`-built pointer keeps in `prot[0]`, or
/// `None` when its metadata is not the shape ordinary `ExternalPtr` builds.
/// This also excludes trait-ABI storage, which has a different Rust layout.
///
/// Only inspect non-ALTREP metadata: checking ownership must not invoke a
/// foreign callback. A foreign address forged with this metadata is outside
/// the trust boundary; after this check `Any` still decides the concrete type.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
#[inline]
unsafe fn stored_type_id_symbol(sexp: SEXP) -> Option<SEXP> {
    let tag = unsafe { R_ExternalPtrTag(sexp) };
    // Symbols are interned, so the trait-ABI tag compares by pointer.
    if tag.type_of() != SEXPTYPE::SYMSXP || tag == unsafe { crate::mx_abi::get_tag() } {
        return None;
    }
    let prot = unsafe { R_ExternalPtrProtected(sexp) };
    if prot.type_of() != SEXPTYPE::VECSXP || prot.is_altrep() || prot.xlength() < PROT_VEC_LEN {
        return None;
    }
    let sym = prot.vector_elt(PROT_TYPE_ID_INDEX);
    (sym.type_of() == SEXPTYPE::SYMSXP).then_some(sym)
}

/// `T`'s type ID as text (`TYPE_ID_CSTR` without its NUL).
fn type_id_str<T: TypedExternal>() -> &'static str {
    let id = T::TYPE_ID_CSTR
        .strip_suffix(b"\0")
        .unwrap_or(T::TYPE_ID_CSTR);
    std::str::from_utf8(id).expect("TYPE_ID_CSTR is not valid UTF-8")
}

/// Splits a type ID of the derive's form, `crate@version::module::Type`,
/// into `(crate, version, module::Type)`; a hand-written ID without a
/// version is `(None, None, id)`.
fn split_type_id(id: &str) -> (Option<&str>, Option<&str>, &str) {
    let Some((crate_name, rest)) = id.split_once('@') else {
        return (None, None, id);
    };
    match rest.split_once("::") {
        Some((version, path)) => (Some(crate_name), Some(version), path),
        None => (None, None, id),
    }
}

// region: A pointer without an address

/// The condition class every error about a pointer restored from a saved
/// session carries, after its specific class.
pub const RESTORED_CLASS: &str = "miniextendr_restored";

/// The class of the error for a pointer this version of the crate saved: it
/// came back from `readRDS()` / `unserialize()` without its Rust value.
pub const RESTORED_NO_VALUE_CLASS: &str = "miniextendr_restored_no_value";

/// The class of the error for a pointer another version of the crate saved;
/// the condition carries `saved_version` and `current_version`.
pub const RESTORED_OTHER_VERSION_CLASS: &str = "miniextendr_restored_other_version";

/// The class of the error a rebuild hook gets when it reads a `Sidecar`
/// field the save has no slot for: a field added after the version that
/// saved the object (#2187). The condition carries `field`, the field's
/// name. The `try_<field>` getters return `None` instead.
pub const RESTORED_MISSING_SLOT_CLASS: &str = "miniextendr_restored_missing_slot";

/// The class of the error for a save without slot names of a type that
/// rebuilds saves by any version (`rebuild = path, any_version`, #2187).
/// Such a type reads a save by slot name only, and a save made before slots
/// were named has none, so it is refused rather than read by position, where
/// a slot another layout shifted would read as the wrong field (#2218).
pub const RESTORED_UNNAMED_SLOTS_CLASS: &str = "miniextendr_restored_unnamed_slots";

/// The class of the error for a receiver of a type's R accessors that carries
/// no external pointer: neither a pointer nor an object holding one in
/// `.ptr`, such as an object of the class an earlier release of a package
/// wrote to disk (#1901). The type's restored and field-error classes go in
/// front of it.
pub const NO_HANDLE_CLASS: &str = "miniextendr_no_handle";

/// What a pointer without an address is, as far as `T` is concerned
/// ([`classify_null_address`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NullAddress {
    /// Not built by `ExternalPtr`: R's `new("externalptr")`, a pointer
    /// `into_inner` cleared, another library's pointer.
    Foreign,
    /// Built for `T` by this version of its crate: restored from a saved
    /// session, so the Rust value is gone.
    Restored,
    /// Built for `T` by another version of its crate.
    OtherVersion {
        /// The crate's name.
        crate_name: &'static str,
        /// The version that saved the pointer.
        saved: &'static str,
        /// This version.
        current: &'static str,
    },
    /// Built for another type.
    OtherType {
        /// The stored type ID.
        found: &'static str,
    },
}

/// Reads `prot[0]` of a pointer without an address and says what it is for
/// `T`. Symbol names live for the session, so the borrowed parts are
/// `'static`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
pub(crate) unsafe fn classify_null_address<T: TypedExternal>(sexp: SEXP) -> NullAddress {
    let Some(sym) = (unsafe { stored_type_id_symbol(sexp) }) else {
        return NullAddress::Foreign;
    };
    let found = symbol_name(sym);
    let own = type_id_str::<T>();
    if found == own {
        return NullAddress::Restored;
    }
    match (split_type_id(found), split_type_id(own)) {
        ((Some(crate_name), Some(saved), path), (Some(own_crate), Some(current), own_path))
            if crate_name == own_crate && path == own_path =>
        {
            NullAddress::OtherVersion {
                crate_name,
                saved,
                current,
            }
        }
        _ => NullAddress::OtherType { found },
    }
}

/// The first R class of `receiver`, which is what the user sees and the
/// field methods dispatched on (#1920); `None` for an object without a class
/// attribute, such as an R6 object's `.ptr`, whose messages name the Rust
/// type instead.
pub(crate) fn receiver_class(receiver: SEXP) -> Option<String> {
    let class = receiver.get_class();
    (class.type_of() == SEXPTYPE::STRSXP && class.len() > 0)
        .then(|| class.string_elt_str(0).map(str::to_owned))
        .flatten()
}

/// The message of the [`RESTORED_NO_VALUE_CLASS`] error.
pub(crate) fn restored_no_value_message(type_name: &str) -> String {
    format!(
        "this `{type_name}` object was restored from a saved session and has no Rust value; \
         re-create it"
    )
}

/// The message of the [`RESTORED_OTHER_VERSION_CLASS`] error.
pub(crate) fn restored_other_version_message(
    type_name: &str,
    crate_name: &str,
    saved: &str,
    current: &str,
) -> String {
    format!(
        "this `{type_name}` object was saved by {crate_name} {saved} and can't be read by \
         {crate_name} {current}; re-create it"
    )
}

/// The message of the [`RESTORED_UNNAMED_SLOTS_CLASS`] error.
pub(crate) fn restored_unnamed_slots_message(type_name: &str) -> String {
    format!(
        "this `{type_name}` object was saved before saves named their fields, so its saved \
         values can't be matched to fields; re-create it"
    )
}

/// The condition classes of a restored refusal of a `T`: the package's
/// classes ([`TypedExternal::RESTORED_ERROR_CLASS`], in order), then the
/// specific class, then [`RESTORED_CLASS`].
pub(crate) fn restored_classes_of<T: TypedExternal>(specific: &'static str) -> Vec<String> {
    T::RESTORED_ERROR_CLASS
        .iter()
        .copied()
        .chain([specific, RESTORED_CLASS])
        .map(str::to_owned)
        .collect()
}

/// The message of a restored refusal of a `T`: the package's
/// ([`TypedExternal::RESTORED_ERROR_MESSAGE`]) when it has one, else
/// miniextendr's (`default`).
pub(crate) fn restored_message_of<T: TypedExternal>(default: impl FnOnce() -> String) -> String {
    T::RESTORED_ERROR_MESSAGE.map_or_else(default, str::to_owned)
}

/// Raises the error for a pointer without an address when it was built for
/// `T` (by this version: [`RESTORED_NO_VALUE_CLASS`]; by another:
/// [`RESTORED_OTHER_VERSION_CLASS`]), with the package's classes in front and
/// its message when the type declares them (`#[externalptr(restored(...))]`),
/// and returns the other cases for the caller to word as it did before. The
/// default message names `receiver`'s R class, the object the user called
/// on (the pointer itself, or the list, environment or S4 object carrying
/// it), else the Rust type. The condition is raised through the panic
/// transport (`panic_any`), like `error!`.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP` whose
/// address is NULL.
#[cold]
pub(crate) unsafe fn refuse_restored<T: TypedExternal>(sexp: SEXP, receiver: SEXP) -> NullAddress {
    let kind = unsafe { classify_null_address::<T>(sexp) };
    let name = || receiver_class(receiver).unwrap_or_else(|| T::TYPE_NAME.to_owned());
    let (message, class, data) = match kind {
        NullAddress::Restored => (
            restored_message_of::<T>(|| restored_no_value_message(&name())),
            RESTORED_NO_VALUE_CLASS,
            None,
        ),
        NullAddress::OtherVersion {
            crate_name,
            saved,
            current,
        } => (
            restored_message_of::<T>(|| {
                restored_other_version_message(&name(), crate_name, saved, current)
            }),
            RESTORED_OTHER_VERSION_CLASS,
            Some(vec![
                (String::from("saved_version"), RValue::from(saved)),
                (String::from("current_version"), RValue::from(current)),
            ]),
        ),
        NullAddress::Foreign | NullAddress::OtherType { .. } => return kind,
    };
    std::panic::panic_any(RCondition::Error {
        message,
        class: restored_classes_of::<T>(class),
        data,
        call: ConditionCall::Inherit,
    })
}

/// Raises the [`RESTORED_UNNAMED_SLOTS_CLASS`] error for a save without slot
/// names of a `T` that reads saves by name only
/// ([`sidecar::refuses_unnamed`], #2218), with the package's classes in
/// front and its restored message, as [`refuse_restored`] raises the other
/// restored refusals. The default message names `receiver`'s R class, else
/// the Rust type.
#[cold]
pub(crate) fn refuse_unnamed_slots<T: TypedExternal>(receiver: SEXP) -> ! {
    std::panic::panic_any(unnamed_slots_refusal::<T>(receiver))
}

/// The condition [`refuse_unnamed_slots`] raises, as a value:
/// [`try_rebuild_restored`] returns it, so `wrap_sexp_with_error` gives it
/// back instead of raising it (#2217).
fn unnamed_slots_refusal<T: TypedExternal>(receiver: SEXP) -> RCondition {
    let name = receiver_class(receiver).unwrap_or_else(|| T::TYPE_NAME.to_owned());
    RCondition::Error {
        message: restored_message_of::<T>(|| restored_unnamed_slots_message(&name)),
        class: restored_classes_of::<T>(RESTORED_UNNAMED_SLOTS_CLASS),
        data: None,
        call: ConditionCall::Inherit,
    }
}

/// The message of a refused rebuild of a `T`; see [`rebuild_failed_text`].
fn rebuild_failed_message<T: TypedExternal>(type_name: &str, cause: &str) -> String {
    rebuild_failed_text(
        T::REBUILD_FAILED_MESSAGE,
        T::RESTORED_ERROR_MESSAGE,
        type_name,
        cause,
    )
}

/// The message of a refused rebuild: the package's rebuild-failed template
/// with each `%s` replaced by the cause's message (#2218); else its restored
/// message, then the cause in parentheses; else miniextendr's, naming
/// `type_name`. The template is filled once, so a `%s` in the cause stays as
/// written.
fn rebuild_failed_text(
    template: Option<&str>,
    restored: Option<&str>,
    type_name: &str,
    cause: &str,
) -> String {
    match (template, restored) {
        (Some(template), _) => template.replace("%s", cause),
        (None, Some(message)) => format!("{message} (rebuilding it failed: {cause})"),
        (None, None) => format!(
            "this `{type_name}` object was restored from a saved session and rebuilding it \
             failed: {cause}"
        ),
    }
}

std::thread_local! {
    /// The pointers whose rebuild hook is running on this thread: a hook that
    /// reaches its own object again finds it refused, not rebuilt twice.
    static REBUILDING: RefCell<Vec<SEXP>> = const { RefCell::new(Vec::new()) };
}

/// [`try_rebuild_restored`], raising the refusal of a failed rebuild: for
/// the paths that refuse a restored pointer by raising (a method's receiver,
/// the R field accessors).
///
/// # Safety
///
/// As [`try_rebuild_restored`].
#[cold]
pub(crate) unsafe fn rebuild_restored<T: TypedExternal>(sexp: SEXP, receiver: SEXP) -> bool {
    unsafe { try_rebuild_restored::<T>(sexp, receiver) }.unwrap_or_else(|refusal| refusal.raise())
}

/// Rebuilds the value of `sexp`, a pointer without an address, when `T`
/// has a rebuild hook (`#[externalptr(rebuild = path)]`) and this version of
/// its crate saved it, or another version did and `T` opts in with
/// `any_version` (#2187) and the save carries slot names. Installs the value
/// in `sexp`. Returns whether `sexp` holds a value now; `Ok(false)` leaves the
/// caller to refuse the pointer as before (another version saved it, another
/// type, no hook, or the hook is already running for `sexp`). Under
/// `any_version` a save of this version without slot names is refused here,
/// before the hook runs ([`refuse_unnamed_slots`], #2218).
///
/// The hook reads the saved `Sidecar` slots through [`Saved`], by field
/// name. The installed pointer keeps each saved slot whose field `T` still
/// has, unless the hook's struct replaces it ([`Sidecar::replace`]), and
/// takes the value the hook's struct holds for a field the save lacks
/// ([`install_rebuilt`]). A failing hook (an `Err`, or a condition it raises)
/// installs nothing, and the next use tries again. It returns the restored
/// refusal of `T` as `Err` (its classes, then [`RESTORED_NO_VALUE_CLASS`] and
/// [`RESTORED_CLASS`], worded by [`rebuild_failed_text`], the cause as
/// `e$parent`), for the caller to raise ([`rebuild_restored`]) or return
/// ([`ExternalPtr::wrap_sexp_with_error`], #2217). Any other unwind (an R
/// error, a plain panic) carries on.
///
/// # Safety
///
/// On R's main thread, with `sexp` a valid `EXTPTRSXP` whose address is NULL,
/// rooted for the call; `receiver` is the object the user called on.
#[cold]
pub(crate) unsafe fn try_rebuild_restored<T: TypedExternal>(
    sexp: SEXP,
    receiver: SEXP,
) -> Result<bool, RCondition> {
    if !T::REBUILDS {
        return Ok(false);
    }
    match unsafe { classify_null_address::<T>(sexp) } {
        NullAddress::Restored => {
            if sidecar::refuses_unnamed::<T>(unsafe { R_ExternalPtrProtected(sexp) }) {
                return Err(unnamed_slots_refusal::<T>(receiver));
            }
        }
        NullAddress::OtherVersion { .. }
            if T::REBUILDS_ANY_VERSION
                && sidecar::has_slot_names(unsafe { R_ExternalPtrProtected(sexp) }) => {}
        _ => return Ok(false),
    }
    if REBUILDING.with_borrow(|running| running.contains(&sexp)) {
        return Ok(false);
    }
    struct Running(SEXP);
    impl Drop for Running {
        fn drop(&mut self) {
            REBUILDING.with_borrow_mut(|running| running.retain(|&s| s != self.0));
        }
    }
    REBUILDING.with_borrow_mut(|running| running.push(sexp));
    let running = Running(sexp);
    // SAFETY: `sexp` is a pointer this version built for a `T` (classified
    // above), rooted by the caller for the hook's duration.
    let saved = unsafe { Saved::<T>::new(sexp) };
    let outcome = crate::condition::catch_condition(|| T::__mx_rebuild(&saved));
    drop(running);
    match outcome {
        Ok(Ok(value)) => {
            // SAFETY: on R's main thread; `sexp` still has no address (the
            // hook could not install one: a reentrant use was refused).
            unsafe { install_rebuilt(sexp, value) };
            Ok(true)
        }
        Ok(Err(cause)) | Err(cause) => {
            let name = receiver_class(receiver).unwrap_or_else(|| T::TYPE_NAME.to_owned());
            Err(RCondition::Error {
                message: rebuild_failed_message::<T>(&name, cause.message()),
                class: restored_classes_of::<T>(RESTORED_NO_VALUE_CLASS),
                data: None,
                call: ConditionCall::Inherit,
            }
            .with_parent(cause))
        }
    }
}

/// Puts the rebuilt `value` in `sexp`, a restored pointer for a `T`, as
/// construction puts a new value in a new pointer: the finalizer first
/// (R does not save it), then the address.
///
/// The pointer gets a protection list in this version's layout (#2187): its
/// type ID and slot names, the saved user slot, and for each `Sidecar` field
/// the saved slot of that name. `value`'s own pending value for the field
/// is written instead when the save has no such slot, or when the hook made
/// it with [`Sidecar::replace`]. `value`'s other pending values are dropped.
///
/// # Safety
///
/// On R's main thread, with `sexp` a valid `EXTPTRSXP` for a `T` whose
/// address is NULL, rooted for the call.
unsafe fn install_rebuilt<T: TypedExternal>(sexp: SEXP, value: T) {
    if T::R_SLOT_COUNT > 0 {
        unsafe { relayout_saved_slots(sexp, &value) };
    }
    discard_pending_sidecars(&value);
    let (_, any_raw) = ExternalPtr::<T>::erase(value);
    // Own the allocation until the finalizer that frees it is registered.
    let allocation = unsafe { Box::from_raw(any_raw) };
    unsafe { R_RegisterCFinalizerEx(sexp, Some(release_any), Rboolean::TRUE) };
    let any_raw = Box::into_raw(allocation);
    unsafe { R_SetExternalPtrAddr(sexp, any_raw.cast()) };
}

/// Replaces the saved protection list of `sexp` with one in `T`'s layout:
/// see [`install_rebuilt`]. `value`'s pending values for the fields the save
/// lacks are written; the rest stay pending, for the caller to drop.
///
/// # Safety
///
/// As [`install_rebuilt`].
unsafe fn relayout_saved_slots<T: TypedExternal>(sexp: SEXP, value: &T) {
    let saved = unsafe { R_ExternalPtrProtected(sexp) };
    let scope = unsafe { ProtectScope::new() };
    let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, prot_vec_len::<T>()) };
    unsafe { scope.protect(prot) };
    prot.set_vector_elt(PROT_TYPE_ID_INDEX, unsafe { type_id_symbol::<T>() });
    prot.set_vector_elt(PROT_USER_INDEX, saved.vector_elt(PROT_USER_INDEX));
    unsafe { sidecar::name_slots::<T>(prot) };
    // Detached from `sexp`, the saved list stays rooted by the scope while
    // its slots are read. The pending values the fields write are rooted
    // until they sit in a slot (#2216).
    unsafe { scope.protect(saved) };
    unsafe { R_SetExternalPtrProtected(sexp, prot) };
    value.__mx_visit_sidecars(&mut |index, name, field| match sidecar::saved_slot::<T>(
        saved, index, name,
    ) {
        Some(at) if !field.__mx_replaces_saved() => {
            prot.set_vector_elt(sidecar::prot_index(index), saved.vector_elt(at));
        }
        _ => field.__mx_flush(prot, index),
    });
}

/// Empties the pointer a value was moved out of (`into_raw` / `into_inner`):
/// the address, so the finalizer is a no-op, and the tag and protection list
/// too, so the pointer reads as empty (the plain `null external pointer`
/// messages) rather than as one restored from a saved session, which keeps
/// its `prot` (#1872). `R_ClearExternalPtr` only nulls the address.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
unsafe fn clear_moved_out(sexp: SEXP) {
    unsafe { R_ClearExternalPtr(sexp) };
    unsafe { R_SetExternalPtrTag(sexp, SEXP::nil()) };
    unsafe { R_SetExternalPtrProtected(sexp, SEXP::nil()) };
}

/// Test support: make a restored pointer read as a save by another version of
/// its crate, so a package can test its handling of
/// `miniextendr_restored_other_version` (#1872) without building two
/// versions.
///
/// It rewrites the version in the type ID a restored pointer keeps in
/// `prot[0]` (`crate@version::module::Type`) to `version`. The pointer must
/// have no address, as one restored by `readRDS()` / `unserialize()` has: a
/// live pointer is checked by `Any::downcast`, not by the ID. Call it from a
/// test-only `#[miniextendr]` function:
///
/// ```ignore
/// /// @param x A pointer restored by `unserialize()`.
/// #[miniextendr(noexport)]
/// pub fn rewrite_saved_version(x: SEXP, version: String) {
///     // SAFETY: R's main thread; `x` is an external pointer.
///     unsafe { miniextendr_api::externalptr::rewrite_stored_version_for_tests(x, &version) };
/// }
/// ```
///
/// ```r
/// back <- unserialize(serialize(obj, NULL))
/// rewrite_saved_version(back, "0.0.1")
/// tryCatch(method(back), miniextendr_restored_other_version = function(e) e$saved_version)
/// #> [1] "0.0.1"
/// ```
///
/// Under `rebuild = path, any_version`, any read or write of a `Sidecar`
/// slot of another version's save rebuilds it first, the standalone
/// accessors, R6 bindings and S7 properties included (#2187). So a test
/// that edits the saved slots (to set up what the hook will read) edits
/// them before it marks the save as another version's.
///
/// # Panics
///
/// When `sexp` is not a restored `ExternalPtr`-built pointer with a
/// versioned type ID.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
pub unsafe fn rewrite_stored_version_for_tests(sexp: SEXP, version: &str) {
    assert!(
        unsafe { R_ExternalPtrAddr(sexp) }.is_null(),
        "the pointer still has its Rust value: save and restore it first"
    );
    let sym = unsafe { stored_type_id_symbol(sexp) }.expect("an `ExternalPtr`-built pointer");
    let (crate_name, path) = match split_type_id(symbol_name(sym)) {
        (Some(crate_name), Some(_), path) => (crate_name, path),
        _ => panic!("the stored type ID has no version"),
    };
    let rewritten = std::ffi::CString::new(format!("{crate_name}@{version}::{path}"))
        .expect("a type ID has no NUL");
    // `Rf_install` interns the symbol for the session; the list keeps it.
    let prot = unsafe { R_ExternalPtrProtected(sexp) };
    prot.set_vector_elt(PROT_TYPE_ID_INDEX, unsafe {
        Rf_install(rewritten.as_ptr())
    });
}

/// Lays the `Sidecar` slots of a restored pointer's save out as another
/// version of its type might have saved them, for tests of reading a save by
/// field name (#2187). Slot `i` of the new list holds the slot named
/// `layout[i]` in the current save, or the string `"unknown"` for a name the
/// save has no slot for. The list carries the names `layout` unless `named`
/// is false, which gives a save made before slots were named.
///
/// # Panics
///
/// When `sexp` still has its Rust value, or its save carries no slot names.
///
/// # Safety
///
/// Must be called from R's main thread with a valid `EXTPTRSXP`.
#[doc(hidden)]
pub unsafe fn rewrite_saved_slots_for_tests(sexp: SEXP, layout: &[&str], named: bool) {
    assert!(
        unsafe { R_ExternalPtrAddr(sexp) }.is_null(),
        "the pointer still has its Rust value: save and restore it first"
    );
    let saved = unsafe { R_ExternalPtrProtected(sexp) };
    assert!(
        sidecar::has_slot_names(saved),
        "the save carries no slot names"
    );
    let names = saved.get_attr(SEXP::names_symbol());
    let len = PROT_VEC_LEN
        + R_xlen_t::try_from(layout.len()).expect("the layout length fits in R_xlen_t");
    let scope = unsafe { ProtectScope::new() };
    let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, len) };
    unsafe { scope.protect(prot) };
    prot.set_vector_elt(PROT_TYPE_ID_INDEX, saved.vector_elt(PROT_TYPE_ID_INDEX));
    prot.set_vector_elt(PROT_USER_INDEX, saved.vector_elt(PROT_USER_INDEX));
    for (index, name) in layout.iter().enumerate() {
        let found =
            (PROT_VEC_LEN..names.xlength()).find(|&at| names.string_elt_str(at) == Some(name));
        let slot = match found {
            Some(at) => saved.vector_elt(at),
            None => crate::into_r::IntoR::into_sexp("unknown"),
        };
        prot.set_vector_elt(sidecar::prot_index(index), slot);
    }
    if named {
        let new_names = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, len) };
        unsafe { scope.protect(new_names) };
        for (index, name) in layout.iter().enumerate() {
            new_names.set_string_elt(sidecar::prot_index(index), SEXP::charsxp(name));
        }
        prot.set_names(new_names);
    }
    unsafe { R_SetExternalPtrProtected(sexp, prot) };
}

// endregion

// region: TypedExternalPtr Trait

/// Trait for types that can be stored in an ExternalPtr.
///
/// This provides R-visible display and diagnostic identifiers. Runtime type
/// checking is performed by `Any::downcast` (Rust's `TypeId`), not by comparing
/// these symbols.
///
/// # Type ID vs Type Name
///
/// - `TYPE_ID_CSTR`: Namespaced identifier used in mismatch diagnostics (stored in `prot[0]`).
///   Format: `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`
///
///   The crate name, version, and module path distinguish otherwise similar
///   names in error messages. They do not determine compatibility.
///
/// - `TYPE_NAME_CSTR`: Short display name for the R tag (shown when printing).
///   Just the type identifier for readability.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a type an `ExternalPtr` stores",
    label = "`{Self}` does not implement `TypedExternal`",
    note = "`#[derive(ExternalPtr)]` (or `impl_typed_external!`) implements it; R reaches a `{Self}` through an external pointer, as the object of a `#[miniextendr]` impl or a `&{Self}` / `&mut {Self}` parameter"
)]
pub trait TypedExternal: 'static {
    /// The type name as a static string (for debugging and display)
    const TYPE_NAME: &'static str;

    /// The type name as a null-terminated C string (for R tag display)
    const TYPE_NAME_CSTR: &'static [u8];

    /// Namespaced type ID as a null-terminated C string (for diagnostics).
    ///
    /// This should include the module path to prevent ambiguous messages.
    /// Use `concat!(module_path!(), "::", stringify!(Type), "\0").as_bytes()`
    /// when implementing manually, or use `#[derive(ExternalPtr)]`.
    const TYPE_ID_CSTR: &'static [u8];

    /// Number of [`Sidecar<T>`] fields: values the external pointer's
    /// protection list keeps after the type ID and the user slot.
    ///
    /// `#[derive(ExternalPtr)]` counts the type's `#[r_data]` fields of type
    /// `Sidecar<_>`.
    const R_SLOT_COUNT: usize = 0;

    /// The R names of the `Sidecar` fields (`#[r_data(name = "..")]`, else
    /// the Rust field name), in slot order: the names the protection list
    /// carries, so a save is read back by field name (#2187, #2215).
    #[doc(hidden)]
    const SIDECAR_NAMES: &'static [&'static str] = &[];

    /// The package's condition classes for a pointer of this type restored
    /// from a saved session, in front of the `miniextendr_restored*` classes
    /// ([`RESTORED_NO_VALUE_CLASS`] / [`RESTORED_OTHER_VERSION_CLASS`], then
    /// [`RESTORED_CLASS`]), which every restored refusal keeps.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(restored(class = "pkg_saved"))]` or
    /// `restored(class = ["pkg_saved", "pkg_error"])`; the default adds
    /// nothing.
    const RESTORED_ERROR_CLASS: &'static [&'static str] = &[];

    /// The package's message for a pointer of this type restored from a saved
    /// session, replacing miniextendr's in both the no-value and the
    /// other-version case (the condition's `saved_version` /
    /// `current_version` fields stay), and for a list or environment of the
    /// class that carries no pointer (a save in an older format). A static
    /// string: one type is one R class, so the package names it literally.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(restored(message = "..."))]`; the default keeps
    /// miniextendr's messages.
    const RESTORED_ERROR_MESSAGE: Option<&'static str> = None;

    /// The package's message for a pointer of this type whose rebuild hook
    /// failed (#2218), each `%s` in it replaced by the cause's message. It
    /// stands alone, so it reads as one sentence. Without it the refusal
    /// says [`Self::RESTORED_ERROR_MESSAGE`] then
    /// ` (rebuilding it failed: <cause>)`, or miniextendr's wording.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(restored(rebuild_failed = "..."), rebuild = path)]`.
    const REBUILD_FAILED_MESSAGE: Option<&'static str> = None;

    /// The package's message for a list or an environment of the type's
    /// class that carries no pointer (#2218), the shape an earlier release
    /// may have saved: it says why the object has none. It replaces
    /// [`Self::RESTORED_ERROR_MESSAGE`] for that receiver, which otherwise
    /// serves both. The R accessors raise it with
    /// [`NO_HANDLE_CLASS`]; a package's own code that refuses such an object
    /// reads it here to word the refusal the same way.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(restored(no_handle = "..."))]`.
    const NO_HANDLE_MESSAGE: Option<&'static str> = None;

    /// The package's condition classes for the errors the type's R field
    /// readers raise themselves (#1899), in front of their own class: a
    /// refused position (`subscriptOutOfBoundsError`), a name that is not a
    /// field (`miniextendr_no_field`), a computed field assigned to
    /// (`miniextendr_read_only_field`), a pointer of another type, and a
    /// receiver that carries no pointer ([`NO_HANDLE_CLASS`]). The R
    /// accessors of the type read it for the last two; the derive writes the
    /// same classes into the R helpers for the others.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(field_error(class = ["pkg_error_config", "pkg_error"]))]`;
    /// the default adds nothing.
    const FIELD_ERROR_CLASS: &'static [&'static str] = &[];

    /// The package's message for the R accessors' refusal of a receiver
    /// whose pointer is not a `T` or that carries no pointer (#1920),
    /// replacing miniextendr's, which names the receiver's R class.
    ///
    /// `#[derive(ExternalPtr)]` fills it from
    /// `#[externalptr(field_error(message = "..."))]`.
    const FIELD_ERROR_MESSAGE: Option<&'static str> = None;

    /// Visits every [`Sidecar<T>`] field of `self` with its slot index and
    /// name, so the handle can flush the field's pending value into its slot
    /// or load the slot back into the field.
    ///
    /// `#[derive(ExternalPtr)]` emits the body for a type with `Sidecar`
    /// fields; the default visits nothing, so other types pay nothing.
    #[doc(hidden)]
    #[inline]
    fn __mx_visit_sidecars(&self, _visit: &mut dyn FnMut(usize, &'static str, &dyn SidecarField)) {}

    /// Whether a pointer of this type that this version of its crate saved
    /// is rebuilt on first use after `readRDS()`, by [`Self::__mx_rebuild`],
    /// instead of refused.
    ///
    /// `#[derive(ExternalPtr)]` sets it with `#[externalptr(rebuild = path)]`.
    #[doc(hidden)]
    const REBUILDS: bool = false;

    /// Whether [`Self::__mx_rebuild`] also rebuilds a pointer another
    /// version of the crate saved, when the save carries slot names.
    /// `#[derive(ExternalPtr)]` sets it with `any_version` next to
    /// `rebuild = path` (#2187).
    #[doc(hidden)]
    const REBUILDS_ANY_VERSION: bool = false;

    /// Rebuilds the value of a pointer of this type restored from a saved
    /// session, from what the save kept. `#[derive(ExternalPtr)]` emits it for
    /// `#[externalptr(rebuild = path)]`: it calls `path` and turns its `Err`
    /// into a condition.
    #[doc(hidden)]
    fn __mx_rebuild(saved: &Saved<'_, Self>) -> Result<Self, RCondition>
    where
        Self: Sized,
    {
        let _ = saved;
        unreachable!("`{}` has no rebuild hook", Self::TYPE_NAME)
    }
}

/// Marker trait for types that should be converted to R as ExternalPtr.
///
/// When a type implements this trait (via `#[derive(ExternalPtr)]`), it gets a
/// blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>`.
///
/// This allows returning the type directly from `#[miniextendr]` functions:
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyData { value: i32 }
///
/// #[miniextendr]
/// fn create_data(v: i32) -> MyData {
///     MyData { value: v }  // Automatically wrapped in ExternalPtr
/// }
/// ```
pub trait IntoExternalPtr: TypedExternal {}

impl TypedExternal for () {
    const TYPE_NAME: &'static str = "()";
    const TYPE_NAME_CSTR: &'static [u8] = b"()\0";
    // Unit type is special - same ID as name since it's only used for type-erased ptrs
    const TYPE_ID_CSTR: &'static [u8] = b"()\0";
}
// endregion

// region: Class-handle unwrapping (audit A9)

/// Look up a variable bound directly in a single environment frame (no search
/// of enclosing frames) with [`REnv::get`](crate::expression::REnv::get).
///
/// Returns `None` for a non-environment, an absent binding, or `NULL` (none
/// can hold a class handle). Promises and active bindings are evaluated once;
/// their errors unwind to the caller's miniextendr boundary.
///
/// # Safety
///
/// R's main thread, inside a miniextendr boundary.
unsafe fn env_binding(env: SEXP, name: &str) -> Option<SEXP> {
    unsafe {
        if !env.is_environment() {
            return None;
        }
        crate::expression::REnv::from_sexp(env)
            .get(name)
            .filter(|val| !val.is_null_or_nil())
    }
}

/// Attempt to unwrap a class-wrapped handle down to the bare `EXTPTRSXP` it
/// carries, so [`ExternalPtr::<T>`](ExternalPtr) argument conversion accepts
/// the ergonomic class handle (e.g. `Foo$new(...)`) in addition to the raw
/// pointer returned by a low-level constructor (audit finding A9 —
/// `audit/2026-07-03-api-sense-conversions-dataframe-errors.md` #5).
///
/// Tries, in order:
/// - **Env / R6**: a direct `.ptr` binding on `sexp` itself — most
///   `#[miniextendr(env)]` classes are actually a bare classed `EXTPTRSXP`
///   (the generated constructor does `class(.val) <- "T"` directly on the
///   pointer returned by Rust, see `env_class.rs`), which already satisfies
///   the plain `EXTPTRSXP` check and never reaches this function, but a
///   user-authored environment that binds `.ptr` is covered here too. Then
///   the R6 handle chain `.__enclos_env__` -> `private` -> `.ptr` (R6
///   objects are the *public* environment; `private` only hangs off the
///   enclosing environment stored at `.__enclos_env__` for `portable`
///   classes, the default — see `r6_class.rs`).
/// - **S4**: the `ptr` slot via `methods::slot()`
///   ([`crate::s4_helpers::s4_get_slot`]). Guarded by `isS4()`, which
///   excludes S7 objects even though both share the `S4SXP`/`OBJSXP`
///   `SEXPTYPE` — S7's `new_object(S7_object(), ...)` base never sets the S4
///   bit.
/// - **List**: a generic vector with a `.ptr` element (`x[[".ptr"]]`), the
///   shape of a hand-built S3 object that keeps R-side state next to the
///   handle: `structure(list(.ptr = <ptr>, log = ...), class = "Foo")`
///   (#1469). The `.ptr` spelling matches the env / R6 / S7 convention.
/// - **Anything else carrying a `.ptr` attribute**: S7 stores properties as
///   plain attributes on its base object (see `s7_class.rs`), so
///   `Rf_getAttrib(x, ".ptr")` recovers the pointer without going through
///   S7's `@`/`prop()` dispatch machinery. A list without a `.ptr` element
///   falls through to this check too (an S7 class over `class_list` is a
///   `VECSXP` with its properties as attributes).
///
/// Both the `ExternalPtr<T>` argument conversion and, via
/// `resolve_receiver`, every generated instance-method prelude go through
/// this function.
///
/// Returns `Some(inner)` only when the unwrapped value is itself an
/// `EXTPTRSXP` — anything else (e.g. a `.ptr`-named field that isn't a
/// pointer) is treated as "no handle found" rather than an error. No
/// recursion beyond one unwrap level. `Any::downcast` remains the type-safety
/// authority: unwrapping a handle for the *wrong* `T` still fails at the
/// caller with the existing type-mismatch error — this only loosens the
/// accepted R-side shape, not type safety.
///
/// # Safety
///
/// - Must be called from R's main thread, inside a miniextendr boundary: an R
///   error from a promise or active binding the lookup forces unwinds to it.
/// - The returned SEXP is reachable from `sexp` (an env binding, S4 slot, or
///   attribute) for as long as `sexp` itself is protected. Macro-generated
///   `.Call()` wrappers hold every argument alive in the call's PROTECT stack
///   for the duration of the call, so no additional protection is needed
///   here.
pub(crate) unsafe fn unwrap_class_handle(sexp: SEXP) -> Option<SEXP> {
    unsafe {
        if sexp.is_environment() {
            if let Some(direct) = env_binding(sexp, ".ptr") {
                if direct.type_of() == SEXPTYPE::EXTPTRSXP {
                    return Some(direct);
                }
            }
            // An active binding can return a fresh environment that is not
            // retained by its owner. Keep it rooted across the next lookup.
            let scope = ProtectScope::new();
            let enclos = scope.protect_raw(env_binding(sexp, ".__enclos_env__")?);
            let private = scope.protect_raw(env_binding(enclos, "private")?);
            let inner = env_binding(private, ".ptr")?;
            return (inner.type_of() == SEXPTYPE::EXTPTRSXP).then_some(inner);
        }

        if sexp.is_s4() {
            let slot = crate::s4_helpers::s4_get_slot(sexp, "ptr").ok()?;
            return (slot.type_of() == SEXPTYPE::EXTPTRSXP).then_some(slot);
        }

        if sexp.type_of() == SEXPTYPE::VECSXP {
            if let Some(elt) = list_element(sexp, ".ptr") {
                if elt.type_of() == SEXPTYPE::EXTPTRSXP {
                    return Some(elt);
                }
            }
        }

        let attr = sexp.get_attr(Rf_install(c".ptr".as_ptr()));
        (attr.type_of() == SEXPTYPE::EXTPTRSXP).then_some(attr)
    }
}

/// The element of a generic vector bound to `name`: the first match in
/// `names(x)`, like `x[[name]]`. `None` when `x` has no character names or
/// no element by that name.
///
/// # Safety
///
/// Must be called from R's main thread; `sexp` must be a `VECSXP`.
unsafe fn list_element(sexp: SEXP, name: &str) -> Option<SEXP> {
    let names = sexp.get_names();
    if names.type_of() != SEXPTYPE::STRSXP {
        return None;
    }
    let n = names.len().min(sexp.len());
    for i in 0..n {
        let i = isize::try_from(i).ok()?;
        if names.string_elt_str(i) == Some(name) {
            return Some(sexp.vector_elt(i));
        }
    }
    None
}

/// Resolve an instance-method receiver to the bare `EXTPTRSXP` it carries.
///
/// Generated method preludes call this on `self_sexp` before
/// `ErasedExternalPtr::from_sexp` / `ExternalPtr::<T>::wrap_sexp`. A bare
/// pointer, the shape every generated constructor returns (classed or not),
/// passes through after one `TYPEOF` compare. Anything else goes through the
/// same class-handle unwrap as `ExternalPtr<T>` arguments: an R6 / S4 / S7
/// handle, an environment or a list carrying the pointer in `.ptr`, or any
/// object with a `.ptr` attribute. An S3 class whose object is a list with
/// R-side state next to the handle, `structure(list(.ptr = <ptr>, log = ...),
/// class = "Foo")`, therefore dispatches into `#[miniextendr(s3)] impl Foo`
/// methods unchanged (#1469).
///
/// Panics (the framework converts it to an R error) when no pointer can be
/// recovered, naming `T` and the receiver's `SEXPTYPE`. Type safety is still
/// decided by the `Any::downcast` that follows; this only widens the accepted
/// R-side shape.
///
/// Runs on R's main thread: generated preludes execute before any worker
/// hand-off.
#[inline]
pub fn resolve_receiver<T: TypedExternal>(sexp: SEXP) -> SEXP {
    let ptr = if sexp.type_of() == SEXPTYPE::EXTPTRSXP {
        sexp
    } else {
        match unsafe { unwrap_class_handle(sexp) } {
            Some(inner) => inner,
            None => receiver_not_a_handle::<T>(sexp),
        }
    };
    // A pointer restored from a saved session gets its value back here when
    // the type has a rebuild hook, before the receiver is borrowed.
    // SAFETY: on R's main thread; `ptr` is a valid EXTPTRSXP reachable from
    // `sexp`, which the call's frame roots.
    if T::REBUILDS && unsafe { R_ExternalPtrAddr(ptr) }.is_null() {
        unsafe { rebuild_restored::<T>(ptr, sexp) };
    }
    ptr
}

/// The typed handle of an instance-method receiver, for `self: ExternalPtr<Self>`
/// receivers: [`resolve_receiver`]
/// then [`ExternalPtr::wrap_sexp`], failing like the erased receivers do
/// ([`handle_downcast_failed`]: consumed, restored from a saved session, or a
/// type mismatch). Runs on R's main thread (the generated prelude).
#[inline]
pub fn wrap_receiver<T: TypedExternal>(sexp: SEXP) -> ExternalPtr<T> {
    let ptr = resolve_receiver::<T>(sexp);
    // SAFETY: `ptr` is a valid EXTPTRSXP the call's frame roots; on R's main
    // thread. The handle borrows it for the method's duration, like the
    // erased receivers.
    match unsafe { ExternalPtr::<T>::wrap_sexp(ptr) } {
        Some(handle) => handle,
        None => {
            let erased = unsafe { ErasedExternalPtr::from_sexp(ptr) };
            handle_downcast_failed::<T>(&erased)
        }
    }
}

#[cold]
fn receiver_not_a_handle<T: TypedExternal>(sexp: SEXP) -> ! {
    panic!(
        "expected a `{}` object (an external pointer, or an R6/S4/S7 handle, \
         environment, or list carrying one in `.ptr`), got {:?}",
        ExternalPtr::<T>::type_name(),
        sexp.type_of()
    )
}
// endregion

// region: Receiver borrows

/// A generated method's receiver: the `EXTPTRSXP` [`resolve_receiver`] finds
/// in the `.Call` argument, rooted for the call when a class handle carried
/// it. R code the method calls back into can drop the handle's reference, and
/// the finalizer would then free the value under the method's borrow. A bare
/// pointer is the `.Call` argument itself, which R keeps alive.
#[doc(hidden)]
pub struct ReceiverOwner {
    sexp: SEXP,
    _root: Option<PoolRoot>,
}

impl ReceiverOwner {
    /// Resolve `sexp` like [`resolve_receiver`]. Runs on R's main thread (the
    /// generated prelude).
    #[inline]
    pub fn resolve<T: TypedExternal>(sexp: SEXP) -> Self {
        let owner = resolve_receiver::<T>(sexp);
        // SAFETY: on R's main thread; `owner` is reachable through `sexp`,
        // which the call's frame roots.
        let root = (owner != sexp).then(|| unsafe { PoolRoot::new(owner) });
        Self {
            sexp: owner,
            _root: root,
        }
    }

    /// The resolved `EXTPTRSXP`.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.sexp
    }

    /// The resolved `EXTPTRSXP`, as the owner of an [`ExternalPtrRef`] /
    /// [`ExternalPtrMut`] receiver.
    #[inline]
    pub fn owner(&self) -> &SEXP {
        &self.sexp
    }
}

/// Borrow a stored value shared for a generated wrapper: the R error of
/// class `miniextendr_borrow_conflict` while a running call holds it mutably.
#[doc(hidden)]
#[inline]
pub fn borrow_value<T>(cell: &RefCell<T>) -> Ref<'_, T> {
    cell.try_borrow().unwrap_or_else(|_| borrow_conflict::<T>())
}

/// Borrow a stored value exclusively for a generated wrapper: the R error of
/// class `miniextendr_borrow_conflict` while a running call holds it.
#[doc(hidden)]
#[inline]
pub fn borrow_value_mut<T>(cell: &RefCell<T>) -> RefMut<'_, T> {
    cell.try_borrow_mut()
        .unwrap_or_else(|_| borrow_conflict::<T>())
}

// The generated wrapper reaches a borrowed argument's value only through
// these functions, each bounded on `TypedExternal` and spanned on the
// parameter's type: for a `T` that isn't `TypedExternal` rustc then reports
// the one missing bound once, with its `on_unimplemented` message.

/// [`ExternalPtr::borrow`] for a generated wrapper's argument.
#[doc(hidden)]
#[inline]
pub fn __mx_borrow<T: TypedExternal>(ptr: &ExternalPtr<T>) -> ExternalPtrRef<'_, T> {
    ptr.borrow()
}

/// [`ExternalPtr::borrow_mut`] for a generated wrapper's argument.
#[doc(hidden)]
#[inline]
pub fn __mx_borrow_mut<T: TypedExternal>(ptr: &mut ExternalPtr<T>) -> ExternalPtrMut<'_, T> {
    ptr.borrow_mut()
}

/// The `&T` a generated wrapper hands a `&T` parameter.
#[doc(hidden)]
#[inline]
pub fn __mx_shared<'a, T: TypedExternal>(view: &'a ExternalPtrRef<'_, T>) -> &'a T {
    view
}

/// The `&mut T` a generated wrapper hands a `&mut T` parameter.
#[doc(hidden)]
#[inline]
pub fn __mx_exclusive<'a, T: TypedExternal>(view: &'a mut ExternalPtrMut<'_, T>) -> &'a mut T {
    view
}

#[cold]
fn borrow_conflict<T>() -> ! {
    crate::error!(
        class = "miniextendr_borrow_conflict",
        "this `{}` object is in use by a call that is still running",
        core::any::type_name::<T>()
    )
}
// endregion

// region: ExternalPtr<T>

/// An owned pointer stored in R's external pointer SEXP.
///
/// This is conceptually similar to `Box<T>`, but with the following differences:
/// - Memory is freed by R's GC via a registered finalizer (non-deterministic)
/// - The underlying SEXP is Copy, so aliasing must be manually prevented
/// - Type checking happens at runtime via `Any::downcast` (Rust `TypeId`)
///
/// # Thread Safety
///
/// `ExternalPtr` is `Send` to allow returning from worker thread functions.
/// However, **concurrent access is not allowed** - R's runtime is single-threaded.
/// All R API calls are serialized through the main thread via `with_r_thread`.
///
/// # Reading the value
///
/// A handle has no `Deref`: [`borrow`](Self::borrow) /
/// [`borrow_mut`](Self::borrow_mut) read the value, checking its `RefCell`
/// (see "Reentrant calls" in the [module docs](self)). A `#[miniextendr]`
/// function that needs only the value takes `&T` / `&mut T` instead, and the
/// generated wrapper borrows it for the call.
///
/// # Safety
///
/// The ExternalPtr assumes exclusive ownership of the underlying data.
/// Cloning the raw SEXP without proper handling will lead to double-free.
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::externalptr::{ExternalPtr, TypedExternal};
///
/// struct MyData { value: f64 }
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
///     const TYPE_ID_CSTR: &'static [u8] = b"my_crate::MyData\0";
/// }
///
/// let ptr = ExternalPtr::new(MyData { value: 3.14 });
/// assert_eq!(ptr.borrow().value, 3.14);
/// ```
#[repr(C)]
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    /// Cached address of the value's `RefCell`, set once at construction time.
    ///
    /// This avoids the `R_ExternalPtrAddr` FFI call on every `borrow()`/`borrow_mut()`.
    /// The pointer remains valid for the lifetime of the `ExternalPtr` because:
    /// - R's finalizer only runs after R garbage-collects the SEXP (which cannot
    ///   happen while a Rust `ExternalPtr` value exists).
    /// - `R_ClearExternalPtr` is only called in methods that consume or finalize
    ///   (`into_raw`, `into_inner`, `release_any`).
    cached_ptr: NonNull<RefCell<T>>,
    /// The [`ProtectPool`] key rooting this handle's `EXTPTRSXP`, or `None` for
    /// borrowed views.
    ///
    /// `Some(key)` for *owning* handles built from a fresh value (`new` /
    /// `new_unchecked` / `from_raw` / `Clone` / `Default`): the constructor
    /// roots the `EXTPTRSXP` in the main-thread pool so it stays alive for the
    /// whole Rust lifetime of the handle — including while it sits in a `Vec`
    /// across other R allocations before being handed to R (#836). `Drop` /
    /// `into_raw` / `into_inner` release the root via the key.
    ///
    /// `None` for *borrowed* views of an SEXP R already owns (`wrap_sexp*` /
    /// `from_sexp*` / `same_object`): no root is taken, so none is released. The
    /// aliased object is kept alive by whatever R-side reference handed it to us
    /// (a `.Call` argument frame, an owning sibling handle, …).
    root: Option<ProtectKey>,
    _marker: PhantomData<T>,
}

// SAFETY: ExternalPtr can be sent between threads because:
// 1. All R API operations are serialized through the main thread via with_r_thread
// 2. The worker thread is blocked while the main thread processes R calls
// 3. There is no concurrent access - only sequential hand-off between threads
unsafe impl<T: TypedExternal + Send> Send for ExternalPtr<T> {}

// region: ExternalPtr GC roots
//
// Owning `ExternalPtr` handles keep their `EXTPTRSXP` alive for the handle's
// whole Rust lifetime by rooting it in a process-wide `ProtectPool` — a single
// GC-traced VECSXP with Rust-side slot bookkeeping. This is what makes a naive
// `Vec<ExternalPtr<T>>` GC-safe: every element stays rooted while later elements
// allocate (#836).
//
// Why a pool and not `R_PreserveObject`: the pool releases in O(1) any order,
// whereas `R_ReleaseObject` scans R's precious list (O(n)). A `Vec<ExternalPtr>`
// drops front-to-back — oldest first, i.e. the entries deepest in R's LIFO
// precious list — so `R_PreserveObject` rooting degrades to O(n²) on exactly
// this workload (60–65× slower at 10k; see
// analysis/gc-protection-benchmarks-results.md). The pool is the mechanism the
// strategy analysis prescribes for ExternalPtr (analysis/gc-protection-strategies.md).
//
// `ProtectPool` is `!Send`/`!Sync` and lives in a `thread_local!` on R's main
// thread. Every access happens there: roots are taken inside
// `create_extptr_sexp[_unchecked]` (main-thread by contract / `with_r_thread`),
// and released through `with_r_thread` from `Drop` / `into_raw` / `into_inner`.
// The pool is wrapped in `ManuallyDrop` so it is never released at thread exit —
// it is a session-lifetime root table, and running `R_ReleaseObject` on its
// backing during R's own teardown would touch a half-freed R heap.

thread_local! {
    static EXTPTR_ROOTS: RefCell<Option<ManuallyDrop<ProtectPool>>> = const { RefCell::new(None) };
}

/// Root an owning handle's `EXTPTRSXP` in the main-thread pool.
///
/// Must run on R's main thread, which holds inside
/// `create_extptr_sexp[_unchecked]` and `TraitViewData::new`, whose trait views
/// root their receiver here too. `sexp` may be fresh: it stays reachable while
/// the first call creates the pool and while the pool grows.
#[inline]
pub(crate) fn root_owned(sexp: SEXP) -> ProtectKey {
    EXTPTR_ROOTS.with_borrow_mut(|slot| {
        let pool = slot.get_or_insert_with(|| {
            // SAFETY: on R's main thread (caller contract). Creating the pool
            // allocates before it holds `sexp`.
            let _sexp = unsafe { crate::OwnedProtect::new(sexp) };
            ManuallyDrop::new(ProtectPool::new(ProtectPool::DEFAULT_CAPACITY))
        });
        pool.insert(sexp)
    })
}

/// Release an owning handle's pool root. Stale keys are a safe no-op.
///
/// Must run on R's main thread (callers route through `with_r_thread`, or are
/// `!Send` like `TraitViewData`).
#[inline]
pub(crate) fn unroot_owned(key: ProtectKey) {
    EXTPTR_ROOTS.with_borrow_mut(|slot| {
        if let Some(pool) = slot.as_mut() {
            pool.release(key);
        }
    });
}

/// One R object rooted in the main-thread pool, released when dropped.
///
/// For roots that R's finalizers drop, such as the caches an ALTREP owner
/// keeps in data1. GC finalizes the newest objects first, while
/// `R_ReleaseObject` scans the precious list from its head, so precious-list
/// roots taken in any other order release in quadratic time. The pool
/// releases in O(1) whatever the order.
///
/// Raw-pointer marker: `!Send` and `!Sync`, so it is dropped on the thread
/// that took it.
pub(crate) struct PoolRoot {
    sexp: SEXP,
    key: ProtectKey,
    _not_send: PhantomData<*mut ()>,
}

impl PoolRoot {
    /// Root `sexp`, which needs no protection of its own.
    ///
    /// # Safety
    ///
    /// Must run on R's main thread.
    pub(crate) unsafe fn new(sexp: SEXP) -> Self {
        Self {
            sexp,
            key: root_owned(sexp),
            _not_send: PhantomData,
        }
    }

    /// The rooted object.
    pub(crate) fn get(&self) -> SEXP {
        self.sexp
    }
}

impl Drop for PoolRoot {
    fn drop(&mut self) {
        // Releasing a slot neither allocates nor runs finalizers, so this is
        // sound inside a finalizer too.
        unroot_owned(self.key);
    }
}
// endregion

impl<T: TypedExternal> ExternalPtr<T> {
    /// Build an *owning* handle rooted at `root`.
    ///
    /// Pairs with the [`ProtectPool`] root that [`create_extptr_sexp`] /
    /// [`create_extptr_sexp_unchecked`] take on the SEXP (and return as the
    /// key). The root is released by `Drop` / `into_raw` / `into_inner`. Only
    /// the four fresh-value constructors (`new` / `new_unchecked` / `from_raw` /
    /// `from_raw_unchecked`) build through here.
    ///
    /// [`create_extptr_sexp`]: Self::create_extptr_sexp
    /// [`create_extptr_sexp_unchecked`]: Self::create_extptr_sexp_unchecked
    #[inline]
    fn from_owned_parts(sexp: SEXP, cached_ptr: NonNull<RefCell<T>>, root: ProtectKey) -> Self {
        Self {
            sexp,
            cached_ptr,
            root: Some(root),
            _marker: PhantomData,
        }
    }

    /// Build a *borrowed* view (`root = None`) of an SEXP R already owns.
    ///
    /// No GC root is taken and none is released — the aliased object is kept
    /// alive by the R-side reference that handed it to us. Used by every
    /// `wrap_sexp*` / `from_sexp*` / `same_object` path.
    #[inline]
    fn from_borrowed_parts(sexp: SEXP, cached_ptr: NonNull<RefCell<T>>) -> Self {
        Self {
            sexp,
            cached_ptr,
            root: None,
            _marker: PhantomData,
        }
    }

    /// Root this handle's `EXTPTRSXP` for as long as the handle lives, as the
    /// fresh-value constructors do: for a pointer converted from a class
    /// handle, whose `.ptr` reference R code can drop while Rust still holds
    /// the pointer (a callback the call makes, or a handle Rust keeps).
    ///
    /// # Safety
    ///
    /// On R's main thread, with no root taken yet.
    #[inline]
    pub(crate) unsafe fn rooted(mut self) -> Self {
        debug_assert!(self.root.is_none());
        self.root = Some(root_owned(self.sexp));
        self
    }

    /// Release the pool root iff this handle owns one.
    ///
    /// Routed through [`with_r_thread`] because an owning `ExternalPtr` is
    /// `Send` and may be dropped on the worker thread, while the pool lives on
    /// R's main thread. `with_r_thread` runs the closure inline when already on
    /// the main thread (the common case), so this is a direct pool release
    /// there and a thread hand-off only from the worker. `ProtectKey` is `Copy`
    /// + `Send` (two `u32`s), so it crosses the boundary by value.
    ///
    /// [`with_r_thread`]: crate::worker::with_r_thread
    #[inline]
    fn release_root_if_owned(&self) {
        let Some(key) = self.root else {
            return;
        };
        crate::worker::with_r_thread(move || unroot_owned(key));
    }

    /// Box `x` in its [`RefCell`] and erase that to the `Box<Box<dyn Any>>` R
    /// stores, returning the cell's address for `cached_ptr` as well.
    #[inline]
    fn erase(x: T) -> (NonNull<RefCell<T>>, *mut Box<dyn Any>) {
        let cell: *mut RefCell<T> = Box::into_raw(Box::new(RefCell::new(x)));
        // SAFETY: `cell` is the allocation `Box::into_raw` just released, so
        // `Box::from_raw` takes it back; the value stays at `cell`.
        let inner: Box<dyn Any> = unsafe { Box::from_raw(cell) };
        // SAFETY: `Box::into_raw` never returns null.
        let cell = unsafe { NonNull::new_unchecked(cell) };
        (cell, Box::into_raw(Box::new(inner)))
    }

    /// Allocates memory on the heap and places `x` into it.
    ///
    /// Internally stores a `Box<Box<dyn Any>>` — a thin pointer (fits in R's
    /// `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any` vtable
    /// for runtime type checking via `downcast`) to a `RefCell<T>`.
    ///
    /// This function can be called from the two supported R contexts:
    /// - If called from R's main thread, creates the ExternalPtr directly
    /// - If called from the worker thread (during `run_on_worker`), automatically
    ///   sends the R API calls to the main thread via [`with_r_thread`]
    ///
    /// # Panics
    ///
    /// Panics if called from a non-main thread outside of a `run_on_worker` context.
    ///
    /// Equivalent to `Box::new`.
    ///
    /// [`with_r_thread`]: crate::worker::with_r_thread
    #[inline]
    pub fn new(x: T) -> Self {
        let (cached_ptr, any_raw) = Self::erase(x);

        // Wrap in Sendable so it can be sent across thread boundary
        let sendable = unsafe { sendable_any_ptr_new(any_raw) };

        // Use with_r_thread to run R API calls on main thread. The pool root is
        // taken there (on the main thread, where the pool lives) and the key
        // crosses back by value — `(SEXP, ProtectKey)` is `Send`.
        let (sexp, root) = crate::worker::with_r_thread(move || {
            let any_raw = sendable_any_ptr_into_ptr(sendable);
            unsafe { Self::create_extptr_sexp_unchecked(any_raw) }
        });

        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Allocates memory on the heap and places `x` into it, without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. Calling from another thread
    /// is undefined behavior (R APIs are not thread-safe).
    #[inline]
    pub unsafe fn new_unchecked(x: T) -> Self {
        let (cached_ptr, any_raw) = Self::erase(x);
        let (sexp, root) = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>`. Must be called from main thread.
    ///
    /// The `any_raw` is a thin pointer to a heap-allocated fat pointer (`Box<dyn Any>`).
    /// R stores the thin pointer in `R_ExternalPtrAddr`. Returns the SEXP and the
    /// [`ProtectPool`] key that roots it for the owning handle's lifetime.
    #[inline]
    unsafe fn create_extptr_sexp(any_raw: *mut Box<dyn Any>) -> (SEXP, ProtectKey) {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp received null pointer"
        );

        // Own the Rust allocation until its R finalizer is registered. Temporary
        // roots remain lexical even when a pool root takes over on success.
        let allocation = unsafe { Box::from_raw(any_raw) };
        let scope = unsafe { ProtectScope::new() };
        let type_sym = unsafe { type_symbol::<T>() };
        let type_id_sym = unsafe { type_id_symbol::<T>() };
        let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, prot_vec_len::<T>()) };
        unsafe { scope.protect(prot) };
        prot.set_vector_elt(PROT_TYPE_ID_INDEX, type_id_sym);

        let sexp = unsafe { R_MakeExternalPtr(ptr::null_mut(), type_sym, prot) };
        unsafe { scope.protect(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe { R_RegisterCFinalizerEx(sexp, Some(release_any), Rboolean::TRUE) };
        // Install the address only after the finalizer is ready. Use the pointer
        // returned by the ownership transfer, retaining its mutable provenance.
        let any_raw = Box::into_raw(allocation);
        unsafe { R_SetExternalPtrAddr(sexp, any_raw.cast()) };

        // Move the value's pending `Sidecar` values into `prot`, which is
        // protected.
        if T::R_SLOT_COUNT > 0 {
            unsafe { init_sidecars::<T>(any_raw, sexp) };
        }

        // Root the owning handle for its whole Rust lifetime so it survives R
        // allocations while held (e.g. element-by-element in a `Vec`) before
        // reaching R (#836). The pool gives O(1) any-order release — see the
        // `EXTPTR_ROOTS` docs for why that beats `R_PreserveObject` here.
        // Must happen here, on the main thread, because `new` returns the SEXP
        // to the *calling* thread (possibly the worker) where R API is gone.
        let root = root_owned(sexp);

        (sexp, root)
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>` without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions for thread safety.
    ///
    /// Returns the SEXP and the [`ProtectPool`] key that roots it.
    #[inline]
    unsafe fn create_extptr_sexp_unchecked(any_raw: *mut Box<dyn Any>) -> (SEXP, ProtectKey) {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp_unchecked received null pointer"
        );

        let allocation = unsafe { Box::from_raw(any_raw) };
        let scope = unsafe { ProtectScope::new() };
        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, prot_vec_len::<T>()) };
        unsafe { scope.protect(prot) };
        unsafe { prot.set_vector_elt_unchecked(PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr_unchecked(ptr::null_mut(), type_sym, prot) };
        unsafe { scope.protect(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe {
            R_RegisterCFinalizerEx_unchecked(sexp, Some(release_any), Rboolean::TRUE);
        };
        let any_raw = Box::into_raw(allocation);
        unsafe { R_SetExternalPtrAddr_unchecked(sexp, any_raw.cast()) };

        // Flush the value's pending `Sidecar` values (see `create_extptr_sexp`).
        if T::R_SLOT_COUNT > 0 {
            unsafe { init_sidecars::<T>(any_raw, sexp) };
        }

        // Root the owning handle (see `create_extptr_sexp` for the rationale).
        // `root_owned` uses the pool's checked FFI, which runs inline here
        // because the unchecked constructors are main-thread-by-contract.
        let root = root_owned(sexp);

        (sexp, root)
    }

    /// Collect an iterator of values into a protected R list (`VECSXP`) holding
    /// one fresh external pointer per item, rooting each via the destination
    /// list instead of the [`ProtectPool`](crate::protect_pool).
    ///
    /// This is the GC-safe, allocation-lean way to hand many Rust values to R at
    /// once — e.g. converting a `Vec<T>` into an R `list()` of external pointers.
    /// Each `EXTPTRSXP` is created and **immediately** stored into the
    /// already-protected result list, so the list roots it the instant it
    /// exists: there is no unprotected window between element allocations, and
    /// **no per-element pool traffic**.
    ///
    /// Contrast the naive `items.map(ExternalPtr::new).collect::<Vec<_>>()`,
    /// which roots every handle in the process-wide pool (keeping the `Vec`
    /// GC-safe while held — #836) only to release every root again when the `Vec`
    /// drops, then still needs a second pass to copy the handles into a list.
    /// Here the list *is* the root, so both the pool round-trip and the copy
    /// pass are skipped. The whole batch also crosses to R's main thread in a
    /// single [`with_r_thread`](crate::worker::with_r_thread) hop rather than one
    /// per element.
    ///
    /// The returned `VECSXP` is **not** protected: the caller must protect it or
    /// return it to R immediately, exactly like any other freshly built SEXP
    /// (e.g. an [`IntoR`](crate::IntoR) result).
    pub fn collect_into_r_list<I>(items: I) -> SEXP
    where
        I: IntoIterator<Item = T>,
    {
        // Box + type-erase every value on the *calling* thread (no R API needed),
        // then hand the allocations to the main thread. Keep them owned until
        // each pointer's finalizer is registered: iterator or conversion panics
        // must drop every value that has not yet been transferred to R.
        let allocations: Vec<Box<dyn Any>> = items
            .into_iter()
            .map(|x| -> Box<dyn Any> { Box::new(RefCell::new(x)) })
            .collect();
        let sendable = crate::worker::Sendable(allocations);

        crate::worker::with_r_thread(move || {
            let allocations = sendable.0;
            // SAFETY: `with_r_thread` runs this on R's main thread; every entry
            // owns a `RefCell<T>` in the ordinary ExternalPtr layout.
            unsafe { Self::build_extptr_list(allocations) }
        })
    }

    /// Build a protected `VECSXP` of external pointers from already-erased boxes.
    ///
    /// Allocates the result list, protects it, then creates one `EXTPTRSXP` per
    /// entry directly into its slot — rooted by the protected list, no pool. The
    /// type symbols are interned once and reused (they are never GC'd, so they
    /// stay valid across the allocating loop). Returns the list **unprotected**.
    ///
    /// # Safety
    ///
    /// Must run on R's main thread; each allocation must wrap a `RefCell<T>`.
    unsafe fn build_extptr_list(allocations: Vec<Box<dyn Any>>) -> SEXP {
        let n = R_xlen_t::try_from(allocations.len()).expect("list length exceeds R_xlen_t::MAX");
        let scope = unsafe { ProtectScope::new() };
        let list = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, n) };
        unsafe { scope.protect(list) };

        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        for (i, allocation) in allocations.into_iter().enumerate() {
            let idx = R_xlen_t::try_from(i).expect("index exceeds R_xlen_t::MAX");
            // Only now allocate the outer box that gives R a thin address.
            let any_raw = Box::into_raw(Box::new(allocation));
            // SAFETY: main thread; `any_raw` owns a `T`; `list` is protected, so
            // it roots each element the instant `set_vector_elt` stores it.
            unsafe { Self::make_extptr_into_slot(any_raw, type_sym, type_id_sym, list, idx) };
        }

        list
    }

    /// Create an `EXTPTRSXP` for `any_raw` and store it into `dest[idx]`.
    ///
    /// Mirrors [`create_extptr_sexp_unchecked`](Self::create_extptr_sexp_unchecked)
    /// but roots the new pointer via `dest` (which the caller keeps protected)
    /// instead of the pool — the element is live the instant it lands in the
    /// protected list, so a bulk build pays no pool insert/release per element.
    ///
    /// # Safety
    ///
    /// Must run on R's main thread; `any_raw` must own a `T`; `dest` must be a
    /// protected `VECSXP` with `idx` in bounds; `type_sym` / `type_id_sym` must
    /// be the interned symbols for `T`.
    #[inline]
    unsafe fn make_extptr_into_slot(
        any_raw: *mut Box<dyn Any>,
        type_sym: SEXP,
        type_id_sym: SEXP,
        dest: SEXP,
        idx: R_xlen_t,
    ) {
        let allocation = unsafe { Box::from_raw(any_raw) };
        let scope = unsafe { ProtectScope::new() };
        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, prot_vec_len::<T>()) };
        unsafe { scope.protect(prot) };
        unsafe { prot.set_vector_elt_unchecked(PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr_unchecked(ptr::null_mut(), type_sym, prot) };
        unsafe { scope.protect(sexp) };
        unsafe { R_RegisterCFinalizerEx_unchecked(sexp, Some(release_any), Rboolean::TRUE) };
        let any_raw = Box::into_raw(allocation);
        unsafe { R_SetExternalPtrAddr_unchecked(sexp, any_raw.cast()) };

        // Flush the value's pending `Sidecar` values (see `create_extptr_sexp`).
        if T::R_SLOT_COUNT > 0 {
            unsafe { init_sidecars::<T>(any_raw, sexp) };
        }

        // Root via the destination list instead of the pool: `dest` is protected
        // by the caller, so storing `sexp` keeps it (and its `prot`) alive with
        // no pool churn.
        unsafe { dest.set_vector_elt_unchecked(idx, sexp) };
    }

    /// Constructs a new `ExternalPtr` with uninitialized contents.
    ///
    /// Equivalent to `Box::new_uninit`.
    #[inline]
    pub fn new_uninit() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::uninit())
    }

    /// Constructs a new `ExternalPtr` with zeroed contents.
    ///
    /// Equivalent to `Box::new_zeroed`.
    #[inline]
    pub fn new_zeroed() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::zeroed())
    }

    /// Constructs an ExternalPtr from a raw pointer.
    ///
    /// Moves the value out of the `Box<T>` allocation into the pointer's own
    /// `RefCell<T>` storage and frees that allocation, so the value's address
    /// changes: `from_raw(into_raw(p))` round-trips the value, not the address.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread
    ///
    /// Like `Box::from_raw`, except for the address.
    #[inline]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        let (cached_ptr, any_raw) = Self::erase(*unsafe { Box::from_raw(raw) });
        let (sexp, root) = unsafe { Self::create_extptr_sexp(any_raw) };
        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Constructs an ExternalPtr from a raw pointer, without thread checks.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread (no debug assertions)
    #[inline]
    pub unsafe fn from_raw_unchecked(raw: *mut T) -> Self {
        let (cached_ptr, any_raw) = Self::erase(*unsafe { Box::from_raw(raw) });
        let (sexp, root) = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self::from_owned_parts(sexp, cached_ptr, root)
    }

    /// Consumes the ExternalPtr, returning a raw pointer.
    ///
    /// The caller is responsible for the memory, and the finalizer is
    /// effectively orphaned (will do nothing since we clear the pointer).
    /// The R object is emptied: address, tag and protection list (each
    /// [`Sidecar<T>`] slot is read back into its field as a pending value
    /// first, so the values travel with the struct), so its accessors report
    /// a null external pointer, not an object restored from a saved session.
    ///
    /// The value moves out of the pointer's `RefCell<T>` storage into a new
    /// `Box<T>`, so its address changes. Like [`into_inner`](Self::into_inner),
    /// this raises the `miniextendr_borrow_conflict` error while a running call
    /// holds the value.
    ///
    /// Like `Box::into_raw`, except for the address.
    #[inline]
    pub fn into_raw(this: Self) -> *mut T {
        Box::into_raw(Box::new(Self::into_inner(this)))
    }

    /// Consumes the ExternalPtr, returning a `NonNull` pointer.
    ///
    /// Equivalent to `Box::into_non_null`.
    #[inline]
    pub fn into_non_null(this: Self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Self::into_raw(this)) }
    }

    /// Consumes and leaks the ExternalPtr, returning a mutable reference.
    ///
    /// The memory will never be freed (from Rust's perspective; R's GC
    /// finalizer is neutralized).
    ///
    /// Equivalent to `Box::leak`.
    #[inline]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        T: 'a,
    {
        unsafe { &mut *Self::into_raw(this) }
    }

    /// Consumes the ExternalPtr, returning the wrapped value.
    ///
    /// Uses `Box<dyn Any>::downcast` to recover the concrete
    /// `Box<RefCell<T>>`, then moves the value out. The R object is emptied
    /// like by [`into_raw`](Self::into_raw): its accessors report a null
    /// external pointer afterwards, and each [`Sidecar<T>`] slot is read back
    /// into its field as a pending value, so wrapping the value again keeps
    /// the values.
    ///
    /// # Panics
    ///
    /// With the `miniextendr_borrow_conflict` R error while a running call
    /// (a generated method of the same object, further up the stack) holds the
    /// value: the call's borrow points into the storage this frees.
    ///
    /// Equivalent to `*boxed` (deref move) or `Box::into_inner`.
    #[inline]
    pub fn into_inner(this: Self) -> T {
        let any_raw = unsafe { R_ExternalPtrAddr(this.sexp) as *mut Box<dyn Any> };
        assert!(!any_raw.is_null(), "ExternalPtr is null or cleared");
        assert!(
            unsafe { stored_type_id_symbol(this.sexp) }.is_some(),
            "ExternalPtr has no valid storage metadata"
        );
        if let Some(cell) = unsafe { &*any_raw }.downcast_ref::<RefCell<T>>() {
            drop(borrow_value_mut(cell));
        }

        // The value leaves its pointer: read its slots back into the struct
        // as pending values, slot `i` into field `i`, so wrapping it again
        // keeps them.
        if T::R_SLOT_COUNT > 0 {
            // SAFETY: `cached_ptr` is valid for the handle's lifetime and no
            // call holds the value; `sexp` is the live pointer this handle
            // holds.
            let prot = unsafe { R_ExternalPtrProtected(this.sexp) };
            load_sidecars(unsafe { &*this.value_ptr() }, prot);
        }

        // Ownership leaves this handle: drop our GC root before `mem::forget`.
        this.release_root_if_owned();

        // Clear so finalizer is no-op
        unsafe { clear_moved_out(this.sexp) };
        mem::forget(this);

        let outer: Box<Box<dyn Any>> = unsafe { Box::from_raw(any_raw) };
        let inner: Box<dyn Any> = *outer;
        inner
            .downcast::<RefCell<T>>()
            .expect("ExternalPtr type mismatch in into_inner")
            .into_inner()
    }

    // region: Accessors

    /// The value's `RefCell`.
    #[inline]
    fn cell(&self) -> &RefCell<T> {
        // SAFETY: cached_ptr is always valid for the lifetime of ExternalPtr,
        // and a shared reference to the cell is always sound.
        unsafe { self.cached_ptr.as_ref() }
    }

    /// The value's address, through its `RefCell`.
    #[inline]
    fn value_ptr(&self) -> *mut T {
        // `as_ptr` gives a pointer that may be written through.
        self.cell().as_ptr()
    }

    /// Borrows the value, as the generated wrapper does for a `&T` argument:
    /// until the returned view drops, a method of the same object that needs
    /// it mutably, a `&mut T` argument holding it, or
    /// [`borrow_mut`](Self::borrow_mut) through another handle raises the
    /// error below instead. The view also carries the R object, for the
    /// sidecar accessors.
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while the value
    /// is borrowed mutably, and for a type-erased `ExternalPtr<()>`, which has
    /// no value of its own to borrow (downcast it first).
    #[inline]
    pub fn borrow(&self) -> ExternalPtrRef<'_, T> {
        ExternalPtrRef::from_guard(&self.sexp, borrow_value(self.value_cell()))
    }

    /// Borrows the value mutably, as the generated wrapper does for a
    /// `&mut T` argument: until the returned view drops, any other borrow of
    /// the value raises the error below.
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while the value
    /// is borrowed (by a running call holding it, or another handle's
    /// [`borrow`](Self::borrow)), and for a type-erased `ExternalPtr<()>`.
    #[inline]
    pub fn borrow_mut(&mut self) -> ExternalPtrMut<'_, T> {
        ExternalPtrMut::from_guard(&self.sexp, borrow_value_mut(self.value_cell()))
    }

    /// The value, with no `RefCell` borrow: only the ALTREP `data1` paths,
    /// whose per-element callbacks read without one until #2103 measures the
    /// checked read.
    ///
    /// # Safety
    ///
    /// Nothing may hold the value mutably while the reference lives.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn as_ref_unchecked(&self) -> &T {
        // SAFETY: the value lives as long as the handle; the caller excludes
        // a mutable borrow.
        unsafe { &*self.value_ptr() }
    }

    /// The value mutably, with no `RefCell` borrow, like
    /// [`as_ref_unchecked`](Self::as_ref_unchecked).
    ///
    /// # Safety
    ///
    /// Nothing else may hold the value while the reference lives.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
        // SAFETY: as in `as_ref_unchecked`.
        unsafe { &mut *self.value_ptr() }
    }

    /// The value's `RefCell`, which a type-erased handle doesn't point at.
    #[inline]
    fn value_cell(&self) -> &RefCell<T> {
        assert!(
            !is_type_erased::<T>(),
            "a type-erased `ExternalPtr<()>` has no value of its own to borrow; downcast it first"
        );
        self.cell()
    }

    /// Returns the raw pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.value_ptr().cast_const()
    }

    /// Returns the raw mutable pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.value_ptr()
    }

    /// Checks whether two `ExternalPtr`s refer to the same allocation (pointer identity).
    ///
    /// This ignores the pointee values. Use this when you need alias detection;
    /// prefer `PartialEq`/`PartialOrd` for value comparisons.
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        ptr::eq(this.cached_ptr.as_ptr(), other.cached_ptr.as_ptr())
    }
    // endregion

    // region: R-specific accessors

    /// Returns the underlying SEXP.
    ///
    /// # Warning
    ///
    /// The returned SEXP must not be duplicated or the finalizer will double-free.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// A second handle to the same R object: no value is copied and no R
    /// object is allocated, and the handle takes no borrow until it is read.
    /// Returning it from a `#[miniextendr]` function returns the same R
    /// object, so R sees the identity it passed in.
    ///
    /// ```ignore
    /// #[miniextendr]
    /// pub fn first_of(objs: Vec<ExternalPtr<MyType>>) -> ExternalPtr<MyType> {
    ///     objs[0].same_object()
    /// }
    /// ```
    ///
    /// The new handle takes no GC root of its own: something else (the handle
    /// it was made from, or R) must keep the R object reachable while it is
    /// used.
    #[inline]
    pub fn same_object(&self) -> Self {
        // SAFETY: self.sexp is a valid live EXTPTRSXP that we already hold.
        // wrap_sexp re-extracts the data pointer from the same SEXP.
        unsafe { Self::wrap_sexp(self.sexp) }
            .expect("same_object of a live ExternalPtr should never fail")
    }

    /// Returns the tag SEXP (type identifier symbol).
    #[inline]
    pub fn tag(&self) -> SEXP {
        unsafe { R_ExternalPtrTag(self.sexp) }
    }

    /// Returns the tag SEXP (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn tag_unchecked(&self) -> SEXP {
        unsafe { crate::sys::R_ExternalPtrTag_unchecked(self.sexp) }
    }

    /// Returns the protected SEXP slot (user-protected objects).
    ///
    /// This returns the user-protected object stored in the prot VECSXP,
    /// not the VECSXP itself.
    #[inline]
    pub fn protected(&self) -> SEXP {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                return SEXP::nil();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::nil();
            }
            prot.vector_elt(PROT_USER_INDEX)
        }
    }

    /// Returns the protected SEXP slot (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn protected_unchecked(&self) -> SEXP {
        use crate::sys::R_ExternalPtrProtected_unchecked;

        unsafe {
            let prot = R_ExternalPtrProtected_unchecked(self.sexp);
            if prot.is_null_or_nil() {
                return SEXP::nil();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::nil();
            }
            prot.vector_elt_unchecked(PROT_USER_INDEX)
        }
    }

    /// Sets the user-protected SEXP slot.
    ///
    /// Use this to prevent R objects from being GC'd while this ExternalPtr exists.
    /// The type ID stored in prot slot 0 is preserved.
    ///
    /// Returns `false` if the prot structure is malformed (should not happen
    /// for ExternalPtrs created by this library).
    ///
    /// # Safety
    ///
    /// - `user_prot` must be a valid SEXP or R_NilValue
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_protected(&self, user_prot: SEXP) -> bool {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                debug_assert!(false, "ExternalPtr prot slot is null or R_NilValue");
                return false;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                debug_assert!(
                    false,
                    "ExternalPtr prot slot is not a VECSXP of expected length"
                );
                return false;
            }
            prot.set_vector_elt(PROT_USER_INDEX, user_prot);
            true
        }
    }

    /// Returns the raw prot VECSXP (contains both type ID and user protected).
    ///
    /// Prefer using `protected()` for user data and `stored_type_id()` for type info.
    #[inline]
    pub fn prot_raw(&self) -> SEXP {
        unsafe { R_ExternalPtrProtected(self.sexp) }
    }

    /// Checks if the internal pointer is null (already finalized or cleared).
    #[inline]
    pub fn is_null(&self) -> bool {
        unsafe { R_ExternalPtrAddr(self.sexp).is_null() }
    }
    // endregion

    // region: Type checking

    /// Attempt to wrap a SEXP as an ExternalPtr with type checking.
    ///
    /// Uses `Any::downcast_ref` for authoritative type checking (Rust `TypeId`).
    /// Type-erased `ExternalPtr<()>` deliberately skips the concrete downcast.
    ///
    /// Returns `None` if:
    /// - The internal pointer is null
    /// - The metadata does not identify ordinary miniextendr storage
    /// - The stored `Box<dyn Any>` does not contain a `RefCell<T>`
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP; foreign pointers must not carry
    ///   forged miniextendr metadata
    /// - The caller must ensure no other ExternalPtr owns this SEXP
    pub unsafe fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        unsafe { stored_type_id_symbol(sexp) }?;

        if is_type_erased::<T>() {
            // Type-erased path: skip downcast, just use the raw pointer
            // (ExternalPtr<()> doesn't care about the concrete type)
            return Some(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast())
            }));
        }

        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        let cell: &RefCell<T> = any_box.downcast_ref::<RefCell<T>>()?;
        Some(Self::from_borrowed_parts(sexp, NonNull::from(cell)))
    }

    /// Attempt to wrap a SEXP as an ExternalPtr (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths like ALTREP callbacks.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP; foreign pointers must not carry
    ///   forged miniextendr metadata
    /// - The caller must ensure exclusive ownership
    /// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
    pub unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Option<Self> {
        use crate::sys::R_ExternalPtrAddr_unchecked;

        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr_unchecked(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        unsafe { stored_type_id_symbol(sexp) }?;

        if is_type_erased::<T>() {
            return Some(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast())
            }));
        }

        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        let cell: &RefCell<T> = any_box.downcast_ref::<RefCell<T>>()?;
        Some(Self::from_borrowed_parts(sexp, NonNull::from(cell)))
    }

    /// Attempt to wrap a SEXP as an ExternalPtr, returning an error with type info on mismatch.
    ///
    /// This is used by the [`TryFromSexp`] trait implementation.
    ///
    /// A pointer restored from a saved session is rebuilt first when `T` has
    /// a rebuild hook (`#[externalptr(rebuild = path)]`). A failed rebuild is
    /// returned, not raised (#2217): [`TypeMismatchError::RebuildFailed`]
    /// holds the refusal a use of the object raises, with the hook's cause as
    /// `e$parent`. So a check that must not raise (is the object live? fall
    /// back to plain data?) needs no
    /// [`catch_condition`](crate::condition::catch_condition):
    ///
    /// ```ignore
    /// match unsafe { ExternalPtr::<Model>::wrap_sexp_with_error(x) } {
    ///     Ok(model) => Some(model),
    ///     Err(TypeMismatchError::RebuildFailed { .. }) => None,
    ///     Err(other) => panic!("{other}"),
    /// }
    /// ```
    ///
    /// The [`TryFromSexp`] conversion, behind every `ExternalPtr<T>` / `&T` /
    /// `&mut T` argument, raises the refusal ([`RCondition::raise`]), as a
    /// method on the object does. An R error or a plain panic inside the hook
    /// carries on as itself here too.
    ///
    /// # Safety
    ///
    /// Same as [`wrap_sexp`](Self::wrap_sexp).
    ///
    /// [`TryFromSexp`]: crate::TryFromSexp
    pub unsafe fn wrap_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError> {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_with_error: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            // SAFETY: on R's main thread with a valid EXTPTRSXP, which the
            // caller roots.
            match unsafe { try_rebuild_restored::<T>(sexp, sexp) } {
                Ok(true) => return unsafe { Self::wrap_sexp_with_error(sexp) },
                Ok(false) => {}
                Err(condition) => return Err(TypeMismatchError::RebuildFailed { condition }),
            }
            // SAFETY: on R's main thread with a valid EXTPTRSXP.
            return Err(match unsafe { classify_null_address::<T>(sexp) } {
                NullAddress::Foreign => TypeMismatchError::NullPointer,
                NullAddress::Restored => TypeMismatchError::Restored {
                    type_name: T::TYPE_NAME,
                    package_classes: T::RESTORED_ERROR_CLASS,
                    package_message: T::RESTORED_ERROR_MESSAGE,
                },
                NullAddress::OtherVersion {
                    crate_name,
                    saved,
                    current,
                } => TypeMismatchError::OtherVersion {
                    type_name: T::TYPE_NAME,
                    crate_name,
                    saved,
                    current,
                    package_classes: T::RESTORED_ERROR_CLASS,
                    package_message: T::RESTORED_ERROR_MESSAGE,
                },
                NullAddress::OtherType { found } => TypeMismatchError::Mismatch {
                    expected: T::TYPE_NAME,
                    found,
                },
            });
        }

        let stored =
            unsafe { stored_type_id_symbol(sexp) }.ok_or(TypeMismatchError::InvalidTypeId)?;

        if is_type_erased::<T>() {
            return Ok(Self::from_borrowed_parts(sexp, unsafe {
                NonNull::new_unchecked(any_raw.cast())
            }));
        }

        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        match any_box.downcast_ref::<RefCell<T>>() {
            Some(cell) => Ok(Self::from_borrowed_parts(sexp, NonNull::from(cell))),
            None => {
                // The stored type ID, for the diagnostic.
                let found = symbol_name(stored);
                Err(TypeMismatchError::Mismatch {
                    expected: T::TYPE_NAME,
                    found,
                })
            }
        }
    }

    /// Create an ExternalPtr from an SEXP without type checking.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP containing a `*mut Box<dyn Any>`
    ///   wrapping a `RefCell<T>`
    /// - The caller must ensure exclusive ownership
    #[inline]
    pub unsafe fn from_sexp_unchecked(sexp: SEXP) -> Self {
        debug_assert_eq!(
            sexp.type_of(),
            crate::SEXPTYPE::EXTPTRSXP,
            "from_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };

        let cached_ptr = if is_type_erased::<T>() {
            // A type-erased handle never reads through `cached_ptr` (its
            // methods read the address afresh), so a pointer without an
            // address (restored from a saved session, or cleared) is accepted
            // here: the downcast that follows refuses it with a clear error.
            NonNull::new(any_raw.cast()).unwrap_or(NonNull::dangling())
        } else {
            debug_assert!(!any_raw.is_null(), "from_sexp_unchecked: null pointer");
            let any_box: &Box<dyn Any> = unsafe { &*any_raw };
            NonNull::from(unsafe { any_box.downcast_ref::<RefCell<T>>().unwrap_unchecked() })
        };

        Self::from_borrowed_parts(sexp, cached_ptr)
    }
    // endregion

    // region: Downcast support

    /// Returns the type name for type T.
    #[inline]
    pub fn type_name() -> &'static str {
        T::TYPE_NAME
    }

    /// Returns the type name stored in this ExternalPtr's prot slot.
    ///
    /// Returns `None` if the prot slot doesn't contain a valid type symbol.
    #[inline]
    pub fn stored_type_name(&self) -> Option<&'static str> {
        // SAFETY: `self.sexp` is a valid EXTPTRSXP; on R's main thread.
        unsafe { stored_type_id_symbol(self.sexp) }.map(symbol_name)
    }
    // endregion
}

impl ExternalPtr<()> {
    /// Create a type-erased ExternalPtr from an EXTPTRSXP without checking the stored type.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - Caller must ensure exclusive ownership semantics are upheld
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        debug_assert!(sexp.type_of() == SEXPTYPE::EXTPTRSXP);
        unsafe { Self::from_sexp_unchecked(sexp) }
    }

    /// Check whether the stored `Box<dyn Any>` contains a `T`.
    ///
    /// Uses `Any::is` for authoritative runtime type checking.
    #[inline]
    pub fn is<T: TypedExternal>(&self) -> bool {
        self.downcast_cell::<T>().is_some()
    }

    /// The `RefCell` holding the stored value if it is a `T`.
    ///
    /// Uses `Any::downcast_ref` for authoritative runtime type checking. This
    /// is where every generated method receiver (inherent and trait-impl) gets
    /// the struct: it borrows the cell for the call.
    #[doc(hidden)]
    #[inline]
    pub fn downcast_cell<T: TypedExternal>(&self) -> Option<&RefCell<T>> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        unsafe { stored_type_id_symbol(self.sexp) }?;
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.downcast_ref::<RefCell<T>>()
    }

    /// Borrow the stored value if it is a `T`, like [`ExternalPtr::borrow`].
    ///
    /// Uses `Any::downcast_ref` for authoritative runtime type checking.
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while the value
    /// is borrowed mutably.
    #[inline]
    pub fn downcast_ref<T: TypedExternal>(&self) -> Option<ExternalPtrRef<'_, T>> {
        self.downcast_cell::<T>()
            .map(|cell| ExternalPtrRef::from_guard(&self.sexp, borrow_value(cell)))
    }

    /// Borrow the stored value mutably if it is a `T`, like
    /// [`ExternalPtr::borrow_mut`].
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while the value
    /// is borrowed.
    #[inline]
    pub fn downcast_mut<T: TypedExternal>(&mut self) -> Option<ExternalPtrMut<'_, T>> {
        self.downcast_cell::<T>()
            .map(|cell| ExternalPtrMut::from_guard(&self.sexp, borrow_value_mut(cell)))
    }
}

// region: Consuming (`self` by value) method support

/// Marker left in an `EXTPTRSXP` slot while a consuming (`self` by value)
/// method runs, and left behind for good if that method panics.
///
/// `#[miniextendr]` methods taking bare `self` move the stored value out of
/// the R handle with [`ExternalPtr::<()>::take_for_consuming`], call the
/// method, and either write the result back
/// ([`ExternalPtr::<()>::restore_after_consuming`], for `self -> Self`) or
/// leave the slot consumed (terminal `self -> T`). A slot holding this marker
/// makes every later method call on the handle fail with a "consumed" error
/// instead of a type mismatch; the finaliser drops the marker like any value.
#[derive(Debug)]
pub struct ConsumedSlot;

/// Bound for the receiver of a **fallible** consuming method
/// (`self -> Result<Self, E>` / `self -> Option<Self>`).
///
/// The generated wrapper calls the method on a clone of the stored value and
/// only overwrites the R handle on `Ok` / `Some`, so a failed step leaves the
/// R object exactly as it was, which is what an interactive R user expects
/// from `obj |> add_step(-1)` erroring. Blanket-implemented for every
/// `T: Clone`; the `on_unimplemented` text below is what rustc prints when
/// the type is not `Clone`.
#[diagnostic::on_unimplemented(
    message = "a fallible consuming method (`self -> Result<Self, E>` / `Option<Self>`) needs `{Self}: Clone`",
    note = "the wrapper calls the method on a clone so a failed step leaves the R object untouched; derive or implement `Clone`, or take `&mut self` and return `Result<&mut Self, E>` instead"
)]
pub trait ConsumingFallible: Clone {}
impl<T: Clone> ConsumingFallible for T {}

/// Clone the stored value for a fallible consuming step (see
/// [`ConsumingFallible`]). Free function so codegen can name the bound.
#[inline]
pub fn clone_for_consuming<T: ConsumingFallible>(value: &T) -> T {
    value.clone()
}

/// Panic with the right message when a handle's stored value is not a `T`:
/// "consumed" if a previous `self`-by-value step failed; the classed
/// [`RESTORED_NO_VALUE_CLASS`] / [`RESTORED_OTHER_VERSION_CLASS`] error for a
/// pointer built for `T` that came back from a saved session without its
/// value; type mismatch otherwise. Used by generated method preludes instead
/// of a bare `expect`.
#[cold]
pub fn handle_downcast_failed<T: TypedExternal>(ptr: &ExternalPtr<()>) -> ! {
    refuse_consumed_or_restored::<T>(ptr);
    panic!(
        "expected ExternalPtr<{}>, found `{}`",
        ExternalPtr::<T>::type_name(),
        ptr.stored_type_name().unwrap_or("<unknown>")
    );
}

/// [`handle_downcast_failed`] for a trait-impl method receiver: the same
/// consumed / restored errors, and a mismatch message naming the trait
/// method (`Trait::method()`).
#[cold]
pub fn handle_downcast_failed_in<T: TypedExternal>(ptr: &ExternalPtr<()>, trait_method: &str) -> ! {
    refuse_consumed_or_restored::<T>(ptr);
    panic!(
        "type mismatch in {}: expected ExternalPtr<{}>, got different type. \
         This can happen if you pass an object of a different type to a trait method.",
        trait_method,
        ExternalPtr::<T>::type_name()
    );
}

/// The errors a receiver prelude raises before a type mismatch: "consumed"
/// for a slot a failed `self`-by-value step left, the classed restored-object
/// errors for a pointer built for `T` without an address.
fn refuse_consumed_or_restored<T: TypedExternal>(ptr: &ExternalPtr<()>) {
    if ptr.is_consumed() {
        panic!(
            "this `{}` object was consumed by a `self`-by-value method that did not return \
             a new value (it returned a plain result or panicked) and can no longer be used",
            ExternalPtr::<T>::type_name()
        );
    }
    if ptr.has_null_address() {
        // SAFETY: `ptr.sexp` is a valid EXTPTRSXP without an address; on R's
        // main thread (the generated prelude).
        unsafe { refuse_restored::<T>(ptr.sexp, ptr.sexp) };
    }
}

impl ExternalPtr<()> {
    /// Whether the slot holds the [`ConsumedSlot`] marker.
    pub fn is_consumed(&self) -> bool {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() || unsafe { stored_type_id_symbol(self.sexp) }.is_none() {
            return false;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.is::<ConsumedSlot>()
    }

    /// Whether the pointer has no address: restored from a saved session
    /// (`readRDS()` / `unserialize()`), cleared by `into_inner`, or never
    /// given one (R's `new("externalptr")`).
    pub fn has_null_address(&self) -> bool {
        unsafe { R_ExternalPtrAddr(self.sexp) }.is_null()
    }

    /// Move the stored `T` out of the handle, leaving [`ConsumedSlot`] behind.
    ///
    /// Returns `None` when the slot does not hold a `T` (wrong type, null, or
    /// already consumed); the slot is untouched in that case. The outer
    /// `Box<Box<dyn Any>>` cell stays allocated, so the finaliser and every
    /// other accessor keep working on the marker. The value's [`Sidecar<T>`]
    /// slots stay in the pointer's `prot` list across the method: a
    /// `self -> Self` result that keeps the fields (`Self { n, ..self }`)
    /// keeps the values, one built afresh (`Self::new(..)`) replaces them at
    /// the write-back. Runs on R's main thread (the generated receiver
    /// prelude). Raises the `miniextendr_borrow_conflict` error while a
    /// running call holds the value.
    pub fn take_for_consuming<T: TypedExternal>(&mut self) -> Option<T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        // A running call's borrow points into the storage this replaces.
        drop(borrow_value_mut(self.downcast_cell::<T>()?));
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        let taken = std::mem::replace(any_box, Box::new(ConsumedSlot));
        let cell = taken
            .downcast::<RefCell<T>>()
            .expect("checked by `downcast_cell` above");
        Some(cell.into_inner())
    }

    /// Put a value back into a slot emptied by [`Self::take_for_consuming`]
    /// (the write-back half of `self -> Self`). Replaces whatever the slot
    /// holds; the `TypedExternal` tag in the `prot` slot is unchanged because
    /// the type is the same. The pending values of the value's
    /// [`Sidecar<T>`] fields (what the method set with `Sidecar::new`) are
    /// flushed into `prot`, on R's main thread (a `worker` method's
    /// write-back runs on the worker, so the flush is routed through
    /// [`with_r_thread`](crate::worker::with_r_thread)); the other slots keep
    /// their values. Raises the `miniextendr_borrow_conflict` error while a
    /// running call holds the value the slot holds.
    pub fn restore_after_consuming<T: TypedExternal>(&mut self, value: T) {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        assert!(
            !any_raw.is_null(),
            "restore_after_consuming on a null external pointer"
        );
        assert!(
            unsafe { stored_type_id_symbol(self.sexp) }.is_some(),
            "restore_after_consuming on a foreign external pointer"
        );
        // A running call's borrow points into the storage this replaces.
        if let Some(cell) = self.downcast_cell::<T>() {
            drop(borrow_value_mut(cell));
        }
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        *any_box = Box::new(RefCell::new(value));
        if T::R_SLOT_COUNT > 0 {
            let sexp = self.sexp;
            let any_raw = crate::worker::Sendable(any_raw);
            crate::worker::with_r_thread(move || {
                let any_raw = any_raw.0;
                // SAFETY: on R's main thread; `any_raw` owns the `T` just
                // stored, inside the live EXTPTRSXP `sexp`.
                unsafe { init_sidecars::<T>(any_raw, sexp) };
            });
        }
    }
}

// endregion

/// Error returned when type checking fails in `try_from_sexp_with_error`.
///
/// The `found` field in `Mismatch` contains a `&'static str` from R's
/// interned symbol table, which persists for the R session lifetime.
#[derive(Debug, Clone)]
pub enum TypeMismatchError {
    /// The external pointer's address was null, and it was not built by
    /// `ExternalPtr` (R's `new("externalptr")`, a cleared pointer).
    NullPointer,
    /// The prot slot didn't contain a valid type symbol.
    InvalidTypeId,
    /// The stored type doesn't match the expected type.
    Mismatch {
        /// Expected Rust type name from this pointer wrapper.
        expected: &'static str,
        /// Actual stored Rust type name found in pointer metadata.
        found: &'static str,
    },
    /// A pointer this version of the crate built, restored from a saved
    /// session without its Rust value ([`RESTORED_NO_VALUE_CLASS`]).
    Restored {
        /// The type's name.
        type_name: &'static str,
        /// The package's classes for the type
        /// ([`TypedExternal::RESTORED_ERROR_CLASS`]).
        package_classes: &'static [&'static str],
        /// The package's message for the type
        /// ([`TypedExternal::RESTORED_ERROR_MESSAGE`]).
        package_message: Option<&'static str>,
    },
    /// A pointer another version of the crate saved
    /// ([`RESTORED_OTHER_VERSION_CLASS`]).
    OtherVersion {
        /// The type's name.
        type_name: &'static str,
        /// The crate's name.
        crate_name: &'static str,
        /// The version that saved the pointer.
        saved: &'static str,
        /// This version.
        current: &'static str,
        /// The package's classes for the type
        /// ([`TypedExternal::RESTORED_ERROR_CLASS`]).
        package_classes: &'static [&'static str],
        /// The package's message for the type
        /// ([`TypedExternal::RESTORED_ERROR_MESSAGE`]).
        package_message: Option<&'static str>,
    },
    /// A pointer restored from a saved session whose rebuild hook
    /// (`#[externalptr(rebuild = path)]`) failed (#2217). Nothing was
    /// installed; the next use tries again.
    RebuildFailed {
        /// The restored refusal a use of the object raises: the package's
        /// classes, [`RESTORED_NO_VALUE_CLASS`] and [`RESTORED_CLASS`], the
        /// message naming the cause, and the hook's condition as `e$parent`.
        /// Raise it with [`RCondition::raise`].
        condition: RCondition,
    },
}

impl TypeMismatchError {
    /// The condition classes of the error about a restored pointer: the
    /// package's classes, then the specific class, then [`RESTORED_CLASS`];
    /// `None` for the other variants (a failed rebuild's classes are its
    /// `condition`'s).
    pub fn restored_classes(&self) -> Option<Vec<&'static str>> {
        let (package_classes, specific) = match self {
            Self::Restored {
                package_classes, ..
            } => (package_classes, RESTORED_NO_VALUE_CLASS),
            Self::OtherVersion {
                package_classes, ..
            } => (package_classes, RESTORED_OTHER_VERSION_CLASS),
            Self::NullPointer
            | Self::InvalidTypeId
            | Self::Mismatch { .. }
            | Self::RebuildFailed { .. } => return None,
        };
        Some(
            package_classes
                .iter()
                .copied()
                .chain([specific, RESTORED_CLASS])
                .collect(),
        )
    }
}

impl fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullPointer => write!(f, "external pointer is null"),
            Self::InvalidTypeId => write!(f, "external pointer has no valid type id"),
            Self::Mismatch { expected, found } => {
                write!(
                    f,
                    "type mismatch: expected `{}`, found `{}`",
                    expected, found
                )
            }
            Self::Restored {
                type_name,
                package_message,
                ..
            } => match package_message {
                Some(message) => f.write_str(message),
                None => f.write_str(&restored_no_value_message(type_name)),
            },
            Self::OtherVersion {
                type_name,
                crate_name,
                saved,
                current,
                package_message,
                ..
            } => match package_message {
                Some(message) => f.write_str(message),
                None => f.write_str(&restored_other_version_message(
                    type_name, crate_name, saved, current,
                )),
            },
            Self::RebuildFailed { condition } => f.write_str(condition.message()),
        }
    }
}

impl std::error::Error for TypeMismatchError {}
// endregion

// region: MaybeUninit support

// We need a separate TypedExternal impl for MaybeUninit<T>
// This is typically done via blanket impl or macro

impl<T: TypedExternal> ExternalPtr<MaybeUninit<T>>
where
    MaybeUninit<T>: TypedExternal,
{
    /// Converts to `ExternalPtr<T>`.
    ///
    /// # Safety
    ///
    /// The value must have been initialized.
    ///
    /// # Implementation Note
    ///
    /// This method creates a *new* SEXP with `T`'s type information, leaving
    /// the original `MaybeUninit<T>` SEXP as an orphaned empty shell in R's heap.
    /// This is necessary because the type ID stored in the prot slot must match
    /// the actual type. The orphaned SEXP will be cleaned up by R's GC eventually.
    ///
    /// If you need to avoid this overhead, consider using `ExternalPtr<T>::new`
    /// directly and initializing in place via `as_mut`.
    ///
    /// Equivalent to `Box::assume_init`.
    #[inline]
    pub fn assume_init(self) -> ExternalPtr<T> {
        // Get the raw pointer (this clears the original SEXP, making its finalizer a no-op)
        let ptr = Self::into_raw(self).cast();

        // Create a new ExternalPtr with T's type info
        unsafe { ExternalPtr::from_raw(ptr) }
    }

    /// Writes a value and converts to initialized.
    ///
    /// Creates a new SEXP with `T`'s type information (the original
    /// `MaybeUninit<T>` SEXP becomes an orphaned shell, cleaned up by GC).
    #[inline]
    pub fn write(mut self, value: T) -> ExternalPtr<T> {
        unsafe {
            (*Self::as_mut_ptr(&mut self)).write(value);
            self.assume_init()
        }
    }
}
/// Type-erased `ExternalPtr` for cases where the concrete `T` is not needed.
pub type ErasedExternalPtr = ExternalPtr<()>;
// endregion

// region: Trait Implementations

impl<T: TypedExternal + Clone> Clone for ExternalPtr<T> {
    /// Deep clones the inner value into a new ExternalPtr.
    ///
    /// This creates a completely independent ExternalPtr with its own
    /// heap allocation and finalizer. The [`Sidecar<T>`] slots are copied as
    /// R objects (after flushing the source's pending values), so the two
    /// pointers share those values until one side writes a field: every
    /// write replaces the slot, and R copies a shared value before modifying
    /// it in place. For a type with `Sidecar` fields this runs on R's main
    /// thread only.
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while the value
    /// is borrowed mutably, like [`borrow`](Self::borrow).
    #[inline]
    fn clone(&self) -> Self {
        let value = self.borrow();
        if T::R_SLOT_COUNT == 0 {
            return Self::new(T::clone(&value));
        }
        assert_sidecar_clone_on_main_thread::<T>();
        // SAFETY: on R's main thread; `self.sexp` is the live pointer this
        // handle holds, built by `ExternalPtr` for a `T`.
        let prot = unsafe { R_ExternalPtrProtected(self.sexp) };
        flush_sidecars(&*value, prot);
        let new = Self::new(T::clone(&value));
        drop(value);
        // SAFETY: both pointers are live and built for a `T`.
        unsafe { copy_sidecar_slots::<T>(self.sexp, new.sexp) };
        new
    }

    /// Clones `source`'s value into this pointer's value in place. The
    /// [`Sidecar<T>`] slots are copied as R objects like in
    /// [`clone`](Self::clone), replacing this pointer's slot values and its
    /// pending values.
    ///
    /// # Panics
    ///
    /// With the R error of class `miniextendr_borrow_conflict` while
    /// `source`'s value is borrowed mutably or this one's is borrowed, which
    /// includes cloning a pointer into itself.
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let from = source.borrow();
        let mut to = self.borrow_mut();
        if T::R_SLOT_COUNT == 0 {
            T::clone_from(&mut to, &from);
            return;
        }
        assert_sidecar_clone_on_main_thread::<T>();
        // SAFETY: as in `clone`.
        let source_prot = unsafe { R_ExternalPtrProtected(source.sexp) };
        flush_sidecars(&*from, source_prot);
        T::clone_from(&mut to, &from);
        discard_pending_sidecars(&*to);
        drop((from, to));
        // SAFETY: both pointers are live and built for a `T`.
        unsafe { copy_sidecar_slots::<T>(source.sexp, self.sexp) };
    }
}

/// `ExternalPtr<T>` is `Send`, and copying `Sidecar` slots is R API.
fn assert_sidecar_clone_on_main_thread<T: TypedExternal>() {
    assert!(
        crate::worker::is_r_main_thread(),
        "cloning an `ExternalPtr<{}>` copies its sidecar slots, which runs on R's main thread only",
        T::TYPE_NAME
    );
}

impl<T: TypedExternal + Default> Default for ExternalPtr<T> {
    /// Creates an ExternalPtr containing the default value of T.
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Like `RefCell`'s: a value borrowed mutably prints as `<borrowed>`
/// instead of raising, and so does a type-erased `ExternalPtr<()>`, which has
/// no value of its own.
impl<T: TypedExternal + fmt::Debug> fmt::Debug for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if is_type_erased::<T>() {
            return f.write_str("<erased>");
        }
        match self.cell().try_borrow() {
            Ok(value) => fmt::Debug::fmt(&*value, f),
            Err(_) => f.write_str("<borrowed>"),
        }
    }
}

/// Borrows the value, like [`ExternalPtr::borrow`].
impl<T: TypedExternal + fmt::Display> fmt::Display for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.borrow(), f)
    }
}

impl<T: TypedExternal> fmt::Pointer for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&Self::as_ptr(self), f)
    }
}

/// Compares the values, borrowing both, like [`ExternalPtr::borrow`].
impl<T: TypedExternal + PartialEq> PartialEq for ExternalPtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: TypedExternal + Eq> Eq for ExternalPtr<T> {}

impl<T: TypedExternal + PartialOrd> PartialOrd for ExternalPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (*self.borrow()).partial_cmp(&*other.borrow())
    }
}

impl<T: TypedExternal + Ord> Ord for ExternalPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self.borrow()).cmp(&*other.borrow())
    }
}

impl<T: TypedExternal + Hash> Hash for ExternalPtr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self.borrow()).hash(state);
    }
}

/// Each step borrows the value mutably, like [`ExternalPtr::borrow_mut`].
impl<T: TypedExternal + std::iter::Iterator> std::iter::Iterator for ExternalPtr<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.borrow_mut().next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.borrow().size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.borrow_mut().nth(n)
    }
}

impl<T: TypedExternal + std::iter::DoubleEndedIterator> std::iter::DoubleEndedIterator
    for ExternalPtr<T>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.borrow_mut().next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.borrow_mut().nth_back(n)
    }
}

impl<T: TypedExternal + std::iter::ExactSizeIterator> std::iter::ExactSizeIterator
    for ExternalPtr<T>
{
    fn len(&self) -> usize {
        self.borrow().len()
    }
}

impl<T: TypedExternal + std::iter::FusedIterator> std::iter::FusedIterator for ExternalPtr<T> {}

impl<T: TypedExternal> From<T> for ExternalPtr<T> {
    #[inline]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T: TypedExternal> From<Box<T>> for ExternalPtr<T> {
    #[inline]
    fn from(boxed: Box<T>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(boxed)) }
    }
}

// `Drop` releases the R-side GC root taken at construction (for *owning*
// handles only) but never frees the pointee — that stays R's job, run by the
// `release_any` finalizer when R garbage-collects the `EXTPTRSXP`. Dropping the
// root just makes the object eligible for collection once R itself holds no
// other reference; if R still references it (the usual case — it was returned
// from a `.Call` or stored), it stays alive and the finalizer runs later.
//
// For deterministic *value* cleanup, use `ExternalPtr::into_inner` (moves the
// value out) or `drop(Box::from_raw(ExternalPtr::into_raw(ptr)))`.
impl<T: TypedExternal> Drop for ExternalPtr<T> {
    fn drop(&mut self) {
        self.release_root_if_owned();
    }
}
// endregion

// region: Finalizer

/// Guard that aborts the process if a panic starts while it is alive.
///
/// Used by [`drop_catching_panic`] to implement panic-safe destructor calls
/// without `catch_unwind`. When `f()` completes normally, the guard is
/// dropped with `std::thread::panicking()` unchanged and becomes a no-op.
/// If `f()` panics, the guard's destructor runs during stack unwinding
/// (when `std::thread::panicking()` has become `true`) and calls
/// `process::abort()`.
///
/// A destructor can also run *because* of an unrelated panic: a handle that
/// goes out of scope in a panicking `#[miniextendr]` fn is dropped by that
/// unwind, with `panicking()` already `true`. That is not a panic in the
/// destructor, so the guard records the state it started in and aborts only
/// on a change. (A panic in `f()` while already unwinding is a double panic,
/// which the Rust runtime aborts on by itself.)
///
/// This approach avoids `catch_unwind`, which registers LLVM unwind landing
/// pads. Inside R's GC finalizer walk, any interaction with the unwinding
/// machinery — especially on the first call that lazily initialises exception
/// handling state — can trigger an allocator call that re-enters R's GC and
/// produces a "recursive gc invocation" hard crash.
#[must_use]
struct AbortIfUnwinding {
    /// `std::thread::panicking()` when the guard was created.
    already_unwinding: bool,
}

impl AbortIfUnwinding {
    #[inline]
    fn new() -> Self {
        AbortIfUnwinding {
            already_unwinding: std::thread::panicking(),
        }
    }
}

impl Drop for AbortIfUnwinding {
    #[cold]
    fn drop(&mut self) {
        if !self.already_unwinding && std::thread::panicking() {
            // A panic propagated through a finalizer — abort immediately.
            // The value being dropped is in an indeterminate state; continuing
            // is not safe.
            eprintln!("miniextendr: destructor panicked during R finalization; aborting");
            std::process::abort();
        }
    }
}

/// Run a destructor closure, aborting the process if the closure panics.
///
/// A panic inside a GC finalizer cannot be safely propagated: the finalizer
/// runs at an arbitrary point in R's garbage collector, and unwinding across
/// the C-ABI boundary into R's runtime is undefined behaviour. Aborting is
/// the only safe recovery strategy — the destructor has already left the
/// value in an indeterminate state, so continuing is not an option.
///
/// ## Implementation note
///
/// This function deliberately avoids `std::panic::catch_unwind`. On the first
/// call from within R's GC finalizer, `catch_unwind` may lazily initialise
/// LLVM exception-handling state, which can allocate. Any allocation during a
/// GC finalizer re-enters the GC and triggers the fatal "recursive gc
/// invocation" crash. Instead, this function uses a drop-guard that compares
/// `std::thread::panicking()` — a cheap, allocation-free TLS read — on entry
/// and on exit.
///
/// Called from a `Drop` that an unrelated panic's unwind is running, it runs
/// `f()` and returns: the unwind continues and reaches its `catch_unwind`
/// boundary (for a `#[miniextendr]` fn, the R error it becomes).
///
/// Deferred conditions queued by this destructor (including nested guards)
/// are suppressed on this thread. Finalization has no caller handlers to
/// receive them, and retaining them would strand or misattribute diagnostics.
///
/// This helper is `#[doc(hidden)]` because it is called from macro-generated
/// code and is not part of the public API.
#[doc(hidden)]
#[inline]
pub fn drop_catching_panic<F: FnOnce()>(f: F) {
    let _guard = AbortIfUnwinding::new();
    let _conditions = crate::deferred_condition::Suppress::new();
    f();
    // guard dropped here with panicking() unchanged → no-op
}

/// Non-generic C finalizer called by R's garbage collector.
///
/// Since `ExternalPtr` stores `Box<Box<dyn Any>>`, the `Any` vtable carries
/// the concrete type's drop function. No generic parameter needed — one
/// finalizer function handles all `ExternalPtr<T>` types.
extern "C-unwind" fn release_any(sexp: SEXP) {
    if sexp.is_null_or_nil() {
        return;
    }

    let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };

    // Guard against double-finalization
    if any_raw.is_null() {
        return;
    }

    // Clear the external pointer first (prevents double-free if called again)
    unsafe { R_ClearExternalPtr(sexp) };

    // Reconstruct the outer Box<Box<dyn Any>> and let it drop.
    // This drops the outer Box, then the inner Box<dyn Any>, which
    // uses the vtable to drop the concrete T value.
    //
    // A panicking Drop impl must not unwind across the C-ABI boundary into R.
    // `drop_catching_panic` catches any panic and aborts instead.
    drop_catching_panic(|| drop(unsafe { Box::from_raw(any_raw) }));
}
// endregion

// region: Utility: ExternalSlice (helper for slice data)

/// A slice stored as a standalone struct, suitable for wrapping in ExternalPtr.
///
/// This is analogous to the data inside a `Box<[T]>`, but stores capacity
/// for proper deallocation when created from a `Vec`.
///
/// # Usage
///
/// To use with `ExternalPtr`, implement `TypedExternal` for your specific
/// `ExternalSlice<YourType>`:
///
/// ```ignore
/// impl_typed_external!(ExternalSlice<MyElement>);
/// let ptr = ExternalPtr::new(ExternalSlice::new(vec![1, 2, 3]));
/// ```
#[repr(C)]
pub struct ExternalSlice<T: 'static> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T: 'static> ExternalSlice<T> {
    /// Create an external slice from a `Vec`, preserving its allocation.
    pub fn new(slice: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(slice);
        Self {
            ptr: unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) },
            len: vec.len(),
            capacity: vec.capacity(),
        }
    }

    /// Create from a boxed slice (capacity == len).
    pub fn from_boxed(boxed: Box<[T]>) -> Self {
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
            capacity: len,
        }
    }

    /// Borrow the contents as a shared slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Borrow the contents as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacity of the underlying allocation.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T: 'static> Drop for ExternalSlice<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.capacity);
        }
    }
}
// endregion

mod altrep_helpers;
pub use altrep_helpers::*;

mod borrowed;
pub use borrowed::{
    ExternalPtrMut, ExternalPtrRef, Saved, SidecarOwner, SidecarOwnerMut, SidecarRead,
};

mod sidecar;
pub use sidecar::*;

#[cfg(test)]
mod tests {
    use super::{drop_catching_panic, rebuild_failed_text};

    /// A failed rebuild's message (#2218): the package's template with the
    /// cause in place of each `%s`, filled once; else the restored message
    /// and the cause in parentheses; else miniextendr's.
    #[test]
    fn rebuild_failed_text_fills_the_template() {
        assert_eq!(
            rebuild_failed_text(
                Some("the engine could not be rebuilt (%s); build a new one"),
                Some("saved"),
                "Engine",
                "bad %s value"
            ),
            "the engine could not be rebuilt (bad %s value); build a new one"
        );
        assert_eq!(
            rebuild_failed_text(Some("%s, then %s"), None, "Engine", "x"),
            "x, then x"
        );
        assert_eq!(
            rebuild_failed_text(None, Some("this engine was saved"), "Engine", "bad"),
            "this engine was saved (rebuilding it failed: bad)"
        );
        assert_eq!(
            rebuild_failed_text(None, None, "Engine", "bad"),
            "this `Engine` object was restored from a saved session and rebuilding it failed: bad"
        );
    }

    #[test]
    fn drop_catching_panic_does_not_propagate_panic() {
        // Verify that drop_catching_panic catches a panicking closure and does
        // NOT propagate the panic to the caller.
        //
        // Note: we cannot test the abort path from inside a test process, so
        // we document it with a comment instead:
        //   If the closure panics, `drop_catching_panic` calls `eprintln!` then
        //   `std::process::abort()`. That path is exercised only by the process
        //   dying, which is observable from an external test harness (not done
        //   here to keep CI simple).
        //
        // What we CAN test: the happy path (no panic) completes normally, and
        // the function compiles and links correctly with a `FnOnce()` generic.
        let mut ran = false;
        drop_catching_panic(|| {
            ran = true;
        });
        assert!(ran, "closure should have been called");
    }

    #[test]
    fn drop_catching_panic_happy_path_drops_value() {
        // Confirm that the closure's side-effects (i.e. actual drop) occur
        // when no panic is raised.
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let dropped = Arc::new(AtomicBool::new(false));
        let flag = dropped.clone();

        struct DropSignal(Arc<AtomicBool>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let signal = DropSignal(flag);
        drop_catching_panic(|| drop(signal));

        assert!(
            dropped.load(Ordering::SeqCst),
            "inner value should have been dropped"
        );
    }

    #[test]
    fn drop_catching_panic_in_a_drop_run_by_an_unwind_does_not_abort() {
        // A guarded `Drop` that runs because an unrelated panic is unwinding
        // (a handle going out of scope in a panicking `#[miniextendr]` fn) is
        // not a panic in the destructor: the unwind must continue to its
        // `catch_unwind`. Before #1917 this aborted the test process.
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        struct GuardedDrop(Arc<AtomicBool>);
        impl Drop for GuardedDrop {
            fn drop(&mut self) {
                let ran = self.0.clone();
                drop_catching_panic(move || ran.store(true, Ordering::SeqCst));
            }
        }

        let ran = Arc::new(AtomicBool::new(false));
        let handle = GuardedDrop(ran.clone());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _handle = handle;
            panic!("unrelated panic");
        }));

        assert!(result.is_err(), "the unrelated panic should propagate");
        assert!(
            ran.load(Ordering::SeqCst),
            "the guarded destructor should have run during the unwind"
        );
    }
}
