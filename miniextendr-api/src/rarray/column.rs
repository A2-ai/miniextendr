//! Copying one selected real column into ordinary R storage.

use std::ffi::c_void;

use crate::from_r::SexpError;
use crate::{Preserved, R_xlen_t, SEXP, SEXPTYPE, SexpExt, sys};

use super::RMatrix;

impl RMatrix<f64> {
    /// Copy one zero-based column into a rooted, ordinary R double vector.
    ///
    /// The result contains the column's values without matrix attributes. An
    /// ordinary matrix takes one bulk copy. For ALTREP, only the selected
    /// range is requested, using bounded scratch space. A provider can
    /// allocate or materialize its own backing during a region callback.
    ///
    /// # Errors
    ///
    /// Returns an error for an out-of-range column, inconsistent dimensions,
    /// or an ALTREP reader that returns an invalid count. R errors propagate
    /// to the enclosing boundary.
    ///
    /// # Safety
    ///
    /// Keep this matrix valid and rooted on R's main thread throughout the
    /// call, inside a miniextendr unwind boundary. Its type and shape must
    /// remain valid across any reentrant R code.
    pub unsafe fn copy_column_to_r(&self, col: usize) -> Result<Preserved, SexpError> {
        struct CopyColumn {
            matrix: RMatrix<f64>,
            col: usize,
            result: Option<Result<Preserved, SexpError>>,
        }

        unsafe extern "C-unwind" fn copy(data: *mut c_void) -> SEXP {
            let data = unsafe { &mut *data.cast::<CopyColumn>() };
            data.result = Some(unsafe { copy_column(data.matrix, data.col) });
            SEXP::nil()
        }

        let mut data = CopyColumn {
            matrix: *self,
            col,
            result: None,
        };
        // The callbacks for dimensions, length and regions can signal an R
        // error. Translate the jump here so the Rust caller's owners unwind.
        unsafe { crate::unwind_protect::call_unwinding(copy, (&raw mut data).cast()) };
        data.result.expect("the column copy callback returned")
    }
}

// Runs inside call_unwinding. Keep only plain stack data across R calls. Its
// R context restores the protect stack automatically if a provider longjmps.
unsafe fn copy_column(matrix: RMatrix<f64>, col: usize) -> Result<Preserved, SexpError> {
    let [rows, cols] = unsafe { matrix.dims() };
    if col >= cols {
        return Err(SexpError::InvalidValue(format!(
            "column index {col} out of bounds for a matrix with {cols} columns"
        )));
    }
    let src = matrix.as_sexp();
    let shape_len = rows
        .checked_mul(cols)
        .and_then(|n| R_xlen_t::try_from(n).ok());
    if shape_len != Some(src.xlength()) {
        return Err(SexpError::InvalidValue(
            "matrix dimensions do not match its length".into(),
        ));
    }
    // The shape check and col < cols bound both conversions by XLENGTH.
    let count = R_xlen_t::try_from(rows).expect("validated column length");
    let start = R_xlen_t::try_from(col * rows).expect("validated column offset");
    let out = unsafe { sys::Rf_protect(sys::Rf_allocVector(SEXPTYPE::REALSXP, count)) };
    let result = if rows == 0 {
        Ok(())
    } else {
        let dst = unsafe { sys::REAL(out) };
        if src.is_altrep() {
            unsafe { copy_regions(src, start, count, dst) }
        } else {
            // Both payloads are nonempty and distinct. Write into fresh
            // storage without creating a Rust slice of uninitialized values.
            let input = unsafe { sys::DATAPTR_RO(src).cast::<f64>().offset(start) };
            unsafe { std::ptr::copy_nonoverlapping(input, dst, rows) };
            Ok(())
        }
    };
    let result = result.map(|()| Preserved::new(out));
    unsafe { sys::Rf_unprotect(1) };
    result
}

unsafe fn copy_regions(
    src: SEXP,
    start: R_xlen_t,
    count: R_xlen_t,
    dst: *mut f64,
) -> Result<(), SexpError> {
    // Called only from the main-thread unwind trampoline. A provider may
    // expose its destination as an initialized &mut [f64].
    let mut scratch = [0.0; 256];
    let block_len = R_xlen_t::try_from(scratch.len()).expect("bounded scratch length");
    let mut done = 0;
    while done < count {
        let wanted = (count - done).min(block_len);
        let got = unsafe {
            sys::REAL_GET_REGION_unchecked(src, start + done, wanted, scratch.as_mut_ptr())
        };
        if !(1..=wanted).contains(&got) {
            return Err(SexpError::InvalidValue(format!(
                "ALTREP column reader returned {got} values for a request of {wanted}"
            )));
        }
        // The reader filled got slots, and done + got <= count.
        unsafe {
            std::ptr::copy_nonoverlapping(
                scratch.as_ptr(),
                dst.offset(done),
                usize::try_from(got).expect("positive region length"),
            );
        }
        done += got;
    }
    Ok(())
}
