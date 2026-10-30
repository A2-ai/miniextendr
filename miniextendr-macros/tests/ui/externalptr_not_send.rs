//! An `ExternalPtr<T>` handle is `!Send`, whatever `T` is. R can pass one
//! object twice (`f(x, x)`), so two handles share one `RefCell`, whose borrow
//! flag is not thread-safe: moving one handle into a scoped or spawned thread
//! would race the other on R's main thread.
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter {
    n: i32,
}

#[miniextendr]
pub fn bump_in_scope(a: ExternalPtr<Counter>, b: ExternalPtr<Counter>) -> i32 {
    std::thread::scope(|s| {
        s.spawn(move || a.borrow_mut().n += 1);
        b.borrow().n
    })
}

#[miniextendr]
pub fn drop_in_spawn(x: ExternalPtr<Counter>) {
    std::thread::spawn(move || drop(x));
}

fn main() {}
