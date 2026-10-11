//! Trait ABI callers bypass the ordinary R wrappers, so shims need preflight too.
mod r_test_utils;

use miniextendr_api::abi::mx_meth;
use miniextendr_api::condition::RCondition;
use miniextendr_api::from_r::{NativeBorrow, SexpError, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::{ExternalPtr, IntoR, SEXP, miniextendr};

/// Advertise a borrowing tuple, but stop before creating any references. A
/// regression must fail an assertion without itself exercising aliased &mut.
pub struct BorrowProbe;

impl TryFromSexp for BorrowProbe {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <(&'static mut i32, &'static i32)>::NATIVE_BORROW;

    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        panic!("argument conversion reached");
    }
}

impl IntoR for BorrowProbe {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        panic!("the test never calls a trait View");
    }
}

#[derive(ExternalPtr)]
pub struct Sink;

#[miniextendr]
pub trait InspectPair {
    fn inspect(&self, pair: BorrowProbe) -> i32;
}

impl InspectPair for Sink {
    fn inspect(&self, _: BorrowProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
}

#[miniextendr]
pub trait InspectGeneric<T> {
    fn inspect(&self, pair: T) -> i32;
}

// This path generates its shim at the concrete impl, not the trait declaration.
#[miniextendr(env, noexport)]
impl InspectGeneric<BorrowProbe> for Sink {
    fn inspect(&self, _pair: BorrowProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
}

fn pair(scope: &ProtectScope, left: SEXP, right: SEXP) -> SEXP {
    unsafe {
        let builder = ListBuilder::new(scope, 2);
        builder.set(0, left);
        builder.set(1, right);
        builder.into_sexp()
    }
}

fn check_shim(shim: mx_meth) {
    unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let other = scope.protect(8i32.into_sexp()).get();
        let mut sink = Sink;
        let data = std::ptr::from_mut(&mut sink).cast();
        for (value, expected) in [
            (pair(&scope, v, v), "aliasing native arguments"),
            (pair(&scope, v, other), "argument conversion reached"),
        ] {
            let args = [value];
            let result = scope.protect(shim(data, 1, args.as_ptr())).get();
            let (_, parts) = RCondition::from_tagged_sexp(result)
                .expect("shim must return a tagged condition")
                .into_parts();
            assert!(parts.message.contains(expected), "{}", parts.message);
        }
        // Arity must be checked before the new preflight dereferences argv.
        let result = scope.protect(shim(data, 0, std::ptr::null())).get();
        let (_, parts) = RCondition::from_tagged_sexp(result)
            .expect("arity mismatch must return a tagged condition")
            .into_parts();
        assert!(parts.message.contains("expected 1 arguments, got 0"));
    }
}

#[test]
fn declaration_shim_checks_native_aliases_before_argument_conversion() {
    r_test_utils::with_r_thread(|| {
        check_shim(__InspectPair_build_vtable::<Sink>().inspect);
    });
}

#[test]
fn concrete_generic_shim_checks_native_aliases_before_argument_conversion() {
    r_test_utils::with_r_thread(|| {
        let entry = miniextendr_api::registry::MX_TRAIT_DISPATCH
            .iter()
            .find(|entry| entry.trait_tag == TAG_InspectGeneric)
            .expect("generic impl must register its concrete vtable");
        let vtable = unsafe { &*entry.vtable.cast::<InspectGenericVTable>() };
        check_shim(vtable.inspect);
    });
}
