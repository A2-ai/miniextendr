mod r_test_utils;

#[cfg(feature = "indexmap")]
use miniextendr_api::IndexMap;
#[cfg(feature = "indexmap")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "indexmap")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "indexmap")]
#[test]
fn indexmap_roundtrip_ints() {
    r_test_utils::with_r_thread(|| {
        let mut map = IndexMap::new();
        map.insert("a".to_string(), 1i32);
        map.insert("b".to_string(), 2i32);
        map.insert("c".to_string(), 3i32);

        let sexp = map.clone().into_sexp();
        let back: IndexMap<String, i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        // Check order preserved
        let keys: Vec<_> = back.keys().cloned().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);

        // Check values
        assert_eq!(back.get("a"), Some(&1));
        assert_eq!(back.get("b"), Some(&2));
        assert_eq!(back.get("c"), Some(&3));
    });
}

#[cfg(feature = "indexmap")]
#[test]
fn indexmap_roundtrip_strings() {
    r_test_utils::with_r_thread(|| {
        let mut map = IndexMap::new();
        map.insert("name".to_string(), "Alice".to_string());
        map.insert("city".to_string(), "Boston".to_string());

        let sexp = map.clone().into_sexp();
        let back: IndexMap<String, String> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.get("name"), Some(&"Alice".to_string()));
        assert_eq!(back.get("city"), Some(&"Boston".to_string()));
    });
}

#[cfg(feature = "indexmap")]
#[test]
fn indexmap_empty() {
    r_test_utils::with_r_thread(|| {
        let map: IndexMap<String, i32> = IndexMap::new();
        let sexp = map.into_sexp();
        let back: IndexMap<String, i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_empty());
    });
}

#[cfg(feature = "indexmap")]
#[test]
fn indexmap_preserves_insertion_order() {
    r_test_utils::with_r_thread(|| {
        // Insert in specific order: z, a, m (not alphabetical)
        let mut map = IndexMap::new();
        map.insert("z".to_string(), 1i32);
        map.insert("a".to_string(), 2i32);
        map.insert("m".to_string(), 3i32);

        let sexp = map.into_sexp();
        let back: IndexMap<String, i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        // Order should be preserved
        let keys: Vec<_> = back.keys().cloned().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);

        let values: Vec<_> = back.values().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    });
}

#[cfg(feature = "indexmap")]
#[test]
fn indexmap_nested() {
    r_test_utils::with_r_thread(|| {
        let mut inner1 = IndexMap::new();
        inner1.insert("x".to_string(), 1.0f64);
        inner1.insert("y".to_string(), 2.0f64);

        let mut inner2 = IndexMap::new();
        inner2.insert("x".to_string(), 3.0f64);
        inner2.insert("y".to_string(), 4.0f64);

        // Note: This tests nested lists, which requires List conversion
        // For now, just test that the inner maps work individually
        let sexp1 = inner1.clone().into_sexp();
        let back1: IndexMap<String, f64> = TryFromSexp::try_from_sexp(sexp1).unwrap();
        assert_eq!(back1.get("x"), Some(&1.0));
        assert_eq!(back1.get("y"), Some(&2.0));
    });
}
