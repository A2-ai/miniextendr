//! Impl-block dots fixtures for class-system methods and constructors.

use miniextendr_api::dots::Dots;
use miniextendr_api::{List, miniextendr, typed_list};

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

/// R6 class exercising the `dots = typed_list!(...)` attribute sugar (golife
/// hiccup #2) on both a constructor and a regular method. The sugar injects a
/// `dots_typed` binding at the top of each body, so the method code can read
/// validated fields directly instead of calling `.typed(...)` by hand.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ImplDotsSugar {
    base: f64,
    scale: f64,
}

#[miniextendr(r6)]
impl ImplDotsSugar {
    /// Construct with a positional `base` and a sugar-validated `scale` dot.
    /// @param base Numeric base value.
    /// @param ... `scale` (numeric scalar), validated via `typed_list!`.
    #[miniextendr(dots = typed_list!(scale => numeric()))]
    pub fn new(base: f64, dots: ...) -> Self {
        // `dots_typed` is injected by the `dots = typed_list!(...)` sugar.
        let scale: f64 = dots_typed.get("scale").expect("scale");
        Self { base, scale }
    }

    /// Return `(base + bump) * scale`, with `bump` a sugar-validated dot.
    /// @param ... `bump` (numeric scalar), validated via `typed_list!`.
    #[miniextendr(dots = typed_list!(bump => numeric()))]
    pub fn scaled(&self, dots: ...) -> f64 {
        let bump: f64 = dots_typed.get("bump").expect("bump");
        (self.base + bump) * self.scale
    }
}

/// Exercise the impl-block `dots = typed_list!(...)` sugar under gctorture
/// without requiring arguments (golife hiccup #2). Returns `(2 + 4) * 3 = 18`.
#[miniextendr]
pub fn gc_stress_impl_dots_sugar() -> f64 {
    let ctor_dots = List::from_pairs(vec![("scale", 3.0f64)]);
    let ctor = Dots {
        inner: ctor_dots.as_sexp(),
    };
    let obj = ImplDotsSugar::new(2.0, &ctor);

    let method_dots = List::from_pairs(vec![("bump", 4.0f64)]);
    let md = Dots {
        inner: method_dots.as_sexp(),
    };
    obj.scaled(&md)
}
