//! Native-borrow preflight benchmarks (#1504).
//!
//! Each case calls the generated C wrapper of a `#[miniextendr]` function
//! directly, with no R wrapper and no `.Call()`, so the numbers are the
//! wrapper's own work: argument conversion, the alias check that runs before
//! it, and a trivial body. The signatures match
//! `rpkg/src/rust/native_borrow_bench_fixtures.rs`, which
//! `rpkg/dev/bench-native-borrow/` measures at the R call level.
//!
//! Groups:
//! - `plain`: no borrowed argument.
//! - `borrowed`: one and three borrowed native vectors (the three include a
//!   mutable one, so the alias check runs), and borrowed strings.
//! - `either`: an `Either` whose right arm borrows, on each arm, alone and
//!   next to a borrowed vector; `left_aliased` passes one vector twice, which
//!   is valid because the owned left arm takes it.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{IntoR, SEXP, miniextendr};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Fixtures

#[miniextendr]
pub fn nbb_plain(x: i32) -> i32 {
    x
}

#[miniextendr]
pub fn nbb_slice(x: &[f64]) -> i32 {
    i32::try_from(x.len()).unwrap_or(i32::MAX)
}

#[miniextendr]
pub fn nbb_slices(a: &[f64], b: &mut [f64], c: &[i32]) -> i32 {
    i32::try_from(a.len() + b.len() + c.len()).unwrap_or(i32::MAX)
}

#[miniextendr]
pub fn nbb_str(s: &str) -> i32 {
    i32::try_from(s.len()).unwrap_or(i32::MAX)
}

#[miniextendr]
pub fn nbb_strs(a: &str, b: &str, c: Vec<&str>) -> i32 {
    i32::try_from(a.len() + b.len() + c.len()).unwrap_or(i32::MAX)
}

#[miniextendr]
pub fn nbb_either(a: Either<i32, &mut [f64]>, b: &[f64]) -> i32 {
    let a = match a {
        Either::Left(n) => n,
        Either::Right(v) => i32::try_from(v.len()).unwrap_or(i32::MAX),
    };
    a.saturating_add(i32::try_from(b.len()).unwrap_or(i32::MAX))
}

#[miniextendr]
pub fn nbb_either_alone(a: Either<i32, &[f64]>) -> i32 {
    match a {
        Either::Left(n) => n,
        Either::Right(v) => i32::try_from(v.len()).unwrap_or(i32::MAX),
    }
}

#[miniextendr]
pub fn nbb_either_alias(a: Either<f64, &mut [f64]>, b: &[f64]) -> f64 {
    let len = |n: usize| u32::try_from(n).map_or(f64::INFINITY, f64::from);
    let a = match a {
        Either::Left(x) => x,
        Either::Right(v) => len(v.len()),
    };
    a + len(b.len())
}
// endregion

// region: Inputs

/// Inputs built once and preserved for the whole run.
struct Inputs {
    nil: SEXP,
    int1: SEXP,
    dbl1: SEXP,
    x: SEXP,
    y: SEXP,
    iv: SEXP,
    abc: SEXP,
    def: SEXP,
    chr3: SEXP,
}

// SAFETY: the benches run on R's main thread only; the SEXPs are preserved.
unsafe impl Send for Inputs {}
unsafe impl Sync for Inputs {}

fn keep(sexp: SEXP) -> SEXP {
    unsafe { miniextendr_api::sys::R_PreserveObject(sexp) };
    sexp
}

fn inputs() -> &'static Inputs {
    static INPUTS: std::sync::OnceLock<Inputs> = std::sync::OnceLock::new();
    INPUTS.get_or_init(|| Inputs {
        nil: SEXP::nil(),
        int1: keep(1i32.into_sexp()),
        dbl1: keep(2.5f64.into_sexp()),
        x: keep((0..1000).map(f64::from).collect::<Vec<f64>>().into_sexp()),
        y: keep((0..1000).map(f64::from).collect::<Vec<f64>>().into_sexp()),
        iv: keep((0..1000).collect::<Vec<i32>>().into_sexp()),
        abc: keep("abc".into_sexp()),
        def: keep("def".into_sexp()),
        chr3: keep(vec!["x", "y", "z"].into_sexp()),
    })
}
// endregion

mod plain {
    use super::*;

    #[divan::bench]
    fn plain() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_plain(i.nil, divan::black_box(i.int1)))
    }
}

mod borrowed {
    use super::*;

    #[divan::bench]
    fn slice() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_slice(i.nil, divan::black_box(i.x)))
    }

    #[divan::bench]
    fn slices() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_slices(
            i.nil,
            divan::black_box(i.x),
            divan::black_box(i.y),
            divan::black_box(i.iv),
        ))
    }

    #[divan::bench]
    fn str() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_str(i.nil, divan::black_box(i.abc)))
    }

    #[divan::bench]
    fn strs() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_strs(
            i.nil,
            divan::black_box(i.abc),
            divan::black_box(i.def),
            divan::black_box(i.chr3),
        ))
    }
}

mod either {
    use super::*;

    #[divan::bench]
    fn left() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_either(
            i.nil,
            divan::black_box(i.int1),
            divan::black_box(i.x),
        ))
    }

    #[divan::bench]
    fn right() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_either(
            i.nil,
            divan::black_box(i.y),
            divan::black_box(i.x),
        ))
    }

    #[divan::bench]
    fn alone_left() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_either_alone(
            i.nil,
            divan::black_box(i.int1),
        ))
    }

    #[divan::bench]
    fn alone_right() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_either_alone(
            i.nil,
            divan::black_box(i.x),
        ))
    }

    #[divan::bench]
    fn left_aliased() -> SEXP {
        let i = inputs();
        divan::black_box(C_native_borrow_nbb_either_alias(
            i.nil,
            divan::black_box(i.dbl1),
            divan::black_box(i.dbl1),
        ))
    }
}
