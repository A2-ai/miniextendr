//! #2020: consumer methods must not intercept framework handle operations.
#![allow(dead_code)]

struct ConsumerFrameworkCalled;

// By-value methods intercept borrowing inherent methods on owned handles.
#[allow(clippy::wrong_self_convention)]
trait ConsumerFramework: Sized {
    fn protect_raw<T>(self, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn rooted<T>(self, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn get(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_sexp(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn column_raw<T>(self, _name: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn nrow(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn names(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn get_or_init<T>(self, _init: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn owner(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn data(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn vtable(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn downcast_cell<T>(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn rooted_native_borrow(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn native_borrow(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn expectation(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn convert<T>(self, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn borrow(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn borrow_mut(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn select<T>(self, _columns: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn select_rows<T>(self, _rows: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_mut_unchecked(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_ref_unchecked(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn take_for_consuming<T>(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn restore_after_consuming<T>(self, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_ptr(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn write<I, T>(self, _index: I, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn field<N, T>(self, _name: N, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn finish(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn debug_struct<T>(self, _name: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn is_data_frame(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn type_of(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_integer(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_real(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn as_logical(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn coerce<T>(self, _kind: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn raw_elt<T>(self, _index: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    // Exact by-value inherent methods are covered too.
    fn new_altrep<A, B>(self, _data1: A, _data2: B) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn new_altrep_unchecked<A, B>(self, _data1: A, _data2: B) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn sexptype(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn set_class_str<T>(self, _classes: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn set_row_names_int<T>(self, _n: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn arg<T>(self, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn named_arg<N, T>(self, _name: N, _value: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn eval<T>(self, _env: T) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn build(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
    fn into_inner(self) -> ConsumerFrameworkCalled { ConsumerFrameworkCalled }
}
impl<T> ConsumerFramework for T {}

include!("common/every_macro.rs");

fn lowered_calls() {
    let _ = miniextendr_api::r!(sum(c(1L, 2L), na.rm = TRUE));
    let _ = miniextendr_api::r!(base::sum(c(1L, 2L)));
}

fn main() {}
