//! Test: `convert` on a `&mut` parameter needs the converter's
//! `NATIVE_BORROW`: the default `None` would let the wrapper's alias check
//! skip it (#2110).

use miniextendr_api::from_r::SexpError;
use miniextendr_api::{SEXP, TryFromSexp, miniextendr};

pub struct Buffer(Vec<i32>);

impl TryFromSexp for &mut Buffer {
    type Error = SexpError;

    fn try_from_sexp(_sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(Box::leak(Box::new(Buffer(Vec::new()))))
    }
}

#[miniextendr]
pub fn fill(#[miniextendr(convert)] buffer: &mut Buffer) {
    buffer.0.push(1);
}

fn main() {}
