# Panic source locations in miniextendr

When a Rust `panic!` becomes an R error, miniextendr appends the Rust source
location to the R condition message:

```
Error in panic_location_main_direct(NULL):
  boom-main-direct
  (at panic_location_tests.rs:46)
```

The location comes from the process **panic hook** (`PanicHookInfo::location()`),
not from `#[track_caller]`.

## How it works

`miniextendr-api/src/backtrace.rs` installs a panic hook at package init. On
every panic the hook records the panic's `(file, line)` into a per-thread,
take-once slot *before* it decides whether to print the stderr traceback
(`MINIEXTENDR_BACKTRACE`). At the panic-stringification sites the framework
folds that location into the message via
`unwind_protect::panic_message_with_location`, producing the trailing
`\n(at file:line)`.

The hook fires on the *panicking* thread, so the slot is per-thread:

- **Main-thread `#[miniextendr]` fns** (anything taking a `SEXP` / borrowed R
  view, returning a `SEXP`, taking `...`, or `check_interrupt`) — panic, hook,
  and `catch_unwind` all run on main; the location is read on main.
- **Worker-thread `#[miniextendr(worker)]` fns** — the panic and hook fire on
  the worker; the location is folded **once, at the worker (origin) thread**
  (`worker.rs`), and the finished message crosses back to main verbatim. No
  double-append, no clobber by main-side re-panics.

## Which errors carry a location

Only the **generic panic** path (`panic!`, `.unwrap()`, `assert!`, …) gets the
`(at …)` suffix — `error_value::kind::PANIC`. The typed condition path is left
byte-for-byte unchanged and carries **no** location:

- `error!()` / `warning!()` / `message!()` / `condition!()`
- returning `Result::Err`
- returning `Option::None`

These are intentional, user-shaped conditions with their own class layering and
messages; a `(at …)` suffix would be noise.

## Why there is no automatic `#[track_caller]`

Earlier versions auto-added `#[track_caller]` to every `#[miniextendr]` fn. That
was **removed** (#1121) because it actively defeats the location above:

- `#[track_caller]` makes `Location::caller()` (and a panicking
  `.unwrap()` / `assert!`) resolve to the function's **call site**. For a
  `#[miniextendr]` fn the caller is the macro-generated wrapper, whose span is
  the `#[miniextendr]` attribute line — i.e. generated glue, not your `panic!`.
- With the attribute gone, a **direct** panic's hook location points at the
  real `panic!` line in your source.
- **Nested** panics (your fn calls a plain Rust helper that panics) were always
  reported at the helper's `panic!` line, with or without the attribute — the
  helper isn't `#[track_caller]`.

Investigation note (`tc-invest`, 2026-07-08): before this feature the automatic
`#[track_caller]` had **no** effect on the R-facing message (which never
contained a location); the only observable difference was the
`MINIEXTENDR_BACKTRACE=true` stderr backtrace. So dropping it costs nothing and
is exactly what makes the newly-surfaced location correct.

The sibling automatic `#[inline(never)]` injection is unrelated and stays — it
keeps function names in stack traces and preserves the worker/unwind boundary.

## Opting into the stderr traceback

Set `MINIEXTENDR_BACKTRACE=true` (or `1`) to also print the full Rust panic
traceback to stderr via Rust's default hook, in addition to the R error. The
`(at file:line)` suffix in the R message is always present, regardless of this
variable.
