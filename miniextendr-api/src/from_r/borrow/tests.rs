use super::*;
use crate::from_r::TryFromSexp;

#[test]
fn native_borrow_metadata_follows_actual_converters() {
    macro_rules! check {
        ($t:ty, $sexptype:ident) => {{
            let shared = Some(NativeBorrow::scalar(SEXPTYPE::$sexptype, false));
            let mutable = Some(NativeBorrow::scalar(SEXPTYPE::$sexptype, true));
            assert_eq!(<&$t>::NATIVE_BORROW, shared);
            assert_eq!(<&mut $t>::NATIVE_BORROW, mutable);
            assert_eq!(<Option<&$t>>::NATIVE_BORROW, shared);
            assert_eq!(<Option<&mut $t>>::NATIVE_BORROW, mutable);
            assert_eq!(
                <Vec<&mut $t>>::NATIVE_BORROW,
                NativeBorrow::in_list(mutable)
            );
            assert_eq!(
                <Box<[Option<&mut $t>]>>::NATIVE_BORROW,
                NativeBorrow::in_list(mutable)
            );
            assert_eq!(
                <Option<Vec<Vec<&mut $t>>>>::NATIVE_BORROW,
                NativeBorrow::in_list(NativeBorrow::in_list(mutable))
            );
            let slice = Some(NativeBorrow::slice(SEXPTYPE::$sexptype, true));
            assert_eq!(<&mut [$t]>::NATIVE_BORROW, slice);
            assert_eq!(
                <Box<[Option<&mut [$t]>]>>::NATIVE_BORROW,
                NativeBorrow::in_list(slice)
            );
            assert_eq!(<Vec<$t>>::NATIVE_BORROW, None);
            assert_eq!(<Box<[$t]>>::NATIVE_BORROW, None);
        }};
    }
    check!(i32, INTSXP);
    check!(f64, REALSXP);
    check!(u8, RAWSXP);
    check!(crate::RLogical, LGLSXP);
    check!(crate::Rcomplex, CPLXSXP);
    type HiddenBorrow = Option<Vec<Vec<&'static mut i32>>>;
    assert!(HiddenBorrow::NATIVE_BORROW.is_some());
}

#[test]
fn forwarding_wrappers_preserve_native_borrow_metadata() {
    type Leaf = &'static mut i32;
    let scalar = Leaf::NATIVE_BORROW;
    let list = NativeBorrow::in_list(scalar);
    assert_eq!(<Result<Leaf, ()>>::NATIVE_BORROW, scalar);
    assert_eq!(<crate::Missing<Leaf>>::NATIVE_BORROW, scalar);
    assert_eq!(
        <std::collections::HashMap<String, Leaf>>::NATIVE_BORROW,
        list
    );
    assert_eq!(
        <Option<std::collections::BTreeMap<String, Leaf>>>::NATIVE_BORROW,
        list
    );
    assert_eq!(
        <Vec<std::collections::HashMap<String, Leaf>>>::NATIVE_BORROW,
        NativeBorrow::in_list(list)
    );
    assert_eq!(
        <std::borrow::Cow<'static, [i32]> as TryFromSexp>::NATIVE_BORROW,
        None
    );
    assert_eq!(
        <std::borrow::Cow<'_, [i32]> as crate::TryFromRoot<'_>>::NATIVE_BORROW,
        <&[i32]>::NATIVE_BORROW
    );
    assert_eq!(
        <crate::RCow<'static, i32>>::NATIVE_BORROW,
        <&[i32]>::NATIVE_BORROW
    );
    #[cfg(feature = "indexmap")]
    assert_eq!(<indexmap::IndexMap<String, Leaf>>::NATIVE_BORROW, list);
}

#[test]
fn custom_reference_conversions_default_to_no_native_borrow() {
    struct RLogical;
    impl TryFromSexp for &RLogical {
        type Error = ();
        fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
            unreachable!()
        }
    }
    assert_eq!(<&RLogical>::NATIVE_BORROW, None);
    assert_eq!(<String>::NATIVE_BORROW, None);
}

#[test]
fn metadata_avoids_unneeded_walks_and_preserves_length_rules() {
    let shared = NativeBorrow::scalar(SEXPTYPE::INTSXP, false);
    let mutable = NativeBorrow::scalar(SEXPTYPE::INTSXP, true);
    assert!(!needs_check(&[]));
    assert!(!needs_check(&[None, Some(shared), Some(shared)]));
    assert!(!needs_check(&[Some(mutable)]));
    assert!(!needs_check(&[
        Some(mutable),
        Some(NativeBorrow::slice(SEXPTYPE::REALSXP, true))
    ]));
    assert!(needs_check(&[Some(mutable), Some(shared)]));
    assert!(needs_check(&[NativeBorrow::in_list(Some(mutable))]));
    assert!(mutable.accepts_length(1));
    assert!(!mutable.accepts_length(0));
    assert!(!mutable.accepts_length(2));
    assert!(NativeBorrow::slice(SEXPTYPE::INTSXP, true).accepts_length(2));
    assert!(!NativeBorrow::slice(SEXPTYPE::INTSXP, true).accepts_length(0));
}

