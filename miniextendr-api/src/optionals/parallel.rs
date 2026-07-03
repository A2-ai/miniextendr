//! Thread pool sizing, CRAN compliance, and cgroup-aware defaults for the
//! rayon bridge.
//!
//! # Precedence
//!
//! ```text
//! effective_threads() =
//!   MINIEXTENDR_NUM_THREADS env        (explicit user/admin override)
//!   else RAYON_NUM_THREADS env         (rayon convention, respected as-is)
//!   else if _R_CHECK_LIMIT_CORES_ is truthy → min(2, available_parallelism())
//!   else available_parallelism()       (cgroup-quota aware already, Rust >=1.61)
//! ```
//!
//! [`ensure_pool`] builds the global rayon pool exactly once, sized by
//! [`effective_threads`]. If a global pool already exists (built by user
//! code before any miniextendr call), it wins — `ensure_pool` does nothing.
//!
//! See `docs/RAYON.md` ("Controlling parallelism from R") for the full
//! design and a decision guide comparing this to R-level parallelism
//! (future/mirai) and worker-thread task queues.

use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

static POOL_READY: Once = Once::new();
static POOL_BUILT: AtomicBool = AtomicBool::new(false);

/// Resolve the thread count [`ensure_pool`] will use, per the precedence
/// table above.
pub fn effective_threads() -> usize {
    resolve(
        std::env::var("MINIEXTENDR_NUM_THREADS").ok().as_deref(),
        std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
        std::env::var("_R_CHECK_LIMIT_CORES_").ok().as_deref(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1),
    )
}

/// Pure resolution logic, factored out of [`effective_threads`] so it's
/// testable without mutating real process environment variables (which
/// would race across parallel `#[test]` fns).
fn resolve(
    mx_num_threads: Option<&str>,
    rayon_num_threads: Option<&str>,
    r_check_limit_cores: Option<&str>,
    available: usize,
) -> usize {
    if let Some(n) = parse_positive(mx_num_threads) {
        return n;
    }
    if let Some(n) = parse_positive(rayon_num_threads) {
        return n;
    }
    if is_truthy(r_check_limit_cores) {
        available.min(2)
    } else {
        available
    }
}

fn parse_positive(v: Option<&str>) -> Option<usize> {
    v.and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
}

/// `_R_CHECK_LIMIT_CORES_` truthiness: R sets it to `"TRUE"` under
/// `--as-cran`. Treat unset, empty, or `false`/`FALSE` as not-limited —
/// anything else (including R's `"TRUE"`) caps at 2 cores.
fn is_truthy(v: Option<&str>) -> bool {
    match v {
        None => false,
        Some(v) => {
            let v = v.trim();
            !(v.is_empty() || v.eq_ignore_ascii_case("false"))
        }
    }
}

/// Build the global rayon pool, sized by [`effective_threads`], the first
/// time any rayon entry point runs. If a global pool already exists (user
/// code called `ThreadPoolBuilder::build_global()` first), leave it alone —
/// explicit user configuration always wins.
///
/// Idempotent and cheap — safe to call at the top of every rayon-backed
/// function.
pub fn ensure_pool() {
    POOL_READY.call_once(|| {
        // Ignore the error case: it means a global pool already exists
        // (ours from a prior call is impossible under `Once`, so this is
        // always someone else's pool winning by construction).
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(effective_threads())
            .build_global();
        POOL_BUILT.store(true, Ordering::Relaxed);
    });
}

/// Whether [`ensure_pool`] has already run in this process. The global pool
/// is fixed at that point — rayon cannot resize it.
pub fn pool_is_built() -> bool {
    POOL_BUILT.load(Ordering::Relaxed)
}

/// Set the thread count for the *next* pool build via
/// `MINIEXTENDR_NUM_THREADS`.
///
/// Errors if the pool has already been built: rayon's global pool cannot be
/// resized once created, so a post-hoc call would otherwise silently no-op.
pub fn set_threads(n: usize) -> Result<(), String> {
    if pool_is_built() {
        return Err(format!(
            "miniextendr: the rayon thread pool is already built with {} threads and \
             cannot be resized. Set MINIEXTENDR_NUM_THREADS (or call this) before the \
             first parallel operation, or restart R.",
            rayon::current_num_threads()
        ));
    }
    // SAFETY: miniextendr's worker-thread model means Rust code runs on a
    // single dedicated thread (or inline on the main thread without
    // `worker-thread`) — this never races another env mutation.
    unsafe {
        std::env::set_var("MINIEXTENDR_NUM_THREADS", n.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_mx_num_threads() {
        assert_eq!(resolve(Some("3"), Some("7"), Some("TRUE"), 16), 3);
    }

    #[test]
    fn resolve_falls_back_to_rayon_num_threads() {
        assert_eq!(resolve(None, Some("5"), None, 16), 5);
    }

    #[test]
    fn resolve_ignores_zero_and_garbage() {
        assert_eq!(resolve(Some("0"), Some("nope"), None, 16), 16);
    }

    #[test]
    fn resolve_caps_under_cran_check() {
        assert_eq!(resolve(None, None, Some("TRUE"), 16), 2);
        assert_eq!(resolve(None, None, Some("TRUE"), 1), 1);
    }

    #[test]
    fn resolve_cran_flag_treats_false_and_empty_as_unset() {
        assert_eq!(resolve(None, None, Some(""), 16), 16);
        assert_eq!(resolve(None, None, Some("false"), 16), 16);
        assert_eq!(resolve(None, None, Some("FALSE"), 16), 16);
    }

    #[test]
    fn resolve_no_env_uses_available() {
        assert_eq!(resolve(None, None, None, 16), 16);
    }

    #[test]
    fn ensure_pool_is_idempotent() {
        ensure_pool();
        assert!(pool_is_built());
        ensure_pool(); // second call: Once short-circuits, must not panic
        assert!(pool_is_built());
    }

    #[test]
    fn set_threads_errors_once_pool_built() {
        ensure_pool();
        let err = set_threads(4).unwrap_err();
        assert!(err.contains("already built"));
    }
}
