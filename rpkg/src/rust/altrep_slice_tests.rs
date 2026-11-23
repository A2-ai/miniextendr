//! ALTREP vectors read from a slice field (#2363), for `test-altrep-slice.R`.
//!
//! `#[derive(Altrep)]` picks the R vector from the element type; the family
//! derives (`AltrepInteger`, `AltrepLogical`, ...) convert each element with
//! `Coerce`. Each fixture returns the ALTREP and the eager conversion of the
//! same elements, which the tests compare with `identical()`.

use miniextendr_api::altrep_registration::RegisterAltrep;
use miniextendr_api::{
    Altrep, AltrepInteger, AltrepLogical, AltrepReal, AltrepString, Coerce, FactorVec, MatchArg,
    RFactor, RNativeType, Rboolean, SEXP, SexpExt, miniextendr,
};

// region: Elements

/// A die's face: a factor under `#[derive(Altrep)]`, the integers 1 to 6
/// under `#[derive(AltrepInteger)]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, RFactor)]
pub enum Die {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Die {
    /// The face's pips.
    pub fn value(self) -> u16 {
        match self {
            Die::One => 1,
            Die::Two => 2,
            Die::Three => 3,
            Die::Four => 4,
            Die::Five => 5,
            Die::Six => 6,
        }
    }
}

impl Coerce<i32> for Die {
    fn coerce(self) -> i32 {
        self.value().into()
    }
}

/// A game's outcome: a factor under `#[derive(Altrep)]`, `TRUE` for a win
/// under `#[derive(AltrepLogical)]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, RFactor)]
pub enum Outcome {
    Loss,
    Win,
}

impl Coerce<Rboolean> for Outcome {
    fn coerce(self) -> Rboolean {
        match self {
            Outcome::Loss => Rboolean::FALSE,
            Outcome::Win => Rboolean::TRUE,
        }
    }
}

/// A speed: a character vector of the choice names under `#[derive(Altrep)]`.
#[derive(Clone, Copy, Debug, PartialEq, MatchArg)]
pub enum Pace {
    Slow,
    Fast,
}

/// Grams, stored as R doubles: R reads a slice of them in place.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, RNativeType)]
pub struct Grams(f64);

const DICE_3: [Die; 3] = [Die::Two, Die::Six, Die::One];
const DICE_5: [Die; 5] = [Die::Five, Die::Five, Die::Three, Die::Four, Die::One];
const OUTCOMES_3: [Outcome; 3] = [Outcome::Win, Outcome::Loss, Outcome::Win];
const OUTCOMES_5: [Outcome; 5] = [
    Outcome::Loss,
    Outcome::Loss,
    Outcome::Win,
    Outcome::Loss,
    Outcome::Win,
];
static PRIMES: [i32; 5] = [2, 3, 5, 7, 11];
// endregion

// region: Owners

/// Dice as an R factor: no attributes needed.
#[derive(Altrep)]
pub struct Dice<const N: usize> {
    value: [Die; N],
    _secret: (),
}

/// Outcomes as an R factor, from a tuple struct.
#[derive(Altrep)]
pub struct Outcomes<const N: usize>([Outcome; N]);

/// Dice as the integers 1 to 6, through `Coerce<i32>`.
#[derive(AltrepInteger)]
pub struct DiceValues<const N: usize> {
    value: [Die; N],
    _secret: (),
}

/// Outcomes as `TRUE` / `FALSE`, through `Coerce<Rboolean>`.
#[derive(AltrepLogical)]
pub struct Wins<const N: usize>([Outcome; N]);

/// An array of any element with an R form.
#[derive(Altrep)]
pub struct SliceArray<T, const N: usize>([T; N]);

/// A `Vec` of any element with an R form.
#[derive(Altrep)]
pub struct SliceVec<T>(Vec<T>);

/// A boxed slice of doubles.
#[derive(Altrep)]
pub struct SliceBox(Box<[f64]>);

/// Borrowed `'static` data: R reads it in place and copies it to write.
#[derive(Altrep)]
#[altrep(class = "SlicePrimes")]
pub struct SliceStatic(&'static [i32]);

/// Several slice fields: `field` picks one.
#[derive(Altrep)]
#[altrep(field = "names")]
pub struct SliceNamed {
    names: Vec<String>,
    _ids: Vec<i32>,
}

/// Family derive on native elements: R reads them in place.
#[derive(AltrepReal)]
pub struct RealSlice(Vec<f64>);

/// Family derive on string elements, borrowed.
#[derive(AltrepString)]
pub struct WordSlice(&'static [&'static str]);

/// A concrete slice owner keeps `serialize`: R saves the faces and reads
/// the ALTREP back.
#[derive(AltrepInteger)]
#[altrep(serialize, class = "SavedDiceValues")]
pub struct SavedDiceValues([Die; 4]);

impl miniextendr_api::altrep_data::AltrepSerialize for SavedDiceValues {
    fn serialized_state(&self) -> SEXP {
        use miniextendr_api::IntoR;
        self.0
            .iter()
            .map(|die| Coerce::<i32>::coerce(*die))
            .collect::<Vec<i32>>()
            .into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        let values: Vec<i32> = miniextendr_api::TryFromSexp::try_from_sexp(state).ok()?;
        let faces: Vec<Die> = values
            .into_iter()
            .map(Die::from_level_index)
            .collect::<Option<_>>()?;
        Some(Self(faces.try_into().ok()?))
    }
}
// endregion

// region: Fixtures

/// Three dice as a slice ALTREP factor, and the same dice as an eager factor.
#[miniextendr(noexport)]
pub fn altrep_slice_dice3() -> (Dice<3>, FactorVec<Die>) {
    (
        Dice {
            value: DICE_3,
            _secret: (),
        },
        FactorVec(DICE_3.to_vec()),
    )
}

/// Five dice as a slice ALTREP factor, and the same dice as an eager factor.
#[miniextendr(noexport)]
pub fn altrep_slice_dice5() -> (Dice<5>, FactorVec<Die>) {
    (
        Dice {
            value: DICE_5,
            _secret: (),
        },
        FactorVec(DICE_5.to_vec()),
    )
}

/// Three outcomes as a slice ALTREP factor, and as an eager factor.
#[miniextendr(noexport)]
pub fn altrep_slice_outcomes3() -> (Outcomes<3>, FactorVec<Outcome>) {
    (Outcomes(OUTCOMES_3), FactorVec(OUTCOMES_3.to_vec()))
}

/// Five outcomes as a slice ALTREP factor, and as an eager factor.
#[miniextendr(noexport)]
pub fn altrep_slice_outcomes5() -> (Outcomes<5>, FactorVec<Outcome>) {
    (Outcomes(OUTCOMES_5), FactorVec(OUTCOMES_5.to_vec()))
}

/// Three dice as an ALTREP integer vector of their pips, and the pips.
#[miniextendr(noexport)]
pub fn altrep_slice_dice_values3() -> (DiceValues<3>, Vec<i32>) {
    (
        DiceValues {
            value: DICE_3,
            _secret: (),
        },
        DICE_3.iter().map(|die| i32::from(die.value())).collect(),
    )
}

/// Five dice as an ALTREP integer vector of their pips, and the pips.
#[miniextendr(noexport)]
pub fn altrep_slice_dice_values5() -> (DiceValues<5>, Vec<i32>) {
    (
        DiceValues {
            value: DICE_5,
            _secret: (),
        },
        DICE_5.iter().map(|die| i32::from(die.value())).collect(),
    )
}

/// Three outcomes as an ALTREP logical vector of wins, and the wins.
#[miniextendr(noexport)]
pub fn altrep_slice_wins3() -> (Wins<3>, Vec<bool>) {
    (
        Wins(OUTCOMES_3),
        OUTCOMES_3.iter().map(|o| *o == Outcome::Win).collect(),
    )
}

/// Five outcomes as an ALTREP logical vector of wins, and the wins.
#[miniextendr(noexport)]
pub fn altrep_slice_wins5() -> (Wins<5>, Vec<bool>) {
    (
        Wins(OUTCOMES_5),
        OUTCOMES_5.iter().map(|o| *o == Outcome::Win).collect(),
    )
}

/// Whether each dice vector has its own instantiation's class and not the
/// other's: `c(dice3 is Dice<3>, dice5 is Dice<5>, dice3 is Dice<5>, dice5 is Dice<3>)`.
#[miniextendr(noexport)]
pub fn altrep_slice_dice_classes(dice3: SEXP, dice5: SEXP) -> Vec<bool> {
    let three = <Dice<3> as RegisterAltrep>::get_or_init_class();
    let five = <Dice<5> as RegisterAltrep>::get_or_init_class();
    // SAFETY: R's main thread; both are live R values.
    unsafe {
        vec![
            three.inherits(dice3),
            five.inherits(dice5),
            three.inherits(dice5),
            five.inherits(dice3),
        ]
    }
}

/// Each array owner of [`altrep_slice_arrays`] next to its eager value.
type ArraysAndValues = (
    SliceArray<i32, 3>,
    Vec<i32>,
    SliceArray<f64, 3>,
    Vec<f64>,
    SliceArray<bool, 3>,
    Vec<bool>,
    SliceArray<String, 2>,
    Vec<String>,
);

/// Each owner of [`altrep_slice_owned`] next to its eager value.
type OwnedAndValues = (
    SliceVec<i32>,
    Vec<i32>,
    SliceVec<f64>,
    Vec<f64>,
    SliceBox,
    Vec<f64>,
    SliceVec<Pace>,
    Vec<Pace>,
    SliceVec<Grams>,
    Vec<f64>,
);

/// Native, logical and character arrays, with their eager values.
#[miniextendr(noexport)]
pub fn altrep_slice_arrays() -> ArraysAndValues {
    let strings = [String::from("one"), String::from("two")];
    (
        SliceArray([1, 2, 3]),
        vec![1, 2, 3],
        SliceArray([0.5, 1.5, 2.5]),
        vec![0.5, 1.5, 2.5],
        SliceArray([true, false, true]),
        vec![true, false, true],
        SliceArray(strings.clone()),
        strings.to_vec(),
    )
}

/// `Vec` owners of two instantiations, a boxed slice, choice names and a
/// native newtype, with their eager values.
#[miniextendr(noexport)]
pub fn altrep_slice_owned() -> OwnedAndValues {
    (
        SliceVec(vec![4, 5]),
        vec![4, 5],
        SliceVec(vec![4.5, 5.5]),
        vec![4.5, 5.5],
        SliceBox(Box::from([7.0, 8.0])),
        vec![7.0, 8.0],
        SliceVec(vec![Pace::Fast, Pace::Slow, Pace::Fast]),
        vec![Pace::Fast, Pace::Slow, Pace::Fast],
        SliceVec(vec![Grams(1.25), Grams(2.5)]),
        vec![1.25, 2.5],
    )
}

/// The primes as a slice ALTREP over `'static` data.
#[miniextendr(noexport)]
pub fn altrep_slice_primes() -> SliceStatic {
    SliceStatic(&PRIMES)
}

/// The `'static` primes themselves, which R writes must leave alone.
#[miniextendr(noexport)]
pub fn altrep_slice_primes_source() -> Vec<i32> {
    PRIMES.to_vec()
}

/// The field `field = "names"` picks, a family derive over native doubles,
/// and one over borrowed strings.
#[miniextendr(noexport)]
pub fn altrep_slice_selected() -> (SliceNamed, RealSlice, WordSlice) {
    (
        SliceNamed {
            names: vec![String::from("a"), String::from("b")],
            _ids: vec![1, 2, 3],
        },
        RealSlice(vec![0.25, 0.75]),
        WordSlice(&["red", "green"]),
    )
}

/// Whether R reads a native `Vec` in place: the data pointer R gets is the
/// `Vec`'s buffer, for the bare derive and the family derive alike.
#[miniextendr(noexport)]
pub fn altrep_slice_reads_in_place() -> Vec<bool> {
    use miniextendr_api::IntoR;

    let ints = vec![1, 2, 3];
    let ints_ptr = ints.as_ptr();
    let reals = vec![0.5, 1.5];
    let reals_ptr = reals.as_ptr();
    // SAFETY: R's main thread; both vectors stay protected while R hands
    // out their data pointers.
    unsafe {
        let ints = miniextendr_api::OwnedProtect::new(SliceVec(ints).into_sexp());
        let reals = miniextendr_api::OwnedProtect::new(RealSlice(reals).into_sexp());
        vec![
            ints.get().as_slice::<i32>().as_ptr() == ints_ptr,
            reals.get().as_slice::<f64>().as_ptr() == reals_ptr,
        ]
    }
}

/// Four dice as a serializing ALTREP integer vector.
#[miniextendr(noexport)]
pub fn altrep_slice_saved_dice() -> SavedDiceValues {
    SavedDiceValues([Die::One, Die::Six, Die::Two, Die::Two])
}
// endregion

// region: gctorture fixture

/// Make slice ALTREP factors of two instantiations, read their codes, set
/// attributes and copy them, so a GC between the ALTREP's allocation and its
/// `levels` / `class` attributes would show.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_altrep_slice_factor() {
    use miniextendr_api::IntoR;

    for _ in 0..4 {
        // SAFETY: R's main thread; each protect is released at the end of
        // the iteration, in reverse order.
        let dice = unsafe {
            miniextendr_api::OwnedProtect::new(
                Dice {
                    value: DICE_5,
                    _secret: (),
                }
                .into_sexp(),
            )
        };
        let outcomes =
            unsafe { miniextendr_api::OwnedProtect::new(Outcomes(OUTCOMES_3).into_sexp()) };
        let codes: Vec<i32> = (0..5).map(|i| dice.get().integer_elt(i)).collect();
        assert_eq!(codes, [5, 5, 3, 4, 1]);
        assert_eq!(dice.get().get_levels().len(), 6);
        assert_eq!(outcomes.get().get_levels().len(), 2);
        assert!(dice.get().inherits_class(c"factor"));
        assert!(outcomes.get().inherits_class(c"factor"));
    }
}
// endregion
