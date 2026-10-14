//! Test: two methods of one impl block marked as the constructor are refused.

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Shape {
    area: f64,
}

#[miniextendr(s7)]
impl Shape {
    #[miniextendr(s7(constructor))]
    pub fn circle(radius: f64) -> Self {
        Shape {
            area: 3.14 * radius * radius,
        }
    }

    #[miniextendr(s7(constructor))]
    pub fn square(side: f64) -> Self {
        Shape { area: side * side }
    }
}

fn main() {}
