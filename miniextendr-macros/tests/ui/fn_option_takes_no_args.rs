use miniextendr_macros::miniextendr;

#[miniextendr(invisible("string_value"))]
pub fn bad_option() -> i32 {
    42
}

fn main() {}
