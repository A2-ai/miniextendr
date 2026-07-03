//! R-callable controls for the rayon thread pool.
//!
//! Thin wrappers over `miniextendr_api::optionals::parallel` — see
//! `docs/RAYON.md` ("Controlling Parallelism from R") for the precedence
//! table and design.

#[cfg(feature = "rayon")]
use miniextendr_api::miniextendr;

/// Report the effective rayon thread count. Follows the precedence in
/// `docs/RAYON.md` ("Controlling parallelism from R"):
/// `MINIEXTENDR_NUM_THREADS` env > `RAYON_NUM_THREADS` env > CRAN
/// `_R_CHECK_LIMIT_CORES_` cap (2) > `available_parallelism()`. Builds the
/// pool on first call, same as any other rayon entry point.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn miniextendr_num_threads() -> i32 {
    miniextendr_api::optionals::parallel::ensure_pool();
    i32::try_from(miniextendr_api::rayon_bridge::perf::num_threads()).unwrap_or(i32::MAX)
}

/// Build the rayon thread pool with exactly `n` threads, immediately. Must be
/// called before the first parallel operation — rayon's global pool cannot be
/// resized once built, so this errors if any pool already exists (whether
/// built by miniextendr or outside it).
/// @param n Number of threads to request (positive integer).
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn miniextendr_set_threads(n: i32) {
    let n = match usize::try_from(n) {
        Ok(n) if n > 0 => n,
        _ => panic!("miniextendr_set_threads: n must be a positive integer, got {n}"),
    };
    if let Err(msg) = miniextendr_api::optionals::parallel::set_threads(n) {
        panic!("{msg}");
    }
}
