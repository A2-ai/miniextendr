/// Shared trait definition for cross-package testing.
///
/// Both producer and consumer packages will vendor this exact trait definition.
/// The #[miniextendr] macro generates ABI-compatible infrastructure (vtables, tags)
/// that allows objects from one package to be used via trait methods in another.

use miniextendr_api::miniextendr;

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}
