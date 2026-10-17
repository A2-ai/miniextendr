use miniextendr_api::miniextendr;

#[miniextendr]
pub trait UnsafeRequired {
    unsafe fn read(&self) -> i32;
}

#[miniextendr]
pub trait UnsafeDefault<T> {
    unsafe fn echo(&self, value: T) -> T { value }
}

#[miniextendr]
pub trait UnsafeStatic {
    unsafe fn create() -> i32 { 1 }
}

fn main() {}
