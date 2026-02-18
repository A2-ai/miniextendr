use miniextendr_macros::miniextendr;

#[miniextendr(dots = typed_list!(x => numeric()))]
pub fn missing_variadic(a: i32) -> i32 {
    a
}

fn main() {}
