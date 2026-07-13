# Plan: #1217 PR A — batch the macro-generated `CoercionMapping::Vec` argument path

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1217-macro-vec-coercion-batching`.

Scope: item 1 of #1217 ONLY (highest user impact — the path a bare
`#[miniextendr] fn f(x: Vec<u32>)` argument actually takes). Items 2+3 are
PR B (`plans/2026-07-11-issue-1217-prb-strict-tinyvec-batching.md`); item 4
(NA policy) is an open maintainer decision — DO NOT change NA behavior.
Related: #1192 (established the grammar + helper — merged), #1097 (outbound
analogue, separate plan).

## The defect (verified)

`miniextendr-macros/src/rust_conversion_builder.rs`, `CoercionMapping::Vec`
arm (`:517-555`): the generated conversion reads a `&[i32]`/`&[f64]` slice,
then

```rust
__r_slice.iter().copied()
    .map(::miniextendr_api::TryCoerce::<#target_elem>::try_coerce)
    .collect::<Result<Vec<_>, _>>()
```

— short-circuits at the first failing element, one message, no index
(`"failed to coerce parameter 'x' to Vec<u32>: element overflow, NaN, or
precision loss: <e>"`). Project principle: collect all errors in vectorized
ops.

## The fix

1. Replace the short-circuit collect in that arm's emitted code with an
   enumerate-and-accumulate loop over `__r_slice`, using the #1192
   accumulator `::miniextendr_api::from_r::BatchedErrors`
   (`miniextendr-api/src/from_r.rs:2011-2047` — `#[doc(hidden)] pub`
   precisely so macro-expanded downstream code can use it; verify the exact
   import path compiles from a user crate — rpkg is the proof). Per element:
   `Ok(v) => out.push(v)`, `Err(e) => errors.push(|| format!("invalid value
   at index {i}: {e}"))`. On `!errors.is_empty()`:
   `errors.into_error(<container label>)` where the container label is
   `Vec<{target_elem}>` (matching #1192's `into_error` grammar), and route
   the resulting error's `Display` through the EXISTING
   `make_rust_condition_value` call with the existing
   `error_msg_coerce`-prefixed format — final shape:
   `failed to coerce parameter 'x' to Vec<u32>: Vec<u32> conversion failed:
   invalid value at index 0: ...; invalid value at index 2: ...; and N more`.
   If that double-`Vec<u32>` read is deemed redundant, simplify the
   `error_msg_coerce` prefix to `failed to coerce parameter '{param}'` and
   let `into_error`'s container carry the type — pick THIS second form
   (decision baked: prefix = `failed to coerce parameter '{param}'`,
   container = `Vec<{target_elem}>`), and keep the trailing
   `element overflow, NaN, or precision loss` hint OUT (the per-index `{e}`
   Display already says why).
2. Keep the FIRST match (slice extraction, `error_msg_convert`) untouched —
   wrong-SEXP-type is a scalar failure, not per-element.
3. Macro insta snapshots: any snapshot embedding the `CoercionMapping::Vec`
   emission changes — rebaseline after diff review (`.snap.new` → `mv`).
4. testthat: update the message pins for the bare-Vec coerce path — grep
   `rpkg/tests/testthat/test-scalar-conversions.R` and
   `rpkg/tests/testthat/test-r-coerce.R` for greps on
   `element overflow|failed to coerce parameter` and update to the new
   grammar. Add the headline regression: an existing bare-Vec fixture (e.g.
   `conv_vec_u8_len(x: Vec<u8>)`, `rpkg/src/rust/conversions.rs:550` — or a
   sibling with a signed target if u8 can't show two distinct failures) fed
   `c(-1, 5, 300)` (two failing elements for u8) reports BOTH indices in one
   error. If no existing fixture can demonstrate two failure indices, add
   `conv_vec_u32_sum(x: Vec<u32>) -> f64` beside `conv_vec_u8_len` (new
   export → ×2 install rule applies).
5. UI tests (`miniextendr-macros/tests/ui/*.stderr`): not expected to change
   (this is emission, not error wording at macro level); if trybuild output
   moves, stop and report (never `TRYBUILD=overwrite` locally — #1239).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2 only needed if item 4 adds a new fixture
cargo test -p miniextendr-macros 2>&1 > /tmp/1217a-macros.log   # Read it
just test 2>&1 > /tmp/1217a-rust.log
just devtools-test 2>&1 > /tmp/1217a-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1217a-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE`/`man` if (and only if) a new fixture export
was added.

## Must NOT touch

- NA semantics of the coerced path (item 4 of #1217 — open decision; the
  batching must preserve exactly which elements fail and which pass today).
- `strict.rs` / `tinyvec_impl.rs` (PR B).
- `BatchedErrors` itself (shared helper — additive use only).
- The `CoercionMapping::Scalar`/other arms of the builder.

## Done criteria

- `f(c(-1, 5, -3))` on a bare unsigned-Vec arg reports all failing indices
  in one batched error; suites, snapshots, three clippy legs green;
  PR references #1217 (partial — item 1 of 4; do NOT `Fixes #1217`).

## Escalation rule

If reality diverges from this plan — the arm's emission differs from the
verified shape, `BatchedErrors` is not reachable from expanded code, NA
behavior changes in any test — **stop, commit nothing further, and report
back. Do not improvise.**
