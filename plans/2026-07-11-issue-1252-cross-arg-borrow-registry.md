# Plan: #1252 — debug borrow registry for cross-argument slice aliasing

Date: 2026-07-11. Anchors verified against main @ 6de43e9b.
Branch: `fix/1252-cross-arg-borrow-registry`.

Maintainer decision (2026-07-11): implement (issue's option 1), don't
document-and-park. The gap: the #1104/#1240 wrapper guard compares
top-level parameter SEXPs (`Vec<_>` params are invisible —
`slice_borrow_kind` returns `None`), and the `Vec<&mut [T]>` conversions
guard only intra-list duplicates — so `f(list(v), v)` yields two live
`&mut` over one buffer with no guard firing.

## Verified anchors

- Wrapper guard: `c_wrapper_builder.rs:434-453` (doc), `build_alias_guard`
  `:454`, emission sites `:509`/`:550`/`:577`/`:607`; debug-only; message
  `"aliasing slice arguments: parameters ..."` `:483-485`. KEEP unchanged
  (it names the offending R parameters — better diagnostics for `f(x, x)`).
- Intra-list guard: `miniextendr-api/src/from_r/references.rs` — duplicate
  data-pointer checks with message "list contains duplicate elements..."
  at `:211`, `:234`, `:282`, `:308` (the `Vec<&mut [T]>` /
  `Vec<Option<&mut [T]>>` impl family); empty slices skipped (R's `0x1`
  sentinel).
- Test pattern to mirror: `rpkg/src/rust/conversions.rs:499`
  `alias_probe(a: &'static mut [i32], b: &'static mut [i32])`;
  `debug_assertions_enabled()` `:517` gates expectations;
  `rpkg/tests/testthat/test-alias-guard.R:30`
  `expect_error(alias_probe(x, x), "aliasing")`.

## Design (all decisions resolved)

New module `miniextendr-api/src/borrow_registry.rs` (never `mod.rs`),
declared from `lib.rs`:

- `thread_local! { static BORROWS: RefCell<Vec<(usize, bool)>> }` —
  `(data_ptr as usize, is_mut)`. A `Vec`, not a set: N is tiny (arity of
  one call), no hashing.
- `pub struct BorrowScope` RAII: `BorrowScope::enter()` **clears** the
  thread-local (self-healing against entries stranded by an R longjmp
  bypassing drops — see the ~8-byte-leak note in
  `miniextendr-api/CLAUDE.md`) and returns the guard; `Drop` clears again.
- `pub fn register_borrow(ptr: *const u8, len: usize, is_mut: bool)
  -> Result<(), String>`: no-op `Ok(())` when `len == 0` (0x1 sentinel);
  conflict when an existing entry has the same ptr AND (`is_mut` or the
  existing entry is mut); on conflict return an `Err` naming the collision
  ("this argument zero-copy borrows a vector already mutably borrowed by an
  earlier argument in the same call; pass distinct vectors or copy with
  as-owned types"); otherwise push and `Ok`.
- **Release builds**: both functions compile to no-ops via internal
  `#[cfg(debug_assertions)]` blocks (`enter()` returns a unit-like guard).
  Codegen stays cfg-free; zero release overhead. Match #1104's debug-only
  framing — document that on the module.
- Registration sites (all in `from_r/references.rs`): the zero-copy slice
  impls — `&mut [T]` (register `is_mut = true`), `&[T]` (register
  `is_mut = false`; a shared borrow over a mutably-borrowed buffer is also
  UB), and the `Vec<&mut [T]>` / `Vec<Option<&mut [T]>>` element loops
  (register each element `is_mut = true`). **Replace** the four local
  duplicate-pointer checks (`:211`/`:234`/`:282`/`:308`) with
  `register_borrow` calls — the registry subsumes intra-list detection
  (same-ptr mut+mut conflicts) and adds cross-argument coverage; keep the
  empty-skip behavior (the registry's `len == 0` no-op provides it). The
  conversion maps the `Err(String)` into its existing error type exactly
  like the current duplicate message does (grep how `:211` wraps it).
  `&str`/`Vec<&str>` do NOT register: no `&mut str` conversion exists, so
  no mut party is possible — note this in the module doc.
- Wrapper emission (`c_wrapper_builder.rs`): in every wrapper variant that
  emits `#conversion_stmts` (grep — the with_r_unwind_protect closure sites
  `:550`/`:577` and the worker variant), emit
  `let __mx_borrow_scope = ::miniextendr_api::borrow_registry::BorrowScope::enter();`
  immediately BEFORE the conversion statements (inside the closure, after
  `#alias_guard`). The binding is intentionally held across the call body
  (drop at closure end); silence unused-binding with a leading underscore
  only if clippy objects — prefer `let _scope = ...` shape used elsewhere
  (grep `let _` guard idioms in the crate and match).

## Work items (flat order)

1. `borrow_registry.rs` module + `lib.rs` wiring; rustdoc with the
   `f(list(v), v)` motivating example from #1252.
2. Rewire the four intra-list checks to `register_borrow`; register in the
   direct `&mut [T]` / `&[T]` impls.
3. `BorrowScope::enter()` emission in `c_wrapper_builder.rs` per design.
4. api unit tests (in-module, no R needed): conflict matrix (mut/mut same
   ptr, mut/shared same ptr, shared/shared same ptr = OK, distinct ptrs =
   OK, len 0 skipped, scope clear resets).
5. rpkg fixture beside `alias_probe` (`conversions.rs`):
   `alias_probe_list(list: Vec<&'static mut [i32]>, direct: &'static mut [i32]) -> i32`
   (sum both, mirroring `alias_probe`'s body/doc style).
6. testthat in `test-alias-guard.R` (gated on `debug_assertions_enabled()`
   like `:30`): `alias_probe_list(list(v), v)` errors matching "borrow";
   `alias_probe_list(list(v1), v2)` works; `alias_probe_list(list(v, v), w)`
   still errors (intra-list regression pin, now via registry);
   zero-length element + same-buffer direct arg does NOT error (0x1 skip);
   `alias_probe(x, x)` unchanged (wrapper guard still first line).
7. No gc-stress fixture: the registry stores raw `usize`, never SEXPs
   (#430 not triggered) — state this in the PR body.
8. Snapshots: wrapper emission adds one line inside the closure — macro
   snapshot tests that pin full C-wrapper output will rebaseline
   (mechanical; review one), and trybuild `.stderr` should be untouched
   (the 5 pre-existing `derive_dataframe_enum_*` mismatches stay — #1239).
9. Docs: the conversion/aliasing section that documents #1104's guard
   (grep `aliasing` in `docs/`) gains the cross-argument coverage note.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1252-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2: new fixture export
just test 2>&1 > /tmp/1252-rust.log
just devtools-test 2>&1 > /tmp/1252-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1252-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

NOTE the debug/release trap: the testthat assertions are conditional on
`debug_assertions_enabled()`. Verify what profile `just rcmdinstall` used
in the worktree (call `debug_assertions_enabled()` in the test log header
or an R one-liner) and confirm the guard tests actually EXECUTED (grep the
devtools log for the test names, not just FAIL 0).

## Must NOT touch

- The wrapper-level SEXP-identity guard (`build_alias_guard`) — it stays,
  unchanged, as the named-parameter first line of defense.
- Conversion behavior for owned types (`Vec<T>`, `Box<[T]>`) — they copy
  and never register.
- Release-build codegen output must be byte-identical in behavior (no-op
  guard); do not cfg-gate the generated code itself.

## Done criteria

- `f(list(v), v)` errors in debug builds with a clean batched-style
  conversion error (no UB, no abort); intra-list and wrapper-level guards
  regress nothing; release builds unaffected; api unit tests + suites +
  snapshots + three clippy legs green; `Fixes #1252`.

## Escalation rule

If reality diverges from this plan — the conversion error-wrapping at the
four rewired sites can't carry the registry message, the scope-guard
emission point isn't uniform across wrapper variants, drops demonstrably
don't run on a path that strands entries (enter()-clears failing to
self-heal) — **stop, commit nothing further, and report back. Do not
improvise.**
