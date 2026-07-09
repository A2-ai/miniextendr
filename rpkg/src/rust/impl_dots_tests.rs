//! Impl-block dots fixtures for class-system methods and constructors.

use miniextendr_api::dots::Dots;
use miniextendr_api::{List, miniextendr};

/// R6 class whose constructor and methods accept dots.
/// @param seed Integer base value.
/// @param ... Additional constructor arguments counted by Rust.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ImplDotsR6 {
    seed: i32,
    ctor_dots: i32,
}

#[miniextendr(r6)]
impl ImplDotsR6 {
    /// Create an R6 dots fixture.
    pub fn new(seed: i32, dots: ...) -> Self {
        Self {
            seed,
            ctor_dots: dots.len() as i32,
        }
    }

    /// Return the constructor dots count.
    pub fn ctor_dots(&self) -> i32 {
        self.ctor_dots
    }

    /// Add a value and the number of method dots.
    /// @param value Integer value to add.
    /// @param ... Additional method arguments counted by Rust.
    pub fn add_with_dots(&self, value: i32, dots: ...) -> i32 {
        self.seed + self.ctor_dots + value + dots.len() as i32
    }

    /// Count explicit `&Dots` method arguments.
    /// @param ... Additional method arguments counted by Rust.
    pub fn explicit_dots(&self, dots: &Dots) -> i32 {
        dots.len() as i32
    }
}

/// S3 class whose constructor and instance method accept dots.
/// @param seed Integer base value.
/// @param ... Additional constructor arguments counted by Rust.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ImplDotsS3 {
    seed: i32,
    ctor_dots: i32,
}

#[miniextendr(s3)]
impl ImplDotsS3 {
    /// Create an S3 dots fixture.
    pub fn new(seed: i32, dots: ...) -> Self {
        Self {
            seed,
            ctor_dots: dots.len() as i32,
        }
    }

    /// Return the constructor dots count.
    pub fn impl_dots_s3_ctor_dots(&self) -> i32 {
        self.ctor_dots
    }

    /// Add a value and the number of method dots.
    /// @param value Integer value to add.
    /// @param ... Additional method arguments counted by Rust.
    pub fn impl_dots_s3_add_with_dots(&self, value: i32, dots: ...) -> i32 {
        self.seed + self.ctor_dots + value + dots.len() as i32
    }
}

/// Exercise impl-block dots storage under gctorture without requiring arguments.
#[miniextendr]
pub fn gc_stress_impl_dots_methods() -> i32 {
    let dots_list = List::from_pairs(vec![("a", 1i32), ("b", 2i32)]);
    let dots = Dots {
        inner: dots_list.as_sexp(),
    };
    let r6 = ImplDotsR6::new(10, &dots);
    let s3 = ImplDotsS3::new(20, &dots);

    r6.add_with_dots(1, &dots) + r6.explicit_dots(&dots) + s3.impl_dots_s3_add_with_dots(1, &dots)
}
