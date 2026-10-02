//! Matrix product and square linear solve through R's own BLAS and LAPACK
//! (`blas-lapack` feature).
//!
//! [`matrix_product`] and [`solve`] wrap [`dgemm_`](crate::sys::dgemm_) and
//! [`dgesv_`](crate::sys::dgesv_): the BLAS and LAPACK R itself was built
//! with, so a package gets R's numerics (and whatever optimised BLAS the R
//! installation uses) without bundling a second library.
//!
//! Both take column-major `f64` buffers with their dimensions and return a
//! new column-major `Vec<f64>`:
//!
//! - The inputs are borrowed and never modified. `solve` factors copies.
//! - Every dimension is checked against R's 32-bit BLAS/LAPACK INTEGER and
//!   every buffer length against its dimensions, with overflow-checked
//!   arithmetic, before any Fortran call. R's `xerbla`, which raises an R
//!   error (a longjmp) on an invalid argument, is therefore never reached.
//! - Zero-size cases are answered in Rust without calling Fortran.
//!
//! Like the raw routines, they must run on R's main thread (from a
//! miniextendr worker the call is routed there) in a process where R is
//! initialised: inside an R package, or after `miniextendr-engine` has
//! started R in a standalone binary.
//!
//! ```ignore
//! use miniextendr_api::linalg::{matrix_product, solve};
//!
//! // [1 3 5]   [1 4]   [22 49]
//! // [2 4 6] * [2 5] = [28 64]
//! //           [3 6]
//! let c = matrix_product(2, 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
//!                        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])?;
//! assert_eq!(c, [22.0, 28.0, 49.0, 64.0]);
//!
//! // [2 1] x = [3]  =>  x = [1, 1]
//! // [1 3]     [4]
//! let x = solve(2, 1, &[2.0, 1.0, 1.0, 3.0], &[3.0, 4.0])?;
//! ```

use std::fmt;
use std::os::raw::c_int;

use crate::sys;

/// Why [`matrix_product`] or [`solve`] refused its input or failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinalgError {
    /// A dimension exceeds R's 32-bit BLAS/LAPACK INTEGER (`i32::MAX`).
    DimensionTooLarge {
        /// Which dimension (`"m"`, `"n"`, `"k"`, `"nrhs"`).
        what: &'static str,
        /// The dimension given.
        value: usize,
    },
    /// The element count of a matrix overflows `usize`.
    SizeOverflow {
        /// Which matrix (`"a"`, `"b"`, `"result"`).
        what: &'static str,
    },
    /// A buffer's length does not match its dimensions.
    LengthMismatch {
        /// Which buffer (`"a"`, `"b"`).
        what: &'static str,
        /// Rows times columns.
        expected: usize,
        /// The buffer's length.
        actual: usize,
    },
    /// DGESV found the pivot `U(index, index)` (1-based) exactly zero: `A`
    /// is singular and no solution was computed (DGESV `INFO > 0`).
    Singular {
        /// 1-based index of the zero pivot.
        index: usize,
    },
    /// DGESV reported argument `position` (1-based) as invalid (`INFO < 0`).
    /// Under R this is not reached: R's `xerbla` raises an R error first,
    /// and the arguments are validated before the call anyway.
    InvalidArgument {
        /// 1-based position of the invalid argument.
        position: usize,
    },
}

impl fmt::Display for LinalgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionTooLarge { what, value } => write!(
                f,
                "dimension `{what}` = {value} exceeds R's 32-bit BLAS/LAPACK integer"
            ),
            Self::SizeOverflow { what } => {
                write!(f, "the element count of `{what}` overflows usize")
            }
            Self::LengthMismatch {
                what,
                expected,
                actual,
            } => write!(
                f,
                "`{what}` has {actual} elements, its dimensions need {expected}"
            ),
            Self::Singular { index } => write!(
                f,
                "matrix is exactly singular: U[{index},{index}] = 0 (DGESV)"
            ),
            Self::InvalidArgument { position } => {
                write!(f, "DGESV reported argument {position} as invalid")
            }
        }
    }
}

impl std::error::Error for LinalgError {}

/// The dimension as R's BLAS/LAPACK INTEGER.
fn blas_int(what: &'static str, value: usize) -> Result<c_int, LinalgError> {
    c_int::try_from(value).map_err(|_| LinalgError::DimensionTooLarge { what, value })
}

/// Checks that `buf` holds exactly `rows * cols` elements.
fn check_len(what: &'static str, rows: usize, cols: usize, buf: &[f64]) -> Result<(), LinalgError> {
    let expected = elements(what, rows, cols)?;
    if buf.len() == expected {
        Ok(())
    } else {
        Err(LinalgError::LengthMismatch {
            what,
            expected,
            actual: buf.len(),
        })
    }
}

fn elements(what: &'static str, rows: usize, cols: usize) -> Result<usize, LinalgError> {
    rows.checked_mul(cols)
        .ok_or(LinalgError::SizeOverflow { what })
}

