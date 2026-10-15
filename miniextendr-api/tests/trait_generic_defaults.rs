//! Generic default methods keep their Rust bodies and expose concrete ABI/R wrappers.
mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, TryFromSexp, miniextendr};

mod declaration {
    #[miniextendr_api::miniextendr]
    pub trait Totals<T = i32, const N: usize = 2> {
        fn total(&self) -> T;

        /// The inherited label.
        #[miniextendr(r_name = "label")]
        fn r_label(&self) -> String {
            "default".into()
        }

        fn values(&self, values: Vec<T>) -> Vec<T> {
            values
        }
        fn array(&self, values: [T; N]) -> [T; N] {
            values
        }
        fn ignored(&self, _: i32) -> i32 {
            7
        }
        fn other(&self, other: &Self) -> T {
            other.total()
        }
        fn title() -> String {
            "totals".into()
        }

        #[miniextendr(skip)]
        fn rust_only(&self, _: std::sync::Mutex<()>) {}
    }
}

use declaration::Totals as RenamedTotals;

#[derive(ExternalPtr)]
pub struct Ledger(i32);

#[miniextendr(env, noexport)]
impl RenamedTotals for Ledger {
    fn total(&self) -> i32 {
        self.0
    }
}

#[derive(ExternalPtr)]
pub struct Override(i32);

#[miniextendr(env, noexport)]
impl RenamedTotals<i32, 3> for Override {
    fn total(&self) -> i32 {
        self.0
    }
    #[miniextendr(r_name = "custom_label")]
    fn r_label(&self) -> String {
        "overridden".into()
    }
}

#[test]
fn default_wrappers_inherit_metadata_and_overrides_win() {
    let inherited = R_WRAPPERS_Ledger_Totals_IMPL.content;
    for name in [
        "total", "label", "values", "array", "ignored", "other", "title",
    ] {
        assert!(
            inherited.contains(&format!("Ledger$Totals${name} <- function(")),
            "{inherited}"
        );
    }
    assert!(!inherited.contains("rust_only"));
    assert!(!inherited.contains("$r_label <-"));
    let own = R_WRAPPERS_Override_Totals_IMPL.content;
    assert!(
        own.contains("Override$Totals$custom_label <- function("),
        "{own}"
    );
    assert!(!own.contains("Override$Totals$label <-"));
    let ledger = Ledger(9);
    ledger.rust_only(std::sync::Mutex::new(()));
    assert_eq!(ledger.r_label(), "default");
    assert_eq!(Override(10).r_label(), "overridden");
    assert_eq!(Ledger::title(), "totals");
}

#[test]
fn default_callbacks_dispatch_and_convert_concrete_arguments() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let host = std::cell::RefCell::new(Ledger(9));
        let data = std::ptr::from_ref(&host).cast_mut().cast();
        let table = &__VTABLE_trait_generic_defaults_Totals_FOR_Ledger;
        let no_args = std::ptr::null();
        let total = scope.protect((table.total)(data, 0, no_args)).get();
        assert_eq!(i32::try_from_sexp(total).unwrap(), 9);
        let label = scope.protect((table.r_label)(data, 0, no_args)).get();
        assert_eq!(String::try_from_sexp(label).unwrap(), "default");
        let input = scope.protect(vec![2i32, 3].into_sexp()).get();
        for callback in [table.values, table.array] {
            let value = scope.protect(callback(data, 1, [input].as_ptr())).get();
            assert_eq!(Vec::<i32>::try_from_sexp(value).unwrap(), [2, 3]);
        }
        let arg = scope.protect(1i32.into_sexp()).get();
        let value = scope
            .protect((table.ignored)(data, 1, [arg].as_ptr()))
            .get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 7);
        let other = scope
            .protect(ExternalPtr::new(Ledger(13)).into_sexp())
            .get();
        let value = scope
            .protect((table.other)(data, 1, [other].as_ptr()))
            .get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 13);

        let host = std::cell::RefCell::new(Override(10));
        let data = std::ptr::from_ref(&host).cast_mut().cast();
        let table = &__VTABLE_trait_generic_defaults_Totals_FOR_Override;
        let value = scope.protect((table.r_label)(data, 0, no_args)).get();
        assert_eq!(String::try_from_sexp(value).unwrap(), "overridden");
        let input = scope.protect(vec![2i32, 3, 4].into_sexp()).get();
        let value = scope
            .protect((table.array)(data, 1, [input].as_ptr()))
            .get();
        assert_eq!(Vec::<i32>::try_from_sexp(value).unwrap(), [2, 3, 4]);
    });
}

#[miniextendr]
pub trait Associated<T> {
    type Item;
    fn marker(&self) -> T;
    fn echoed(&self, value: Self::Item) -> Self::Item {
        value
    }
}

#[miniextendr(env, noexport)]
impl Associated<i32> for Ledger {
    type Item = i32;
    fn marker(&self) -> i32 {
        self.0
    }
}

#[test]
fn defaults_can_name_the_implementations_associated_types() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let host = std::cell::RefCell::new(Ledger(9));
        let data = std::ptr::from_ref(&host).cast_mut().cast();
        let arg = scope.protect(42i32.into_sexp()).get();
        let table = &__VTABLE_trait_generic_defaults_Associated_FOR_Ledger;
        let value = scope.protect((table.echoed)(data, 1, [arg].as_ptr())).get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 42);
    });
}
