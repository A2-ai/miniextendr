# Scaffolding-perf roadmap — handoff for next session

State as of 2026-05-20. This is the **forward-looking plan** consolidating
the work in `scaffolding-bench-2026-05-20.md`,
`scaffolding-deep-findings-2026-05-20.md`, and
`scaffolding-perf-investigation-plan.md`. Read those first if you want the
evidence; this doc covers what's done, what's pending, and how to finish.

## Status

### Done

| Item | Where |
|---|---|
| M1+M6 measurement: per-line R-wrapper attribution | `scaffolding-strip-bench.R` |
| M9: bytecompile sensitivity (≤ 5 % impact) | `scaffolding-bench-deep.R` |
| M10: 5-rep variance (CV 0.0–2.6 %) | same |
| M11: hand-strip installed wrapper cross-check (41 ns from hand-written) | `scaffolding-bench-installed.R` |
| M12: multi-arg / multi-shape sweep | same |
| M13: Rprof over testthat (wrapper is 1.1 % of testthat time) | `scaffolding-bench-rprof.R` |
| **M2: C-side flamegraph attribution** — `with_r_unwind_protect` = ~38 ns, conversion = ~27 ns, R-side `.Call` dominates the floor (~125 ns) | `c_side_attribution.rs` |
| **M4: error-path attribution** — C-side panic catch + build = ~6 μs, R-side re-raise = ~7 μs, panic infrastructure itself is the bigger half | `error_path_attribution.rs` |
| **Fn-level `#[miniextendr(no_preconditions)]`** | `miniextendr_fn.rs`, `lib.rs` |
| **Fn-level `#[miniextendr(no_call_attribution)]`** | same |
| **Fn-level `#[miniextendr(fast)]` bundle alias** | same |
| **Impl-level `#[miniextendr(r6, fast)] impl` propagated to all 6 class generators** | `ImplAttrs`, `ParsedImpl`, `MethodContext::with_fast_flags`, `r_class_formatter.rs`, `miniextendr_impl/{s4,s7}_class.rs` |
| `rpkg/src/rust/fast_fixtures.rs` — 8 fixture fns (6 standalone + 2 R6 classes) | new file |
| `test-fast-fixtures.R` — 13 testthat tests, all pass (5967/5967) | new file |
| Bench validates fn-level 8.25× / impl-level 2.19–3.36× speedups | `scaffolding-fast-bench.R` |
| **Commit e3279011** — committed | git log |

### Pending — high priority

(P1–P4 are DONE — moved to the "Done" table above.)

### Deferred — tracked as GitHub issues

All deferred items live in the issue tracker with justifications and labels.
Re-read the issue bodies before picking up: M2/M4 evidence may have shifted
priorities since the last roadmap revision.

