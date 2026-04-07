//! Either adapter tests
use miniextendr_api::either_impl::Either;
use miniextendr_api::miniextendr;

/// Test dispatching an Either<i32, String> and returning a tagged string.
/// @param value Either an integer or a string from R.
#[miniextendr]
pub fn either_int_or_str(value: Either<i32, String>) -> String {
    match value {
        Either::Left(n) => format!("int:{n}"),
        Either::Right(s) => format!("str:{s}"),
    }
}

/// Test dispatching an Either<f64, Vec<i32>> and returning a tagged string.
/// @param value Either a double or an integer vector from R.
#[miniextendr]
pub fn either_dbl_or_vec(value: Either<f64, Vec<i32>>) -> String {
    match value {
        Either::Left(d) => format!("dbl:{d}"),
        Either::Right(v) => format!("vec:{v:?}"),
    }
}

/// Test creating a Left(i32) variant of Either.
/// @param n Integer value for the Left variant.
#[miniextendr]
pub fn either_make_left(n: i32) -> Either<i32, String> {
    Either::Left(n)
}

/// Test creating a Right(String) variant of Either.
/// @param s String value for the Right variant.
#[miniextendr]
pub fn either_make_right(s: String) -> Either<i32, String> {
    Either::Right(s)
}

/// Test whether an Either value was parsed as Left (integer).
/// @param value Either an integer or a string from R.
#[miniextendr]
pub fn either_is_left(value: Either<i32, String>) -> bool {
    value.is_left()
}

/// Test whether an Either value was parsed as Right (string).
/// @param value Either an integer or a string from R.
#[miniextendr]
pub fn either_is_right(value: Either<i32, String>) -> bool {
    value.is_right()
}

/// Test nested Either dispatch: Either<bool, Either<i32, String>>.
/// @param value Nested Either value from R.
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

/// Test that zero is correctly represented as Left(0) in Either.
#[miniextendr]
pub fn either_zero() -> Either<i32, String> {
    Either::Left(0)
}
