mod r_test_utils;

#[cfg(feature = "indexmap")]
use miniextendr_api::IndexMap;
#[cfg(feature = "indexmap")]
use miniextendr_api::SEXPTYPE;
#[cfg(feature = "indexmap")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "indexmap")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "indexmap")]
use miniextendr_api::prelude::{SEXP, SexpExt};
#[cfg(feature = "indexmap")]
use miniextendr_api::sexp_types::cetype_t;
#[cfg(feature = "indexmap")]
use miniextendr_api::sys::{Rf_allocVector, Rf_mkCharLenCE, Rf_protect, Rf_unprotect};

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

// region: degenerate name coverage (exercises from_r::charsxp_to_string_lossy
// via the owned CHARSXP->String extraction in the IndexMap TryFromSexp path)

/// Builds `list(a = 1L, <NA> = 2L, `` = 3L, <invalid-utf8> = 4L)` by hand —
/// `names<-` in R rejects `NA_character_`/`""` names on `data.frame`s but a
/// plain list tolerates them, and a raw `Rf_mkCharLenCE(..., CE_BYTES)` name
/// can carry non-UTF-8 bytes no R-level assignment could produce cleanly.
#[cfg(feature = "indexmap")]
#[test]
fn indexmap_degenerate_names_use_charsxp_to_string_lossy_fallbacks() {
    r_test_utils::with_r_thread(|| unsafe {
        let list = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, 4));
        list.set_vector_elt(0, SEXP::scalar_integer(1));
        list.set_vector_elt(1, SEXP::scalar_integer(2));
        list.set_vector_elt(2, SEXP::scalar_integer(3));
        list.set_vector_elt(3, SEXP::scalar_integer(4));

        let names = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, 4));
        names.set_string_elt(0, SEXP::charsxp("a"));
        names.set_string_elt(1, SEXP::na_string());
        names.set_string_elt(2, SEXP::charsxp(""));
        let invalid_utf8: &[u8] = &[0x62, 0xFF, 0xFE]; // "b" + invalid bytes
        names.set_string_elt(
            3,
            Rf_mkCharLenCE(
                invalid_utf8.as_ptr().cast(),
                invalid_utf8.len() as i32,
                cetype_t::CE_BYTES,
            ),
        );
        list.set_names(names);

        let map: IndexMap<String, i32> = TryFromSexp::try_from_sexp(list).unwrap();
        Rf_unprotect(2);

        let keys: Vec<_> = map.keys().cloned().collect();

        // Named element keeps its name; NA and empty names fall back to the
        // auto-generated "V<position>" scheme (1-indexed).
        assert_eq!(keys[0], "a");
        assert_eq!(keys[1], "V2");
        assert_eq!(keys[2], "V3");

        // Invalid UTF-8 is lossily decoded (replacement char), not discarded
        // to an auto-name — the valid "b" prefix survives.
        assert!(keys[3].starts_with('b'));
        assert!(keys[3].contains('\u{FFFD}'));
        assert_ne!(keys[3], "V4");

        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("V2"), Some(&2));
        assert_eq!(map.get("V3"), Some(&3));
        assert_eq!(map.get(keys[3].as_str()), Some(&4));
    });
}
// endregion
