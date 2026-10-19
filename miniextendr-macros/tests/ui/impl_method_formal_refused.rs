//! Test: the method-level `formal(p = "..")` (#1842) must name a parameter of
//! the method, must not take the receiver's name, and is refused on an S7
//! property accessor, whose arguments S7 fixes.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Fit;

#[miniextendr(s3)]
impl Fit {
    pub fn new() -> Self {
        Fit
    }

    #[miniextendr(formal(conf_levl = "conf.level"))]
    pub fn interval(&self, conf_level: f64) -> f64 {
        conf_level
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Model;

#[miniextendr(s3)]
impl Model {
    pub fn new() -> Self {
        Model
    }

    #[miniextendr(formal(data = "x"))]
    pub fn predict(&self, data: f64) -> f64 {
        data
    }
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Level;

#[miniextendr(s7)]
impl Level {
    pub fn new() -> Self {
        Level
    }

    #[miniextendr(s7(getter))]
    pub fn level(&self) -> f64 {
        0.0
    }

    #[miniextendr(s7(setter, prop = "level"), formal(value = "new.level"))]
    pub fn set_level(&mut self, value: f64) {
        let _ = value;
    }
}

fn main() {}
