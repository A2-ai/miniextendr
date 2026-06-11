# `borrow_args` (#664): copy-vs-borrow audit of `&[T]` / `&str` arguments

Date: 2026-06-09

Issue #664 proposes a `#[miniextendr(borrow_args)]` knob that would emit `&[T]`
/ `&str` slice views over R's vector data pointers instead of copying into
`Vec<T>` / `String`. This audit determines what `#[miniextendr]` actually emits
for borrowed arguments *today*, before deciding whether the knob is needed.

## TL;DR

- **`&[T]` arguments already borrow zero-copy on `main`.** No knob needed; that
  part of #664 is moot.
- **`&str` arguments copied into an owning `String` unconditionally** — even on
  the main-thread default where a borrow is just as safe as `&[T]`. The copy is
  only *required* on the worker-thread path (`!Send`).
- **Fix shipped (this PR): `&str` borrows zero-copy by default on the
  main-thread path**, matching `&[T]`. The `String` copy is kept only for the
  worker path. No knob — it's a transparent, always-sound default.

## How borrowed arguments are lowered

Argument conversion lives in
`miniextendr-macros/src/rust_conversion_builder.rs`. The C-wrapper builder
(`c_wrapper_builder.rs`) chooses between two code paths based on thread strategy
(`lib.rs:913` — `requires_main_thread`; main thread is the default, worker is
opt-in via `#[miniextendr(worker)]`):

- **Main-thread default** → `build_conversion_stmts` → `build_conversions` →
  `build_conversion` (single scope; all `let`s run inside the same
  `with_r_unwind_protect` closure).
- **Worker** → `build_conversion_stmts_split` → `build_conversion_split`
  (two phases: owned conversions run on the main thread *before* the worker
  closure so they can be `move`d in; borrows run *inside* the closure).

### `&[T]` — BORROW (pre-existing, zero-copy)

`rust_conversion_builder.rs` (slice arm) emits:

```rust
let x = ::miniextendr_api::TryFromSexp::try_from_sexp(arg_0)?;
```

which resolves to the blanket `impl<T> TryFromSexp for &[T]`
(`miniextendr-api/src/from_r.rs:586`) →
`sexp.as_slice::<T>()` (`sexp_ext.rs:489`) →
`std::slice::from_raw_parts(DATAPTR_RO(sexp), len)`. **No allocation** — the
slice is a direct view over R's vector data.

### `&str` — COPY (before this PR)

The `&str` arm emitted an owning-`String` detour, on *both* paths:

```rust
let __owned_s: String = ::miniextendr_api::TryFromSexp::try_from_sexp(arg_0)?; // .to_owned() — HEAP COPY
let s: &str = ::std::borrow::Borrow::borrow(&__owned_s);
```

The `String` impl (`from_r/strings.rs:245`) calls `charsxp_to_str(...).to_owned()`
— a fresh heap allocation. Note the `&'static str` impl
(`from_r/strings.rs:47`) was already zero-copy (`charsxp_to_str`, no alloc); the
codegen just never used it for arguments.

The doc comment on the `&str` arm said *"Convert to String, then borrow ... This
allows the String to be moved into worker thread closures."* — i.e. the copy
existed for the worker path's `Send` requirement, but was applied universally.

## Why borrow-by-default is sound (no knob)

A `&str` / `&[T]` view over R's CHARSXP/data pool is valid only while the SEXP is
alive and ungc'd. On the **main-thread path** both conditions hold for the whole
call:

1. **Protect lifetime.** The argument SEXPs are parameters of the generated
   `extern "C-unwind"` wrapper. They are passed in by R's `.Call` machinery and
   remain reachable/protected for the entire wrapper body, which runs inside a
   single `with_r_unwind_protect` closure. The borrow never outlives the SEXP.

2. **No-store-beyond-the-call.** The borrow's lifetime is the anonymous lifetime
   of the `try_from_sexp` call inside the closure. The borrow checker forbids
   returning it or storing it past the call — `#[miniextendr]` already rejects
   explicit lifetime params (MXL112), so a user can't thread a longer lifetime
   out. Storing-beyond-the-call is a compile error, exactly as #664 asked for —
   and it's already guaranteed by `&[T]` today.

3. **Same thread.** Main-thread codegen never sends the borrow across threads,
   so the `!Send`-ness of an R-backed `&str` is a non-issue.

The worker path keeps the `String` copy precisely because (3) fails there: a
borrowed view over R memory is `!Send` and can't `move` into the worker closure,
and SEXP data must only be touched on the main thread.

## Why this is *not* a knob

Making borrow the main-thread default is strictly better than a `borrow_args`
knob:

- It is always sound (the SEXP is protected for the call; lifetime is scoped).
- It requires no user opt-in and no new surface area.
- It already matches the `&[T]` behaviour users get today, so it removes a
  surprising asymmetry (`&[T]` borrowed but `&str` copied).
- The worker path — the only place the copy is mandatory — is unaffected.

## Performance note (corrected evidence chain)

#664's deprioritisation cited `analysis/scaffolding-perf-roadmap.md` (option E),
`analysis/scaffolding-bench-2026-05-20.md`, and a `fast` knob that "already does
the heavy lifting." **None of those exist on `main` or any git ref** — they were
part of an unmerged perf sprint (commit `e3279011` is also not on `main`). So:

- The `fast`-covers-it mitigation is moot (no `fast` knob exists).
- The bench table in #664 is unverified (the bench file isn't in the tree).

The *other* mitigation #664 cited — `r_slice()` / `r_slice_mut()` in
`miniextendr-api/src/from_r.rs` — **does** exist, and is exactly what the `&[T]`
borrow path uses under the hood.

We deliberately did **not** re-run the missing benches. The change is a pure
removal of a per-`&str`-argument heap allocation on the hot main-thread path;
correctness and the avoided allocation are the justification, not a measured
delta. At small N the wrapper still dominates; the win shows up for large
strings / many string args.

## What shipped

- `rust_conversion_builder.rs`: split the `&str` arm on a new `zero_copy_str`
  flag. `build_conversion` (main thread) passes `true` → direct zero-copy borrow;
  `build_conversion_split` (worker) passes `false` → keep `String`-then-borrow.
- Unit tests: `test_str_conversion_main_thread_borrows_zero_copy` (single
  `TryFromSexp` binding, no `String`/`Borrow`) and
  `test_str_conversion_worker_copies_then_borrows` (owned `String` + borrow).
- rpkg fixtures: `str_borrow_len(s: &str) -> i32` (round-trips a borrowed arg)
  and `gc_stress_str_borrow()` (no-arg gctorture fixture exercising the
  zero-copy CHARSXP borrow under `gctorture(TRUE)`), plus testthat coverage.

## Recommendation

Close #664 as resolved: `&[T]` already borrowed; `&str` now borrows by default
on the main-thread path. A `borrow_args` knob is unnecessary.
