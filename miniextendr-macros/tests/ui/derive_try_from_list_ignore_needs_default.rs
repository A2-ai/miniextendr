//! Test: `TryFromList` fills an `#[into_list(ignore)]` field with
//! `Default::default()`, so the field's type must be `Default`; the error
//! points at the field's type.

use miniextendr_api::TryFromList;

struct Opaque(std::rc::Rc<i32>);

#[derive(TryFromList)]
struct In {
    a: i32,
    #[into_list(ignore)]
    skipped: Opaque,
}

fn main() {}
