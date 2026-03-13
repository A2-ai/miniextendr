//! Benchmarks for RFactor enum ↔ R factor conversions.
//!
//! Compares:
//! - Cached levels (via `#[derive(RFactor)]` with inline OnceLock)
//! - Uncached levels (fresh STRSXP allocation each call)
//!
//! Key finding: ~4x speedup for single value conversions (the primary use case).
//! Vector conversions show minimal difference since vector allocation dominates.

use miniextendr_api::factor::{build_factor, build_levels_sexp};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::{FactorVec, IntoR, MatchArg, RFactor};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Enum definitions

/// Uses `#[derive(RFactor)]` which generates IntoR with inline OnceLock caching.
/// The levels STRSXP is allocated once and reused for all subsequent conversions.
#[derive(Copy, Clone, Debug, RFactor)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

/// Manual implementation without caching - each conversion allocates fresh STRSXP.
/// Used as baseline to measure caching benefit.
#[derive(Copy, Clone, Debug)]
pub enum ColorUncached {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

impl MatchArg for ColorUncached {
    const CHOICES: &'static [&'static str] = &[
        "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "White", "Black",
    ];

    fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "Red" => Some(Self::Red),
            "Green" => Some(Self::Green),
            "Blue" => Some(Self::Blue),
            "Yellow" => Some(Self::Yellow),
            "Cyan" => Some(Self::Cyan),
            "Magenta" => Some(Self::Magenta),
            "White" => Some(Self::White),
            "Black" => Some(Self::Black),
            _ => None,
        }
    }

    fn to_choice(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
            Self::Cyan => "Cyan",
            Self::Magenta => "Magenta",
            Self::White => "White",
            Self::Black => "Black",
        }
    }
}

impl RFactor for ColorUncached {
    fn to_level_index(self) -> i32 {
        match self {
            Self::Red => 1,
            Self::Green => 2,
            Self::Blue => 3,
            Self::Yellow => 4,
            Self::Cyan => 5,
            Self::Magenta => 6,
            Self::White => 7,
            Self::Black => 8,
        }
    }

    fn from_level_index(idx: i32) -> Option<Self> {
        match idx {
            1 => Some(Self::Red),
            2 => Some(Self::Green),
            3 => Some(Self::Blue),
            4 => Some(Self::Yellow),
            5 => Some(Self::Cyan),
            6 => Some(Self::Magenta),
            7 => Some(Self::White),
            8 => Some(Self::Black),
            _ => None,
        }
    }
}

impl IntoR for ColorUncached {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        // No caching - allocates fresh levels STRSXP each time
        build_factor(&[self.to_level_index()], build_levels_sexp(Self::CHOICES))
    }
}
// endregion

// region: Single factor conversion (primary use case)

/// Single enum → factor with cached levels (~4x faster).
#[divan::bench]
fn single_cached() -> SEXP {
    divan::black_box(Color::Green.into_sexp())
}

/// Single enum → factor without caching (baseline).
#[divan::bench]
fn single_uncached() -> SEXP {
    divan::black_box(ColorUncached::Green.into_sexp())
}
// endregion

// region: Repeated single conversions (shows amortized benefit)

/// 100 conversions with caching - levels allocated once.
#[divan::bench]
fn repeated_100_cached() -> SEXP {
    let mut last = SEXP::null();
    for _ in 0..100 {
        last = Color::Blue.into_sexp();
    }
    divan::black_box(last)
}

/// 100 conversions without caching - 100 allocations.
#[divan::bench]
fn repeated_100_uncached() -> SEXP {
    let mut last = SEXP::null();
    for _ in 0..100 {
        last = ColorUncached::Blue.into_sexp();
    }
    divan::black_box(last)
}
// endregion

// region: Vector factor conversion (levels overhead is minimal)

const VEC_SIZES: &[usize] = &[1, 16, 256, 4096];

fn make_color_vec(n: usize) -> Vec<Color> {
    use Color::*;
    let colors = [Red, Green, Blue, Yellow, Cyan, Magenta, White, Black];
    (0..n).map(|i| colors[i % colors.len()]).collect()
}

/// FactorVec conversion - levels allocated per call (generic impl limitation).
/// At large sizes, vector allocation dominates and caching has minimal impact.
#[divan::bench(args = VEC_SIZES)]
fn vec_factor_vec(n: usize) -> SEXP {
    let vec = make_color_vec(n);
    divan::black_box(FactorVec(vec).into_sexp())
}
// endregion
