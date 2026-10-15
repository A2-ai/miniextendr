//! Integration tests for ExternalPtr.

mod r_test_utils;

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::sys::Rf_install;
use std::ffi::CString;

#[test]
fn externalptr_suite() {
    r_test_utils::with_r_thread(|| {
        test_basic_access();
        test_tag_and_protected();
        test_wrap_sexp();
        test_into_raw();
    });
}

fn test_basic_access() {
    let mut ext = ExternalPtr::new(10i32);
    assert_eq!(*ext.borrow(), 10);

    *ext.borrow_mut() = 42;
    assert_eq!(*ext.borrow(), 42);

    // Debug never raises: a value borrowed mutably prints as `<borrowed>`.
    let alias = unsafe { ExternalPtr::<i32>::wrap_sexp(ext.as_sexp()) }.unwrap();
    let held = ext.borrow_mut();
    assert_eq!(format!("{alias:?}"), "<borrowed>");
    drop(held);
    assert_eq!(format!("{alias:?}"), "42");
}

fn test_tag_and_protected() {
    let ext = ExternalPtr::new(5i32);

    let tag = ext.tag();
    let expected = unsafe {
        let c_str = CString::new("i32").unwrap();
        Rf_install(c_str.as_ptr())
    };
    assert!(std::ptr::eq(tag.0, expected.0));

    let protected = SEXP::scalar_integer(123);
    let ok = unsafe { ext.set_protected(protected) };
    assert!(ok);
    let stored = ext.protected();
    assert!(std::ptr::eq(stored.0, protected.0));
}

fn test_wrap_sexp() {
    let ext = ExternalPtr::new(7i32);
    let sexp: SEXP = ext.as_sexp();

    let same = unsafe { ExternalPtr::<i32>::wrap_sexp(sexp) };
    assert!(same.is_some());

    let wrong = unsafe { ExternalPtr::<f64>::wrap_sexp(sexp) };
    assert!(wrong.is_none());
}

fn test_into_raw() {
    let ext = ExternalPtr::new(99i32);
    let ptr = ExternalPtr::into_raw(ext);
    assert!(!ptr.is_null());

    unsafe {
        let boxed = Box::from_raw(ptr);
        assert_eq!(*boxed, 99);
        drop(boxed);
    }
}
