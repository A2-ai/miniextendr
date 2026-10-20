//! `serde_condition!` without miniextendr-api's `serde` feature (#2217): a
//! compile error naming the feature, not an `RCondition` built some other
//! way. (The UI crate builds miniextendr-api without `serde`.)

fn main() {
    let _condition = miniextendr_api::serde_condition!(&String::from("boom"));
}
