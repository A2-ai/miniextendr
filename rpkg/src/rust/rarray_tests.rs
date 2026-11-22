//! Test fixtures for RArray/RMatrix/RVector.

use std::cell::Cell;

use miniextendr_api::SexpError;
use miniextendr_api::expression::RCall;
use miniextendr_api::list::List;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::prelude::*;
use miniextendr_api::rarray::{RMatrix, RVector};
use miniextendr_api::{RNativeType, SEXPTYPE};

fn strings_from_sexp(sexp: SEXP) -> Vec<String> {
    Vec::<String>::try_from_sexp(sexp).expect("expected character names")
}

/// Get dimensions of a matrix as integer vector.
/// @param x A double matrix.
#[miniextendr]
pub fn rarray_matrix_dims(x: RMatrix<f64>) -> [i32; 2] {
    // R stores each dimension in an integer vector.
    unsafe { x.dims() }.map(|n| i32::try_from(n).unwrap())
}

/// Get total length of a matrix.
/// @param x A double matrix.
#[miniextendr]
pub fn rarray_matrix_len(x: RMatrix<f64>) -> usize {
    x.len()
}

/// Sum all elements of a numeric vector via RVector.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr]
pub fn rarray_vector_sum(x: SEXP) -> f64 {
    let vec = unsafe { RVector::<f64>::from_sexp(x).expect("expected numeric vector") };
    let slice = unsafe { vec.as_slice() };
    slice.iter().sum()
}

/// Copy one column of a double matrix to an ordinary, unnamed double vector.
/// @param x A double matrix.
/// @param col 1-based column index. Errors if `col` is not in `1..=ncol`.
#[miniextendr]
pub fn rarray_matrix_column(x: RMatrix<f64>, col: i32) -> Result<Preserved, SexpError> {
    if col < 1 {
        let ncol = unsafe { x.ncol() };
        return Err(SexpError::InvalidValue(format!(
            "column {col} is out of bounds (must be a positive 1-based index, matrix has {ncol} columns)"
        )));
    }
    // The .Call arguments root x for the full read on R's main thread.
    unsafe { x.copy_column_to_r(usize::try_from(col - 1).unwrap()) }
}

/// Read one double matrix cell.
/// @param x A double matrix.
/// @param row,col 1-based row and column indices.
#[miniextendr(noexport)]
pub fn rarray_matrix_cell(x: RMatrix<f64>, row: usize, col: usize) -> f64 {
    let row = row
        .checked_sub(1)
        .expect("row must be a positive 1-based index");
    let col = col
        .checked_sub(1)
        .expect("column must be a positive 1-based index");
    unsafe { x.get_rc(row, col) }
}

/// Does this matrix provider currently expose a contiguous double buffer?
/// @param x A double matrix.
#[miniextendr(noexport)]
pub fn rarray_matrix_has_buffer(x: RMatrix<f64>) -> bool {
    !unsafe { miniextendr_api::sys::REAL_OR_NULL(x.as_sexp()) }.is_null()
}

/// Run `f::<T>` for the native element type of `x`.
fn with_native_type<R>(x: SEXP, f: impl NativeTypeFn<R>) -> Result<R, SexpError> {
    use miniextendr_api::{RLogical, Rcomplex};
    match x.type_of() {
        SEXPTYPE::INTSXP => f.call::<i32>(x),
        SEXPTYPE::REALSXP => f.call::<f64>(x),
        SEXPTYPE::LGLSXP => f.call::<RLogical>(x),
        SEXPTYPE::RAWSXP => f.call::<u8>(x),
        SEXPTYPE::CPLXSXP => f.call::<Rcomplex>(x),
        other => Err(SexpError::InvalidValue(format!(
            "expected a native vector, got {other:?}"
        ))),
    }
}

/// A function generic over the native element type.
trait NativeTypeFn<R> {
    fn call<T: RNativeType>(self, x: SEXP) -> Result<R, SexpError>;
}

/// Copy one column of a matrix of any native type; the result keeps the type.
/// @param x An integer, double, logical, raw or complex matrix.
/// @param col 1-based column index.
#[miniextendr(noexport)]
pub fn rarray_native_matrix_column(x: SEXP, col: usize) -> Result<Preserved, SexpError> {
    struct Column(usize);
    impl NativeTypeFn<Preserved> for Column {
        fn call<T: RNativeType>(self, x: SEXP) -> Result<Preserved, SexpError> {
            let matrix = RMatrix::<T>::try_from_sexp(x)?;
            let index = self
                .0
                .checked_sub(1)
                .ok_or_else(|| SexpError::InvalidValue("column indices start at 1".into()))?;
            // The .Call arguments root x for the full read on R's main thread.
            unsafe { matrix.copy_column_to_r(index) }
        }
    }
    with_native_type(x, Column(col))
}

