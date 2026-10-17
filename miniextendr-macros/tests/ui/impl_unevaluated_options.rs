//! Methods must reject shapes/options that force an unevaluated promise.
use miniextendr_api::miniextendr;

pub struct Table;
#[miniextendr(env)]
impl Table {
    #[miniextendr(defaults(expr = "NULL"))]
    fn required(&self, expr: miniextendr_api::Quoted) { let _ = expr; }
}

#[miniextendr(env)]
impl Table {
    fn nested(&self, expr: Option<miniextendr_api::Quoted>) { let _ = expr; }
}

#[miniextendr]
pub trait Capture {
    fn checked(&self, #[miniextendr(no_na)] expr: miniextendr_api::Quosure);
}

#[miniextendr(env)]
impl Table {
    fn optional(
        &self,
        #[miniextendr(default = "NULL", no_na)] expr: miniextendr_api::Missing<miniextendr_api::Quoted>,
    ) { let _ = expr; }
}

fn main() {}
