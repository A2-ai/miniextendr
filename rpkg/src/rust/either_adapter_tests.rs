//! Either adapter tests
use miniextendr_api::either_impl::Either;
use miniextendr_api::{AsNumericVec, DataFrame, miniextendr};

/// `num:<length>` for numbers, `frame:<rows>x<cols>` for a data frame.
fn describe_numbers_or_frame(numbers: Either<usize, DataFrame>) -> String {
    match numbers {
        Either::Left(n) => format!("num:{n}"),
        Either::Right(frame) => format!("frame:{}x{}", frame.nrow(), frame.ncol()),
    }
}

/// Numbers or a data frame: a list that is not a data frame is refused by
/// both arms.
/// @param x A numeric vector or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn either_num_or_df(x: Either<Vec<f64>, DataFrame>) -> String {
    describe_numbers_or_frame(x.map_left(|v| v.len()))
}

/// Numbers read like `as.numeric()`, behind a newtype the macro knows
/// nothing about: its argument errors say what it accepts only through the
/// conversion error.
#[derive(miniextendr_api::TryFromSexp)]
pub struct OpaqueNumbers(pub AsNumericVec);

/// Opaque numbers or a data frame: the expectation comes from the two arms'
/// errors at run time.
/// @param x Numbers, strings or factor labels, or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn either_opaque_or_df(x: Either<OpaqueNumbers, DataFrame>) -> String {
    describe_numbers_or_frame(x.map_left(|v| v.0.0.len()))
}

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

/// `no_na` on an `Either` with a vector arm and a scalar arm: the message
/// says `contain`, since the value may hold several.
/// @param x A raw vector, or a number that is not missing.
/// @noRd
#[miniextendr(noexport)]
pub fn either_no_na_raw_or_number(
    #[miniextendr(no_na)] x: Either<Vec<u8>, miniextendr_api::AsNumeric>,
) -> String {
    match x {
        Either::Left(raw) => format!("raw:{}", raw.len()),
        Either::Right(n) => format!("number:{}", n.0.unwrap_or(f64::NAN)),
    }
}
