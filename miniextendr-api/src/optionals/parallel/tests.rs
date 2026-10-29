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
