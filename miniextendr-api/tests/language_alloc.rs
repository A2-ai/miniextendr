//! Portable allocation of language objects, including R 4.4.0.

mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{SEXPTYPE, SexpExt};

#[test]
fn language_allocation_handles_empty_and_positive_lengths() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        for n in [-1, 0] {
            assert!(scope.alloc_lang(n).get().is_null_or_nil());
        }
        for n in [1, 2, 5] {
            let call = scope.alloc_lang(n).get();
            assert_eq!(call.type_of(), SEXPTYPE::LANGSXP);
            assert_eq!(call.xlength(), isize::try_from(n).unwrap());
        }
    });
}
