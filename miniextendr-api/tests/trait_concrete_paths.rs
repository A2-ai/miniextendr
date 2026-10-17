//! Concrete type paths must survive trait wrapper and dispatch generation.
mod r_test_utils;

use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::{ExternalPtr, IntoR, SEXP, TryFromSexp};

mod types {
    #[derive(miniextendr_api::ExternalPtr)]
    pub struct Host(pub i32);

    pub fn erased(value: i32) -> *mut miniextendr_api::abi::mx_erased {
        __mx_wrap_Host(Host(value))
    }
}

mod r#type {
    #[allow(non_camel_case_types)]
    #[derive(miniextendr_api::ExternalPtr)]
    pub struct r#gen;

    pub fn erased() -> *mut miniextendr_api::abi::mx_erased {
        __mx_wrap_gen(r#gen)
    }
}

mod protocols {
    use miniextendr_api::miniextendr;

    #[miniextendr]
    pub trait Counter {
        const LIMIT: i32;
        fn read(&self) -> i32;
        fn add(&mut self, other: &Self) -> i32;
        fn total(left: &Self, right: &Self) -> i32;
        fn plus(&self, amount: i32) -> i32 {
            self.read() + amount
        }
    }

    #[miniextendr]
    pub trait DefaultRead {
        fn default_read(&self) -> i32 {
            17
        }
    }

    #[miniextendr]
    pub trait AliasRead {
        fn alias_read(&self) -> i32 {
            23
        }
    }

    #[miniextendr]
    pub trait Scale<T> {
        fn scale(&self, by: T) -> T;
    }
}

mod implementations {
    use super::*;
    use miniextendr_api::miniextendr;

    #[allow(dead_code)]
    struct Host;

    #[miniextendr(env, noexport)]
    impl crate::protocols::Counter for crate::types::Host {
        const LIMIT: i32 = 100;
        fn read(&self) -> i32 {
            self.0
        }
        fn add(&mut self, other: &Self) -> i32 {
            self.0 += other.0;
            self.0
        }
        fn total(left: &Self, right: &Self) -> i32 {
            left.0 + right.0
        }
    }

    impl crate::protocols::DefaultRead for crate::types::Host {}
    #[miniextendr(env, noexport)]
    impl crate::protocols::DefaultRead for crate::types::Host {}

    type HostAlias = crate::types::Host;
    impl crate::protocols::AliasRead for HostAlias {}
    #[miniextendr(env, noexport)]
    impl crate::protocols::AliasRead for HostAlias {}

    impl crate::protocols::DefaultRead for crate::r#type::r#gen {}
    #[miniextendr(env, noexport)]
    impl crate::protocols::DefaultRead for crate::r#type::r#gen {}

    #[miniextendr(env, noexport)]
    impl crate::protocols::Scale<i32> for crate::types::Host {
        fn scale(&self, by: i32) -> i32 {
            self.0 * by
        }
    }
    #[test]
    fn qualified_impl_wrappers_use_the_concrete_type_for_every_member() {
        r_test_utils::with_r_thread(|| unsafe {
            let scope = ProtectScope::new();
            let left = scope
                .protect(ExternalPtr::new(types::Host(10)).into_sexp())
                .get();
            let right = scope
                .protect(ExternalPtr::new(types::Host(4)).into_sexp())
                .get();
            let two = scope.protect(2_i32.into_sexp()).get();
            let read = |value| i32::try_from_sexp(value).unwrap();
            let refused = scope.protect(C_trait_concrete_paths_Host__Counter__add(
                SEXP::nil(),
                left,
                left,
            ));
            let (_, parts) = miniextendr_api::RCondition::from_tagged_sexp(refused.get())
                .expect("mutable receiver must refuse its own &Self argument")
                .into_parts();
            assert!(parts.message.contains("alias the same object"));

            assert_eq!(
                read(C_trait_concrete_paths_Host__Counter__read(
                    SEXP::nil(),
                    left
                )),
                10
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__Counter__LIMIT(SEXP::nil())),
                100
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__Counter__total(
                    SEXP::nil(),
                    left,
                    right
                )),
                14
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__Counter__add(
                    SEXP::nil(),
                    left,
                    right
                )),
                14
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__Counter__plus(
                    SEXP::nil(),
                    left,
                    two
                )),
                16
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__DefaultRead__default_read(
                    SEXP::nil(),
                    left
                )),
                17
            );
            assert_eq!(
                read(C_trait_concrete_paths_Host__Scale__scale(
                    SEXP::nil(),
                    left,
                    two
                )),
                28
            );
        });
    }
}

#[test]
fn consumer_views_find_impls_outside_the_types_defining_module() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(types::erased(10)));
        let view =
            protocols::CounterView::try_from_sexp(owner.get()).expect("qualified populated impl");
        assert_eq!(view.read(), 10);
        assert_eq!(view.plus(2), 12);
        let default =
            protocols::DefaultReadView::try_from_sexp(owner.get()).expect("qualified empty impl");
        assert_eq!(default.default_read(), 17);
        let alias = protocols::AliasReadView::try_from_sexp(owner.get())
            .expect("a type alias preserves dispatch identity");
        assert_eq!(alias.alias_read(), 23);
        let odd = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(crate::r#type::erased()));
        let odd_view = protocols::DefaultReadView::try_from_sexp(odd.get())
            .expect("raw type and module identifiers use canonical identity");
        assert_eq!(odd_view.default_read(), 17);

        // Generic traits have no generated View. Query and call their declared
        // callback layout directly, as a consumer of that ABI would.
        let table = miniextendr_api::mx_abi::mx_query(owner.get(), protocols::TAG_Scale);
        assert!(!table.is_null());
        let table =
            &*table.cast::<implementations::__VTABLE_trait_concrete_paths_Scale_FOR_Host_Layout>();
        let raw = miniextendr_api::mx_abi::mx_get(owner.get());
        let data = raw.cast::<u8>().add((*(*raw).base).data_offset).cast();
        let by = OwnedProtect::new(3_i32.into_sexp());
        let args = [by.get()];
        let scaled = OwnedProtect::new((table.scale)(data, 1, args.as_ptr()));
        assert_eq!(i32::try_from_sexp(scaled.get()).unwrap(), 30);
    });
}
