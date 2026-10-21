use miniextendr_api::miniextendr;

#[miniextendr]
pub unsafe fn unchecked(value: i32) -> i32 {
    value
}

fn main() {}