#[test]
fn tuple_metadata_keeps_field_shapes_and_owned_positions() {
    type Pair = (&'static mut i32, &'static i32);
    assert_eq!(
        Pair::NATIVE_BORROW,
        NativeBorrow::tuple(&[<&mut i32>::NATIVE_BORROW, <&i32>::NATIVE_BORROW])
    );
    assert!(needs_check(&[Pair::NATIVE_BORROW]));
    assert_eq!(<(i32, String)>::NATIVE_BORROW, None);
    assert!(!needs_check(&[<(i32, &mut i32)>::NATIVE_BORROW]));
    assert!(!needs_check(&[<(&i32, &i32)>::NATIVE_BORROW]));
    assert!(!needs_check(&[<(&mut i32, &f64)>::NATIVE_BORROW]));
    assert!(needs_check(&[<((i32, &mut i32), (&i32,))>::NATIVE_BORROW]));
    assert!(needs_check(&[
        <(&mut [i32], String)>::NATIVE_BORROW,
        <&[i32]>::NATIVE_BORROW,
    ]));
    assert!(needs_check(&[<(Vec<&mut i32>,)>::NATIVE_BORROW,]));
    assert!(needs_check(&[NativeBorrow::in_list(Pair::NATIVE_BORROW)]));
}

#[cfg(feature = "either")]
#[test]
fn either_metadata_takes_both_arms_and_marks_the_top_level() {
    use crate::either_impl::Either;
    type MutLeft = Either<&'static mut i32, String>;
    let mutable = <&mut i32>::NATIVE_BORROW;
    assert_eq!(
        MutLeft::NATIVE_BORROW,
        NativeBorrow::either(&[<&mut i32>::NATIVE_BORROW, None])
    );
    assert_eq!(
        <Either<String, &'static mut i32>>::NATIVE_BORROW,
        NativeBorrow::either(&[None, <&mut i32>::NATIVE_BORROW])
    );
    // Neither arm borrows: nothing to check.
    assert_eq!(<Either<i32, String>>::NATIVE_BORROW, None);
    assert!(NativeBorrow::selects(MutLeft::NATIVE_BORROW));
    // The wrappers that prepare as the `Either` keep its top level.
    assert!(NativeBorrow::selects(<Option<MutLeft>>::NATIVE_BORROW));
    assert!(NativeBorrow::selects(
        <crate::Missing<MutLeft>>::NATIVE_BORROW
    ));
    assert!(NativeBorrow::selects(<Result<MutLeft, ()>>::NATIVE_BORROW));
    assert!(NativeBorrow::selects(
        <crate::Nullable<MutLeft>>::NATIVE_BORROW
    ));
    // Inside a list or tuple the check takes both arms.
    assert!(!NativeBorrow::selects(<(MutLeft,)>::NATIVE_BORROW));
    assert!(!NativeBorrow::selects(
        <std::collections::HashMap<String, MutLeft>>::NATIVE_BORROW
    ));
    assert!(!NativeBorrow::selects(mutable));
    assert!(!NativeBorrow::selects(None));

    // The union meets another argument's leaves.
    assert!(needs_check(&[
        MutLeft::NATIVE_BORROW,
        <&i32>::NATIVE_BORROW
    ]));
    assert!(needs_check(&[
        <Either<i32, &'static mut [f64]>>::NATIVE_BORROW,
        <&[f64]>::NATIVE_BORROW,
    ]));
    // Only one arm converts: two mutable arms never borrow together.
    assert!(!needs_check(&[
        <Either<&'static mut i32, &'static mut i32>>::NATIVE_BORROW
    ]));
    // Shared arms next to shared arguments never conflict.
    assert!(!needs_check(&[
        <Either<&'static i32, String>>::NATIVE_BORROW,
        <&i32>::NATIVE_BORROW,
    ]));
    // Other storage types never conflict.
    assert!(!needs_check(&[
        MutLeft::NATIVE_BORROW,
        <&f64>::NATIVE_BORROW
    ]));
    // A nested `Either` keeps every arm.
    type Nested = Either<String, Either<i32, &'static mut i32>>;
    assert!(NativeBorrow::selects(Nested::NATIVE_BORROW));
    assert!(needs_check(&[Nested::NATIVE_BORROW, <&i32>::NATIVE_BORROW]));
    // An `Either` field of a tuple still meets the tuple's other fields.
    assert!(needs_check(&[<(MutLeft, &'static i32)>::NATIVE_BORROW]));
}

#[test]
fn choice_metadata_is_the_other_arm_for_non_character_input() {
    static RIGHT: Option<NativeBorrow> = Some(NativeBorrow::slice(SEXPTYPE::REALSXP, true));
    static OWNED: Option<NativeBorrow> = None;
    let choice = NativeBorrow::choice_or(&RIGHT);
    assert!(choice.is_some());
    assert_eq!(NativeBorrow::choice_or(&OWNED), None);
    // The input's type decides the arm: nothing to prepare.
    assert!(!NativeBorrow::selects(choice));
    assert!(needs_check(&[choice, <&[f64]>::NATIVE_BORROW]));
    assert!(!needs_check(&[choice, <&[i32]>::NATIVE_BORROW]));
    assert!(!needs_check(&[choice]));
}
