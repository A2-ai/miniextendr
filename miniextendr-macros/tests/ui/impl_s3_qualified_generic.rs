use miniextendr_macros::miniextendr;

struct Rows;

#[miniextendr(s3)]
impl Rows {
    #[miniextendr(s3(generic = "bad_pkg::as_tibble"))]
    pub fn rows(&self) -> i32 { 1 }
}

fn main() {}
