//! Test: `s7(dispatch = "...")` names the receiver and then the method's
//! leading parameters. `y` is not the method's first parameter (`other` is),
//! so S7 would reject the method at load; it is a compile error instead.

use miniextendr_macros::miniextendr;

struct Dog {
    age: i32,
}

#[miniextendr(s7)]
impl Dog {
    #[miniextendr(s7(dispatch = "x, y"))]
    fn compare(&self, other: i32) -> i32 {
        self.age - other
    }
}

fn main() {}