/// Copy one column of an integer matrix. An `RMatrix` argument is not
/// materialized (a bare `SEXP` one is), so a compact matrix is read through
/// its regions.
/// @param x An integer matrix.
/// @param col 1-based column index.
#[miniextendr(noexport)]
pub fn rarray_int_matrix_column(x: RMatrix<i32>, col: usize) -> Result<Preserved, SexpError> {
    let index = col
        .checked_sub(1)
        .ok_or_else(|| SexpError::InvalidValue("column indices start at 1".into()))?;
    // The .Call arguments root x for the full read on R's main thread.
    unsafe { x.copy_column_to_r(index) }
}

/// Does this integer matrix currently expose a buffer? Asking doesn't
/// materialize it.
/// @param x An integer matrix.
#[miniextendr(noexport)]
pub fn rarray_int_matrix_has_buffer(x: RMatrix<i32>) -> bool {
    !unsafe { i32::dataptr_or_null(x.as_sexp()) }.is_null()
}

/// A matrix provider that fills at most two values per region call.
#[derive(miniextendr_api::AltrepReal)]
#[altrep(manual, class = "MatrixColumnRegions")]
pub struct MatrixColumnRegions {
    stall: bool,
    signal_once: Cell<bool>,
}

impl AltrepLen for MatrixColumnRegions {
    fn altrep_len(&self) -> usize {
        10
    }
}

impl AltRealData for MatrixColumnRegions {
    fn elt(&self, i: usize) -> f64 {
        if i == 7 && self.signal_once.replace(false) {
            // The default ALTREP guard and eval_with_handlers propagate R's
            // original condition to the enclosing Elt or Get_region caller.
            unsafe {
                let base = miniextendr_api::sys::R_BaseEnv;
                let condition = OwnedProtect::new(
                    RCall::new("errorCondition")
                        .arg_value("matrix provider read failed")
                        .named_value("class", "matrix_provider_error")
                        .named_value("offset", i32::try_from(i).unwrap())
                        .eval_with_handlers(base),
                );
                RCall::new("stop")
                    .arg(condition.get())
                    .eval_with_handlers(base);
            }
        }
        f64::from(u8::try_from(i).unwrap())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        if self.stall && start >= 7 {
            return 0;
        }
        // Force an R allocation while the caller holds its unfinished column.
        let count = len.min(2).min(self.altrep_len().saturating_sub(start));
        if count > 0 {
            // Same size and R allocation class as the unfinished output.
            let _churn = [-999.0_f64; 5].into_sexp();
            for (i, slot) in buf[..count].iter_mut().enumerate() {
                *slot = self.elt(start + i);
            }
        }
        count
    }
}

/// Create a 5-by-2 double ALTREP matrix with short region reads.
/// @param stall Whether the second column stops after its first two values.
/// @param signal Whether R signals its own error once at zero-based offset 7.
#[miniextendr(noexport)]
pub fn rarray_region_matrix(
    stall: bool,
    #[miniextendr(default = "FALSE")] signal: bool,
) -> Preserved {
    let matrix = Preserved::new(MatrixColumnRegions {
        stall,
        signal_once: Cell::new(signal),
    });
    let dims = unsafe { OwnedProtect::new([5_i32, 2].into_sexp()) };
    matrix.as_sexp().set_dim(dims.get());
    matrix
}

/// Hold a Rust drop sentinel across a read that may signal an R condition.
/// @param x A matrix from `rarray_region_matrix()`.
/// @param column Read its second column or, if false, the third cell of that column.
#[miniextendr(noexport)]
pub fn rarray_guarded_read(x: RMatrix<f64>, column: bool) -> Result<Preserved, SexpError> {
    let _sentinel = crate::quoted_tests::DropSentinel;
    if column {
        unsafe { x.copy_column_to_r(1) }
    } else {
        Ok(Preserved::new(unsafe { x.get_rc(2, 1) }))
    }
}

/// Return row names through `RMatrix::get_rownames`.
/// @param x A double matrix.
#[miniextendr(noexport)]
pub fn rarray_matrix_rownames(x: RMatrix<f64>) -> Vec<String> {
    let names = unsafe { x.get_rownames().expect("expected row names") };
    strings_from_sexp(names)
}

/// Return column names through `RMatrix::get_colnames`.
/// @param x A double matrix.
#[miniextendr(noexport)]
pub fn rarray_matrix_colnames(x: RMatrix<f64>) -> Vec<String> {
    let names = unsafe { x.get_colnames().expect("expected column names") };
    strings_from_sexp(names)
}

/// Return row names through `List::get_rownames` for a VECSXP matrix.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr(noexport)]
pub fn list_matrix_rownames(x: SEXP) -> Vec<String> {
    let list = List::try_from_sexp(x).expect("expected list matrix");
    strings_from_sexp(list.get_rownames().expect("expected row names"))
}

/// Return column names through `List::get_colnames` for a VECSXP matrix.
/// @param x An R vector or matrix accepted by the selected helper.
#[miniextendr(noexport)]
pub fn list_matrix_colnames(x: SEXP) -> Vec<String> {
    let list = List::try_from_sexp(x).expect("expected list matrix");
    strings_from_sexp(list.get_colnames().expect("expected column names"))
}
