use super::*;

#[test]
fn dots_empty_creates_nil() {
    let dots = Dots::empty();
    assert_eq!(dots.inner, SEXP::nil());
}
