//! Test: a type with several receiver-less methods returning `Self` and no
//! declared constructor is refused, at each candidate.
//!
//! Without a method marked `<system>(constructor)` or named `new`, the one
//! receiver-less method returning `Self` is the constructor. With two, which
//! one is meant is a guess.

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Shape {
    area: f64,
}

#[miniextendr(r6)]
impl Shape {
    pub fn circle(radius: f64) -> Self {
        Shape {
            area: 3.14 * radius * radius,
        }
    }

    pub fn square(side: f64) -> Self {
        Shape { area: side * side }
    }

    // Not a candidate: a fallible factory is never the inferred constructor.
    pub fn parse(text: &str) -> Result<Self, String> {
        text.parse()
            .map(|area| Shape { area })
            .map_err(|_| text.to_string())
    }

    pub fn area(&self) -> f64 {
        self.area
    }
}

fn main() {}
