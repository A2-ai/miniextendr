use super::*;

macro_rules! owner {
    ($name:ident, $elem:ty) => {
        struct $name(Vec<$elem>);

        impl TypedExternal for $name {
            const TYPE_NAME: &'static str = stringify!($name);
            const TYPE_NAME_CSTR: &'static [u8] = concat!(stringify!($name), "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] =
                concat!("slice_tests::", stringify!($name), "\0").as_bytes();
        }

        impl AltrepSlice for $name {
            type Element = $elem;

            fn altrep_slice(&self) -> &[$elem] {
                &self.0
            }

            fn altrep_slice_mut(&mut self) -> Option<&mut [$elem]> {
                Some(&mut self.0)
            }
        }
    };
}

owner!(Ints, i32);
owner!(Reals, f64);
owner!(Flags, bool);
owner!(RFlags, RLogical);
owner!(Bytes, u8);
owner!(Complexes, Rcomplex);
owner!(Strings, String);

/// Borrowed data: R reads it, never writes it.
struct Borrowed(&'static [i32]);

impl TypedExternal for Borrowed {
    const TYPE_NAME: &'static str = "Borrowed";
    const TYPE_NAME_CSTR: &'static [u8] = b"Borrowed\0";
    const TYPE_ID_CSTR: &'static [u8] = b"slice_tests::Borrowed\0";
}

impl AltrepSlice for Borrowed {
    type Element = i32;

    fn altrep_slice(&self) -> &[i32] {
        self.0
    }

    fn altrep_slice_mut(&mut self) -> Option<&mut [i32]> {
        None
    }
}

#[test]
fn element_family_picks_the_base() {
    assert_eq!(<Ints as InferBase>::BASE, RBase::Int);
    assert_eq!(<Reals as InferBase>::BASE, RBase::Real);
    assert_eq!(<Flags as InferBase>::BASE, RBase::Logical);
    assert_eq!(<RFlags as InferBase>::BASE, RBase::Logical);
    assert_eq!(<Bytes as InferBase>::BASE, RBase::Raw);
    assert_eq!(<Complexes as InferBase>::BASE, RBase::Complex);
    assert_eq!(<Strings as InferBase>::BASE, RBase::String);
    assert_eq!(<Borrowed as InferBase>::BASE, RBase::Int);
}

#[test]
fn elements_read_through_their_family() {
    let ints = Ints(vec![3, 1, 2]);
    assert_eq!(ints.altrep_len(), 3);
    assert_eq!(AltIntegerData::elt(&ints, 1), 1);

    let flags = Flags(vec![true, false]);
    assert_eq!(AltLogicalData::elt(&flags, 0), Logical::True);
    assert_eq!(AltLogicalData::elt(&flags, 1), Logical::False);

    let rflags = RFlags(vec![RLogical::TRUE, RLogical::NA]);
    assert_eq!(AltLogicalData::elt(&rflags, 1), Logical::Na);

    let strings = Strings(vec!["a".to_owned(), "b".to_owned()]);
    assert_eq!(AltStringData::elt(&strings, 1), Some("b"));

    let borrowed = Borrowed(&[7, 8]);
    assert_eq!(borrowed.altrep_len(), 2);
    assert_eq!(AltIntegerData::elt(&borrowed, 0), 7);
}

#[test]
fn native_elements_expose_their_storage() {
    let ints = Ints(vec![1, 2, 3]);
    assert_eq!(AltIntegerData::as_slice(&ints), Some(&[1, 2, 3][..]));

    let rflags = RFlags(vec![RLogical::TRUE, RLogical::FALSE]);
    assert_eq!(AltLogicalData::as_r_slice(&rflags), Some(&[1, 0][..]));

    // `bool` is one byte: R gets a copy, never the Rust slice.
    let flags = Flags(vec![true]);
    assert_eq!(AltLogicalData::as_r_slice(&flags), None);
}

#[test]
fn native_view_matches_only_the_same_type() {
    let ints = [1i32, 2];
    assert_eq!(native_view::<i32, i32>(&ints), Some(&ints[..]));
    let shorts = [1u16, 2];
    assert_eq!(native_view::<u16, i32>(&shorts), None);

    let mut reals = [1.0f64, 2.0];
    native_view_mut::<f64, f64>(&mut reals).unwrap()[0] = 5.0;
    assert_eq!(reals, [5.0, 2.0]);
}

#[test]
fn rboolean_reads_as_a_logical() {
    assert_eq!(Logical::from(crate::Rboolean::TRUE), Logical::True);
    assert_eq!(Logical::from(crate::Rboolean::FALSE), Logical::False);
}
