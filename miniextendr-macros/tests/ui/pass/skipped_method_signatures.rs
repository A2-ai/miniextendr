#[path = "../../../../rpkg/src/rust/skipped_method_tests.rs"]
mod skipped_method_tests;

fn main() {
    assert_eq!(
        skipped_method_tests::skipped_methods_rust_only(),
        vec![1, 2, 3, 4, 11, 12, 13, 14],
    );
}
