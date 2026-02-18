use miniextendr_macros::miniextendr;

struct MyVctr;

#[miniextendr(s3)]
impl MyVctr {
    #[miniextendr(vctrs(invalid_protocol))]
    pub fn bad_method(&self) -> String {
        "test".to_string()
    }
}

fn main() {}
