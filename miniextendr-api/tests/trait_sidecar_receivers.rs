//! Explicit owner receivers work for ordinary pointers and erased trait views.
mod r_test_utils;

use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::trait_abi::{TraitView, ccall};
use miniextendr_api::{
    ExternalPtr, ExternalPtrMut, ExternalPtrRef, IntoR, RCondition, SEXP, TryFromSexp,
    TypedExternal, miniextendr,
};

#[miniextendr]
pub trait SlotProbe: TypedExternal {
    fn read(self: ExternalPtrRef<'_, Self>) -> Vec<i32>;
    fn write(self: ExternalPtrMut<'_, Self>, values: Vec<i32>);
    fn plain(&self) -> i32;
    fn binding(self: ExternalPtrRef<'_, Self>, __miniextendr_self: i32) -> i32;
    fn compare(self: ExternalPtrMut<'_, Self>, other: &Self) -> i32;
    fn read_default(self: ExternalPtrRef<'_, Self>) -> Vec<i32> {
        Self::read(self)
    }
}

#[derive(ExternalPtr)]
pub struct SlotHost {
    generation: i32,
    #[r_data]
    keys: Sidecar<Vec<i32>>,
}

fn host() -> SlotHost {
    SlotHost {
        generation: 0,
        keys: Sidecar::new(vec![3, 5, 8]),
    }
}

#[miniextendr(env, noexport)]
impl SlotProbe for SlotHost {
    fn read(self: ExternalPtrRef<'_, Self>) -> Vec<i32> {
        Self::keys(&self)
    }
    fn write(mut self: ExternalPtrMut<'_, Self>, values: Vec<i32>) {
        Self::set_keys(&mut self, values);
        self.generation += 1;
    }
    fn plain(&self) -> i32 {
        self.generation
    }
    fn binding(self: ExternalPtrRef<'_, Self>, __miniextendr_self: i32) -> i32 {
        self.generation + __miniextendr_self
    }
    fn compare(self: ExternalPtrMut<'_, Self>, other: &Self) -> i32 {
        self.generation - other.generation
    }
}

#[miniextendr]
pub trait GenericSlots<T>: TypedExternal {
    fn first(self: ExternalPtrRef<'_, Self>) -> T;
}

#[miniextendr(env, noexport)]
impl GenericSlots<i32> for SlotHost {
    fn first(self: ExternalPtrRef<'_, Self>) -> i32 {
        Self::keys(&self)[0]
    }
}

#[test]
fn ordinary_handles_can_call_the_same_trait_functions() {
    r_test_utils::with_r_thread(|| {
        let mut ptr = ExternalPtr::new(host());
        assert_eq!(SlotProbe::read(ExternalPtrRef::from(&ptr)), [3, 5, 8]);
        SlotProbe::write(ExternalPtrMut::from(&mut ptr), vec![13, 21]);
        assert_eq!(SlotHost::keys(&ptr), [13, 21]);
        assert_eq!(ptr.generation, 1);
    });
}

#[test]
fn erased_views_read_write_and_keep_sidecars_after_temporary_roots_drop() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut view = {
            let owner = OwnedProtect::new(ccall::mx_wrap(__mx_wrap_SlotHost(host())));
            SlotProbeView::from_sexp(owner.get())
        };
        miniextendr_api::sys::R_gc();
        assert_eq!(view.read_default(), [3, 5, 8]);
        view.write(vec![34, 55]);
        miniextendr_api::sys::R_gc();
        assert_eq!(view.read(), [34, 55]);
        assert_eq!(view.plain(), 1);
        assert_eq!(view.binding(7), 8);
    });
}

#[test]
fn concrete_generic_shims_receive_the_same_owner() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(ccall::mx_wrap(__mx_wrap_SlotHost(host())));
        let vtable = ccall::mx_query(owner.get(), TAG_GenericSlots).cast::<GenericSlotsVTable>();
        assert!(!vtable.is_null());
        let erased = ccall::mx_get(owner.get());
        let data = erased
            .cast::<u8>()
            .add((*(*erased).base).data_offset)
            .cast();
        let result = OwnedProtect::new(((*vtable).first)(data, owner.get(), 0, core::ptr::null()));
        assert_eq!(i32::try_from_sexp(result.get()).unwrap(), 3);
    });
}

#[test]
fn owner_receivers_keep_self_argument_alias_preflight() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let mut ptr = ExternalPtr::new(host());
        let data = ptr.as_mut_ptr().cast();
        let args = [ptr.as_sexp()];
        let table = __SlotProbe_build_vtable::<SlotHost>();
        let result = scope.protect((table.compare)(data, ptr.as_sexp(), 1, args.as_ptr()));
        let (_, parts) = RCondition::from_tagged_sexp(result.get())
            .unwrap()
            .into_parts();
        assert!(
            parts.message.contains("alias the same object"),
            "{}",
            parts.message
        );
    });
}

#[test]
fn raw_views_without_an_owner_refuse_owner_methods() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut ptr = ExternalPtr::new(host());
        let table = __SlotProbe_build_vtable::<SlotHost>();
        let view = <SlotProbeView as TraitView>::from_raw_parts(
            ptr.as_mut_ptr().cast(),
            core::ptr::from_ref(&table).cast(),
            None,
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| view.read()));
        assert!(result.is_err());
        assert_eq!(view.plain(), 0);
    });
}

// Find by the semantic suffix; the registration prefix uses this test crate's
// name. Checking arity before the cast keeps these calls matched to the C ABI.
unsafe fn registration(suffix: &str, arity: i32) -> miniextendr_api::sys::R_CallMethodDef {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .expect("generated trait wrapper must be registered");
    assert_eq!(entry.numArgs, arity);
    *entry
}

#[test]
fn call_wrappers_create_owner_receivers_and_refuse_aliases() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let owner = ExternalPtr::new(host());
        let write = std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP,
        >(registration("SlotHost__SlotProbe__write", 3).fun.unwrap());
        let read = std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(registration("SlotHost__SlotProbe__read", 2).fun.unwrap());
        let compare = std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP) -> SEXP,
        >(registration("SlotHost__SlotProbe__compare", 3).fun.unwrap());
        let values = scope.protect(vec![89_i32, 144].into_sexp());
        write(SEXP::nil(), owner.as_sexp(), values.get());
        let result = scope.protect(read(SEXP::nil(), owner.as_sexp()));
        assert_eq!(Vec::<i32>::try_from_sexp(result.get()).unwrap(), [89, 144]);
        assert_eq!(owner.generation, 1);
        let refused = scope.protect(compare(SEXP::nil(), owner.as_sexp(), owner.as_sexp()));
        let (_, parts) = RCondition::from_tagged_sexp(refused.get())
            .unwrap()
            .into_parts();
        assert!(
            parts.message.contains("alias the same object"),
            "{}",
            parts.message
        );
    });
}
