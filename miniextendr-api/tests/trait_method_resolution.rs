//! Trait callbacks select the declared interface despite same-named supertraits.
mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, TryFromSexp, miniextendr};

pub trait RustBase {
    fn value(&self) -> i32 {
        -10
    }
    fn add(&mut self, n: i32) -> i32 {
        n - 20
    }
}

#[miniextendr]
pub trait Interface: RustBase {
    fn value(&self) -> i32;
    fn add(&mut self, n: i32) -> i32;
    fn inherited(&self) -> i32 {
        <Self as Interface>::value(self) + 1
    }
}

#[miniextendr]
pub trait Associated: RustBase {
    type Value;
    fn value(&self) -> Self::Value;
}

// Generic traits already use concrete, qualified callbacks at the impl site.
#[miniextendr]
pub trait Generic<T>: RustBase {
    fn value(&self) -> T;
}

#[derive(ExternalPtr)]
pub struct Host(i32);

impl RustBase for Host {}

// These inherent methods must not capture either callback path.
impl Host {
    fn value(&self) -> i32 {
        -100
    }
    fn add(&mut self, n: i32) -> i32 {
        n - 200
    }
}

#[miniextendr(env, noexport)]
impl Interface for Host {
    fn value(&self) -> i32 {
        self.0
    }
    fn add(&mut self, n: i32) -> i32 {
        self.0 += n;
        self.0
    }
}

#[miniextendr(env, noexport)]
impl Associated for Host {
    type Value = i32;
    fn value(&self) -> i32 {
        self.0 + 10
    }
}

#[miniextendr(env, noexport)]
impl Generic<i32> for Host {
    fn value(&self) -> i32 {
        self.0 + 20
    }
}

#[test]
fn ordinary_rust_dispatch_remains_explicit_and_object_safe() {
    let mut host = Host(1);
    assert_eq!(host.value(), -100);
    assert_eq!(host.add(1), -199);
    assert_eq!(RustBase::value(&host), -10);
    assert_eq!(RustBase::add(&mut host, 1), -19);
    let interface: &dyn Interface = &host;
    assert_eq!(Interface::value(interface), 1);
    assert_eq!(interface.inherited(), 2);
}

#[test]
fn declared_and_concrete_callbacks_select_the_intended_trait() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let host = std::cell::RefCell::new(Host(3));
        let data = std::ptr::from_ref(&host).cast_mut().cast();
        let declared = __Interface_build_vtable::<Host>();
        let concrete = &__VTABLE_trait_method_resolution_Interface_FOR_Host;
        for (value, add, inherited) in [
            (declared.value, declared.add, declared.inherited),
            (concrete.value, concrete.add, concrete.inherited),
        ] {
            let before = host.borrow().0;
            let value = scope.protect(value(data, 0, std::ptr::null())).get();
            assert_eq!(i32::try_from_sexp(value).unwrap(), before);
            let n = scope.protect(2i32.into_sexp()).get();
            let value = scope.protect(add(data, 1, [n].as_ptr())).get();
            assert_eq!(i32::try_from_sexp(value).unwrap(), before + 2);
            assert_eq!(host.borrow().0, before + 2);
            let value = scope.protect(inherited(data, 0, std::ptr::null())).get();
            assert_eq!(i32::try_from_sexp(value).unwrap(), before + 3);
        }
        let associated = __Associated_build_vtable::<Host>();
        let value = scope
            .protect((associated.value)(data, 0, std::ptr::null()))
            .get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 17);
        let generic = &__VTABLE_trait_method_resolution_Generic_FOR_Host;
        let value = scope
            .protect((generic.value)(data, 0, std::ptr::null()))
            .get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 27);
    });
}
