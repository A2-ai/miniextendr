# Plan: #1283 — fix struct-level `#[miniextendr(prefer = "native")]` unconditional E0119

Date: 2026-07-11. Anchors verified against origin/main @ 372841ec.
Branch: `fix/1283-prefer-native-e0119`.

**Sequencing (binding): dispatch after PR #1285 merges; first step
`git merge origin/main`.** The docs work item below edits lines that PR #1285
adds to `docs/MINIEXTENDR_ATTRIBUTE.md`. If #1285 is not merged when you
start, stop and report back.

## Verified state (all anchors re-checked on origin/main @ 372841ec)

**The E0119 mechanism.** `expand_struct` in
`miniextendr-macros/src/struct_enum_dispatch.rs:309-327` (the default /
ExternalPtr branch, which also serves `prefer = "native"`) emits BOTH:

1. `crate::externalptr_derive::derive_external_ptr(derive_input.clone())`
   (`struct_enum_dispatch.rs:312`) — which unconditionally calls
   `generate_into_external_ptr` (`miniextendr-macros/src/externalptr_derive.rs:1182`,
   invoked at `:1209`, emitting the marker impl at `:1187`:
   `impl IntoExternalPtr for #name`). That marker enables the blanket
   `impl<T: crate::externalptr::IntoExternalPtr> IntoR for T` at
   `miniextendr-api/src/into_r/large_integers.rs:493` (the marker trait itself
   is `miniextendr-api/src/externalptr.rs:328`).
2. `crate::list_derive::derive_prefer_rnative(derive_input)`
   (`struct_enum_dispatch.rs:315-316`, guarded by
   `attrs.prefer.as_deref() == Some("native")`) — the fn at
   `miniextendr-macros/src/list_derive.rs:496` emits a **concrete**
   `impl IntoR for #name` routing through `AsRNative`.

Two `IntoR` impls for the same type → E0119, unconditionally, for every
struct. Repro (verified via real `cargo check` from `rpkg/src/rust`,
rustc 1.97.0):

```rust
use miniextendr_api::prelude::*;

#[miniextendr(prefer = "native")]
pub struct Wrapped(pub i32);
```

```
error[E0119]: conflicting implementations of trait `miniextendr_api::IntoR` for type `Wrapped`
   = note: conflicting implementation in crate `miniextendr_api`:
           - impl<T> miniextendr_api::IntoR for T
             where T: IntoExternalPtr;
```

**Dead-code evidence.** No struct-level `prefer = "native"` usage exists
anywhere in the repo (grep `'prefer = "native"'` and `PreferRNativeType`,
excluding target dirs): the only usages are function-level
(`rpkg/src/rust/convert_pref_tests.rs:123-131`, tested by
`rpkg/tests/testthat/test-convert-pref.R` "prefer = 'native' attribute ..."),
and the **direct** `#[derive(RNativeType, PreferRNativeType)]` form (no
ExternalPtr) exercised in `miniextendr-api/src/macro_coverage/derive_matrix.rs`
— both unaffected by this fix.

**#870 distinction.** #870's guided conflict marker
(`prefer_conflict_marker`, `list_derive.rs:351`) fires only when two `Prefer*`
derives are stacked (duplicate fixed-name const → guided E0592). Here only ONE
`Prefer*` derive is present; the conflicting `IntoR` comes from the
`IntoExternalPtr` blanket, so #870's guidance never fires. Do not touch #870's
mechanism.

## The fix (committed — no alternatives)

Thread a marker-suppression flag: change
`externalptr_derive::derive_external_ptr(input: DeriveInput)` to
`derive_external_ptr(input: DeriveInput, emit_into_r_marker: bool)`. When
`false`, skip the `generate_into_external_ptr(&input)` call at
`externalptr_derive.rs:1209` (emit an empty `TokenStream` in its place);
everything else (`TypedExternal`, sidecar accessors, erased wrapper) is
emitted unchanged. There are exactly two callers: `miniextendr-macros/src/lib.rs:2076`
(the standalone `#[derive(ExternalPtr)]` proc macro — passes `true`,
behavior unchanged) and `struct_enum_dispatch.rs:312` (passes
`attrs.prefer.as_deref() != Some("native")`). Result: on the native path the
concrete `IntoR` from `derive_prefer_rnative` is the type's sole `IntoR`,
exactly what the docs table has always described. The other direction —
dropping the concrete `IntoR` from `derive_prefer_rnative` and relying on
blanket routing — is worse because `derive_prefer_rnative` is also the
backend of the standalone `#[derive(PreferRNativeType)]` (registered at
`miniextendr-macros/src/lib.rs:2347`), used without `ExternalPtr` in
`macro_coverage/derive_matrix.rs`; gutting it breaks that working, tested
form.

