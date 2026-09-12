use miniextendr_api::miniextendr;

#[miniextendr(serialize)]
pub unsafe extern "C-unwind" fn raw(value: miniextendr_api::SEXP) -> miniextendr_api::SEXP {
    value
}

fn main() {}
