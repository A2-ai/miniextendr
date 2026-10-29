use super::IntoR;

struct Fallible;

impl IntoR for &[Fallible] {
    type Error = &'static str;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Err("checked slice error")
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Err("unchecked slice error")
    }
}

#[test]
fn array_forwards_slice_errors_without_panicking() {
    assert_eq!(
        [Fallible].try_into_sexp().unwrap_err(),
        "checked slice error"
    );
    // The test slice conversion performs no R calls.
    assert_eq!(
        unsafe { [Fallible].try_into_sexp_unchecked() }.unwrap_err(),
        "unchecked slice error"
    );
    assert_eq!(
        Some([Fallible]).try_into_sexp().unwrap_err(),
        "checked slice error"
    );
    assert_eq!(
        unsafe { Some([Fallible]).try_into_sexp_unchecked() }.unwrap_err(),
        "unchecked slice error"
    );
}
