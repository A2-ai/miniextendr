use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn conflicting_name(_dots: i32, ...) -> i32 {
    _dots
}

fn main() {}
