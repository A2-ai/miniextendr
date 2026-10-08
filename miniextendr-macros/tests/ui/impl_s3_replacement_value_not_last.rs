//! Test: an impl-block S3 method for a replacement generic must take the new
//! value last, as `value`; a trailing `&Dots` would make `...` the last
//! formal (#1853).

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Slots {
    values: Vec<f64>,
}

#[miniextendr(s3)]
impl Slots {
    pub fn new() -> Self {
        Slots { values: Vec::new() }
    }

    #[miniextendr(s3(generic = "[<-"))]
    pub fn set_many(
        &mut self,
        i: Vec<i32>,
        value: Vec<f64>,
        _rest: &miniextendr_api::dots::Dots,
    ) {
        let _ = (i, value, &self.values);
    }
}

fn main() {}
