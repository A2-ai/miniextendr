use crate::declarations::{Existing, S3Compatible, WrongSystem};
use crate::{DefaultHost, Host as ImportedHost};
use miniextendr_api::miniextendr;

#[miniextendr(s7)]
impl WrongSystem for ImportedHost {
    fn wrong_value(&self) -> i32 {
        1
    }
}

impl Existing for ImportedHost {
    fn existing_value(&self) -> i32 {
        2
    }
}

#[miniextendr(env)]
impl Existing for ImportedHost {}

#[miniextendr(s3)]
impl S3Compatible for ImportedHost {
    fn compatible_value(&self) -> i32 {
        3
    }
}

#[miniextendr]
impl S3Compatible for DefaultHost {
    fn compatible_value(&self) -> i32 {
        4
    }
}

impl Existing for DefaultHost {
    fn existing_value(&self) -> i32 {
        5
    }
}

#[miniextendr]
impl Existing for DefaultHost {}
