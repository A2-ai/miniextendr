//! Test: an S7 fast-path shortcut name that collides with another generated
//! function (here a static method aliased via `r_name`) is a compile error.

use miniextendr_macros::miniextendr;

struct Foo {
    value: i32,
}

#[miniextendr(s7)]
impl Foo {
    // Instance method emits the shortcut `Foo_value`.
    fn value(&self) -> i32 {
        self.value
    }

    // Static method renamed to `value` emits `Foo_value` too — collision.
    #[miniextendr(s7(r_name = "value"))]
    fn make() -> i32 {
        0
    }
}

fn main() {}
