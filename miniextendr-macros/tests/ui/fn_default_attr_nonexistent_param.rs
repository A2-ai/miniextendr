use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(defaults(ghost = "42"))]
    pub fn bad_default(&self, x: i32) -> i32 {
        x
    }
}

fn main() {}
