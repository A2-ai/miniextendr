use super::*;

#[test]
fn markers_hold_their_sexps_only() {
    assert_eq!(
        std::mem::size_of::<Quoted<'static>>(),
        2 * std::mem::size_of::<SEXP>()
    );
    assert_eq!(
        std::mem::size_of::<Quosure<'static>>(),
        3 * std::mem::size_of::<SEXP>()
    );
}
