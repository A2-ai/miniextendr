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
