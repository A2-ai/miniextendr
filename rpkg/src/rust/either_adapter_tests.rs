//! Either adapter tests
use miniextendr_api::either_impl::Either;
use miniextendr_api::{miniextendr, miniextendr_module};

/// Accepts either an integer or a string. Returns "int:N" or "str:S".
/// @noRd
#[miniextendr]
pub fn either_int_or_str(value: Either<i32, String>) -> String {
    match value {
        Either::Left(n) => format!("int:{n}"),
        Either::Right(s) => format!("str:{s}"),
    }
}

/// Accepts either a double or a vector of integers.
/// @noRd
#[miniextendr]
pub fn either_dbl_or_vec(value: Either<f64, Vec<i32>>) -> String {
    match value {
        Either::Left(d) => format!("dbl:{d}"),
        Either::Right(v) => format!("vec:{v:?}"),
    }
}

/// Returns Left(i32) by creating one
/// @noRd
#[miniextendr]
pub fn either_make_left(n: i32) -> Either<i32, String> {
    Either::Left(n)
}

/// Returns Right(String) by creating one
/// @noRd
#[miniextendr]
pub fn either_make_right(s: String) -> Either<i32, String> {
    Either::Right(s)
}

miniextendr_module! {
    mod either_adapter_tests;
    fn either_int_or_str;
    fn either_dbl_or_vec;
    fn either_make_left;
    fn either_make_right;
}
