# Plan: #1017 — per-method `fast`/`no_fast` override inside impl blocks

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1017-per-method-fast-override`.

Note: #666 (block-level `fast` + `fast-default` cargo feature) is CLOSED
(re-landed via PR #1210) — this is its explicitly-deferred follow-up. #667
(should `internal` imply fast) is a separate open decision — do not touch
`internal` semantics here.

## Verified state

- Block level: `ImplAttrs` parses `no_preconditions`/`no_call_attribution`
  as `Option<bool>` (`miniextendr_impl.rs:864-865`, keyword arm `:1128`),
  with `fast`/`no_fast` sugar; methods inherit as plain bools
  (`miniextendr_impl.rs:739-744` — "Inherited from ImplAttrs").
- Bare-fn level: `MiniextendrFnAttrs` already accepts
  `no_preconditions, no_call_attribution, fast, no_fast`
  (`miniextendr_fn.rs:1070` accepted-keyword list, fields `:1133-1146`).
- Per-method `#[miniextendr(...)]` attrs inside impl blocks already exist as
  a parse surface (e.g. `s7(no_shortcut)`) — find the per-method attr parse
  point in `miniextendr_impl.rs` (grep `no_shortcut`) and extend it, do not
  invent a second parser.

## Semantics (from the issue's acceptance criteria — bake exactly)

- Method-level `fast` ⇒ both knobs `Some(true)`; `no_fast` ⇒ both
  `Some(false)`; individual `no_preconditions`/`no_call_attribution` set
  their own knob `Some(true)`. `fast` + `no_fast` on one method =
  `compile_error!` (mirror however the fn-level parser rejects that combo —
  read it first; if it doesn't reject, make BOTH levels reject in this PR
  and note it).
- Resolution per method per knob: method-level `Some(x)` wins over the
  block-level resolved bool (which itself already folds `fast-default`).
- Applies to all 6 class systems — the resolution must happen where the
  per-method bools at `:739-744` are populated, BEFORE the 6 generators
  consume them, so the generators need no per-system logic. If any generator
  reads `ImplAttrs` directly instead of the per-method bools, fix it to read
  the resolved per-method values (that's the point of the change).
- CLAUDE.md rule applies: fields added to `MiniextendrFnAttrs`/`ImplAttrs`/
  `ParsedImpl` (or the per-method struct) → update the destructuring in
  `lib.rs` AND all 6 class generators.

## Work items

1. Parse + resolve per the semantics above.
2. UI tests (`miniextendr-macros/tests/ui/`): (a) `fast`+`no_fast` on one
   method errors; (b) whatever placement-invalid case the existing
   per-method attr rejects (mirror its fixture) — the issue asks for the
   parse-error case explicitly. New `.stderr` via
   `TRYBUILD=overwrite cargo test -p miniextendr-macros` for NEW fixtures
   only; CI output is authoritative on mismatch (#1239).
3. Macro unit tests: block `fast` + method `no_fast` → method's resolved
   knobs false; block plain + method `fast` → true; block
   `no_preconditions` only + method `no_call_attribution` → mixed.
4. rpkg fixtures: extend `rpkg/src/rust/fast_fixtures.rs` (mirror its
   existing style, `:80-130` has the FastCounter impls): one impl block
   `#[miniextendr(env, fast)]` with a `#[miniextendr(no_fast)]` method, and
   one plain block with a `#[miniextendr(fast)]` method. testthat in
   `test-fast-fixtures.R`: the no_fast method raises the R-side
   stopifnot error on bad input (not `rust_error`); the fast method raises
   `rust_error`; call attribution present/absent accordingly (copy the
   file's existing conditionCall assertions). New exports → ×2 install.
5. Snapshots rebaseline (insta) where impl emission changes.
6. Docs: the fast-knob docs page (grep `no_fast` in `docs/`) gains the
   per-method override with the issue's example.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-macros 2>&1 > /tmp/1017-macros.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1017-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1017-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- `fast-default` crate-wide resolution (#1210) and the #1244 leg fixtures.
- `internal` semantics (#667 pending decision).
- Bare-fn knob behavior.

## Done criteria

- Both acceptance-criteria directions work in all 6 class systems (fixture
  covers env; unit tests cover resolution generically); parse errors pinned;
  suites + snapshots + three clippy legs green; `Fixes #1017`.

## Escalation rule

If reality diverges from this plan — the per-method attr surface can't carry
these keywords without grammar conflicts, a generator bypasses the resolved
bools in a way that can't be fixed locally — **stop, commit nothing further,
and report back. Do not improvise.**
