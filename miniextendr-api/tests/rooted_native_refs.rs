//! Shared references are bounded by their Root, including trait-shim arguments.

mod r_test_utils;

use miniextendr_api::from_r::SexpError;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, RLogical, SEXP, SexpExt, TryFromRoot, miniextendr};

#[test]
fn shared_native_roots_preserve_pointer_type_length_and_null_semantics() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let source = scope.protect(vec![3_i32, 5, 8].into_sexp());
        let slice = <&[i32]>::try_from_root(source).unwrap();
        assert_eq!(slice, [3, 5, 8]);
        assert_eq!(slice.as_ptr(), source.get().as_slice::<i32>().as_ptr());
        assert!(matches!(
            <&i32>::try_from_root(source),
            Err(SexpError::Length(_))
        ));
        assert!(<&[f64]>::try_from_root(source).is_err());

        let scalar = scope.protect(13_i32.into_sexp());
        assert_eq!(*<&i32>::try_from_root(scalar).unwrap(), 13);
        assert_eq!(
            <Option<&i32>>::try_from_root(scalar).unwrap().copied(),
            Some(13)
        );
        let logical = scope.protect(RLogical::NA.into_sexp());
        assert_eq!(*<&RLogical>::try_from_root(logical).unwrap(), RLogical::NA);

        let nil = scope.protect(SEXP::nil());
        assert!(<Option<&i32>>::try_from_root(nil).unwrap().is_none());
        assert!(<Option<&[i32]>>::try_from_root(nil).unwrap().is_none());
        let empty = scope.protect(Vec::<i32>::new().into_sexp());
        assert!(<&[i32]>::try_from_root(empty).unwrap().is_empty());
    });
}

#[test]
fn shared_character_roots_preserve_na_and_null_semantics() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let text = scope.protect("hello".into_sexp());
        assert_eq!(<&str>::try_from_root(text).unwrap(), "hello");
        assert_eq!(<Option<&str>>::try_from_root(text).unwrap(), Some("hello"));

        let na = scope.protect(vec![None::<String>].into_sexp());
        assert_eq!(<&str>::try_from_root(na).unwrap(), "");
        assert_eq!(<Option<&str>>::try_from_root(na).unwrap(), None);
        let nil = scope.protect(SEXP::nil());
        assert_eq!(<Option<&str>>::try_from_root(nil).unwrap(), None);
        let wrong = scope.protect(1_i32.into_sexp());
        assert!(matches!(
            <&str>::try_from_root(wrong),
            Err(SexpError::Type(_))
        ));
    });
}

// A trait parameter also needs `IntoR` for the View, which `&i32` lacks, so
// the shim's references are the slice and string forms.
#[miniextendr]
pub trait RootedReader {
    fn read(&self, values: &[i32], label: &str) -> String;
}

#[derive(ExternalPtr)]
pub struct Reader;

#[miniextendr(env, noexport)]
impl RootedReader for Reader {
    fn read(&self, values: &[i32], label: &str) -> String {
        format!("{label}:{}", values.iter().sum::<i32>())
    }
}

#[test]
fn trait_shim_shared_references_keep_call_scoped_roots() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let owner = scope.protect(miniextendr_api::mx_abi::mx_wrap(__mx_wrap_Reader(Reader)));
        let view = RootedReaderView::from_sexp(owner.get());
        assert_eq!(view.read(&[5, 8], "sum"), "sum:13");
    });
}

#[miniextendr]
pub trait GenericRootedReader<T> {
    fn total(&self, values: &[T]) -> i32;
}

#[miniextendr(env, noexport)]
impl GenericRootedReader<i32> for Reader {
    fn total(&self, values: &[i32]) -> i32 {
        values.iter().sum()
    }
}

#[test]
fn concrete_generic_shim_uses_shared_native_roots() {
    r_test_utils::with_r_thread(|| unsafe {
        let entry = miniextendr_api::registry::MX_TRAIT_DISPATCH
            .iter()
            .find(|entry| entry.trait_tag == TAG_GenericRootedReader)
            .expect("generic impl must register its concrete vtable");
        let vtable = &*entry.vtable.cast::<GenericRootedReaderVTable>();
        let scope = ProtectScope::new();
        let source = scope.protect(vec![3_i32, 5, 8].into_sexp());
        let args = [source.get()];
        // The data pointer of a trait object points at its value's `RefCell`.
        let reader = std::cell::RefCell::new(Reader);
        let result = scope.protect((vtable.total)(
            std::ptr::from_ref(&reader).cast_mut().cast(),
            1,
            args.as_ptr(),
        ));
        assert_eq!(*<&i32>::try_from_root(result).unwrap(), 16);
    });
}
