# Plan: #348 — remove call attribution entirely (audit, then remove)

Date: 2026-07-11. Anchors verified against main @ 6de43e9b.
Branch: `refactor/348-remove-call-attribution`.

Maintainer decision (2026-07-11, recorded on #348): call attribution buys
nothing — all errors forward to the R wrapper anyway — so `.call` slots, the
`no_call_attribution` knob, and the match.call() shipping machinery come out
"from everywhere in the miniextendr world". This plan is the audit + removal.

**Sequencing**: dispatch AFTER `ci/1244-fast-default-leg` and all
`fix/1217*`/`fix/1097` PRs have merged (they touch the same fixtures/strict
files); dispatch BEFORE `feat/1017-per-method-fast-override` (whose semantics
this PR changes — #1017 gets re-planned against the post-removal knob set).
First step on checkout: `git merge origin/main`.

## How attribution works today (verified, the removal map)

R wrapper → C wrapper → condition value → R raise helper:

1. **R side**: `DotCallBuilder`
   (`miniextendr-macros/src/r_wrapper_builder.rs:341-423`) emits
   `.Call(C_f, .call = match.call(), args…)`; `call_expr: Option<String>`
   field (`:352`), `null_call_attribution()` (`:396-399`) emits
   `.call = NULL` for lambda contexts (R6 finalizer/deep_clone, S7 property
   getter/setter/validator), `build()` (`:402-422`) defaults to
   `match.call()`.
2. **C side**: `c_wrapper_builder.rs:343-344` prepends
   `__miniextendr_call: SEXP` as the first param of EVERY `#[miniextendr]`
   C wrapper; `:255` documents how `numArgs` accounts for the synthetic
   param (read that comment and find the corresponding `+1` in the
   registration emission — keep the accounting consistent when the param
   goes). The value is passed as `Some(__miniextendr_call)` into
   `with_r_unwind_protect(f, call)` (sites `:556`, `:583`, worker variant
   `:653`) and into `make_rust_condition_value(msg, kind, None, call)` in
   the RNG panic handler (`:533-538`).
3. **api**: `unwind_protect.rs:589` `with_r_unwind_protect<F>(f, call:
   Option<SEXP>)`; `:411` `with_r_unwind_protect_or_raise` (legacy,
   test/bench only) also takes `call`. `error_value.rs`:
   `make_rust_condition_value_with_data` builds the 5-element tagged list
   (message/kind/class/call/data); `make_rust_condition_value` is the
   no-data wrapper. **PROTECT-sensitive file** (#344 `af6b4875`): dropping
   the `call` element shrinks the list to 4 — keep the PROTECT pattern
   intact, adjust indices carefully, and re-run the condition gctorture
   tests (see Verification).
4. **R raise helper**: emitted into wrappers.R by
   `miniextendr-api/src/registry.rs:1185-1217`
   (`.miniextendr_raise_condition(.val, .call_default)`): line `:1187`
   `.call <- (if (is.null(.val$call)) .call_default else .val$call)`. Every
   generated check line already passes `sys.call()` as `.call_default`
   (`method_return_builder.rs:32-70`: `condition_check_lines`,
   `condition_check_inline_block`, `standalone_body`).

**The key fact making removal near-free**: the raise helper ALREADY falls
back to the wrapper's `sys.call()` whenever `.val$call` is NULL (that is
exactly how `no_call_attribution`/lambda contexts work today, and
`rpkg/tests/testthat/test-fast-fixtures.R` pins it: "error$call falls back
to sys.call()"). Removal = make that fallback the only path. `conditionCall`
on `rust_error` conditions remains the user-facing wrapper call — sys.call()
(call as written) instead of match.call() (canonicalized argument names).
That rendering difference is the entire user-visible delta.

## Step 1 — audit (pre-specified branches, no judgment calls)

Run and record in the PR body:

(a) Grep for consumers of the canonicalized call:
    `grep -rn "conditionCall" rpkg/ minirextendr/ tests/ docs/` — classify
    each hit: does any assert on match.call()-canonicalized output
    (named-arg normalization) rather than just "the call names the wrapper"?
(b) `cargo bench -p miniextendr-bench --bench error_path_attribution` (or
    `just bench` equivalent — check `miniextendr-bench/CLAUDE.md` for the
    invocation) BEFORE the change; re-run AFTER; record both.
(c) Check the trait-ABI wrappers
    (`miniextendr_impl_trait/r_wrappers.rs`) and cross-package consumer for
    any `.call`-slot dependency beyond the same DotCallBuilder emission.

- **Branch A (expected)**: no consumer depends on canonicalization; nothing
  outside the mapped machinery touches `.val$call`. → proceed to Step 2.
- **Branch B**: some consumer genuinely needs the captured match.call()
  (not satisfiable by sys.call() at raise time) — **stop, commit nothing
  further, report to the maintainer with the evidence.**

## Step 2 — removal (flat order)

1. `r_wrapper_builder.rs`: delete `call_expr` field +
   `null_call_attribution()`; `build()` emits `.Call(C_f, args…)` /
   `.Call(C_f)`. Fix every caller of `null_call_attribution()` (grep; R6
   finalizer/deep_clone, S7 property lambdas) — they just use the plain
   builder now. Update the DotCallBuilder rustdoc + module examples.
2. `c_wrapper_builder.rs`: drop the `__miniextendr_call` param (`:343-344`),
   the `Some(__miniextendr_call)` at all pass sites (`:537`, `:556`, `:583`,
   `:653` — grep for stragglers), and fix the `numArgs` accounting per the
   `:255` comment. Check `externalptr_derive.rs` needs nothing (sidecars
   never had the slot) — this PR makes every wrapper sidecar-shaped, which
   RESOLVES the "two C-wrapper codegen paths" gotcha: update
   `miniextendr-macros/CLAUDE.md` (gotcha bullet + `c_wrapper_builder.rs`
   layout line) and the root-CLAUDE.md architecture pointer if it mentions
   the call slot; mirror in the sibling AGENTS.md files.
3. `unwind_protect.rs`: remove the `call: Option<SEXP>` param from
   `with_r_unwind_protect` (`:589`) and `with_r_unwind_protect_or_raise`
   (`:411`); fix all callers (bench + engine tests included).
4. `error_value.rs`: remove the `call` element from
   `make_rust_condition_value_with_data`/`make_rust_condition_value`
   (5-element list → 4). PRESERVE the PROTECT pattern; adjust
   `VECTOR_ELT` indices AND the R-side reader in lockstep (grep
   `\$call` / index usage). This file segfaulted under R-devel GC in #344 —
   change minimally, keep protection spans identical in shape.
5. `registry.rs:1185-1217`: raise helper drops `.call_default` fallback
   logic — signature `.miniextendr_raise_condition(.val, .call)` where
   `.call` IS the wrapper's `sys.call()` (keep passing it from the check
   lines; `message` kind keeps `call = NULL` as today `:1207`). Update the
   helper's comment block and the parse test at `:2513-2514` if the
   signature string changes.
6. `method_return_builder.rs:32-70`: check lines keep `sys.call()` — only
   the helper's second-arg name/meaning changes (no `.val$call` branch).
7. Knob removal: delete `no_call_attribution` from `MiniextendrFnAttrs`
   (`miniextendr_fn.rs:1145-1147`, parse `:1295`/`:1352-1353`, accepted-list
   `:1070`, rustdoc `:1096`,`:1100`) and from `ImplAttrs`
   (`miniextendr_impl.rs:864-865` region, keyword arm `:1128`; per-method
   inheritance `:739-744`). `fast` becomes sugar for `no_preconditions`
   alone (keep both keywords + `no_fast`; update rustdoc). CLAUDE.md rule:
   fields removed from `MiniextendrFnAttrs`/`ImplAttrs` → update the
   destructuring in `lib.rs` AND all 6 class generators.
8. Fixtures/tests: `rpkg/src/rust/fast_fixtures.rs` — the
   `no_call_attribution`-only fixtures collapse into the `no_preconditions`
   story; rewrite `rpkg/tests/testthat/test-fast-fixtures.R` conditionCall
   assertions: ALL wrappers now behave like the old fallback path (error$call
   == the wrapper call via sys.call()); delete the match.call()-vs-NULL
   distinction tests. `test-conditions-comprehensive.R` — update any
   `.val$call` / conditionCall expectations (grep it first).
9. Bench: rewrite or delete `miniextendr-bench/benches/error_path_attribution.rs`
   (it benchmarks a removed distinction). If deleted, note the before/after
   numbers from the audit in the PR body instead.
10. UI tests: `#[miniextendr(no_call_attribution)]` must now fail with the
    unknown-keyword error — add a trybuild fixture pinning that;
    `TRYBUILD=overwrite` for the NEW fixture only (#1239: never rebaseline
    the 5 pre-existing `derive_dataframe_enum_*` mismatches).
11. Docs: the fast-knob page (grep `no_call_attribution` in `docs/` —
    FAST_WRAPPERS or similar), `docs/CONDITIONS*.md` if it names `.call`,
    scaffolding perf evidence cross-references. Update `#348`-adjacent
    prose in `minirextendr` templates if any template R code carries
    `.call =` (grep `minirextendr/inst/templates` — then
    `just templates-approve` if the rpkg→templates delta changes).
12. Regen: wrappers.R/wasm_registry regenerate on install (gitignored);
    commit regenerated `NAMESPACE`/`man` only if roxygen output changes
    (knob docs may appear in man pages — check the diff). Cross-package
    wrappers (`producer.pkg`/`consumer.pkg` tracked wrappers.R) regenerate
    via `just cross-document` — commit those (CI sync gate #1276).
13. Snapshots: EVERY wrapper-emission snapshot loses `.call = match.call()`
    — large mechanical rebaseline; diff one of each kind (fn, each class
    system, trait, vctrs) carefully, `mv` the rest.

## Verification (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2 install: wrapper ABI changed everywhere
just test 2>&1 > /tmp/348-rust-test.log          # Read it
just devtools-test 2>&1 > /tmp/348-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/348-devtools.log   # devtools::test always exits 0
# Condition-path gctorture (error_value.rs changed — #344 history):
MINIEXTENDR_SKIP_STRESS= Rscript -e 'testthat::test_file("rpkg/tests/testthat/test-conditions-comprehensive.R")' 2>&1 > /tmp/348-gcstress.log
just cross-test 2>&1 > /tmp/348-cross.log        # trait-ABI numArgs changed
just minirextendr-test 2>&1 > /tmp/348-minir.log
just templates-check                             # after templates-approve if delta changed
just r-cmd-check 2>&1 > /tmp/348-rcmdcheck.log   # zero new WARNINGs
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- The tagged-condition transport itself (kinds, classes, `data` splicing —
  #1192) — only the `call` element goes.
- `with_r_unwind_protect_shim` (`unwind_protect.rs:507`) — trait-ABI vtable
  path, no call param today; leave it.
- `no_preconditions` behavior and the `fast-default` cargo feature (its
  meaning narrows to preconditions-only — that's a doc change, not a
  behavior change to preconditions).
- ALTREP guard modes, `Rf_error` sites (MXL300 scope), `error_direct` (#665
  is parked).

## Done criteria

- No `.call =`, `__miniextendr_call`, `no_call_attribution`, or
  `null_call_attribution` anywhere outside historical docs
  (`git grep` each); `conditionCall(e)` still names the user-facing wrapper
  call in tests; audit results + bench delta in the PR body; all suites,
  snapshots, three clippy legs, templates-check green; `Fixes #348`;
  PR body notes that #1017's plan must be re-cut (per-method `fast` =
  per-method `no_preconditions` only) and links this decision.

## Escalation rule

If reality diverges from this plan — Branch B fires, the PROTECT
restructuring in `error_value.rs` needs more than element removal, the
numArgs accounting doesn't isolate to one `+1`, a consumer of `.val$call`
exists outside the mapped machinery — **stop, commit nothing further, and
report back. Do not improvise.**
