//! Test (#1682): `Option<T>` / `Vec<T>` of a type that converts from R but
//! is neither a `#[derive(TryFromSexp)]` newtype nor a string-parsed type
//! fails with E0277 naming `TryFromSexpElement`, not with E0275 (overflow).
//! The container blankets bound the element's inner type through helper
//! traits whose self type is the projection itself, so an element that does
//! not implement the trait stops the search at once.

use miniextendr_api::from_r::SexpError;
use miniextendr_api::{SEXP, TryFromSexp, miniextendr};

pub struct Local;

impl TryFromSexp for Local {
    type Error = SexpError;
    fn try_from_sexp(_: SEXP) -> Result<Self, SexpError> {
        Ok(Local)
    }
}

#[miniextendr]
pub fn optional_local(_x: Option<Local>) {}

#[miniextendr]
pub fn local_vector(_x: Vec<Local>) {}

fn main() {}
