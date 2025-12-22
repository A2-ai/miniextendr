//! Unwind protection benchmarks.

use miniextendr_api::unwind_protect::with_r_unwind_protect;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn unwind_protect_noop() {
    let out: i32 = with_r_unwind_protect(|| 42, None);
    divan::black_box(out);
}

#[divan::bench]
fn direct_noop() {
    let out: i32 = 42;
    divan::black_box(out);
}
