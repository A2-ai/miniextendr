//! Re-exporting a trait must carry enough metadata for ABI registration.
mod r_test_utils;

use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{ExternalPtr, RClone, RToVec, miniextendr};

#[derive(Debug, ExternalPtr)]
pub struct RootPath;

#[miniextendr(env, noexport)]
impl miniextendr_api::RDebug for RootPath {}

#[derive(Clone, ExternalPtr)]
pub struct Imported;

#[miniextendr(env, noexport)]
impl RClone for Imported {}

#[derive(ExternalPtr)]
pub struct Generic(Vec<i32>);

#[miniextendr(env, noexport)]
impl RToVec<i32> for Generic {
    fn r_to_vec(&self) -> Vec<i32> {
        self.0.clone()
    }
    fn r_len(&self) -> i64 {
        i64::try_from(self.0.len()).unwrap()
    }
    fn r_is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

mod defining {
    // The default callback's argument type must resolve here, even though the
    // consumer only imports the trait (under another name).
    pub type Count = i32;

    #[miniextendr_api::miniextendr]
    pub trait Counter {
        fn read(&self) -> i32;
        fn plus(&self, amount: Count) -> Count {
            self.read() + amount
        }
    }

    #[miniextendr_api::miniextendr]
    pub trait Associated {
        type Item;
        #[miniextendr(r_name = "value")]
        fn r_value(&self) -> Self::Item;
    }
}

// Neither re-export brings the vtable, tag, builder or local Count alias into
// scope. The macro namespace follows these same imports automatically.
pub use defining::Counter as RenamedCounter;
mod facade {
    pub use super::defining::Associated as RenamedAssociated;
    pub use miniextendr_api::RDebug as DebugAlias;
}

#[derive(ExternalPtr)]
pub struct CounterHost(i32);

#[miniextendr(env, noexport)]
impl crate::RenamedCounter for CounterHost {
    fn read(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, ExternalPtr)]
pub struct AssociatedHost;

#[miniextendr(env, noexport)]
impl facade::RenamedAssociated for AssociatedHost {
    type Item = i32;
    fn r_value(&self) -> i32 {
        9
    }
}

// An empty impl through a user re-export uses the declaration's name too.
#[miniextendr(env, noexport)]
impl facade::DebugAlias for AssociatedHost {}

// An empty impl of a trait this crate defines, by the crate path and by the
// bare name (#2072).
#[miniextendr]
pub trait Greet {
    fn hello(&self) -> String {
        "hi".to_string()
    }
}

#[derive(ExternalPtr)]
pub struct CratePathGreeter;

impl Greet for CratePathGreeter {}

#[miniextendr(env, noexport)]
impl crate::Greet for CratePathGreeter {}

#[derive(ExternalPtr)]
pub struct BareGreeter;

impl Greet for BareGreeter {}

#[miniextendr(env, noexport)]
impl Greet for BareGreeter {}

#[test]
fn import_spelling_does_not_change_r_names_or_trait_object_compatibility() {
    assert!(
        R_WRAPPERS_RootPath_RDebug_IMPL
            .content
            .contains("RootPath$RDebug$debug_str")
    );
    assert!(
        R_WRAPPERS_Imported_RClone_IMPL
            .content
            .contains("Imported$RClone$clone")
    );
    assert!(
        R_WRAPPERS_Generic_RToVec_IMPL
            .content
            .contains("Generic$RToVec$to_vec")
    );
    assert!(
        R_WRAPPERS_AssociatedHost_Associated_IMPL
            .content
            .contains("AssociatedHost$Associated$value")
    );
    assert!(
        R_WRAPPERS_AssociatedHost_RDebug_IMPL
            .content
            .contains("AssociatedHost$RDebug$debug_str")
    );
    assert!(
        R_WRAPPERS_CounterHost_Counter_IMPL
            .content
            .contains("CounterHost$Counter$read")
    );
    assert!(
        R_WRAPPERS_CratePathGreeter_Greet_IMPL
            .content
            .contains("CratePathGreeter$Greet$hello")
    );
    assert!(
        R_WRAPPERS_BareGreeter_Greet_IMPL
            .content
            .contains("BareGreeter$Greet$hello")
    );
    let concrete = CounterHost(40);
    let object: &dyn RenamedCounter = &concrete;
    assert_eq!(object.plus(2), 42);
}

#[test]
fn a_consumer_view_dispatches_default_methods_through_the_reexported_impl() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = OwnedProtect::new(miniextendr_api::mx_abi::mx_wrap(__mx_wrap_CounterHost(
            CounterHost(40),
        )));
        let view = defining::CounterView::try_from_sexp(owner.get()).expect("canonical trait tag");
        assert_eq!(view.read(), 40);
        assert_eq!(view.plus(2), 42);

        for greeter in [
            miniextendr_api::mx_abi::mx_wrap(__mx_wrap_CratePathGreeter(CratePathGreeter)),
            miniextendr_api::mx_abi::mx_wrap(__mx_wrap_BareGreeter(BareGreeter)),
        ] {
            let greeter = OwnedProtect::new(greeter);
            let view = GreetView::try_from_sexp(greeter.get()).expect("empty impl registers");
            assert_eq!(view.hello(), "hi");
        }

        // Also check the public declaration-side builder: it must select the
        // same associated callbacks, and preserve its nominal vtable type.
        let table: defining::CounterVTable = defining::__Counter_build_vtable::<CounterHost>();
        assert_eq!(
            std::mem::size_of_val(&table),
            std::mem::size_of_val(&__VTABLE_trait_reexports_Counter_FOR_CounterHost)
        );
        assert_eq!(
            std::mem::align_of_val(&table),
            std::mem::align_of_val(&__VTABLE_trait_reexports_Counter_FOR_CounterHost)
        );
    });
}
