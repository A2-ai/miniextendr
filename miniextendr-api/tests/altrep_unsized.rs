//! Unsized ALTREP views keep their metadata; ownership stays with a sized owner.
use miniextendr_api::SEXP;
use miniextendr_api::altrep_data::{AltrepExtract, AltrepLen};

// A custom trailing DST, not just the built-in slice type.
struct Tagged<T: ?Sized> {
    tag: u32,
    values: T,
}

impl AltrepLen for Tagged<[i32]> {
    fn len(&self) -> usize {
        self.values.len()
    }
}

// Custom storage can extract a DST without erasing its slice metadata.
impl AltrepExtract for Tagged<[i32]> {
    unsafe fn altrep_extract_ref(x: SEXP) -> &'static Self {
        unsafe { <Box<Tagged<[i32]>> as AltrepExtract>::altrep_extract_ref(x) }
    }

    unsafe fn altrep_extract_mut(x: SEXP) -> &'static mut Self {
        unsafe { <Box<Tagged<[i32]>> as AltrepExtract>::altrep_extract_mut(x) }
    }
}

impl miniextendr_api::externalptr::TypedExternal for Box<Tagged<[i32]>> {
    const TYPE_NAME: &'static str = "TaggedSlice";
    const TYPE_NAME_CSTR: &'static [u8] = b"TaggedSlice\0";
    const TYPE_ID_CSTR: &'static [u8] = b"tests::TaggedSlice\0";
}

#[test]
fn trailing_dst_implements_length_and_extraction_as_a_view() {
    let owner: Box<Tagged<[i32]>> = Box::new(Tagged {
        tag: 7,
        values: [1, 2, 3],
    });
    assert_eq!(owner.tag, 7);
    assert_eq!(AltrepLen::len(&*owner), 3);
    fn accepts_extraction_view<T: AltrepExtract + ?Sized>() {}
    accepts_extraction_view::<Tagged<[i32]>>();
}

mod r_test_utils;

#[test]
fn boxed_slice_marker_transfers_the_original_allocation_to_r() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::altrep_traits::{AltInteger, AltVec};
        use miniextendr_api::{Altrep, IntoR, OwnedProtect, SexpExt};

        let marker: Box<Altrep<[i32]>> = Box::new(Altrep([10, 20, 30]));
        let original = marker.as_ptr();
        let x = unsafe { OwnedProtect::new(marker.into_sexp()) };
        assert!(x.get().is_altrep());
        assert_eq!(
            <Box<[i32]> as AltVec>::dataptr_or_null(x.get()),
            original.cast()
        );
        let ptr = <Box<[i32]> as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *ptr.add(1) = 77 };
        assert_eq!(<Box<[i32]> as AltInteger>::elt(x.get(), 1), 77);
    });
}

#[test]
fn boxed_trailing_dst_keeps_metadata_and_drops_each_element_once() {
    use miniextendr_api::Altrep;
    use std::cell::Cell;
    use std::rc::Rc;

    struct CountDrop(Rc<Cell<usize>>);
    impl Drop for CountDrop {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let marker: Box<Altrep<Tagged<[CountDrop]>>> = Box::new(Altrep(Tagged {
        tag: 9,
        values: [CountDrop(drops.clone()), CountDrop(drops.clone())],
    }));
    let original = marker.values.as_ptr();
    let owner = marker.into_boxed_inner();
    assert_eq!(owner.tag, 9);
    assert_eq!(owner.values.len(), 2);
    assert_eq!(owner.values.as_ptr(), original);
    assert_eq!(drops.get(), 0);
    drop(owner);
    assert_eq!(drops.get(), 2);
}

#[test]
fn boxed_trait_object_marker_preserves_the_vtable() {
    use miniextendr_api::Altrep;
    let marker: Box<Altrep<dyn std::fmt::Display>> = Box::new(Altrep(42_i32));
    let owner = marker.into_boxed_inner();
    assert_eq!(owner.to_string(), "42");
}
