use super::*;

#[test]
fn markers_are_transparent_over_sexp() {
    assert_eq!(std::mem::size_of::<Call>(), std::mem::size_of::<SEXP>());
    assert_eq!(
        std::mem::size_of::<CallerCall>(),
        std::mem::size_of::<SEXP>()
    );
    let raw = SEXP::null();
    let call = Call::from_sexp(raw);
    assert_eq!(call.sexp(), raw);
    assert_eq!(SEXP::from(call), raw);
    let caller = CallerCall::from_sexp(raw);
    assert_eq!(Call::from(caller), call);
    assert_eq!(*caller, raw);
}
