use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(as = "tibble")]
    pub fn bad_coercion(&self) -> String {
        "test".to_string()
    }
}

fn main() {}
