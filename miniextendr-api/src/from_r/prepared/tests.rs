use super::*;
use crate::SEXPTYPE;
use crate::from_r::SexpError;

#[test]
fn arms_pop_in_the_order_they_were_entered() {
    // A leaf under `Either<_, Either<leaf, _>>`: entered left (inner), then
    // right (outer); the outer arm comes out first.
    let arms = Arms::LEAF.under(false).unwrap().under(true).unwrap();
    let (right, rest) = arms.pop();
    assert!(right);
    let (right, rest) = rest.pop();
    assert!(!right);
    assert_eq!(rest, Arms::LEAF);
}

#[test]
fn arms_stop_at_sixty_four_levels() {
    let mut arms = Arms::LEAF;
    for level in 0..64 {
        arms = arms
            .under(level % 2 == 0)
            .expect("64 levels fit in the path");
    }
    assert_eq!(arms.under(true), None);
    // The level entered last comes out first.
    for level in (0..64).rev() {
        let (right, rest) = arms.pop();
        assert_eq!(right, level % 2 == 0);
        arms = rest;
    }
    assert_eq!(arms, Arms::LEAF);
}

#[test]
fn prepared_states_borrow_what_their_conversion_will() {
    let declared = Some(NativeBorrow::scalar(SEXPTYPE::INTSXP, true));
    let leaf = Some(NativeBorrow::slice(SEXPTYPE::REALSXP, false));
    assert_eq!(Prepared::<i32>::Unprepared.borrow(declared), declared);
    assert_eq!(Prepared::<i32>::Ready(1).borrow(declared), None);
    assert_eq!(
        Prepared::<i32>::Failed(SexpError::InvalidValue("x".into())).borrow(declared),
        None
    );
    let deferred = Prepared::<i32>::Deferred {
        borrow: leaf,
        arms: Arms::LEAF,
    };
    assert_eq!(deferred.borrow(declared), leaf);
    assert_eq!(Prepared::<i32>::Undecided(leaf).borrow(declared), leaf);
}

#[test]
fn an_arm_nests_its_path_and_gives_up_past_the_limit() {
    let either = Some(NativeBorrow::scalar(SEXPTYPE::INTSXP, true));
    let leaf = Some(NativeBorrow::scalar(SEXPTYPE::INTSXP, false));
    let deferred = Prepared::<&'static i32>::Deferred {
        borrow: leaf,
        arms: Arms::LEAF,
    };
    match deferred.arm::<Option<&'static i32>>(true, Some, either) {
        Prepared::Deferred { borrow, arms } => {
            assert_eq!(borrow, leaf);
            assert_eq!(arms.pop(), (true, Arms::LEAF));
        }
        _ => panic!("a deferred arm stays deferred"),
    }

    let mut full = Arms::LEAF;
    for _ in 0..64 {
        full = full.under(false).unwrap();
    }
    let deep = Prepared::<&'static i32>::Deferred {
        borrow: leaf,
        arms: full,
    };
    match deep.arm::<Option<&'static i32>>(false, Some, either) {
        Prepared::Undecided(borrow) => assert_eq!(borrow, either),
        _ => panic!("past 64 levels the arm is undecided"),
    }

    match Prepared::<i32>::Ready(3).arm::<Option<i32>>(false, Some, either) {
        Prepared::Ready(Some(3)) => {}
        _ => panic!("a ready arm wraps its value"),
    }
    match Prepared::<i32>::Undecided(leaf).arm::<Option<i32>>(false, Some, either) {
        Prepared::Undecided(borrow) => assert_eq!(borrow, leaf),
        _ => panic!("an undecided arm stays undecided"),
    }
}
