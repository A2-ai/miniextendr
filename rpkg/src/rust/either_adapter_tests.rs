//! Either adapter tests
use miniextendr_api::either_impl::Either;
use miniextendr_api::miniextendr;

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

/// Check if the value was parsed as Left (integer)
/// @noRd
#[miniextendr]
pub fn either_is_left(value: Either<i32, String>) -> bool {
    value.is_left()
}

/// Check if the value was parsed as Right (string)
/// @noRd
#[miniextendr]
pub fn either_is_right(value: Either<i32, String>) -> bool {
    value.is_right()
}

/// Nested either: Either<bool, Either<i32, String>>
/// @noRd
#[miniextendr]
pub fn either_nested(value: Either<bool, Either<i32, String>>) -> String {
    match value {
        Either::Left(b) => format!("bool:{b}"),
        Either::Right(inner) => match inner {
            Either::Left(n) => format!("int:{n}"),
            Either::Right(s) => format!("str:{s}"),
        },
    }
}

/// Either with zero value (edge case for i32)
/// @noRd
#[miniextendr]
pub fn either_zero() -> Either<i32, String> {
    Either::Left(0)
}
