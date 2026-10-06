//! Test (#1815): `#[try_from_sexp(...)]` takes only `validate = path`; any
//! other key is a spanned error on that key.

use miniextendr_api::TryFromSexp;

#[derive(TryFromSexp)]
#[try_from_sexp(check = plain_number)]
pub struct Elapsed(f64);

fn main() {}
