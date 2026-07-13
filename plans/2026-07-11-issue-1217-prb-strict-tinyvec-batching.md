# Plan: #1217 PR B — batch strict `checked_vec_*` + tinyvec `coerce_slice_to_vec`

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1217-strict-tinyvec-batching`.

Scope: items 2+3 of #1217. Item 1 is PR A
(`plans/2026-07-11-issue-1217-pra-macro-vec-coercion-batching.md` — no file
overlap; either merge order works). Item 4 (NA policy) is an open maintainer
decision — DO NOT change NA behavior. Reuses #1192's
`BatchedErrors` (`miniextendr-api/src/from_r.rs:2011-2047`) and its grammar.

## Item 2 — strict outbound `checked_vec_*` (`miniextendr-api/src/strict.rs`)

Verified shape: `checked_vec_i64_into_sexp` (`:85`) and
`checked_vec_u64_into_sexp` (`:106`) `panic!` on the FIRST out-of-range
element, message
`"strict conversion failed: i64 value {x} is outside R integer range (...); use a non-strict function to allow lossy f64 widening"`,
no index. `checked_vec_isize/usize` (`:126-132`) delegate. The Option
variants `checked_vec_option_i64/u64` (`:137,:161`, delegates `:184-185`)
share the shape.

1. In each of the four non-delegating fns: walk with `enumerate`, collect
   in-range values, accumulate failures into `BatchedErrors`
   (`errors.push(|| format!("invalid value at index {i}: {x} is outside R
   integer range ({lo}..={hi})"))` — keep the existing range wording per
   element, minus the shared hint). After the walk, if `!errors.is_empty()`,
   `panic!` ONCE with:
   `"strict conversion failed: {batched}; use a non-strict function to allow lossy f64 widening"`
   where `{batched}` is `errors.into_error("Vec<i64>")`'s Display (or format
   the same grammar directly if `into_error`'s `SexpError` wrapper reads
   awkwardly in a panic — decision baked: format directly with the SAME
   `invalid value at index <i>: ...; and N more` grammar, container label
   `Vec<i64>`/`Vec<u64>`/`Vec<Option<i64>>`/`Vec<Option<u64>>`).
   Keep the leading `strict conversion failed:` prefix — R-side tests pin it.
2. Preserve: NA sentinel handling in the Option variants exactly as-is (only
   the error aggregation changes); the exclusive `i32::MIN` lower bound on
   the i64 path (`x > i32::MIN as i64` — NA sentinel exclusion).
3. Unit tests: extend the existing strict.rs test module (or add one
   following crate convention) — a two-bad-element `Vec<i64>` panics with
   both indices; an 11+-bad-element vector produces `and N more`.
4. R-side pins: grep `rpkg/tests/testthat/` for `strict conversion failed`
   and update any grep that pinned the old single-value message shape.

## Item 3 — tinyvec `coerce_slice_to_vec` (`miniextendr-api/src/optionals/tinyvec_impl.rs`)

Verified: `:288-299` — `.map(...).collect()` short-circuit +
`SexpError::InvalidValue(format!("{e:?}"))` (Debug, inconsistent with the
crate-wide `{e}` Display from #1192).

5. Rewrite with enumerate + `BatchedErrors` (same pattern as #1192's
   `coerce_slice_to_vec` in `from_r.rs` — read that fn and mirror it), with
   `{e}` Display per element; container label matches the target
   (`TinyVec<...>` — use whatever type label the surrounding impls at
   `:324,:365` present; pick the same string both call sites report today if
   one exists, else `TinyVec`).
6. Tests: tinyvec is feature-gated — run
   `cargo test -p miniextendr-api --features tinyvec` and extend its test
   module with the two-bad-element + `and N more` cases.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1217b-api.log            # Read it
cargo test -p miniextendr-api --features tinyvec 2>&1 > /tmp/1217b-tinyvec.log
just configure && just rcmdinstall && just force-document
# (no new exports — single install)
just test 2>&1 > /tmp/1217b-rust.log
just devtools-test 2>&1 > /tmp/1217b-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1217b-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

No snapshot churn expected (api-crate only). Strict runtime coverage rides
the weekly `r6-default` leg (`strict-default` rides with it in the
feature-legs matrix) — note in the PR body that the leg exercises it.

## Must NOT touch

- `rust_conversion_builder.rs` (PR A's file).
- NA semantics anywhere; the strict scalar (non-vec) paths; `BatchedErrors`.
- `error_value.rs` (PROTECT-sensitive) — panics here are pre-FFI, no change.

## Done criteria

- Both strict vec families and the tinyvec path report ALL failing indices
  in one error with `{e}` Display; unit tests pin two-failure and `and N
  more` shapes; suites + three clippy legs green;
  PR references #1217 (items 2+3 — do NOT `Fixes #1217`; item 4 remains).

## Escalation rule

If reality diverges from this plan — anchors don't match, a testthat pin
depends on the panic aborting at the first element (behavioral coupling),
tinyvec's container label is ambiguous across call sites — **stop, commit
nothing further, and report back. Do not improvise.**
