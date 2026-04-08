+++
title = "`#[track_caller]` in miniextendr"
weight = 54
+++

The `#[miniextendr]` macro automatically adds `#[track_caller]` to Rust functions for better panic location reporting.

## What it does

When a panic occurs (e.g., from `.unwrap()`, `.expect()`, or `assert!()`), Rust reports the source location. With `#[track_caller]`, the location propagates through the call chain, showing where the panicking function was *called from* rather than where the panic *originated*.

## Example

```rust
#[miniextendr]
pub fn process_data(x: i32) {
    let result: Option<i32> = None;
    result.unwrap();  // panics here
}
```

**Without `#[track_caller]`:** Panic reports the location inside `Option::unwrap()` in the standard library.

**With `#[track_caller]` (automatic):** Panic reports `lib.rs:4` - the line where `.unwrap()` was called in your code.

## When it helps

The automatic `#[track_caller]` is most useful for panics from:
- `.unwrap()` and `.expect()` on `Option` and `Result`
- `assert!()`, `assert_eq!()`, `assert_ne!()`
- Any function that uses `std::panic::Location::caller()`

## When it doesn't help

`#[track_caller]` does NOT affect:
- Direct `panic!()` calls - these always report their own location
- Argument conversion errors in the generated wrapper - these report the `#[miniextendr]` line
- `extern "C-unwind"` functions - these don't support `#[track_caller]`

## Propagation through call chains

For `#[track_caller]` to propagate through multiple function calls, ALL functions in the chain need the attribute:

```rust
#[track_caller]
fn helper() {
    let x: Option<i32> = None;
    x.unwrap();
}

#[miniextendr]  // automatically adds #[track_caller]
pub fn my_function() {
    helper();  // panic location will be here if helper has #[track_caller]
}
```

Without `#[track_caller]` on `helper()`, the panic location shows inside `helper()` where `unwrap()` was called.

## Skipped cases

The macro does NOT add `#[track_caller]` when:
1. The function already has `#[track_caller]`
2. The function has an explicit ABI (e.g., `extern "C-unwind"`)
