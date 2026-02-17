//! Test: unknown #[into_list] field option.

use miniextendr_macros::IntoList;

#[derive(IntoList)]
struct Bad {
    #[into_list(typo)]
    x: f64,
}

fn main() {}
