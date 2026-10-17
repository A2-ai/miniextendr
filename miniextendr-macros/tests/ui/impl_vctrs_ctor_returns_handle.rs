//! Explicit pointer constructors cannot supply a vctrs vector payload.
use miniextendr_macros::miniextendr;

struct Currency;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "$"))]
impl Currency {
    pub fn new() -> miniextendr_api::ExternalPtr<Self> {
        unimplemented!()
    }
}

fn main() {}