Accepted consequence (state in PR body, no issue needed — the mode never
compiled at all, so nothing regresses): a struct-level `prefer = "native"`
type does not implement `IntoExternalPtr`, so the call-site
`AsExternalPtr<T>` wrapper (`miniextendr-api/src/convert.rs:363`, bound
`T: IntoExternalPtr`) is not available on it.

## Work items (flat)

1. **Macro change** — `miniextendr-macros/src/externalptr_derive.rs`: add the
   `emit_into_r_marker: bool` parameter to `derive_external_ptr` (`:1201`);
   replace `let into_external_ptr = generate_into_external_ptr(&input);`
   (`:1209`) with a conditional:
   `let into_external_ptr = if emit_into_r_marker { generate_into_external_ptr(&input) } else { proc_macro2::TokenStream::new() };`
   Update the fn's doc comment (step 4 in its list) to say the marker is
   suppressed for struct-level `prefer = "native"` (cite #1283). Update the
   two callers: `lib.rs:2076` → `derive_external_ptr(input, true)`;
   `struct_enum_dispatch.rs:312` →
   `derive_external_ptr(derive_input.clone(), attrs.prefer.as_deref() != Some("native"))`.
   Also fix the stale dispatch doc comment at `struct_enum_dispatch.rs:191`
   (says `PreferRNative`; the real marker is `PreferRNativeType`).

2. **Compile-pass trybuild test** (decision: trybuild, not a unit test —
   sibling prefer/derive behavior is pinned via trybuild in
   `miniextendr-macros/tests/ui/` (`derive_prefer_conflict.rs` compile-fail)
   and the harness `miniextendr-macros/tests/ui.rs` already runs
   `t.pass("tests/ui/pass/*.rs")` with 8 existing pass tests; there is no
   unit-test convention for dispatch behavior). Add
   `miniextendr-macros/tests/ui/pass/struct_prefer_native.rs`:

   ```rust
   //! Compile-pass test: struct-level `#[miniextendr(prefer = "native")]`
   //! (#1283 regression — used to E0119 unconditionally: the ExternalPtr
   //! derive's IntoExternalPtr blanket IntoR collided with
   //! derive_prefer_rnative's concrete IntoR).

   #![allow(dead_code)]

   use miniextendr_macros::miniextendr;

   #[miniextendr(prefer = "native")]
   #[derive(Copy, Clone, miniextendr_api::RNativeType)]
   pub struct Wrapped(pub i32);

   fn main() {}
   ```

   (`miniextendr-api` is already a dev-dependency of miniextendr-macros;
   the existing pass tests use the same import pattern. `RNativeType` must be
   derived by the user — the attr does not emit it; `AsRNative` requires it.
   No `.stderr` file for pass tests.)

3. **rpkg fixture + testthat round-trip** — in
   `rpkg/src/rust/convert_pref_tests.rs` (keep related fixtures together),
   after the existing attr-form fixtures (`:100-131`), add:

   ```rust
   // Struct-level prefer = "native" (#1283): the attribute on the STRUCT
   // (not the fn) selects the type's default IntoR = AsRNative routing.
   #[miniextendr(prefer = "native")]
   #[derive(Copy, Clone, Debug, miniextendr_api::RNativeType)]
   pub struct StructPreferNative(pub i32);

   #[miniextendr]
   /// @title Struct-level prefer = "native" returns the native R type
   /// @rdname convert_pref_tests
   /// @description Regression fixture for #1283: a struct carrying
   ///   `#[miniextendr(prefer = "native")]` returned bare from a fn converts
   ///   via `AsRNative` to a length-1 integer vector.
   /// @examples
   /// struct_prefer_native(3L)
   pub fn struct_prefer_native(x: i32) -> StructPreferNative {
       StructPreferNative(x)
   }
   ```

   And in `rpkg/tests/testthat/test-convert-pref.R`, after the
   "prefer = 'native' attribute ..." test:

   ```r
   test_that("struct-level prefer = 'native' compiles and returns the native type (#1283)", {
     result <- struct_prefer_native(3L)
     expect_identical(typeof(result), "integer")
     expect_identical(result, 3L)
     expect_identical(result, hybrid_as_native(3L))
   })
   ```

   **New export → ×2 install rule**: `struct_prefer_native` is a new export;
   it is not runtime-callable until the second install (see commands block).
   Commit the regenerated `NAMESPACE` + `man/convert_pref_tests.Rd` in the
   same commit as the Rust change (`rpkg/R/miniextendr-wrappers.R` is
   gitignored — do NOT add it).

4. **Docs un-warning** — `docs/MINIEXTENDR_ATTRIBUTE.md`, editing exactly what
   PR #1285 (commit af8ba199) added:
   - Delete the entire 6-line `> **Warning**: ...` blockquote at lines
     508-513 of the #1285 version (begins
     `> **Warning**: struct-level \`prefer = "native"\` currently fails to compile`,
     ends `> the tested path (\`rpkg/src/rust/convert_pref_tests.rs\`).`)
     plus its trailing blank line.
   - In the table row at line 524 of the #1285 version, delete the tail
     `— intended; currently broken at the struct level, see #1283` so the
     row reads:
     `| \`#[miniextendr(prefer = "native")]\` | \`ExternalPtr\` + \`PreferRNativeType\` marker (the only \`prefer\` value that stays marker-only) |`
   - Do not touch the other prefer rows or the full-mode prose paragraph
     (#1268's content stays).
   - Note in passing (one sentence after the table is fine): struct-level
     `prefer = "native"` types don't implement `IntoExternalPtr`, so the
     `AsExternalPtr` call-site wrapper doesn't apply to them.

5. **gc-stress consideration** — explicitly: NO gc-stress fixture needed per
   #430. `StructPreferNative` is a `Copy` newtype over `i32`; neither the
   fixture nor the macro change stores SEXPs across allocations (no
   `Vec<SEXP>`, no sidecars, no generic-list buffers).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # re-verify at start of EVERY R batch
just worktree-sync                              # FIRST (fresh worktree rv/library)
git merge origin/main                           # picks up merged #1285 (never rebase)

# Rust-side verification (before any R install):
cargo test -p miniextendr-macros 2>&1 > /tmp/1283-macros-test.log   # trybuild ui incl. new pass test; Read the log
just test 2>&1 > /tmp/1283-just-test.log

# R side — new export needs the double install:
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1283-devtools-test.log
grep -E '\[ FAIL [0-9]+' /tmp/1283-devtools-test.log                # must be FAIL 0 (devtools::test always exits 0)

# CI clippy reproduction — three legs, -D warnings. Read the CURRENT feature
# list from .github/workflows/ci.yml `clippy_all` step (do NOT hard-code it):
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-targets --locked --features "<list from ci.yml clippy_all>" -- -D warnings
cargo clippy --workspace --all-targets --locked --features "<same list with full-codegen-s7 instead of full-codegen>" -- -D warnings

cargo fmt --all                                  # before push; CI rejects on rustfmt
git status --short                               # expected: macros x3 files, rpkg fixture, testthat, NAMESPACE, man/, docs x1
```

All compiling commands need `dangerouslyDisableSandbox: true`. Never end the
turn while a suite is running — foreground bounded polls only.

## Must NOT touch

- Function-level `prefer = "native"` behavior: `convert_pref_tests.rs`'s
  existing fixtures and every existing `test-convert-pref.R` test must stay
  green **unchanged** (additions only).
- #870's guided-error mechanism (`prefer_conflict_marker`,
  `list_derive.rs:351`) and the `derive_prefer_rnative` emission body
  (`list_derive.rs:496`) — the fix is entirely on the ExternalPtr-marker
  side.
- The resolution of the other prefer modes in `expand_struct`
  (`effective_list` / `effective_dataframe` / `effective_externalptr`,
  `struct_enum_dispatch.rs:206-212`) and the standalone
  `#[derive(ExternalPtr)]` proc macro's output (its caller passes `true`).

## Done criteria

- `#[miniextendr(prefer = "native")]` on a struct compiles (trybuild pass
  test green) and round-trips: `struct_prefer_native(3L)` returns integer
  `3L` identical to `hybrid_as_native(3L)`.
- The #1285 Warning block and "currently broken" row tail are removed from
  `docs/MINIEXTENDR_ATTRIBUTE.md`; grep `see #1283` in `docs/` returns
  nothing.
- `cargo test -p miniextendr-macros`, `just test`, `just devtools-test`
  (FAIL 0), all three clippy legs, and `cargo fmt --all` clean.
- Regenerated `NAMESPACE` + `man/convert_pref_tests.Rd` committed in sync
  with the Rust change; `rpkg/R/miniextendr-wrappers.R` NOT committed.
- PR body: `Fixes #1283`, notes the `AsExternalPtr`-unavailable consequence,
  and the standard AI-attribution format (TODO first line + details block).

## Escalation rule

If reality diverges from this plan — the flag threading breaks a caller this
plan didn't enumerate, the trybuild pass test cannot compile for a reason
other than the E0119 (e.g. a linker/dev-dep gap), the fixture's R round-trip
returns something other than an integer scalar, or #1285's docs lines are not
on main as cited — **stop, commit nothing further, and report back. Do not
improvise.**
