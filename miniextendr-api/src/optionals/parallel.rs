//! Thread pool sizing, CRAN compliance, and cgroup-aware defaults for the
//! rayon bridge.
//!
//! # Precedence
//!
//! ```text
//! effective_threads() =
//!   MINIEXTENDR_NUM_THREADS env        (explicit user/admin override)
//!   else RAYON_NUM_THREADS env         (rayon convention, respected as-is)
//!   else RAYON_RS_NUM_CPUS env         (rayon's deprecated fallback, still honored)
//!   else if _R_CHECK_LIMIT_CORES_ is truthy → min(2, available_parallelism())
//!   else available_parallelism()       (cgroup-quota aware already, Rust >=1.61)
//! ```
//!
//! # Divergence from rayon-core
//!
//! The two `RAYON_*` vars are parsed *exactly* as `rayon-core` parses them
//! (`usize::from_str`, no whitespace trim) so "respected as-is" is literally
//! true — a value rayon would reject, we reject too. Two deliberate exceptions:
//!
//! - **Zero is treated as unset, not "use the default".** rayon-core reads
//!   `RAYON_NUM_THREADS=0` as an explicit request for its default (all cores)
//!   and short-circuits. We instead fall through to the `_R_CHECK_LIMIT_CORES_`
//!   cap and `available_parallelism()`, so a stray `0` can never bypass CRAN's
//!   core limit.
//! - **`MINIEXTENDR_NUM_THREADS` (our own var) is lenient:** surrounding
//!   whitespace is trimmed before parsing. The `RAYON_*` vars are not — they
//!   match rayon byte-for-byte.
//!
//! [`ensure_pool`] builds the global rayon pool exactly once, sized by
//! [`effective_threads`]. If a global pool already exists (built by user
//! code before any miniextendr call), it wins — `ensure_pool` does nothing.
//!
//! See `docs/RAYON.md` ("Controlling parallelism from R") for the full
//! design and a decision guide comparing this to R-level parallelism
//! (future/mirai) and worker-thread task queues.

use std::sync::Once;

static POOL_READY: Once = Once::new();

/// Resolve the thread count [`ensure_pool`] will use, per the precedence
/// table above.
pub fn effective_threads() -> usize {
    resolve(
        std::env::var("MINIEXTENDR_NUM_THREADS").ok().as_deref(),
        std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
        std::env::var("RAYON_RS_NUM_CPUS").ok().as_deref(),
        std::env::var("_R_CHECK_LIMIT_CORES_").ok().as_deref(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1),
    )
}

/// Pure resolution logic, factored out of [`effective_threads`] so it's
/// testable without mutating real process environment variables (which
/// would race across parallel `#[test]` fns). See the module-level
/// "Divergence from rayon-core" note for the parse semantics.
fn resolve(
    mx_num_threads: Option<&str>,
    rayon_num_threads: Option<&str>,
    rayon_rs_num_cpus: Option<&str>,
    r_check_limit_cores: Option<&str>,
    available: usize,
) -> usize {
    // Our own override — lenient (surrounding whitespace trimmed).
    if let Some(n) = parse_positive(mx_num_threads) {
        return n;
    }
    // rayon's own convention: RAYON_NUM_THREADS, then the deprecated
    // RAYON_RS_NUM_CPUS fallback, parsed byte-for-byte as rayon-core does.
    if let Some(n) = parse_rayon(rayon_num_threads).or_else(|| parse_rayon(rayon_rs_num_cpus)) {
        return n;
    }
    if is_truthy(r_check_limit_cores) {
        available.min(2)
    } else {
        available
    }
}

/// Lenient positive-integer parse for our own `MINIEXTENDR_NUM_THREADS`:
/// trims surrounding whitespace, rejects zero and non-numeric input.
fn parse_positive(v: Option<&str>) -> Option<usize> {
    v.and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
}

/// Parse a `RAYON_*` var exactly as `rayon-core` does (`usize::from_str`, no
/// trim), keeping only positive values. Zero and unparseable input yield
/// `None` so the resolver falls through — see the module "Divergence" note on
/// why we do *not* mirror rayon's "0 = use the default" short-circuit.
fn parse_rayon(v: Option<&str>) -> Option<usize> {
    v.and_then(|v| v.parse::<usize>().ok()).filter(|&n| n > 0)
}

/// `_R_CHECK_LIMIT_CORES_` truthiness. R sets it to `"TRUE"` under
/// `--as-cran`. Unset defaults to not-limited; otherwise
/// [`crate::env_flag::parse_bool`] decides, and any *unrecognized* value
/// (e.g. garbage) fails safe toward limiting — under a CRAN check we would
/// rather cap cores than not.
fn is_truthy(v: Option<&str>) -> bool {
    match v {
        None => false,
        Some(v) => crate::env_flag::parse_bool(v).unwrap_or(true),
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
    });
}

