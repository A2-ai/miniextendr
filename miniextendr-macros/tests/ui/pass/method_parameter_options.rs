use miniextendr_api::{ExternalPtr, MatchArg, miniextendr};

#[derive(Clone, Copy, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum Mode { Fast, Safe }

mod declaration {
    use super::*;
    type PrivateMode = Mode;
    #[miniextendr]
    pub trait Options {
        fn mode(&self, #[miniextendr(match_arg, default = "safe")] mode: PrivateMode) -> Mode { mode }
        fn value(&self, #[miniextendr(coerce, no_na)] value: i32) -> i32 { value }
    }
}

#[derive(ExternalPtr)]
pub struct Counter;
impl declaration::Options for Counter {}
#[miniextendr(env)]
impl declaration::Options for Counter {}

#[miniextendr(env)]
impl Counter {
    pub fn new() -> Self { Self }
    pub fn mode(&self, #[miniextendr(match_arg)] mode: Mode) -> Mode { mode }
    pub fn defaulted(&self, #[miniextendr(default = "1L")] count: i32) -> i32 { count }
    pub fn renamed(&self, #[miniextendr(no_na)] _: f64) -> i32 { 0 }
    pub fn pair(&self, #[miniextendr(default = "list(1L, 2L)")] (a, b): (i32, i32)) -> i32 { a + b }
}

#[miniextendr]
trait Generic<T> {
    fn value(&self) -> T;
    fn mode(&self, #[miniextendr(choices("fast", "safe"))] mode: String) -> String { mode }
}
#[miniextendr(env)]
impl Generic<i32> for Counter {
    fn value(&self) -> i32 { 1 }
}

fn main() {
    // Parameter metadata must not make otherwise object-safe traits unusable.
    let _: &dyn declaration::Options = &Counter;
}
