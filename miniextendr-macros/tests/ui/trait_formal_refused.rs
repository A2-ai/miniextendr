//! Test: `formal(...)` (#1842) is refused on trait methods, declared or
//! implemented: a trait method's R formals are the trait declaration's
//! parameter names, which every implementing type shares.

use miniextendr_macros::miniextendr;

struct Dummy;

#[miniextendr]
trait Declared {
    #[miniextendr(formal(conf_level = "conf.level"))]
    fn interval(&self, conf_level: f64) -> f64;
}

#[miniextendr]
trait Scale {
    fn scale(&self, x_factor: f64) -> f64;
}

#[miniextendr(env)]
impl Scale for Dummy {
    #[miniextendr(formal(x_factor = "x.factor"))]
    fn scale(&self, x_factor: f64) -> f64 {
        x_factor
    }
}

fn main() {}
