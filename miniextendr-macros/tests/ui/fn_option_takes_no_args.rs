use miniextendr_macros::miniextendr;

#[miniextendr(invisible(true))]
pub fn bad_option() -> i32 {
    42
}

fn main() {}
