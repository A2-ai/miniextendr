use miniextendr_api::{ExternalPtr, SEXP, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

#[miniextendr]
pub trait RustOnly {
    #[miniextendr(skip)]
    unsafe fn unchecked(&self, value: i32) -> i32 { value }
    fn safe(&self) -> i32 { 1 }
}
impl RustOnly for Host {}
#[miniextendr(env, noexport)]
impl RustOnly for Host {}

#[miniextendr]
pub trait GenericRustOnly<T> {
    #[miniextendr(skip)]
    unsafe fn unchecked(&self, value: T) -> T { value }
    fn safe(&self) -> T;
}
#[miniextendr(env, noexport)]
impl GenericRustOnly<i32> for Host {
    unsafe fn unchecked(&self, value: i32) -> i32 { value }
    fn safe(&self) -> i32 { 2 }
}

#[miniextendr]
pub fn safe_adapter(value: i32) -> i32 {
    // This fixture's unchecked operation has no extra preconditions.
    unsafe { RustOnly::unchecked(&Host, value) }
}

// Raw registration generates no Rust wrapper around this entry point.
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn raw_identity(value: SEXP) -> SEXP {
    value
}

fn main() {
    assert_eq!(safe_adapter(4), 4);
}
