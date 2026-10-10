//! Unsized ALTREP views retain metadata and use only receiver-level operations.
use miniextendr_api::SEXP;
use miniextendr_api::altrep_data::{AltrepExtract, AltrepLen, AltrepSerialize};

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

impl AltrepSerialize for Tagged<[i32]> {
    fn serialized_state(&self) -> SEXP {
        use miniextendr_api::IntoR;
        self.values.as_ref().into_sexp()
    }
}

// Custom storage can extract a DST without erasing its slice metadata.
impl AltrepExtract for Tagged<[i32]> {
    unsafe fn altrep_extract_ref(x: SEXP) -> &'static Self {
        let owner = unsafe { <Box<Tagged<[i32]>> as AltrepExtract>::altrep_extract_ref(x) };
        owner
    }

    unsafe fn altrep_extract_mut(x: SEXP) -> &'static mut Self {
        let owner = unsafe { <Box<Tagged<[i32]>> as AltrepExtract>::altrep_extract_mut(x) };
        owner
    }
}

impl miniextendr_api::externalptr::TypedExternal for Box<Tagged<[i32]>> {
    const TYPE_NAME: &'static str = "TaggedSlice";
    const TYPE_NAME_CSTR: &'static [u8] = b"TaggedSlice\0";
    const TYPE_ID_CSTR: &'static [u8] = b"tests::TaggedSlice\0";
}

#[test]
fn trailing_dst_supports_length_and_save_side_without_an_owned_factory() {
    let owner: Box<Tagged<[i32]>> = Box::new(Tagged {
        tag: 7,
        values: [1, 2, 3],
    });
    assert_eq!(owner.tag, 7);
    assert_eq!(AltrepLen::len(&*owner), 3);
    fn accepts_serialization_view<T: AltrepSerialize + ?Sized>(_: &T) {}
    accepts_serialization_view(&*owner);
    fn accepts_extraction_view<T: AltrepExtract + ?Sized>() {}
    accepts_extraction_view::<Tagged<[i32]>>();
}
