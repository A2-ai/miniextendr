//! Benchmarks for RFactor enum ↔ R factor conversions.
//!
//! Compares:
//! - Cached levels (via `#[derive(RFactor)]` with inline OnceLock)
//! - Uncached levels (fresh STRSXP allocation each call)

use miniextendr_api::ffi::SEXP;
use miniextendr_api::factor::{build_factor, build_factor_with_levels, build_levels_sexp_preserved};
use miniextendr_api::{FactorVec, IntoR, RFactor};
use std::sync::OnceLock;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Enum definitions
// =============================================================================

/// Cached version - uses derive macro which generates inline OnceLock caching.
#[derive(Copy, Clone, Debug, RFactor)]
pub enum CachedColor {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

/// Uncached version - manually implements IntoR without caching.
/// Each conversion allocates fresh STRSXP for levels.
#[derive(Copy, Clone, Debug)]
pub enum UncachedColor {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

impl RFactor for UncachedColor {
    const LEVELS: &'static [&'static str] = &[
        "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "White", "Black",
    ];

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

impl IntoR for UncachedColor {
    fn into_sexp(self) -> SEXP {
        // No caching - allocates fresh levels STRSXP each time
        build_factor(&[self.to_level_index()], Self::LEVELS)
    }
}

// =============================================================================
// Single factor conversion benchmarks
// =============================================================================

#[divan::bench]
fn single_cached() -> SEXP {
    // Uses derive-generated IntoR with OnceLock caching
    divan::black_box(CachedColor::Green.into_sexp())
}

#[divan::bench]
fn single_uncached() -> SEXP {
    // Allocates fresh levels STRSXP each time
    divan::black_box(UncachedColor::Green.into_sexp())
}

// =============================================================================
// Vector factor conversion benchmarks
// =============================================================================

const VEC_SIZES: &[usize] = &[1, 16, 256, 4096];

fn make_cached_vec(n: usize) -> Vec<CachedColor> {
    use CachedColor::*;
    let colors = [Red, Green, Blue, Yellow, Cyan, Magenta, White, Black];
    (0..n).map(|i| colors[i % colors.len()]).collect()
}

fn make_uncached_vec(n: usize) -> Vec<UncachedColor> {
    use UncachedColor::*;
    let colors = [Red, Green, Blue, Yellow, Cyan, Magenta, White, Black];
    (0..n).map(|i| colors[i % colors.len()]).collect()
}

/// FactorVec with cached type - note: FactorVec itself doesn't cache levels
/// because it's generic. This tests the baseline for Vec conversion.
#[divan::bench(args = VEC_SIZES)]
fn vec_factor_vec_wrapper(n: usize) -> SEXP {
    let vec = make_cached_vec(n);
    divan::black_box(FactorVec(vec).into_sexp())
}

/// Manual uncached vec conversion - builds fresh levels STRSXP.
#[divan::bench(args = VEC_SIZES)]
fn vec_uncached(n: usize) -> SEXP {
    let vec = make_uncached_vec(n);
    let indices: Vec<i32> = vec.iter().map(|c| c.to_level_index()).collect();
    divan::black_box(build_factor(&indices, UncachedColor::LEVELS))
}

/// Manual cached vec conversion - pre-caches levels STRSXP.
#[divan::bench(args = VEC_SIZES)]
fn vec_cached_manual(n: usize) -> SEXP {
    static LEVELS_CACHE: OnceLock<SEXP> = OnceLock::new();

    let vec = make_cached_vec(n);
    let indices: Vec<i32> = vec.iter().map(|c| c.to_level_index()).collect();
    let levels = *LEVELS_CACHE
        .get_or_init(|| build_levels_sexp_preserved(CachedColor::LEVELS));
    divan::black_box(build_factor_with_levels(&indices, levels))
}

// =============================================================================
// Repeated single conversions (amortization test)
// =============================================================================

/// Many single conversions with caching - shows amortized benefit.
#[divan::bench]
fn repeated_100_cached() -> SEXP {
    let mut last = SEXP::null();
    for _ in 0..100 {
        last = CachedColor::Blue.into_sexp();
    }
    divan::black_box(last)
}

/// Many single conversions without caching - shows allocation overhead.
#[divan::bench]
fn repeated_100_uncached() -> SEXP {
    let mut last = SEXP::null();
    for _ in 0..100 {
        last = UncachedColor::Blue.into_sexp();
    }
    divan::black_box(last)
}
