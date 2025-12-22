# miniextendr-macros

Procedural macros for exporting Rust functions and types to R.

Most users should depend on `miniextendr-api` and use its re-exports, but this
crate can be used directly if you want only the macros.

## Macros

### `#[miniextendr]`

Exports a Rust function to R and generates the necessary C and R wrappers.

```rust
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### `miniextendr_module!`

Registers exported functions and ALTREP classes for a package/module.

```rust
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod mypkg;
    fn add;
}
```

### `#[r_ffi_checked]`

Wraps `extern "C-unwind"` blocks with thread assertions in debug builds.

### Derives

- `ExternalPtr` – implement `TypedExternal` for external pointers.
- `RNativeType` – mark newtypes as R-native for vector conversions.

## Notes

- Attributes like `#[miniextendr(unsafe(main_thread))]` and
  `#[miniextendr(coerce)]` can be used to control wrapper behavior.
- See `docs.md` and the `miniextendr-api` docs for full usage examples.

## Testing

This crate uses `trybuild` for compile-fail tests:

```sh
cargo test -p miniextendr-macros
```
