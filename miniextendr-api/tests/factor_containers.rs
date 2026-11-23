//! The containers of a `#[derive(RFactor)]` enum convert as the enum does
//! (#2365): `Vec<T>`, `Box<[T]>`, `Option<Vec<T>>` and `Vec<Option<T>>`
//! become a factor of the type's levels (`NA` for `None`), and `Vec<T>`,
//! `Box<[T]>`, `Option<T>` and `Vec<Option<T>>` read one. A plain `MatchArg`
//! enum's `Vec<T>` stays a character vector.

mod r_test_utils;

use miniextendr_api::factor::build_factor_with_levels;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, MatchArg, RFactor, RNativeType, SEXP, SexpExt, TryFromSexp};

#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
enum Color {
    Red,
    Green,
    Blue,
}

/// Interaction factor: levels `Small.Red`, `Small.Green`, ... (outer slowest).
#[derive(Copy, Clone, Debug, PartialEq, RFactor)]
#[r_factor(interaction = ["Red", "Green", "Blue"])]
enum SizedColor {
    Small(Color),
    Large(Color),
}

#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
enum Mode {
    Fast,
    Safe,
}

/// The factor's codes and its levels.
fn codes_and_levels(sexp: SEXP) -> (Vec<i32>, Vec<String>) {
    assert!(sexp.is_factor(), "not a factor");
    let codes = unsafe { sexp.as_slice::<i32>() }.to_vec();
    let levels = sexp.get_levels();
    let levels = (0..levels.len())
        .map(|i| levels.string_elt_str(i as isize).unwrap().to_string())
        .collect();
    (codes, levels)
}

fn color_levels() -> Vec<String> {
    ["Red", "Green", "Blue"].map(String::from).to_vec()
}

#[test]
fn vec_and_boxed_slice_are_factors() {
    r_test_utils::with_r_thread(|| {
        let values = vec![Color::Blue, Color::Red, Color::Blue];
        let expected = (vec![3, 1, 3], color_levels());

        let vec = unsafe { OwnedProtect::new(values.clone().into_sexp()) };
        assert_eq!(codes_and_levels(vec.get()), expected);

        let boxed = unsafe { OwnedProtect::new(values.clone().into_boxed_slice().into_sexp()) };
        assert_eq!(codes_and_levels(boxed.get()), expected);

        let some = unsafe { OwnedProtect::new(Some(values).into_sexp()) };
        assert_eq!(codes_and_levels(some.get()), expected);

        // An empty vector keeps every level.
        let empty = unsafe { OwnedProtect::new(Vec::<Color>::new().into_sexp()) };
        assert_eq!(codes_and_levels(empty.get()), (vec![], color_levels()));
    });
}

#[test]
fn vec_of_options_is_a_factor_with_na() {
    r_test_utils::with_r_thread(|| {
        let values = vec![Some(Color::Green), None, Some(Color::Red)];
        let factor = unsafe { OwnedProtect::new(values.into_sexp()) };
        assert_eq!(
            codes_and_levels(factor.get()),
            (vec![2, i32::R_NA, 1], color_levels())
        );
    });
}

#[test]
fn interaction_vec_is_a_factor_of_the_combined_levels() {
    r_test_utils::with_r_thread(|| {
        let values = vec![
            SizedColor::Large(Color::Green),
            SizedColor::Small(Color::Blue),
        ];
        let factor = unsafe { OwnedProtect::new(values.clone().into_sexp()) };
        let (codes, levels) = codes_and_levels(factor.get());
        assert_eq!(codes, vec![5, 3]);
        assert_eq!(levels, SizedColor::CHOICES);

        let back = Vec::<SizedColor>::try_from_sexp(factor.get()).unwrap();
        assert_eq!(back, values);
    });
}

#[test]
fn containers_read_a_factor() {
    r_test_utils::with_r_thread(|| {
        let factor = unsafe {
            OwnedProtect::new(build_factor_with_levels(
                &[2, 3, 1],
                &["Red", "Green", "Blue"],
            ))
        };
        let expected = vec![Color::Green, Color::Blue, Color::Red];
        assert_eq!(Vec::<Color>::try_from_sexp(factor.get()).unwrap(), expected);
        assert_eq!(
            Box::<[Color]>::try_from_sexp(factor.get()).unwrap(),
            expected.clone().into_boxed_slice()
        );
        assert_eq!(
            Vec::<Option<Color>>::try_from_sexp(factor.get()).unwrap(),
            expected.into_iter().map(Some).collect::<Vec<_>>()
        );

        // What a container writes reads back unchanged.
        let values = vec![Some(Color::Blue), None];
        let written = unsafe { OwnedProtect::new(values.clone().into_sexp()) };
        assert_eq!(
            Vec::<Option<Color>>::try_from_sexp(written.get()).unwrap(),
            values
        );
    });
}

#[test]
fn option_reads_a_scalar_factor_and_na() {
    r_test_utils::with_r_thread(|| {
        let blue = unsafe { OwnedProtect::new(Color::Blue.into_sexp()) };
        assert_eq!(
            Option::<Color>::try_from_sexp(blue.get()).unwrap(),
            Some(Color::Blue)
        );

        let na = unsafe {
            OwnedProtect::new(build_factor_with_levels(
                &[i32::R_NA],
                &["Red", "Green", "Blue"],
            ))
        };
        assert_eq!(Option::<Color>::try_from_sexp(na.get()).unwrap(), None);
        assert_eq!(Option::<Color>::try_from_sexp(SEXP::nil()).unwrap(), None);
        let plain_na = unsafe { OwnedProtect::new(Option::<bool>::None.into_sexp()) };
        assert_eq!(
            Option::<Color>::try_from_sexp(plain_na.get()).unwrap(),
            None
        );

        let two = unsafe {
            OwnedProtect::new(build_factor_with_levels(&[1, 2], &["Red", "Green", "Blue"]))
        };
        assert!(Option::<Color>::try_from_sexp(two.get()).is_err());
    });
}

#[test]
fn containers_refuse_what_the_scalar_refuses() {
    r_test_utils::with_r_thread(|| {
        // Not a factor.
        let text = unsafe { OwnedProtect::new(vec!["Red".to_string()].into_sexp()) };
        assert!(Vec::<Color>::try_from_sexp(text.get()).is_err());
        assert!(Vec::<Option<Color>>::try_from_sexp(text.get()).is_err());
        assert!(Option::<Color>::try_from_sexp(text.get()).is_err());

        // Other levels.
        let other = unsafe { OwnedProtect::new(build_factor_with_levels(&[1], &["Red", "Green"])) };
        assert!(Vec::<Color>::try_from_sexp(other.get()).is_err());

        // `NA` in a `Vec<T>` is reported with every other failing element.
        let bad = unsafe {
            OwnedProtect::new(build_factor_with_levels(
                &[i32::R_NA, 1, 4],
                &["Red", "Green", "Blue"],
            ))
        };
        let error = Vec::<Color>::try_from_sexp(bad.get())
            .unwrap_err()
            .to_string();
        for reason in ["NA is not allowed", "code 4 is not a level"] {
            assert!(error.contains(reason), "{error}");
        }
    });
}

#[test]
fn match_arg_vec_stays_character() {
    r_test_utils::with_r_thread(|| {
        let modes = unsafe { OwnedProtect::new(vec![Mode::Safe, Mode::Fast].into_sexp()) };
        let modes = modes.get();
        assert!(!modes.is_factor());
        assert_eq!(modes.string_elt_str(0), Some("Safe"));
        assert_eq!(modes.string_elt_str(1), Some("Fast"));
    });
}
