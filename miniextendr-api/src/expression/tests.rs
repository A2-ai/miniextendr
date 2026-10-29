use super::*;

// These tests verify compilation and basic invariants.
// Full integration tests require the R runtime.

#[test]
fn renv_types_are_sized() {
    // Just verify types compile and are sized
    fn assert_sized<T: Sized>() {}
    assert_sized::<RSymbol>();
    assert_sized::<RCall>();
    assert_sized::<REnv>();
}

#[test]
fn renv_constructors_compile() {
    // Verify all REnv constructor signatures compile.
    // Actual testing requires the R runtime.
    fn assert_env_fn<F: FnOnce() -> REnv>(_f: F) {}
    fn assert_env_result_fn<F: FnOnce() -> Result<REnv, REvalError>>(_f: F) {}

    assert_env_fn(|| unsafe { REnv::global() });
    assert_env_fn(|| unsafe { REnv::base() });
    assert_env_fn(|| unsafe { REnv::empty() });
    assert_env_fn(REnv::base_namespace);
    assert_env_fn(|| unsafe { REnv::caller() });
    assert_env_result_fn(|| unsafe { REnv::package_namespace("base") });
}
