use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn conflicting_name(__miniextendr_dots: i32, ...) -> i32 {
    __miniextendr_dots
}

fn main() {}
