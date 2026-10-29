use super::*;

#[test]
fn s4_is_compiles() {
    // Verify the function signature compiles.
    // Actual testing requires the R runtime.
    fn assert_fn<F: Fn(SEXP) -> bool>(_f: F) {}
    assert_fn(|s| unsafe { s4_is(s) });
}

#[test]
fn s4_class_name_compiles() {
    fn assert_fn<F: Fn(SEXP) -> Option<String>>(_f: F) {}
    assert_fn(|s| unsafe { s4_class_name(s) });
}
