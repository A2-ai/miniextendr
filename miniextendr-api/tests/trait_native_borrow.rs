//! Trait ABI callers bypass the ordinary R wrappers, so shims need preflight too.
mod r_test_utils;

use miniextendr_api::abi::mx_meth;
use miniextendr_api::condition::RCondition;
use miniextendr_api::from_r::{NativeBorrow, SexpError, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::{ExternalPtr, IntoR, RCow, SEXP, miniextendr};
use std::cell::RefCell;

/// Advertise a native borrow, but stop before creating any reference. A
/// regression must fail an assertion without itself exercising aliased &mut.
macro_rules! probe {
    ($name:ident, $borrow:ty) => {
        pub struct $name;

        impl TryFromSexp for $name {
            type Error = SexpError;
            const NATIVE_BORROW: Option<NativeBorrow> = <$borrow>::NATIVE_BORROW;

            fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
                panic!("argument conversion reached");
            }
        }

        impl IntoR for $name {
            type Error = std::convert::Infallible;

            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                panic!("the test never calls a trait View");
            }
        }
    };
}

probe!(WriteProbe, &'static mut i32);
probe!(ReadProbe, &'static i32);
probe!(WriteListProbe, Vec<&'static mut i32>);
probe!(PairProbe, (&'static mut i32, &'static i32));

#[derive(ExternalPtr)]
pub struct Sink;

// `other` sits between the native arguments, so the preflight must read the
// original argv positions 0 and 2. `inspect_view` binds a borrowed view, which
// has no `TryFromSexp` and takes its borrow metadata from the view type.
#[miniextendr]
pub trait Inspect {
    fn inspect(&self, write: WriteProbe, other: &Self, read: ReadProbe) -> i32;
    fn inspect_list(&self, items: WriteListProbe) -> i32;
    fn inspect_pair(&self, pair: PairProbe) -> i32;
    fn inspect_view(&self, values: RCow<'_, i32>, write: WriteProbe) -> i32;
}

impl Inspect for Sink {
    fn inspect(&self, _: WriteProbe, _: &Self, _: ReadProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
    fn inspect_list(&self, _: WriteListProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
    fn inspect_pair(&self, _: PairProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
    fn inspect_view(&self, _: RCow<'_, i32>, _: WriteProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
}

#[miniextendr]
pub trait InspectGeneric<T> {
    fn inspect(&self, write: T, other: &Self, read: ReadProbe) -> i32;
    fn inspect_list(&self, items: WriteListProbe) -> i32;
    fn inspect_pair(&self, pair: PairProbe) -> i32;
    fn inspect_view(&self, values: RCow<'_, i32>, write: T) -> i32 {
        let _ = (values, write);
        panic!("the converter must stop before the method");
    }
}

// This path generates its shims at the concrete impl, not the trait declaration.
#[miniextendr(env, noexport)]
impl InspectGeneric<WriteProbe> for Sink {
    // Deliberately differ from the declaration. Both shim paths must name
    // the declaration's arguments, including raw identifiers and `_` patterns.
    fn inspect(&self, _write: WriteProbe, _other: &Self, r#read: ReadProbe) -> i32 {
        let _ = r#read;
        panic!("the converter must stop before the method");
    }
    fn inspect_list(&self, _: WriteListProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
    fn inspect_pair(&self, _pair: PairProbe) -> i32 {
        panic!("the converter must stop before the method");
    }
}

fn list(scope: &ProtectScope, left: SEXP, right: SEXP) -> SEXP {
    unsafe {
        let builder = ListBuilder::new(scope, 2);
        builder.set(0, left);
        builder.set(1, right);
        builder.into_sexp()
    }
}

/// Call a shim and return the message of the tagged condition it must return.
fn shim_message(
    scope: &ProtectScope,
    shim: mx_meth,
    data: *const RefCell<Sink>,
    args: &[SEXP],
) -> String {
    unsafe {
        let argc = i32::try_from(args.len()).unwrap();
        let argv = if args.is_empty() {
            std::ptr::null()
        } else {
            args.as_ptr()
        };
        let result = scope
            .protect(shim(data.cast_mut().cast(), argc, argv))
            .get();
        let (_, parts) = RCondition::from_tagged_sexp(result)
            .expect("shim must return a tagged condition")
            .into_parts();
        parts.message
    }
}

fn check_shims(
    inspect: mx_meth,
    inspect_list: mx_meth,
    inspect_pair: mx_meth,
    inspect_view: mx_meth,
) {
    unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let w = scope.protect(8i32.into_sexp()).get();
        let other = ExternalPtr::new(Sink);
        // The shims borrow the receiver's RefCell, as in trait-ABI storage.
        let sink = RefCell::new(Sink);
        let data = std::ptr::from_ref(&sink);

        let message = shim_message(&scope, inspect, data, &[v, other.as_sexp(), v]);
        assert!(
            message.contains(
                "aliasing native arguments: parameters `write` and `read` borrow the same"
            ),
            "{message}"
        );
        let message = shim_message(&scope, inspect, data, &[v, other.as_sexp(), w]);
        assert!(message.contains("argument conversion reached"), "{message}");

        let message = shim_message(&scope, inspect_list, data, &[list(&scope, v, v)]);
        assert!(
            message.contains("aliasing native arguments: parameter `items` contains duplicate"),
            "{message}"
        );
        let message = shim_message(&scope, inspect_list, data, &[list(&scope, v, w)]);
        assert!(message.contains("argument conversion reached"), "{message}");

        // Tuple fields are compared with each other (#2014).
        let message = shim_message(&scope, inspect_pair, data, &[list(&scope, v, v)]);
        assert!(
            message.contains("aliasing native arguments: parameter `pair` contains duplicate"),
            "{message}"
        );
        let message = shim_message(&scope, inspect_pair, data, &[list(&scope, v, w)]);
        assert!(message.contains("argument conversion reached"), "{message}");

        // The view's shared borrow counts against the probe's mutable one.
        let message = shim_message(&scope, inspect_view, data, &[v, v]);
        assert!(
            message.contains(
                "aliasing native arguments: parameters `values` and `write` borrow the same"
            ),
            "{message}"
        );
        // A view that cannot bind fails like any other conversion.
        let text = scope.protect("text".into_sexp()).get();
        let message = shim_message(&scope, inspect_view, data, &[text, w]);
        assert!(
            message.contains("invalid 'values' argument: expected integer, got character"),
            "{message}"
        );
        let message = shim_message(&scope, inspect_view, data, &[v, w]);
        assert!(message.contains("argument conversion reached"), "{message}");

        // Arity must be checked before the preflight dereferences argv.
        let message = shim_message(&scope, inspect, data, &[]);
        assert!(message.contains("expected 3 arguments, got 0"), "{message}");
        let message = shim_message(&scope, inspect_list, data, &[]);
        assert!(message.contains("expected 1 arguments, got 0"), "{message}");
        let message = shim_message(&scope, inspect_view, data, &[]);
        assert!(message.contains("expected 2 arguments, got 0"), "{message}");
    }
}

#[test]
fn declaration_shim_checks_native_aliases_before_argument_conversion() {
    r_test_utils::with_r_thread(|| {
        let vtable = __Inspect_build_vtable::<Sink>();
        check_shims(
            vtable.inspect,
            vtable.inspect_list,
            vtable.inspect_pair,
            vtable.inspect_view,
        );
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
        check_shims(
            vtable.inspect,
            vtable.inspect_list,
            vtable.inspect_pair,
            vtable.inspect_view,
        );
    });
}
