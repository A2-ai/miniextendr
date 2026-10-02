//! A plain cargo binary that depends on miniextendr-api (with `blas-lapack`)
//! and calls R's BLAS/LAPACK. This proves that miniextendr-api's build
//! script link directives (`R CMD config LAPACK_LIBS` / `BLAS_LIBS`) reach a
//! dependent's targets, and that the binary runs once R is started through
//! `miniextendr-engine`: R's `xerbla` lives in libR, so the BLAS needs an
//! initialised R. `harness = false` (see Cargo.toml) keeps R on the main
//! thread, as in a CLI.

use miniextendr_api::linalg::{LinalgError, matrix_product, solve};
use miniextendr_api::sys::{dgemm_, dgesv_};
use std::os::raw::c_int;

fn main() {
    // SAFETY: R is started once, on this (the main) thread, before any R
    // call. The engine has no shutdown: R stays up until the process exits.
    let _engine = unsafe {
        miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .interactive(false)
            .signal_handlers(false)
            .init()
            .expect("failed to start R")
    };
    miniextendr_api::miniextendr_runtime_init();

    // Raw dgemm_ with a transposed A: A' B for A = [1 2; 3 4] (column-major
    // [1, 3, 2, 4]) and B = I gives A' = [1 3; 2 4].
    let (n, one, zero) = (2, 1.0, 0.0);
    let a = [1.0, 3.0, 2.0, 4.0];
    let b = [1.0, 0.0, 0.0, 1.0];
    let mut c = [0.0; 4];
    // SAFETY: valid 2 x 2 operands with leading dimension 2, a fresh output,
    // valid flags and one hidden length per flag; R is initialised.
    unsafe {
        dgemm_(
            c"T".as_ptr(),
            c"N".as_ptr(),
            &n,
            &n,
            &n,
            &one,
            a.as_ptr(),
            &n,
            b.as_ptr(),
            &n,
            &zero,
            c.as_mut_ptr(),
            &n,
            1,
            1,
        );
    }
    assert_eq!(c, [1.0, 2.0, 3.0, 4.0]);

    // Raw dgesv_: [0 1; 1 0] x = [2; 3] needs a row interchange.
    let mut lu = [0.0, 1.0, 1.0, 0.0];
    let mut x = [2.0, 3.0];
    let mut ipiv: [c_int; 2] = [0; 2];
    let (nrhs, mut info) = (1, -1);
    // SAFETY: valid 2 x 2 and 2 x 1 buffers with leading dimension 2.
    unsafe {
        dgesv_(
            &n,
            &nrhs,
            lu.as_mut_ptr(),
            &n,
            ipiv.as_mut_ptr(),
            x.as_mut_ptr(),
            &n,
            &mut info,
        );
    }
    assert_eq!((info, x), (0, [3.0, 2.0]));

    // The safe adapters.
    let product = matrix_product(2, 1, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[1.0, 1.0, 1.0]);
    assert_eq!(product, Ok(vec![9.0, 12.0]));
    assert_eq!(
        solve(2, 1, &[2.0, 1.0, 1.0, 3.0], &[3.0, 4.0]),
        Ok(vec![1.0, 1.0])
    );
    assert_eq!(
        solve(2, 1, &[1.0, 2.0, 2.0, 4.0], &[1.0, 1.0]),
        Err(LinalgError::Singular { index: 2 })
    );

    println!("blas_lapack_link: R's BLAS/LAPACK linked and callable from a dependent binary");
}
