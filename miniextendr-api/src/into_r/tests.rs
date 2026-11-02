use super::IntoR;
use crate::condition::RConditionData;

struct Fallible;

/// A refusal with only a message.
#[derive(Debug, PartialEq)]
struct SliceError(&'static str);

impl RConditionData for SliceError {
    fn message(&self) -> String {
        self.0.to_owned()
    }
}

impl IntoR for &[Fallible] {
    type Error = SliceError;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Err(SliceError("checked slice error"))
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Err(SliceError("unchecked slice error"))
    }
}

#[test]
fn array_forwards_slice_errors_without_panicking() {
    assert_eq!(
        [Fallible].try_into_sexp().unwrap_err(),
        SliceError("checked slice error")
    );
    // The test slice conversion performs no R calls.
    assert_eq!(
        unsafe { [Fallible].try_into_sexp_unchecked() }.unwrap_err(),
        SliceError("unchecked slice error")
    );
    assert_eq!(
        Some([Fallible]).try_into_sexp().unwrap_err(),
        SliceError("checked slice error")
    );
    assert_eq!(
        unsafe { Some([Fallible]).try_into_sexp_unchecked() }.unwrap_err(),
        SliceError("unchecked slice error")
    );
}
