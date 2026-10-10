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

// Registration and ownership stay on the sized Box; its callbacks can expose
// the custom unsized view without changing ExternalPtr's storage representation.
impl AltrepLen for Box<Tagged<[i32]>> {
    fn len(&self) -> usize {
        AltrepLen::len(&**self)
    }
}
impl miniextendr_api::altrep_data::AltIntegerData for Box<Tagged<[i32]>> {
    fn elt(&self, i: usize) -> i32 {
        self.values[i]
    }
}
miniextendr_api::impl_altinteger_from_data!(Box<Tagged<[i32]>>);

impl miniextendr_api::altrep::RegisterAltrep for Box<Tagged<[i32]>> {
    fn get_or_init_class() -> miniextendr_api::sys::altrep::R_altrep_class_t {
        use miniextendr_api::altrep_data::InferBase;
        use std::sync::OnceLock;
        static CLASS: OnceLock<miniextendr_api::sys::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| unsafe {
            let class = Self::make_class(
                c"TestTaggedSlice".as_ptr(),
                miniextendr_api::AltrepPkgName::as_ptr(),
            );
            Self::install_methods(class);
            class
        })
    }
}

#[test]
fn custom_dst_owner_reaches_r_and_extracts_a_view_with_metadata() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::altrep_traits::AltInteger;
        use miniextendr_api::{Altrep, IntoR, OwnedProtect, SexpExt, TryFromSexp};

        let marker: Box<Altrep<Tagged<[i32]>>> = Box::new(Altrep(Tagged {
            tag: 11,
            values: [10, 20, 30],
        }));
        let original = marker.values.as_ptr();
        let x = OwnedProtect::new(marker.into_sexp());
        assert!(x.get().is_altrep());
        let view = Tagged::<[i32]>::altrep_extract_ref(x.get());
        assert_eq!(view.tag, 11);
        assert_eq!(view.values.as_ptr(), original);
        assert_eq!(view.values.len(), 3);
        Tagged::<[i32]>::altrep_extract_mut(x.get()).values[1] = 77;
        assert_eq!(<Box<Tagged<[i32]>> as AltInteger>::elt(x.get(), 1), 77);
        assert_eq!(Vec::<i32>::try_from_sexp(x.get()).unwrap(), [10, 77, 30]);
    });
}
