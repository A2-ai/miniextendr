//! Test: a standalone S3 method for a replacement generic must take the new
//! value last, as `value` (#1853).

use miniextendr_macros::miniextendr;

#[miniextendr(s3(generic = "$<-", class = "thing"))]
pub fn dollar_assign_thing(
    x: miniextendr_api::List,
    name: &str,
    new_value: miniextendr_api::SEXP,
) -> miniextendr_api::SEXP {
    let _ = (name, new_value);
    x.as_sexp()
}

fn main() {}
