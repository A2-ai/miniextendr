//! Rust-only methods must not enter ABI or R-wrapper lowering (#2151).
#![allow(dead_code, async_fn_in_trait)]

use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, Invisible, LazyDots, Missing, miniextendr};

#[derive(ExternalPtr)]
pub struct SkippedMethodHost(i32);

#[miniextendr(env, noexport)]
impl SkippedMethodHost {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn exposed(&self) -> i32 {
        self.0
    }

    /// # Safety
    /// This signature-only fixture has no additional preconditions.
    #[miniextendr(env(ignore))]
    pub unsafe fn unchecked(&self) -> i32 {
        self.0
    }

    // This receiver shape is the regression: R wrappers cannot accept it.
    #[allow(clippy::boxed_local)]
    #[miniextendr(env(ignore))]
    pub fn boxed(self: Box<Self>) -> i32 {
        self.0
    }

    #[miniextendr(env(ignore))]
    pub fn missing(&self, value: Missing<Missing<i32>>) {
        let _ = value;
    }

    #[miniextendr(env(ignore))]
    pub fn two_dots(&self, first: &Dots, second: &Dots) {
        let _ = (first, second);
    }

    #[miniextendr(env(ignore))]
    pub fn visibility(&self) -> Option<Invisible<i32>> {
        None
    }
}

#[miniextendr]
pub trait SkippedPlain {
    fn visible(&self) -> i32;

    #[miniextendr(skip)]
    fn consume(self) -> Vec<i32>;

    #[miniextendr(skip)]
    fn map<T>(&self, value: T) -> T {
        value
    }

    #[miniextendr(skip)]
    fn array<const N: usize>(&self, value: [i32; N]) -> [i32; N] {
        value
    }

    #[miniextendr(skip)]
    async fn fetch(&self) -> i32 {
        self.visible()
    }

    #[miniextendr(skip)]
    fn lazy(&self, dots: &LazyDots);
}

#[miniextendr(env, noexport)]
impl SkippedPlain for SkippedMethodHost {
    fn visible(&self) -> i32 {
        self.0
    }

    fn consume(self) -> Vec<i32> {
        vec![self.0]
    }

    fn map<T>(&self, value: T) -> T {
        value
    }

    fn lazy(&self, dots: &LazyDots) {
        let _ = dots;
    }
}

#[miniextendr]
pub trait SkippedGeneric<T> {
    fn generic_visible(&self) -> T;

    #[miniextendr(skip)]
    async fn generic_fetch(&self) -> T {
        self.generic_visible()
    }

    #[miniextendr(skip)]
    fn generic_map<U>(&self, value: U) -> U;

    #[miniextendr(skip)]
    fn generic_lazy(&self, dots: &LazyDots);
}

#[miniextendr(env, noexport)]
impl SkippedGeneric<i32> for SkippedMethodHost {
    fn generic_visible(&self) -> i32 {
        self.0 + 1
    }

    fn generic_map<U>(&self, value: U) -> U {
        value
    }

    fn generic_lazy(&self, dots: &LazyDots) {
        let _ = dots;
    }
}

// An impl containing only skipped methods must still be emitted as Rust.
#[miniextendr]
pub trait SkippedOnly {
    #[miniextendr(skip)]
    fn only(self) -> i32;
}

#[miniextendr(env, noexport)]
impl SkippedOnly for SkippedMethodHost {
    fn only(self) -> i32 {
        self.0
    }
}

// The empty-impl path must not request wrappers for skipped default methods.
#[miniextendr]
pub trait SkippedDefaults {
    fn default_visible(&self) -> i32 {
        40
    }

    #[miniextendr(skip)]
    fn default_map<U>(&self, value: U) -> U {
        value
    }
}

impl SkippedDefaults for SkippedMethodHost {}

#[miniextendr(env, noexport)]
impl SkippedDefaults for SkippedMethodHost {}

/// @noRd
#[miniextendr(noexport, no_worker)]
pub fn skipped_methods_rust_only() -> Vec<i32> {
    let host = SkippedMethodHost(11);
    // The futures remain ordinary Rust values; no executor or R conversion is needed.
    std::mem::drop(host.fetch());
    std::mem::drop(host.generic_fetch());
    vec![
        host.map(1),
        host.generic_map(2),
        host.array([3])[0],
        host.default_map(4),
        unsafe { host.unchecked() },
        Box::new(SkippedMethodHost(12)).boxed(),
        SkippedMethodHost(13).consume()[0],
        SkippedMethodHost(14).only(),
    ]
}
