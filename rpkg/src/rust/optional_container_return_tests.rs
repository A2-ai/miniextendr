//! Optional container/reference methods share standalone IntoR semantics (#2088).
use miniextendr_api::{ExternalPtr, miniextendr};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

macro_rules! optional_container_class {
    ($name:ident, $system:ident, $visibility:ident) => {
        #[derive(ExternalPtr, Default)]
        pub struct $name;

        /// @noRd
        #[miniextendr($system, $visibility)]
        impl $name {
            pub fn new() -> Self {
                Self
            }

            pub fn oc_integers(&self, present: bool) -> Option<Vec<i32>> {
                present.then(|| vec![1, 2])
            }
            pub fn oc_strings(&self, present: bool) -> Option<Vec<String>> {
                present.then(|| vec!["a".into(), "b".into()])
            }
            pub fn oc_hash_map(&self, present: bool) -> Option<HashMap<String, i32>> {
                present.then(|| HashMap::from([("a".into(), 1)]))
            }
            pub fn oc_tree_map(&self, present: bool) -> Option<BTreeMap<String, Vec<i32>>> {
                present.then(|| BTreeMap::from([("a".into(), vec![1, 2])]))
            }
            pub fn oc_hash_set(&self, present: bool) -> Option<HashSet<i32>> {
                present.then(|| HashSet::from([42]))
            }
            pub fn oc_tree_set(&self, present: bool) -> Option<BTreeSet<String>> {
                present.then(|| BTreeSet::from(["a".into(), "b".into()]))
            }
            pub fn oc_reference(&self, present: bool) -> Option<&'static i32> {
                present.then_some(&42)
            }
            pub fn oc_empty(&self, present: bool) -> Option<Vec<i32>> {
                present.then(Vec::new)
            }
            pub fn oc_boxed(&self, present: bool) -> Option<Box<[i32]>> {
                present.then(|| vec![1, 2].into_boxed_slice())
            }
            pub fn oc_slice(&self, present: bool) -> Option<&'static [i32]> {
                present.then_some(&[1, 2])
            }
            pub fn oc_array(&self, present: bool) -> Option<[i32; 2]> {
                present.then_some([1, 2])
            }
            pub fn oc_tuple(&self, present: bool) -> Option<(i32, String)> {
                present.then(|| (1, "a".into()))
            }
        }
    };
}
optional_container_class!(OptionalContainerEnv, env, noexport);
optional_container_class!(OptionalContainerR6, r6, noexport);
optional_container_class!(OptionalContainerS3, s3, internal);
optional_container_class!(OptionalContainerS4, s4, noexport);
optional_container_class!(OptionalContainerS7, s7, noexport);

#[miniextendr]
pub trait OptionalContainerReturns {
    fn oct_integers(&self, present: bool) -> Option<Vec<i32>>;
    fn oct_map(&self, present: bool) -> Option<BTreeMap<String, Vec<i32>>>;
    fn oct_set(&self, present: bool) -> Option<BTreeSet<String>>;
    fn oct_reference(&self, present: bool) -> Option<&'static i32>;
}

#[miniextendr(env, noexport)]
impl OptionalContainerReturns for OptionalContainerEnv {
    fn oct_integers(&self, present: bool) -> Option<Vec<i32>> {
        self.oc_integers(present)
    }
    fn oct_map(&self, present: bool) -> Option<BTreeMap<String, Vec<i32>>> {
        self.oc_tree_map(present)
    }
    fn oct_set(&self, present: bool) -> Option<BTreeSet<String>> {
        self.oc_tree_set(present)
    }
    fn oct_reference(&self, present: bool) -> Option<&'static i32> {
        self.oc_reference(present)
    }
}

#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;
    pub struct OptionalContainerVctrs;
    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "oc"), noexport)]
    impl OptionalContainerVctrs {
        #[allow(clippy::new_ret_no_self)]
        pub fn new(values: Vec<f64>) -> Vec<f64> {
            values
        }

        pub fn oc_integers(_values: Vec<f64>, present: bool) -> Option<Vec<i32>> {
            present.then(|| vec![1, 2])
        }
        pub fn oc_strings(_values: Vec<f64>, present: bool) -> Option<Vec<String>> {
            present.then(|| vec!["a".into(), "b".into()])
        }
        pub fn oc_hash_map(_values: Vec<f64>, present: bool) -> Option<HashMap<String, i32>> {
            present.then(|| HashMap::from([("a".into(), 1)]))
        }
        pub fn oc_tree_map(_values: Vec<f64>, present: bool) -> Option<BTreeMap<String, Vec<i32>>> {
            present.then(|| BTreeMap::from([("a".into(), vec![1, 2])]))
        }
        pub fn oc_hash_set(_values: Vec<f64>, present: bool) -> Option<HashSet<i32>> {
            present.then(|| HashSet::from([42]))
        }
        pub fn oc_tree_set(_values: Vec<f64>, present: bool) -> Option<BTreeSet<String>> {
            present.then(|| BTreeSet::from(["a".into(), "b".into()]))
        }
        pub fn oc_reference(_values: Vec<f64>, present: bool) -> Option<&'static i32> {
            present.then_some(&42)
        }
        pub fn oc_empty(_values: Vec<f64>, present: bool) -> Option<Vec<i32>> {
            present.then(Vec::new)
        }
        pub fn oc_boxed(_values: Vec<f64>, present: bool) -> Option<Box<[i32]>> {
            present.then(|| vec![1, 2].into_boxed_slice())
        }
        pub fn oc_slice(_values: Vec<f64>, present: bool) -> Option<&'static [i32]> {
            present.then_some(&[1, 2])
        }
        pub fn oc_array(_values: Vec<f64>, present: bool) -> Option<[i32; 2]> {
            present.then_some([1, 2])
        }
        pub fn oc_tuple(_values: Vec<f64>, present: bool) -> Option<(i32, String)> {
            present.then(|| (1, "a".into()))
        }
    }
}
