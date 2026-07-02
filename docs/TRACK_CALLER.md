# `#[track_caller]` in miniextendr

The `#[miniextendr]` macro automatically adds `#[track_caller]` to Rust functions.

## What it actually does

`#[track_caller]` changes what `std::panic::Location::caller()` reports inside
a function: instead of the exact line where `caller()` (or a panicking
`.unwrap()` / `.expect()` / `assert!`) sits, the location resolves to the
function's **call site**. When several `#[track_caller]` functions call each
other, the location keeps walking up until it exits the attributed chain.

For a `#[miniextendr]` function, the caller is the macro-generated wrapper,
whose call-site span resolves to the **`#[miniextendr]` attribute line** of
your function. The observable behavior (pinned by the
`track_caller_is_active()` / `track_caller_chain_location()` fixtures in
`rpkg/src/rust/r_wrapper_attrs.rs`):

```rust
#[miniextendr]              // <- panic locations report THIS line
pub fn process_data(x: i32) {
    let result: Option<i32> = None;
    result.unwrap();        // NOT this line
}
```

- **Without the automatic attribute**, `.unwrap()` (which is itself
  `#[track_caller]` in std) would report the exact `.unwrap()` line.
- **With it**, the location walks past the function body to the wrapper's
  call site — the `#[miniextendr]` line.

So the automatic attribute trades line-level precision inside the function
for a stable function-level location: a panic always identifies *which
exported function* failed, in *your* source file, never a location inside
generated code or the standard library.

## Propagation through call chains

Marking helpers `#[track_caller]` extends the walk — their reported location
also resolves to the `#[miniextendr]` line, not to the helper call site:

```rust
#[track_caller]
fn helper() {
    let x: Option<i32> = None;
    x.unwrap();             // reports the #[miniextendr] line below
}

#[miniextendr]              // <- reported location
pub fn my_function() {
    helper();
}
```

Without `#[track_caller]` on `helper()`, the panic reports the `.unwrap()`
line inside `helper()` — standard Rust behavior.

## Where the location surfaces

The panic *message* transported to R (the `rust_panic` condition) contains
only the payload text, not the location. The location appears on stderr via
the panic hook, and in `Location::caller()`-based code like the fixtures
above.

## Skipped cases

The macro does NOT add `#[track_caller]` when:
1. The function already has `#[track_caller]`
2. The function has an explicit ABI (e.g., `extern "C-unwind"` — the
   attribute is not supported there)
