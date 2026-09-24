use miniextendr_api::miniextendr;

#[miniextendr(wrap = "r8")]
pub fn unknown() -> i32 { 0 }

#[miniextendr(wrap = "s7")]
pub fn conflict() -> miniextendr_api::WrapAsR6<i32> { miniextendr_api::WrapAsR6(0) }

#[miniextendr]
pub fn nested() -> miniextendr_api::WrapAsR6<Vec<i32>> { miniextendr_api::WrapAsR6(vec![]) }

#[miniextendr]
pub fn argument(value: miniextendr_api::WrapAsR6<i32>) -> i32 { value.0 }

#[miniextendr(wrap = "r6", serialize)]
pub fn serialization() -> i32 { 0 }

#[miniextendr(wrap = "r6", unwrap_in_r)]
pub fn result_value() -> Result<i32, String> { Ok(0) }

#[miniextendr(wrap = "r6")]
pub unsafe extern "C-unwind" fn raw(value: miniextendr_api::SEXP) -> miniextendr_api::SEXP { value }

fn main() {}
