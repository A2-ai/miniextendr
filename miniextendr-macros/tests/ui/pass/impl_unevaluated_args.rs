//! Class and trait methods capture unevaluated arguments.
use miniextendr_api::{ExternalPtr, Missing, Quosure, Quoted, miniextendr};

#[derive(ExternalPtr)]
pub struct Table { rows: i32 }

#[miniextendr(env)]
impl Table {
    fn new(rows: i32) -> Self { Self { rows } }
    fn filter(&self, cond: Quoted) -> i32 { let _ = cond; self.rows }
    fn tidy(&self, cond: Quosure) -> i32 { let _ = cond; self.rows }
    #[miniextendr(defaults(cond = "stop('unused default')"))]
    fn optional(&self, cond: Missing<Quoted>) -> bool { cond.is_missing() }
}

#[miniextendr]
pub trait Capture {
    fn capture(&self, cond: Quoted) -> bool { cond.as_name().is_some() }
    fn tidy_capture(&self, cond: Missing<Quosure>) -> bool { cond.is_missing() }
}
impl Capture for Table {}
#[miniextendr(env)]
impl Capture for Table {}

#[miniextendr]
pub trait GenericCapture<T> {
    fn generic_capture(&self, cond: Quoted) -> bool;
}
#[miniextendr(env)]
impl GenericCapture<i32> for Table {
    fn generic_capture(&self, cond: Quoted) -> bool { cond.as_name().is_some() }
}
fn main() {}