/// The matrix product `A B` through R's BLAS (`dgemm`).
///
/// `a` is `A`, `m x k`, and `b` is `B`, `k x n`, both column-major; the
/// result is `A B`, `m x n`, column-major. Neither input is modified.
///
/// With `m = 0` or `n = 0` the result is empty; with `k = 0` it is `m x n`
/// zeros, as for R's `%*%`. Neither case calls the BLAS.
///
/// This is DGEMM's result. With NaN or infinite inputs it can differ from
/// R's `%*%`, which checks for NA/NaN first and then uses its own loop: a
/// BLAS may skip terms whose multiplier is zero, so `0 * Inf` and `0 * NaN`
/// can vanish from a sum.
///
/// # Errors
///
/// [`LinalgError::DimensionTooLarge`] when a dimension exceeds `i32::MAX`,
/// [`LinalgError::SizeOverflow`] when an element count overflows `usize`,
/// and [`LinalgError::LengthMismatch`] when `a.len() != m * k` or
/// `b.len() != k * n`.
///
/// # Panics
///
/// Off R's main thread and outside a miniextendr worker (the checked
/// [`sys::dgemm_`] cannot route the call).
pub fn matrix_product(
    m: usize,
    n: usize,
    k: usize,
    a: &[f64],
    b: &[f64],
) -> Result<Vec<f64>, LinalgError> {
    let (m_int, n_int, k_int) = (blas_int("m", m)?, blas_int("n", n)?, blas_int("k", k)?);
    check_len("a", m, k, a)?;
    check_len("b", k, n, b)?;
    let len = elements("result", m, n)?;
    if len == 0 || k == 0 {
        return Ok(vec![0.0; len]);
    }
    let mut c = vec![0.0; len];
    let (alpha, beta) = (1.0, 0.0);
    let no_trans = c"N".as_ptr();
    // SAFETY: all dimensions are positive and fit c_int; a is m x k with
    // lda = m, b is k x n with ldb = k, c is m x n with ldc = m (lengths
    // checked above); c is a fresh buffer, so it aliases neither input. The
    // flags are 'N', valid, so xerbla is not called. One hidden length per
    // flag.
    unsafe {
        sys::dgemm_(
            no_trans,
            no_trans,
            &m_int,
            &n_int,
            &k_int,
            &alpha,
            a.as_ptr(),
            &m_int,
            b.as_ptr(),
            &k_int,
            &beta,
            c.as_mut_ptr(),
            &m_int,
            1,
            1,
        );
    }
    Ok(c)
}

/// Solve `A X = B` for a square `A` through R's LAPACK (`dgesv`, LU with
/// partial pivoting).
///
/// `a` is `A`, `n x n`, and `b` is `B`, `n x nrhs`, both column-major; the
/// result is `X`, `n x nrhs`, column-major. Neither input is modified: the
/// factorisation works on copies.
///
/// With `n = 0` or `nrhs = 0` there is nothing to solve: the result is empty
/// and LAPACK is not called, so `A` is not inspected (`base::solve()`
/// errors in both cases instead).
///
/// There is no fallback (no pseudoinverse, no regularisation), and only an
/// exactly zero pivot is reported: unlike `base::solve()`, which also
/// rejects a reciprocal condition number below `tol`, a nearly singular `A`
/// returns an inaccurate `X`.
///
/// # Errors
///
/// [`LinalgError::Singular`] when DGESV finds an exactly zero pivot,
/// [`LinalgError::DimensionTooLarge`] when `n` or `nrhs` exceeds
/// `i32::MAX`, [`LinalgError::SizeOverflow`] when an element count
/// overflows `usize`, and [`LinalgError::LengthMismatch`] when
/// `a.len() != n * n` or `b.len() != n * nrhs`.
///
/// # Panics
///
/// Off R's main thread and outside a miniextendr worker (the checked
/// [`sys::dgesv_`] cannot route the call).
pub fn solve(n: usize, nrhs: usize, a: &[f64], b: &[f64]) -> Result<Vec<f64>, LinalgError> {
    let (n_int, nrhs_int) = (blas_int("n", n)?, blas_int("nrhs", nrhs)?);
    check_len("a", n, n, a)?;
    check_len("b", n, nrhs, b)?;
    if n == 0 || nrhs == 0 {
        return Ok(Vec::new());
    }
    let mut lu = a.to_vec();
    let mut x = b.to_vec();
    let mut ipiv: Vec<c_int> = vec![0; n];
    let mut info: c_int = 0;
    // SAFETY: n and nrhs are positive and fit c_int; lu is n x n with
    // lda = n, x is n x nrhs with ldb = n (lengths checked above), ipiv
    // holds n entries. All three are separate fresh buffers, so DGESV's
    // writes alias neither each other nor the caller's data, and valid
    // arguments mean xerbla is not called.
    unsafe {
        sys::dgesv_(
            &n_int,
            &nrhs_int,
            lu.as_mut_ptr(),
            &n_int,
            ipiv.as_mut_ptr(),
            x.as_mut_ptr(),
            &n_int,
            &mut info,
        );
    }
    let position = || usize::try_from(info.unsigned_abs()).expect("a c_int magnitude fits usize");
    match info {
        0 => Ok(x),
        1.. => Err(LinalgError::Singular { index: position() }),
        _ => Err(LinalgError::InvalidArgument {
            position: position(),
        }),
    }
}
