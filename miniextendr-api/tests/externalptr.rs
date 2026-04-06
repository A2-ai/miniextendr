//! Integration tests for ExternalPtr.

mod r_test_utils;

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::ffi::{Rf_install, SEXP};
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
    assert_eq!(ext.as_ref().copied(), Some(10));

    if let Some(val) = ext.as_mut() {
        *val = 42;
    }
    assert_eq!(ext.as_ref().copied(), Some(42));
}

fn test_tag_and_protected() {
    let ext = ExternalPtr::new(5i32);

    let tag = ext.tag();
    let expected = unsafe {
        let c_str = CString::new("i32").unwrap();
        Rf_install(c_str.as_ptr())
    };
    assert!(std::ptr::eq(tag.0, expected.0));

    let protected = unsafe { SEXP::scalar_integer(123) };
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
