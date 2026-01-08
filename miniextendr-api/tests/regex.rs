mod r_test_utils;

#[cfg(feature = "regex")]
use miniextendr_api::Regex;
#[cfg(feature = "regex")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "regex")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "regex")]
#[test]
fn regex_from_pattern() {
    r_test_utils::with_r_thread(|| {
        let pattern = r"^\d{4}-\d{2}-\d{2}$".to_string();
        let sexp = pattern.into_sexp();
        let re: Regex = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(re.is_match("2024-01-15"));
        assert!(!re.is_match("not-a-date"));
    });
}

#[cfg(feature = "regex")]
#[test]
fn regex_option_none() {
    r_test_utils::with_r_thread(|| {
        let opt: Option<String> = None;
        let sexp = opt.into_sexp();
        let back: Option<Regex> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_none());
    });
}

#[cfg(feature = "regex")]
#[test]
fn regex_option_some() {
    r_test_utils::with_r_thread(|| {
        let pattern = r"\w+".to_string();
        let sexp = pattern.into_sexp();
        let opt: Option<Regex> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(opt.is_some());
        assert!(opt.unwrap().is_match("hello"));
    });
}

#[cfg(feature = "regex")]
#[test]
fn regex_invalid_pattern_error() {
    r_test_utils::with_r_thread(|| {
        let invalid = "[invalid".to_string();
        let sexp = invalid.into_sexp();
        let result: Result<Regex, _> = TryFromSexp::try_from_sexp(sexp);
        assert!(result.is_err());
    });
}

#[cfg(feature = "regex")]
#[test]
fn regex_complex_pattern() {
    r_test_utils::with_r_thread(|| {
        // Email-like pattern
        let pattern = r"^[\w.+-]+@[\w-]+\.[\w.-]+$".to_string();
        let sexp = pattern.into_sexp();
        let re: Regex = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(re.is_match("test@example.com"));
        assert!(re.is_match("user.name+tag@sub.domain.org"));
        assert!(!re.is_match("not-an-email"));
    });
}
