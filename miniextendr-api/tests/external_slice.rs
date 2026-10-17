//! ExternalSlice is usable from a consumer crate without a local newtype.

mod r_test_utils;

use miniextendr_api::{ExternalPtr, ExternalSlice, TypedExternal};
use std::cell::Cell;
use std::rc::Rc;

struct Entry {
    value: i32,
    dropped: Rc<Cell<usize>>,
}

impl Drop for Entry {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

#[test]
fn direct_storage_keeps_allocation_and_releases_non_send_elements_once() {
    r_test_utils::with_r_thread(|| {
        // Entry is local to this consumer, neither TypedExternal nor Send.
        let dropped = Rc::new(Cell::new(0));
        let mut values = Vec::with_capacity(8);
        for value in [1, 2, 3] {
            values.push(Entry {
                value,
                dropped: Rc::clone(&dropped),
            });
        }
        let allocation = values.as_ptr();
        let capacity = values.capacity();
        let mut owner = ExternalPtr::new(ExternalSlice::new(values));
        assert_eq!(owner.borrow().as_slice().as_ptr(), allocation);
        assert_eq!(owner.borrow().capacity(), capacity);
        owner.borrow_mut().as_mut_slice()[1].value = 20;

        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(dropped.get(), 0);
        assert_eq!(owner.borrow().as_slice()[1].value, 20);

        drop(owner);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(dropped.get(), 3);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(dropped.get(), 3);
    });
}

#[test]
fn shared_display_metadata_does_not_allow_a_different_element_type() {
    assert_eq!(ExternalSlice::<i32>::TYPE_NAME, "ExternalSlice");
    assert_eq!(ExternalSlice::<i32>::TYPE_NAME_CSTR, b"ExternalSlice\0");
    assert_eq!(
        ExternalSlice::<i32>::TYPE_ID_CSTR,
        ExternalSlice::<String>::TYPE_ID_CSTR
    );
    let id = std::ffi::CStr::from_bytes_with_nul(ExternalSlice::<i32>::TYPE_ID_CSTR)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(id.starts_with("miniextendr-api@"));
    assert!(id.ends_with("::externalptr::ExternalSlice"));

    r_test_utils::with_r_thread(|| {
        let owner = ExternalPtr::new(ExternalSlice::new(vec![1_i32, 2, 3]));
        let same = unsafe { ExternalPtr::<ExternalSlice<i32>>::wrap_sexp(owner.as_sexp()) }
            .expect("the same concrete type");
        assert_eq!(same.borrow().as_slice(), &[1, 2, 3]);
        let wrong = unsafe { ExternalPtr::<ExternalSlice<String>>::wrap_sexp(owner.as_sexp()) };
        assert!(wrong.is_none());
    });
}

#[test]
fn boxed_empty_and_zero_sized_slices_store_directly() {
    r_test_utils::with_r_thread(|| {
        let owner = ExternalPtr::new(ExternalSlice::from_boxed(
            vec![String::from("one"), String::from("two")].into_boxed_slice(),
        ));
        let slice = ExternalPtr::into_inner(owner);
        assert_eq!(slice.capacity(), 2);
        assert_eq!(slice.as_slice(), &["one", "two"]);

        let empty = ExternalPtr::new(ExternalSlice::<u8>::new(Vec::new()));
        assert!(empty.borrow().is_empty());
        assert_eq!(empty.borrow().as_slice(), &[]);
        let empty_boxed = ExternalPtr::new(ExternalSlice::<u8>::from_boxed(Box::new([])));
        assert!(empty_boxed.borrow().is_empty());

        let zst = ExternalPtr::new(ExternalSlice::from_boxed(Box::new([(); 3])));
        assert_eq!(zst.borrow().len(), 3);
        assert_eq!(ExternalPtr::into_inner(zst).as_slice(), &[(); 3]);
    });
}
