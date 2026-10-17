use miniextendr_macros::miniextendr;

#[miniextendr(s3(generic = "::as_tibble", class = "rows"))]
fn empty_package() {}

#[miniextendr(s3(generic = "bad_pkg::as_tibble", class = "rows"))]
fn invalid_package() {}

#[miniextendr(s3(generic = "tibble::as_tibble::extra", class = "rows"))]
fn nested_qualifier() {}

#[miniextendr(s3(generic = "tibble::", class = "rows"))]
fn empty_generic() {}

fn main() {}
