//! Compile-pass companion of the class member collision checks (#1677): the
//! names below look like collisions but define distinct R members, so the
//! whole expansion must still compile.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct R6Members {
    value: i32,
}

#[miniextendr(r6)]
impl R6Members {
    // The constructor is `initialize` itself.
    #[miniextendr(r6(constructor))]
    pub fn initialize(value: i32) -> Self {
        Self { value }
    }

    // Instance methods named like generator bindings live on the object.
    pub fn set(&mut self, amount: i32) {
        self.value = amount;
    }

    pub fn lock(&self) -> i32 {
        self.value
    }

    // An active binding under a renamed property.
    #[miniextendr(r6(active, prop = "state"))]
    pub fn current(&self) -> i32 {
        self.value
    }

    // A renamed static method keeps clear of the generator's `new`.
    #[miniextendr(r_name = "fresh")]
    pub fn new() -> i32 {
        0
    }
}

#[derive(ExternalPtr)]
pub struct R6StaticInitialize {
    value: i32,
}

#[miniextendr(r6)]
impl R6StaticInitialize {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    // A static method may be called `initialize`: it lands on the generator.
    pub fn initialize() -> i32 {
        1
    }
}

#[derive(ExternalPtr)]
pub struct EnvNoConstructor {
    value: i32,
}

#[miniextendr(env)]
impl EnvNoConstructor {
    // No constructor, so a static `new` is the only `EnvNoConstructor$new`.
    pub fn new() -> i32 {
        0
    }
}

#[derive(ExternalPtr)]
pub struct EnvRenamedNew {
    value: i32,
}

#[miniextendr(env)]
impl EnvRenamedNew {
    #[miniextendr(env(constructor))]
    pub fn create() -> Self {
        Self { value: 0 }
    }

    // Renamed away from the constructor's `new`.
    #[miniextendr(r_name = "initial")]
    pub fn new() -> i32 {
        0
    }
}

#[derive(ExternalPtr)]
pub struct S4Generics {
    value: i32,
}

#[miniextendr(s4)]
impl S4Generics {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn kind(&self) -> i32 {
        self.value
    }

    // An explicit generic is used verbatim, so `s4_kind` stays free.
    #[miniextendr(s4(generic = "category"))]
    pub fn r#type(&self) -> i32 {
        self.value
    }

    // Static methods emit `S4Generics_kind()`, no `setMethod()`.
    #[miniextendr(r_name = "kind")]
    pub fn make() -> i32 {
        0
    }
}

fn main() {}
