//! Trait declaration metadata must reach populated and blanket impl wrappers.
use miniextendr_api::{ExternalPtr, miniextendr};

mod declarations {
    #[miniextendr_api::miniextendr]
    pub trait Collection<T> {
        #[miniextendr(r_name = "values")]
        fn r_values(&self) -> Vec<T>;

        // This argument deliberately has no R conversion. Inheriting skip
        // must keep it out of both the C wrapper and the generic vtable.
        #[miniextendr(skip)]
        fn rust_only(&self, action: std::sync::Mutex<()>);
    }

    #[miniextendr_api::miniextendr]
    pub trait Associated {
        type Item;
        #[miniextendr(r_name = "value")]
        fn r_value(&self) -> Self::Item;
    }
}

#[derive(ExternalPtr)]
pub struct Bag(Vec<i32>);

#[miniextendr(env, noexport)]
impl declarations::Collection<i32> for Bag {
    fn r_values(&self) -> Vec<i32> {
        self.0.clone()
    }
    fn rust_only(&self, _action: std::sync::Mutex<()>) {}
}

#[miniextendr(env, noexport)]
impl declarations::Associated for Bag {
    type Item = i32;
    fn r_value(&self) -> i32 {
        self.0[0]
    }
}

#[derive(Debug, ExternalPtr)]
pub struct Override;

#[miniextendr(env, noexport)]
impl declarations::Associated for Override {
    type Item = i32;
    #[miniextendr(r_name = "custom")]
    fn r_value(&self) -> i32 {
        42
    }
}

// Empty impls still consume the same metadata macro's signature arm.
#[miniextendr(env, noexport)]
impl miniextendr_api::adapter_traits::RDebug for Override {}

// A blanket registration with signatures also inherits declaration metadata.
#[derive(ExternalPtr)]
pub struct BlanketBag(Vec<i32>);

impl<'a> IntoIterator for &'a BlanketBag {
    type Item = &'a i32;
    type IntoIter = std::slice::Iter<'a, i32>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[miniextendr(env, noexport, blanket)]
impl miniextendr_api::adapter_traits::RToVec<i32> for BlanketBag {
    fn r_to_vec(&self) -> Vec<i32> {
        unreachable!()
    }
    fn r_len(&self) -> i64 {
        unreachable!()
    }
    fn r_is_empty(&self) -> bool {
        unreachable!()
    }
}

#[test]
fn wrappers_inherit_names_and_skip_but_keep_explicit_overrides() {
    let generic = R_WRAPPERS_Bag_Collection_IMPL.content;
    assert!(
        generic.contains("Bag$Collection$values <- function("),
        "{generic}"
    );
    assert!(!generic.contains("$r_values <-"));
    assert!(!generic.contains("rust_only"));
    assert!(
        R_WRAPPERS_Bag_Associated_IMPL
            .content
            .contains("Bag$Associated$value <- function(")
    );
    assert!(
        R_WRAPPERS_Override_Associated_IMPL
            .content
            .contains("Override$Associated$custom <- function(")
    );
    assert!(
        R_WRAPPERS_Override_RDebug_IMPL
            .content
            .contains("Override$RDebug$debug_str <- function(")
    );
    let blanket = R_WRAPPERS_BlanketBag_RToVec_IMPL.content;
    for name in ["to_vec", "len", "is_empty"] {
        assert!(
            blanket.contains(&format!("BlanketBag$RToVec${name} <- function(")),
            "{blanket}"
        );
        assert!(!blanket.contains(&format!("$r_{name} <-")));
    }
    let bag = Bag(vec![1, 2]);
    declarations::Collection::rust_only(&bag, std::sync::Mutex::new(()));
    assert_eq!(declarations::Collection::r_values(&bag), [1, 2]);
}
