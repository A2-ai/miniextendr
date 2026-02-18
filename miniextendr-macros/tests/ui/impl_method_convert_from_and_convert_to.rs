use miniextendr_macros::miniextendr;

struct TypeA;

#[miniextendr(s7)]
impl TypeA {
    #[miniextendr(s7(convert_from = "TypeB", convert_to = "TypeC"))]
    pub fn conflicting(&self) {}
}

fn main() {}
