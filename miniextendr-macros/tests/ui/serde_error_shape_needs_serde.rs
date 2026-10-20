//! `serde_error_shape!` without miniextendr-api's `serde` feature (#2207): a
//! compile error, not a declaration that silently does nothing. (The UI crate
//! builds miniextendr-api without `serde`.)

miniextendr_api::serde_error_shape!(String = miniextendr_api::condition::SerdeErrorShape::new());

fn main() {}
