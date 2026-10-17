use miniextendr_api::miniextendr;

trait Reader {}
struct Container;

#[miniextendr(s3(r_data_accessors, fallback = "Self::column"))]
impl Reader for Container {}

fn main() {}
