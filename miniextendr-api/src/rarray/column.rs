//! Copying one selected matrix column into ordinary R storage.

use crate::from_r::SexpError;
use crate::{Preserved, R_xlen_t, RNativeType, SEXP, SexpExt, sys};

use super::RMatrix;

impl<T: RNativeType> RMatrix<T> {
    /// Copy one zero-based column into a rooted, ordinary R vector of the
    /// matrix's type (integer, double, logical, raw or complex).
    ///
    /// The result contains the column's values without matrix attributes. An
    /// ordinary matrix or ALTREP with an exposed buffer takes one bulk copy.
    /// An ALTREP without a buffer is read in bounded regions
    /// ([`RNativeType::get_region`]). A provider can allocate or materialize
    /// its own backing during a callback.
    ///
    /// # Errors
    ///
    /// Returns an error for an out-of-range column, inconsistent dimensions,
    /// or an ALTREP reader that returns an invalid count. R errors propagate
    /// to the enclosing R handler. Standalone Rust callers receive an error.
    ///
    /// # Safety
    ///
    /// Keep this matrix valid and rooted on R's main thread throughout the
    /// call. Its type and shape must remain valid across reentrant R code.
    pub unsafe fn copy_column_to_r(self, col: usize) -> Result<Preserved, SexpError> {
        // The result allocation and the provider can both signal. The bridge
        // lets Rust owners unwind before R resumes the original condition.
        let dim = self.as_sexp().get_dim();
        let _dim = unsafe { crate::OwnedProtect::new(dim) };
        unsafe { crate::unwind_protect::call_r_callback(|| copy_column(self, dim, col)) }?
    }
}

// Keep only plain stack data across R calls in the bridge. Its R context
// restores the protect stack automatically if a provider longjmps.
unsafe fn copy_column<T: RNativeType>(
    matrix: RMatrix<T>,
    dim: SEXP,
    col: usize,
) -> Result<Preserved, SexpError> {
    let src = matrix.as_sexp();
    // This whole operation is already inside the callback bridge.
    let [rows, cols] = unsafe { super::get_dims(src, dim) };
    if col >= cols {
        return Err(SexpError::InvalidValue(format!(
            "column index {col} out of bounds for a matrix with {cols} columns"
        )));
    }
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
    let out = unsafe { sys::Rf_protect(sys::Rf_allocVector(T::SEXP_TYPE, count)) };
    let result = if rows == 0 {
        Ok(())
    } else {
        // A non-empty vector: the pointer is R's real storage.
        let dst = unsafe { T::dataptr_mut(out) };
        let input = unsafe { T::dataptr_or_null(src) };
        if input.is_null() {
            unsafe { copy_regions(src, start, count, dst) }
        } else {
            // No R call intervenes between obtaining the exposed buffer and
            // copying. Destination is fresh and distinct from the input.
            unsafe { std::ptr::copy_nonoverlapping(input.offset(start), dst, rows) };
            Ok(())
        }
    };
    let result = result.map(|()| Preserved::new(out));
    unsafe { sys::Rf_unprotect(1) };
    result
}

unsafe fn copy_regions<T: RNativeType>(
    src: SEXP,
    start: R_xlen_t,
    count: R_xlen_t,
    dst: *mut T,
) -> Result<(), SexpError> {
    // Called only from the main-thread unwind trampoline. The scratch is
    // initialized, so a provider may expose its destination as a `&mut [T]`.
    let mut scratch = [T::R_NA; 256];
    let block_len = R_xlen_t::try_from(scratch.len()).expect("bounded scratch length");
    let mut done = 0;
    while done < count {
        let wanted = (count - done).min(block_len);
        let got = unsafe { T::get_region(src, start + done, wanted, scratch.as_mut_ptr()) };
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
