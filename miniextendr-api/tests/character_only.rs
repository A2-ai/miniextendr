//! `TryFromSexp::CHARACTER_ONLY`: which conversions read only character or
//! factor input (other than `NULL`). `#[miniextendr]` rejects such a type as
//! the `R` arm of an `Either<T, R>` choice parameter, so the table below pins
//! what is set, what is forwarded, and what stays `false`.
//!
//! Constants only: no R session. The feature-gated rows run under
//! `--features either,regex,log`.

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::match_arg::{EitherArmProbe, EitherArmProbeFallback};
use miniextendr_api::{
    AsCharacter, AsCharacterVec, AsFromStr, AsFromStrVec, DataFrame, List, Missing,
    ProtectedStrVec, SEXP, StrVec,
};

/// Read through a generic, so the rows are not `assert!` on constants.
fn character_only<T: TryFromSexp>() -> bool {
    T::CHARACTER_ONLY
}

type Label = String;

/// Delegates to `String` and forwards its const (the rustdoc example).
#[allow(dead_code)] // only the const is read
struct Username(String);

impl TryFromSexp for Username {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = <String as TryFromSexp>::CHARACTER_ONLY;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        String::try_from_sexp(sexp).map(Username)
    }
}

/// Reads only strings, but leaves the default.
#[allow(dead_code)] // only the const is read
struct Loose(String);

impl TryFromSexp for Loose {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        String::try_from_sexp(sexp).map(Loose)
    }
}

#[test]
fn character_only_table() {
    #[allow(unused_mut)]
    let mut rows: Vec<(&str, bool, bool)> = vec![
        // Set on the string and factor conversions.
        ("String", character_only::<String>(), true),
        ("&'static str", character_only::<&'static str>(), true),
        ("char", character_only::<char>(), true),
        ("Option<String>", character_only::<Option<String>>(), true),
        ("Vec<String>", character_only::<Vec<String>>(), true),
        ("HashSet<String>", character_only::<HashSet<String>>(), true),
        ("PathBuf", character_only::<PathBuf>(), true),
        (
            "Cow<'static, str>",
            character_only::<Cow<'static, str>>(),
            true,
        ),
        ("StrVec", character_only::<StrVec<'static>>(), true),
        ("ProtectedStrVec", character_only::<ProtectedStrVec>(), true),
        ("AsFromStr<i32>", character_only::<AsFromStr<i32>>(), true),
        (
            "AsFromStrVec<i32>",
            character_only::<AsFromStrVec<i32>>(),
            true,
        ),
        // Forwarded by the layers.
        ("Missing<String>", character_only::<Missing<String>>(), true),
        ("Box<[String]>", character_only::<Box<[String]>>(), true),
        (
            "Option<Vec<String>>",
            character_only::<Option<Vec<String>>>(),
            true,
        ),
        (
            "Option<HashSet<String>>",
            character_only::<Option<HashSet<String>>>(),
            true,
        ),
        ("Label", character_only::<Label>(), true),
        ("Option<Label>", character_only::<Option<Label>>(), true),
        ("Username", character_only::<Username>(), true),
        // Read more than character input.
        ("AsCharacter", character_only::<AsCharacter>(), false),
        ("AsCharacterVec", character_only::<AsCharacterVec>(), false),
        (
            "Option<AsCharacter>",
            character_only::<Option<AsCharacter>>(),
            false,
        ),
        ("DataFrame", character_only::<DataFrame>(), false),
        ("List", character_only::<List>(), false),
        ("SEXP", character_only::<SEXP>(), false),
        ("f64", character_only::<f64>(), false),
        ("Option<f64>", character_only::<Option<f64>>(), false),
        ("Vec<f64>", character_only::<Vec<f64>>(), false),
        ("Box<[f64]>", character_only::<Box<[f64]>>(), false),
        (
            "Vec<Vec<String>>",
            character_only::<Vec<Vec<String>>>(),
            false,
        ),
        // Reads only strings, but leaves the default: not checked.
        ("Loose", character_only::<Loose>(), false),
    ];
    #[cfg(feature = "either")]
    {
        use miniextendr_api::either_impl::Either;
        rows.push((
            "Either<String, char>",
            character_only::<Either<String, char>>(),
            true,
        ));
        rows.push((
            "Either<String, f64>",
            character_only::<Either<String, f64>>(),
            false,
        ));
    }
    #[cfg(feature = "regex")]
    rows.push(("Regex", character_only::<miniextendr_api::Regex>(), true));
    #[cfg(feature = "log")]
    rows.push((
        "LevelFilter",
        character_only::<miniextendr_api::optionals::log_impl::log::LevelFilter>(),
        true,
    ));

    let wrong: Vec<_> = rows
        .iter()
        .filter(|(_, got, want)| got != want)
        .map(|(name, got, _)| format!("{name}: CHARACTER_ONLY = {got}"))
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A type without `TryFromSexp`: the probe falls back to `false` and the
/// path still compiles, so the macro's guard adds no error of its own.
struct NoConv;

#[test]
fn either_arm_probe_reads_the_const_or_falls_back() {
    // `EitherArmProbeFallback` in scope is what resolves the `NoConv` row.
    let rows = [
        ("String", EitherArmProbe::<String>::CHARACTER_ONLY, true),
        ("f64", EitherArmProbe::<f64>::CHARACTER_ONLY, false),
        ("NoConv", EitherArmProbe::<NoConv>::CHARACTER_ONLY, false),
    ];
    for (name, got, want) in rows {
        assert_eq!(got, want, "{name}");
    }
}
