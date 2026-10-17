use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Collector;

#[miniextendr(r6)]
impl Collector {
    /// @param rest,... Values.
    pub fn collect(&self, rest: miniextendr_api::LazyDots) -> i32 {
        let _ = rest;
        0
    }
}

fn main() {}