/// Whether the pool size has been locked in — by [`ensure_pool`] or a
/// successful [`set_threads`]. The global pool is fixed at that point —
/// rayon cannot resize it.
pub fn pool_is_built() -> bool {
    POOL_READY.is_completed()
}

/// Build the global rayon pool with exactly `n` threads, immediately.
///
/// Errors if a pool already exists: rayon's global pool cannot be resized
/// once created, so a post-hoc call would otherwise silently no-op. Building
/// eagerly is the only exact probe rayon exposes, and it detects pools built
/// outside miniextendr too (a user `ThreadPoolBuilder::build_global()`, or a
/// raw rayon call that lazily created the default pool) — a stashed-count
/// design can't, and would return `Ok` for a setting that never takes effect.
pub fn set_threads(n: usize) -> Result<(), String> {
    if n == 0 {
        return Err("miniextendr: thread count must be positive".to_string());
    }
    let mut outcome = None;
    POOL_READY.call_once(|| {
        outcome = Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global(),
        );
    });
    match outcome {
        // Once had already completed: ensure_pool or a prior set_threads won.
        None => Err(format!(
            "miniextendr: the rayon thread pool is already built with {} threads and \
             cannot be resized. Set MINIEXTENDR_NUM_THREADS (or call this) before the \
             first parallel operation, or restart R.",
            rayon::current_num_threads()
        )),
        Some(Ok(())) => Ok(()),
        // Our build ran but rayon reports a global pool already existed —
        // one built outside miniextendr entirely.
        Some(Err(_)) => Err(format!(
            "miniextendr: a global rayon pool is already built with {} threads \
             (created outside miniextendr) and cannot be resized. Restart R and set \
             the thread count before the first parallel operation.",
            rayon::current_num_threads()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_mx_num_threads() {
        assert_eq!(
            resolve(Some("3"), Some("7"), Some("9"), Some("TRUE"), 16),
            3
        );
    }

    #[test]
    fn resolve_falls_back_to_rayon_num_threads() {
        assert_eq!(resolve(None, Some("5"), None, None, 16), 5);
    }

    #[test]
    fn resolve_ignores_zero_and_garbage() {
        assert_eq!(resolve(Some("0"), Some("nope"), None, None, 16), 16);
    }

    #[test]
    fn resolve_caps_under_cran_check() {
        assert_eq!(resolve(None, None, None, Some("TRUE"), 16), 2);
        assert_eq!(resolve(None, None, None, Some("TRUE"), 1), 1);
    }

    #[test]
    fn resolve_cran_flag_treats_false_and_empty_as_unset() {
        assert_eq!(resolve(None, None, None, Some(""), 16), 16);
        assert_eq!(resolve(None, None, None, Some("false"), 16), 16);
        assert_eq!(resolve(None, None, None, Some("FALSE"), 16), 16);
    }

    #[test]
    fn resolve_no_env_uses_available() {
        assert_eq!(resolve(None, None, None, None, 16), 16);
    }

    #[test]
    fn resolve_honors_deprecated_rayon_rs_num_cpus() {
        // Used when RAYON_NUM_THREADS is unset, matching rayon-core.
        assert_eq!(resolve(None, None, Some("6"), None, 16), 6);
        // ...but RAYON_NUM_THREADS wins over it (rayon's own priority).
        assert_eq!(resolve(None, Some("4"), Some("6"), None, 16), 4);
    }

    #[test]
    fn resolve_rayon_vars_match_rayon_no_trim() {
        // Our own var trims surrounding whitespace...
        assert_eq!(resolve(Some(" 3 "), None, None, None, 16), 3);
        // ...but the rayon vars are parsed byte-for-byte, so whitespace is a
        // parse failure that falls through (exactly as rayon-core would).
        assert_eq!(resolve(None, Some(" 5 "), None, None, 16), 16);
        assert_eq!(resolve(None, None, Some(" 6 "), None, 16), 16);
    }

    #[test]
    fn resolve_rayon_zero_falls_through_to_cap() {
        // Deliberate divergence from rayon-core (which reads 0 as "use the
        // default", all cores): we treat 0 as unset so the CRAN core cap still
        // applies and can never be bypassed.
        assert_eq!(resolve(None, Some("0"), None, Some("TRUE"), 16), 2);
        assert_eq!(resolve(None, Some("0"), None, None, 16), 16);
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

    #[test]
    fn set_threads_rejects_zero_without_touching_the_pool() {
        // Guard fires before the Once — must not lock in a pool size.
        let err = set_threads(0).unwrap_err();
        assert!(err.contains("positive"));
    }
}