| Item | Issue | Label | Justification (one line) |
|---|---|---|---|
| `infallible` knob (option C / P5) | [#663](https://github.com/A2-ai/miniextendr/issues/663) | enhancement | M2 says only ~38 ns savings; complexity outweighs |
| `borrow_args` knob (option E / P6) | [#664](https://github.com/A2-ai/miniextendr/issues/664) | enhancement | Wrapper dominates at small N; matters only for large-N hot paths |
| `error_direct` knob (option F / P7) | [#665](https://github.com/A2-ai/miniextendr/issues/665) | enhancement | M4 confirms ~7 μs savings half; worth it but error path isn't usually hot |
| `default-fast` cargo feature (P8) | [#666](https://github.com/A2-ai/miniextendr/issues/666) | enhancement, good first issue | 3-line change; ship when a project wants it |
| Should `internal` imply `fast`? (P9) | [#667](https://github.com/A2-ai/miniextendr/issues/667) | suggestion | Audit required; could be a silent regression |
| S7 dispatch deep-dive (P10 / M5) | [#668](https://github.com/A2-ai/miniextendr/issues/668) | suggestion | Upstream / docs only; not a miniextendr fix |
| Surface the new knobs in docs (P11/P12) | [#669](https://github.com/A2-ai/miniextendr/issues/669) | documentation | Knobs are functional but undocumented |
| Typed-error transport (post-M4) | [#670](https://github.com/A2-ai/miniextendr/issues/670) | suggestion | Bigger redesign; potentially halves the remaining C-side error cost |

## Long-term design principles (to honor in subsequent sessions)

1. **Independent knobs first, bundles compose them.** `no_preconditions` and
   `no_call_attribution` are independent; `fast` is just a parse-arm that
   sets both. When new knobs land (`infallible`, `error_direct`,
   `borrow_args`), expose them as independent flags first, then consider a
   `fast_max` or `unsafe_fast` bundle.

2. **Error-UX degradation is opt-in.** Even the "match.call is cosmetic"
   change is wrapped in `no_call_attribution`. We never silently make
   error messages worse without the user asking. The one exception:
   `internal` may eventually imply `fast` (P9) because internal fns
   don't have user-facing error UX.

3. **`unsafe` gate for skipping the unwind machinery.** `infallible` and
   `error_direct` both can crash R if misused. Spell them with `unsafe`
   syntax (parallel to `_unchecked` FFI variants) so they stand out at
   call sites: `#[miniextendr(unsafe(infallible))]`.

4. **Per-fn scoping by default, project-wide via cargo features only when
   broad opt-in is desired.** `default-fast` should be tightly scoped
   (CI-internal experiments, framework-internal crates).

5. **Every new knob ships with a measurement.** No knob without a
   `bench_X` fixture in `rpkg/src/rust/fast_fixtures.rs` (or similar)
   and a `scaffolding-X-bench.R` script under `analysis/`. The bench
   must run in < 1 min so it's not a maintenance burden.

6. **The cost-attribution table is the source of truth.** When deciding
   whether to build knob X, refer to the M1+M6+M11+M12 numbers, not to
   intuition. The deep-findings doc has the per-layer breakdown.

7. **No changes to default codegen** without strong evidence the success
   path improves *and* no semantic regression. The current default
   (with stopifnot + match.call + post-check) is conservative for good
   reason — it's what shipped, and changing it would silently affect
   error messages and `.call` slots across every user's codebase.
   `internal`-implies-`fast` (P9) is the *one* case where I think the
   default-shift is safe.

## How to pick up next session

Quick start:

```bash
# 1. Confirm fn-level work is still installed + tests pass
cd /Users/elea/Documents/GitHub/miniextendr
Rscript -e 'library(miniextendr); fast_i32_fast(42L)'

# 2. Resume P1 — impl-method codegen
#    Read miniextendr-macros/src/miniextendr_impl.rs:738 (ImplAttrs)
#    and the 6 class generators in miniextendr_impl/*.rs
#    Mirror no_preconditions + no_call_attribution fields.

# 3. After codegen change:
just configure
just rcmdinstall                                # ~7 min
just force-document                             # regenerates wrappers.R + NAMESPACE
just devtools-test                              # confirm no regressions

# 4. Add an impl-method fixture to fast_fixtures.rs (e.g. SimpleCounter::value
#    with #[miniextendr(fast)]) and bench R6/S4/S7 dispatch with the knob on.
```

Notes:
- Pre-commit hook (`.githooks/pre-commit`) enforces that
  `R/miniextendr-wrappers.R` is staged with matching `NAMESPACE`. Use
  `git config core.hooksPath .githooks` once per clone.
- Compilation needs `dangerouslyDisableSandbox: true` when invoked
  through the Bash tool. Plain Rscript benches don't.
- The `default-fast` feature (P8) lives in `miniextendr-macros/Cargo.toml`
  alongside `default-strict` and friends. Mirror the parse-arm pattern
  used in `MiniextendrFnAttrs::parse`.

## File inventory

### Code (Rust)
- `miniextendr-macros/src/miniextendr_fn.rs` — `MiniextendrFnAttrs` struct, parse arms, struct init
- `miniextendr-macros/src/lib.rs` — destructure, conditional `.call` emission, conditional precondition emission
- `rpkg/src/rust/fast_fixtures.rs` — 6 fixture fns
- `rpkg/src/rust/lib.rs` — `mod fast_fixtures;`

### Tests (R)
- `rpkg/tests/testthat/test-fast-fixtures.R` — 9 tests

### Generated (do not edit by hand)
- `rpkg/R/miniextendr-wrappers.R`
- `rpkg/NAMESPACE`
- `rpkg/man/fast_fixtures.Rd`

### Bench / analysis (read-only, dated)
- `analysis/scaffolding-bench-2026-05-20.md` — first-pass measurement
- `analysis/scaffolding-deep-findings-2026-05-20.md` — M9–M13 deep findings
- `analysis/scaffolding-perf-investigation-plan.md` — original plan; superseded by this roadmap for what's pending
- `analysis/scaffolding-bench.R` + `-output.txt` — first-pass script
- `analysis/scaffolding-strip-bench.R` + `-output.txt` — M1+M6
- `analysis/scaffolding-bench-deep.R` + `-output.txt` — M9+M10+M12
- `analysis/scaffolding-bench-installed.R` + `-output.txt` — M11
- `analysis/scaffolding-bench-rprof.R` + `-output.txt` — M13
- `analysis/scaffolding-fast-bench.R` + `-output.txt` — end-to-end fast knob validation
- `analysis/scaffolding-perf-roadmap.md` — **this file**

## Completion state

P1 (impl methods) and P2 (commit) shipped in **commit e3279011**.

M2 (P3) and M4 (P4) measurement done; both surface new evidence that
sharpens the case against `infallible` (#663) and clarifies what
`error_direct` (#665) buys.

What's left is tracked in the issues above; pick whichever next.

## Headline numbers (recap)

| Variant | min (ns) | Speedup |
|---|---:|---:|
| 1-arg standalone fn (default) | 2870 | — |
| 1-arg standalone fn (`fast`) | **369** | **7.78×** |
| 3-arg standalone fn (default) | 4551 | — |
| 3-arg standalone fn (`fast`) | **533** | **8.54×** |
| R6 method `value()` (default) | 2337 | — |
| R6 method `value()` (`fast`) | **1066** | **2.19×** |
| R6 method `add(1L)` (default) | 3854 | — |
| R6 method `add(1L)` (`fast`) | **1148** | **3.36×** |

Floor accounting (`fast`, 1-arg standalone, 287 ns total):

- ~38 ns: C-side `with_r_unwind_protect` machinery
- ~30 ns: TryFromSexp + IntoR for i32
- ~125 ns: R-side `.Call` symbol lookup + arg marshalling (irreducible)
- ~94 ns: R closure dispatch + inherits/attr post-check

Error path (`demo_error` ≈ 21 μs vs `stop()` ≈ 7 μs):

- ~7 μs: C-side panic catch + tagged SEXP build
- ~7 μs: R-side `.miniextendr_raise_condition` → `stop(structure(...))`

---

## Status (2026-06-13)

The original sprint commit (`e3279011`) that produced this roadmap and the
`fast` knob infrastructure was **never merged**. The work has since re-landed
piecemeal under the G6 issue group:

- **`fast` knob base + `fast-default` cargo feature** — re-landed via PR #1018
  (#666), a fresh re-implementation against current macro internals.
- **`error_direct` C-side direct error raise** — landed via PR #950 (#665).
- **#667 (`internal` ⇒ `fast`)** — closed as wontfix; the repo's own test suite
  asserts error UX against `internal`-marked fixtures, so coupling doc-visibility
  to error-UX degradation would break those tests. The escape hatch is the
  explicit `#[miniextendr(internal, fast)]` spelling or the crate-wide
  `fast-default` feature, both shipped by PR #1018.
- **M2 (C-side) + M4 (error-path) attribution benches** — re-landed in this PR
  alongside this roadmap and the supporting `scaffolding-*` evidence files.

The numbers above are the original `e3279011` measurements, preserved as dated
evidence. Re-run `analysis/scaffolding-fast-bench.R` (or the new
`cargo bench` targets `c_side_attribution` / `error_path_attribution`) against
an installed `fast-default` build to refresh them.
