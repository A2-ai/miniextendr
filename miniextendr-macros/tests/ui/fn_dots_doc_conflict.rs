use miniextendr_macros::miniextendr;

/// @param sources Values.
/// @param ... Other values.
#[miniextendr]
pub fn collect(sources: ...) -> i32 {
    let _ = sources;
    0
}

fn main() {}
