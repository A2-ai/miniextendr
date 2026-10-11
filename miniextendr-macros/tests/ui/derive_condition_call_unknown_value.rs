//! Test: `#[derive(RCondition)]` accepts only `call = none`; any other
//! `call` value is rejected at the value.

use miniextendr_api::condition::RCondition;

#[derive(Debug, RCondition)]
#[condition(class = "pkg_warning")]
enum PkgWarning {
    #[condition(call = wrapper, message = "row {row} overrides an earlier one")]
    Overridden { row: i32 },
}

fn main() {}
