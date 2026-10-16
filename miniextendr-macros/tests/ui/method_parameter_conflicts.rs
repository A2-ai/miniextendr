use miniextendr_api::miniextendr;

struct Inherent;
#[miniextendr(env)]
impl Inherent {
    #[miniextendr(no_na(n))]
    pub fn mixed(&self, #[miniextendr(coerce)] n: i32) -> i32 { n }
}

#[miniextendr]
trait InvalidDeclaration {
    fn invalid(&self, #[miniextendr(several_ok)] n: i32) -> i32;
}

struct Implementer;
#[miniextendr]
trait ValidDeclaration { fn value(&self, n: i32) -> i32; }
#[miniextendr(env)]
impl ValidDeclaration for Implementer {
    #[miniextendr(defaults(n = "1L"))]
    fn value(&self, #[miniextendr(no_na)] n: i32) -> i32 { n }
}

struct Receiver;
#[miniextendr(env)]
impl Receiver {
    pub fn value(#[miniextendr(no_na)] &self) -> i32 { 0 }
}

fn main() {}
