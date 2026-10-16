//! Default methods keep their Rust bodies and expose concrete ABI/R wrappers,
//! whether the trait is generic or not.
mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, IntoR, TryFromSexp, miniextendr};

mod declaration {
    type LocalValues<T> = Vec<T>;
    type LocalNumber = i32;
    #[miniextendr_api::miniextendr]
    pub trait Totals<T = i32, const N: usize = 2> {
        fn total(&self) -> T;

        /// The inherited label.
        #[miniextendr(r_name = "label")]
        fn r_label(&self) -> String {
            "default".into()
        }

        fn values(&self, values: LocalValues<T>) -> LocalValues<T> {
            values
        }
        fn maybe_values(&self, values: Option<LocalValues<T>>) -> LocalValues<T> {
            values.unwrap_or_default()
        }
        fn total_slice(&self, values: &[LocalNumber]) -> LocalNumber {
            values.iter().sum()
        }
        extern "Rust" fn abi_values(&self, values: LocalValues<T>) -> LocalValues<T> {
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

// Deliberately shadow the private declaration aliases with different types.
pub type LocalValues<T> = Vec<Vec<T>>;
pub type LocalNumber = f64;

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
        "total",
        "label",
        "values",
        "maybe_values",
        "array",
        "ignored",
        "other",
        "title",
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
        let table = &__VTABLE_trait_default_methods_Totals_FOR_Ledger;
        let no_args = std::ptr::null();
        let total = scope.protect((table.total)(data, 0, no_args)).get();
        assert_eq!(i32::try_from_sexp(total).unwrap(), 9);
        let label = scope.protect((table.r_label)(data, 0, no_args)).get();
        assert_eq!(String::try_from_sexp(label).unwrap(), "default");
        let input = scope.protect(vec![2i32, 3].into_sexp()).get();
        for callback in [table.values, table.array, table.abi_values] {
            let value = scope.protect(callback(data, 1, [input].as_ptr())).get();
            assert_eq!(Vec::<i32>::try_from_sexp(value).unwrap(), [2, 3]);
        }
        let total = scope
            .protect((table.total_slice)(data, 1, [input].as_ptr()))
            .get();
        assert_eq!(i32::try_from_sexp(total).unwrap(), 5);
        let pointer = scope.protect(ExternalPtr::new(Ledger(9)).into_sexp()).get();
        let value = scope
            .protect(C_trait_default_methods_Ledger__Totals__values(
                miniextendr_api::SEXP::nil(),
                pointer,
                input,
            ))
            .get();
        assert_eq!(Vec::<i32>::try_from_sexp(value).unwrap(), [2, 3]);
        for (argument, expected) in [(input, vec![2, 3]), (miniextendr_api::SEXP::nil(), vec![])] {
            let value = scope
                .protect(C_trait_default_methods_Ledger__Totals__maybe_values(
                    miniextendr_api::SEXP::nil(),
                    pointer,
                    argument,
                ))
                .get();
            assert_eq!(Vec::<i32>::try_from_sexp(value).unwrap(), expected);
        }
        let total = scope
            .protect(C_trait_default_methods_Ledger__Totals__total_slice(
                miniextendr_api::SEXP::nil(),
                pointer,
                input,
            ))
            .get();
        assert_eq!(i32::try_from_sexp(total).unwrap(), 5);
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
        let table = &__VTABLE_trait_default_methods_Totals_FOR_Override;
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
        let table = &__VTABLE_trait_default_methods_Associated_FOR_Ledger;
        let value = scope.protect((table.echoed)(data, 1, [arg].as_ptr())).get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 42);
    });
}

// A non-generic trait's omitted defaults get the R wrappers a generic trait's
// do: instance and static defaults, the declaration's `r_name`, nothing for a
// skipped one, and argument types read in the declaration's module.
mod plain_declaration {
    type LocalWords = Vec<String>;
    type LocalNumber = i32;
    #[miniextendr_api::miniextendr]
    pub trait Plain {
        fn read(&self) -> i32;

        fn echo(&self, words: LocalWords) -> LocalWords {
            words
        }
        fn maybe(&self, words: Option<LocalWords>) -> LocalWords {
            words.unwrap_or_default()
        }
        fn total(&self, values: &[LocalNumber]) -> LocalNumber {
            values.iter().sum()
        }
        #[miniextendr(r_name = "label")]
        fn r_label(&self) -> String {
            "plain".into()
        }
        fn ignored(&self, _: i32) -> i32 {
            7
        }
        fn kind() -> String {
            "plain".into()
        }

        #[miniextendr(skip)]
        fn rust_only(&self, _: std::sync::Mutex<()>) {}
    }
}

use plain_declaration::Plain;

// Deliberately shadow the private declaration alias with a different type.
pub type LocalWords = Vec<i32>;

#[derive(ExternalPtr)]
pub struct Reader(i32);

#[miniextendr(env, noexport)]
impl Plain for Reader {
    fn read(&self) -> i32 {
        self.0
    }
}

#[test]
fn non_generic_defaults_get_wrappers_too() {
    let wrappers = R_WRAPPERS_Reader_Plain_IMPL.content;
    for name in ["read", "echo", "maybe", "total", "label", "ignored", "kind"] {
        assert!(
            wrappers.contains(&format!("Reader$Plain${name} <- function(")),
            "{wrappers}"
        );
    }
    assert!(
        wrappers.contains("Reader$Plain$ignored <- function(.mx_self, arg0)"),
        "{wrappers}"
    );
    assert!(!wrappers.contains("rust_only"));
    assert!(!wrappers.contains("$r_label <-"));
    Reader(1).rust_only(std::sync::Mutex::new(()));

    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let pointer = scope.protect(ExternalPtr::new(Reader(9)).into_sexp()).get();
        let input = scope
            .protect(vec!["one".to_owned(), "two".to_owned()].into_sexp())
            .get();
        let value = scope
            .protect(C_trait_default_methods_Reader__Plain__echo(
                miniextendr_api::SEXP::nil(),
                pointer,
                input,
            ))
            .get();
        assert_eq!(Vec::<String>::try_from_sexp(value).unwrap(), ["one", "two"]);
        for (argument, expected) in [(input, 2), (miniextendr_api::SEXP::nil(), 0)] {
            let value = scope
                .protect(C_trait_default_methods_Reader__Plain__maybe(
                    miniextendr_api::SEXP::nil(),
                    pointer,
                    argument,
                ))
                .get();
            assert_eq!(Vec::<String>::try_from_sexp(value).unwrap().len(), expected);
        }
        let numbers = scope.protect(vec![2i32, 3].into_sexp()).get();
        let value = scope
            .protect(C_trait_default_methods_Reader__Plain__total(
                miniextendr_api::SEXP::nil(),
                pointer,
                numbers,
            ))
            .get();
        assert_eq!(i32::try_from_sexp(value).unwrap(), 5);
        let value = scope
            .protect(C_trait_default_methods_Reader__Plain__kind(
                miniextendr_api::SEXP::nil(),
            ))
            .get();
        assert_eq!(String::try_from_sexp(value).unwrap(), "plain");
    });
}
