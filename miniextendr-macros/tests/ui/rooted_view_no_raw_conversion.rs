//! #1427: `StrVec` and `RCow` have no raw `TryFromSexp` impl. A copyable
//! `SEXP` carries no lifetime, so that impl let safe code pick any view
//! lifetime, `'static` included. `try_from_root` takes a `Root<'a>` instead.

use miniextendr_api::{RCow, SEXP, StrVec, TryFromSexp};

fn strings(sexp: SEXP) -> StrVec<'static> {
    StrVec::try_from_sexp(sexp).unwrap()
}

fn values(sexp: SEXP) -> RCow<'static, i32> {
    RCow::try_from_sexp(sexp).unwrap()
}

fn main() {}
