//! Test: a `Quoted` parameter on a class method.
//!
//! Unevaluated arguments are a standalone-function feature (standalone S3
//! methods included); a class method's wrapper forces its arguments (#1835).

use miniextendr_api::{ExternalPtr, Quoted, miniextendr};

#[derive(ExternalPtr)]
pub struct Table {
    rows: i32,
}

#[miniextendr(env)]
impl Table {
    fn new(rows: i32) -> Self {
        Self { rows }
    }

    fn filter(&self, cond: Quoted) -> i32 {
        let _ = cond;
        self.rows
    }
}

fn main() {}
