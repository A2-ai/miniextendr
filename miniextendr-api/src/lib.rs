pub use miniextendr_macros::miniextendr;
pub use miniextendr_macros::miniextendr_module;

pub mod ffi;

pub mod dots {
    use crate::ffi::SEXP;

    #[derive(Debug)]
    pub struct Dots {
        // Dots is always passed to us, they need no protection.
        pub inner: SEXP,
    }
}
