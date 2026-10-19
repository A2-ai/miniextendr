//! Compile-fail test: a field `#[derive(AllEqual)]` compares must convert to
//! R: a clone with `IntoR`, or a `with` getter's value (#2149). The error
//! points at the field's type, or at the getter's path.

use miniextendr_api::{AllEqual, ExternalPtr};

struct Opaque;

#[derive(ExternalPtr, AllEqual)]
struct NotConvertible {
    handle: Opaque,
}

#[derive(ExternalPtr, AllEqual)]
struct GetterNotConvertible {
    #[all_equal(with = "Self::handle")]
    value: f64,
}

impl GetterNotConvertible {
    fn handle(&self) -> Opaque {
        let _ = self.value;
        Opaque
    }
}

fn main() {}
