use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(typo_option)]
    pub fn bad_attr(&self) -> i32 {
        42
    }
}

fn main() {}
