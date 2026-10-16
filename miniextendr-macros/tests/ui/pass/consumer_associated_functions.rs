//! #2020: type-qualified trait functions still see consumer blanket traits.
#![allow(dead_code)]

struct ConsumerAssociatedCalled;

trait ConsumerAssociated: Sized {
    fn from<T>(_value: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn try_from<T>(_value: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn default() -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn get_or_init_class() -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn rows_into_dataframe<T>(_rows: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    // Inherent associated functions must keep taking precedence too.
    fn try_from_dataframe<T>(_sexp: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn try_from_dataframe_par<T>(_sexp: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn try_from_sexp<T>(_sexp: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn to_dataframe_split<T>(_rows: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
}
impl<T> ConsumerAssociated for T {}

include!("common/every_macro.rs");

// An inherent function must not silently replace the registration trait either.
impl Repeated {
    fn get_or_init_class() -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
}

fn condition_single_pair() -> ! {
    miniextendr_api::rust_error!(data = ("value", 1), "refused");
}

fn condition_computed_pair(name: &str) -> ! {
    miniextendr_api::rust_error!(data = (name, 1), "refused");
}

fn condition_pair_list(name: &str) -> ! {
    miniextendr_api::rust_error!(data = [("value", 1), (name, 2)], "refused");
}

fn condition_keyed() -> ! {
    miniextendr_api::rust_error!(data = { value = 1 }, "refused");
}

fn main() {}
