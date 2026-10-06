//! Compile-pass test (#1766): a string-parsed type defined outside
//! `miniextendr-api` gets `T`, `Option<T>`, `Vec<T>` and `Vec<Option<T>>`
//! through the public `try_from_sexp_via_str_parse!`, using only the public
//! API. The macro emits impls on the local type alone (the parse hook,
//! `TryFromSexp`, `TryFromSexpElement`); the three container impls are the
//! `TryFromSexpElement` blankets in `miniextendr-api`, which an impl here
//! could not provide (E0117).

#![allow(dead_code)]

use miniextendr_api::{TryFromSexp, miniextendr};

#[derive(Debug)]
pub struct Slug(String);

impl std::str::FromStr for Slug {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || !s.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            return Err("expected lowercase letters and '-'");
        }
        Ok(Slug(s.to_owned()))
    }
}

miniextendr_api::try_from_sexp_via_str_parse!(Slug, "slug", |s| s.parse::<Slug>());

/// A parse body that is not `FromStr`, with an error type that only
/// implements `Display`.
#[derive(Debug)]
pub struct Even(u32);

pub enum NotEven {
    NotANumber,
    Odd(u32),
}

impl std::fmt::Display for NotEven {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotEven::NotANumber => f.write_str("not a number"),
            NotEven::Odd(n) => write!(f, "{n} is odd"),
        }
    }
}

miniextendr_api::try_from_sexp_via_str_parse!(Even, "even number", |s| match s.trim().parse::<u32>() {
    Ok(n) if n % 2 == 0 => Ok(Even(n)),
    Ok(n) => Err(NotEven::Odd(n)),
    Err(_) => Err(NotEven::NotANumber),
});

#[miniextendr]
pub fn slug_scalar(x: Slug) -> i32 {
    x.0.len() as i32
}

#[miniextendr]
pub fn slug_option(x: Option<Slug>) -> bool {
    x.is_some()
}

#[miniextendr]
pub fn slug_vec(x: Vec<Slug>) -> i32 {
    x.len() as i32
}

#[miniextendr]
pub fn slug_vec_option(x: Vec<Option<Slug>>) -> i32 {
    x.iter().filter(|s| s.is_none()).count() as i32
}

/// All four in one signature, beside a boxed slice of the same type.
#[miniextendr]
pub fn slug_all(
    a: Slug,
    b: Option<Slug>,
    c: Vec<Slug>,
    d: Vec<Option<Slug>>,
    e: Box<[Slug]>,
    f: Vec<Option<Even>>,
) -> i32 {
    let _ = (a, b, d, f);
    (c.len() + e.len()) as i32
}

// All four shapes read only character input.
const _: () = assert!(<Slug as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Option<Slug> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Vec<Slug> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Vec<Option<Slug>> as TryFromSexp>::CHARACTER_ONLY);

fn main() {}
