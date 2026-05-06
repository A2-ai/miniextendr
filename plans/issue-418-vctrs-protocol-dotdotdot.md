# Issue #418 — vctrs protocol-method R wrappers should accept `...`

## Decision: Option 3 from the issue (explicit drop)

Generate the protocol-method R wrapper as:

    format.Currency <- function(amounts, ...) {
      .val <- .Call(C_Currency__format_currency, .call = match.call(), amounts)
      ...
    }

Trailing `...` accepts (and discards) extra args, making the method compatible
with `format(x, nsmall = 2)`-style invocation. The underlying `.Call` ignores
`...` since the Rust function has a fixed signature.

## Implementation

`miniextendr-macros/src/miniextendr_impl/vctrs_class.rs`, in the static-method
codegen branch (around line 310-348 added by #414's rectification):

When `is_protocol = true` (the static method is annotated with
`#[miniextendr(vctrs(<protocol>))]`), append `, ...` to the R formals.
Only protocol methods get `...` — regular static helpers keep their
existing fixed-formals shape.

## Tests

- Update the snapshot test `vctrs_protocol_method_override` in
  `tests.rs` so the assertion is
  `assert!(wrapper.contains("format.Currency <- function(amounts, ...)"))`.
- Add a snapshot assertion that a *non*-protocol static helper still emits
  without `, ...` (`assert!(wrapper.contains("currency_symbol <- function(amounts)"))`).

## Acceptance

- [ ] Existing snapshots updated and `cargo test -p miniextendr-macros` passes.
- [ ] Generated `format.Currency` has `function(amounts, ...)`.
- [ ] Non-protocol static helpers unchanged.
- [ ] `just check && just clippy` clean.

## Out of scope

- Passing `...` *into* the Rust call — Rust has fixed signatures; would require
  variadic arg support.
- Other vctrs protocols (cast, ptype2, etc.) — same pattern applies but only
  `format` is currently exercised; extend if other protocols add fixtures.
