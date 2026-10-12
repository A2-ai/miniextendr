//! Independently declared interface versions must be rejected before dispatch.
mod r_test_utils;

use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{ExternalPtr, miniextendr};

// The unused associated type keeps these declarations independent of TPIE's
// crate-root macro names. It does not appear in their exported signatures.
#[allow(dead_code)]
mod producer {
    #[miniextendr_api::miniextendr]
    pub trait Versioned {
        type Marker;
        fn read(&self, offset: i32) -> i32;
        fn reset(&mut self);
    }
}

macro_rules! consumer {
    ($module:ident; $($methods:tt)*) => {
        #[allow(dead_code)]
        mod $module {
            #[miniextendr_api::miniextendr]
            pub trait Versioned {
                type Marker;
                $($methods)*
            }
        }
    };
}

consumer! { compatible;
    fn read(&self, differently_named: i32) -> i32;
    fn reset(&mut self) -> ();
}
consumer! { added;
    fn read(&self, offset: i32) -> i32;
    fn size(&self) -> i32;
    fn reset(&mut self);
}
consumer! { removed;
    fn read(&self, offset: i32) -> i32;
}
consumer! { reordered;
    fn reset(&mut self);
    fn read(&self, offset: i32) -> i32;
}
consumer! { changed_argument;
    fn read(&self, offset: f64) -> i32;
    fn reset(&mut self);
}
consumer! { changed_return;
    fn read(&self, offset: i32) -> f64;
    fn reset(&mut self);
}
consumer! { changed_receiver;
    fn read(&mut self, offset: i32) -> i32;
    fn reset(&mut self);
}
consumer! { changed_r_name;
    #[miniextendr(r_name = "peek")]
    fn read(&self, offset: i32) -> i32;
    fn reset(&mut self);
}

#[derive(ExternalPtr)]
pub struct VersionedCounter(i32);

#[miniextendr(env, noexport)]
impl producer::Versioned for VersionedCounter {
    type Marker = ();

    fn read(&self, offset: i32) -> i32 {
        self.0 + offset
    }

    fn reset(&mut self) {
        self.0 = 0;
    }
}

#[test]
fn only_the_matching_interface_can_query_and_call_the_producer() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(
            __mx_wrap_VersionedCounter(VersionedCounter(40)),
        ));
        let sexp = owner.get();
        // Never call through the incompatible Views, even if query regresses.
        assert!(added::VersionedView::try_from_sexp(sexp).is_none());
        assert!(removed::VersionedView::try_from_sexp(sexp).is_none());
        assert!(reordered::VersionedView::try_from_sexp(sexp).is_none());
        assert!(changed_argument::VersionedView::try_from_sexp(sexp).is_none());
        assert!(changed_return::VersionedView::try_from_sexp(sexp).is_none());
        assert!(changed_receiver::VersionedView::try_from_sexp(sexp).is_none());
        assert!(changed_r_name::VersionedView::try_from_sexp(sexp).is_none());
        assert!(
            miniextendr_api::mx_abi::mx_query(
                sexp,
                miniextendr_api::abi::mx_tag_from_path("Versioned"),
            )
            .is_null()
        );

        let mut view = compatible::VersionedView::from_sexp(sexp);
        assert_eq!(view.read(2), 42);
        view.reset();
        assert_eq!(view.read(2), 2);
    });
}
