use super::*;

#[test]
fn catch_unwind_returns_value_on_success() {
    let result = guarded_ffi_call(|| 42, GuardMode::CatchUnwind, PanicSource::Worker);
    assert_eq!(result, 42);
}

#[test]
fn fallback_returns_value_on_success() {
    let result = guarded_ffi_call_with_fallback(|| 42, -1, PanicSource::Connection);
    assert_eq!(result, 42);
}

#[test]
fn fallback_returns_fallback_on_panic() {
    let result = guarded_ffi_call_with_fallback(|| panic!("boom"), -1, PanicSource::Connection);
    assert_eq!(result, -1);
}
