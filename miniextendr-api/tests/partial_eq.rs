//! Exact comparison codegen must work without optional integrations.
use miniextendr_api::{ExternalPtr, RPartialEq as Equality, miniextendr};

#[derive(ExternalPtr, PartialEq)]
pub struct Opaque(i32);
#[miniextendr(s3, class = "exact_opaque")]
impl Opaque {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
}
#[miniextendr(s3)]
impl Equality for Opaque {}

#[derive(ExternalPtr, PartialEq, Debug)]
pub struct Printable(i32);
#[miniextendr(s3)]
impl Printable {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
}
#[miniextendr(s3, blanket)]
impl Equality for Printable {
    fn r_eq(&self, other: &Self) -> bool {
        unreachable!("metadata only")
    }
}

#[test]
fn default_features_register_exact_methods_for_both_impl_paths() {
    let wrappers = miniextendr_api::registry::collect_r_wrappers().join("\n");
    assert!(wrappers.contains("`all.equal.exact_opaque`"));
    assert!(wrappers.contains("`all.equal.Printable`"));
    assert!(!wrappers.contains("RPartialEq$r_eq"));
    assert!(Equality::r_eq(&Opaque(1), &Opaque(1)));
    assert!(!Equality::r_eq(&Opaque(1), &Opaque(2)));
    assert!(Equality::r_eq(&Printable(1), &Printable(1)));
}

#[test]
fn diagnostics_use_debug_without_requiring_it() {
    use miniextendr_api::adapter_traits::{PartialEqFallback as _, PartialEqProbe};
    assert_eq!(
        PartialEqProbe::new(&Opaque(1)).differences(&Opaque(1), &Opaque(2), "Opaque"),
        ["Rust values of Opaque differ"]
    );
    assert_eq!(
        PartialEqProbe::new(&Printable(1)).differences(&Printable(1), &Printable(2), "Printable"),
        [
            "Rust values of Printable differ",
            "target: Printable(1)\ncurrent: Printable(2)"
        ]
    );
}
