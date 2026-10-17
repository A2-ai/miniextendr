//! Comparison providers are exclusive by canonical type, including aliases.
use miniextendr_api::{AllEqual, ExternalPtr, RPartialEq, miniextendr};

#[derive(ExternalPtr, AllEqual, PartialEq)]
pub struct Both(i32);
#[miniextendr(s3)]
impl Both {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
}
type Alias = Both;
#[miniextendr(s3)]
impl RPartialEq for Alias {}

#[test]
fn derive_and_exact_adapter_on_one_type_are_rejected() {
    let panic = std::panic::catch_unwind(miniextendr_api::registry::collect_r_wrappers)
        .expect_err("two comparison providers must not be silently overwritten");
    let message = panic.downcast_ref::<String>().expect("writer diagnostic");
    assert!(
        message.contains("more than one all.equal provider"),
        "{message}"
    );
    assert!(message.contains(std::any::type_name::<Both>()), "{message}");
}
