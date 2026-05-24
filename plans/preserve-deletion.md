# Delete deprecated `preserve.rs` module

## Base branch

`origin/refactor/api-sys-split` (PR #675 still open at start of work). The
renamed `pub mod sys` (was `pub mod ffi`) means `preserve.rs` references
`crate::sys::PairListExt` — already correct on this branch.

## Scope

`miniextendr-api/src/preserve.rs` is marked **Deprecated** in its own
docstring and at `lib.rs:619-621`:

> "Deprecated: DLL preserve list. Use ProtectPool or R_PreserveObject instead.
>  Kept for benchmark comparisons."

There are zero non-doc, non-bench callers of `preserve::insert` /
`preserve::release` / `preserve::count`. Per CLAUDE.md "No backwards
compatibility": delete, don't shim.

## Confirmed (`rg` triage)

### Live callers — none
- No call sites in `miniextendr-api/src/` outside the file itself.
- No call sites in `miniextendr-macros/`, `miniextendr-engine/`,
  `miniextendr-lint/`, `rpkg/`, `minirextendr/`, or cross-package tests.
- `miniextendr-api/src/allocator.rs` uses `R_PreserveObject_unchecked`
  (the R API) — `docs/ALLOCATOR.md` is **incorrect** when it says
  `preserve::insert` (aspirational/stale). Fix as part of this PR.

### Files to delete (`git rm`)
- `miniextendr-api/src/preserve.rs` (~300 LOC, the module)
- `miniextendr-api/tests/preserve.rs` (only-consumer integration test)
- `miniextendr-bench/benches/preserve.rs` (preserve-only bench, 58 LOC)
- `miniextendr-bench/src/bench_plan/preserve.rs` (preserve-only doc-only
  module, 12 LOC)

### Files to edit

**Module reachability + feature**
- `miniextendr-api/src/lib.rs`
  - Line 619–621: remove `// Deprecated: DLL preserve list. Use ...` +
    `pub mod preserve;`.
  - Line 181: remove `debug-preserve` row from feature-flags table.
  - Lines 60–66: drop "R objects surviving across `.Call`s -> preserve"
    code example block (preserve example block).
  - Lines 42–46: drop "Preserve list" row from three-strategies table.
- `miniextendr-api/Cargo.toml`
  - Line 120–122: remove `## Enable preserve::count() / count_unchecked() …`
    + `debug-preserve = []`.
  - Line 139: remove `"debug-preserve"` from `full = [...]`.

**Doc tables / cross-references**
- `miniextendr-api/src/gc_protect.rs`
  - Lines 25–26 (table row), 35–38 (when-to-use bullet), 59–64
    (ASCII art block) — remove all preserve references.
- `miniextendr-api/src/externalptr.rs`
  - Line 32 (table row), 46–48 (when-to-use bullet) — remove preserve.

**Benchmarks (pure subtraction, keep file otherwise)**
- `miniextendr-bench/Cargo.toml`
  - Remove `debug-preserve = ["miniextendr-api/debug-preserve"]`
    (line 21).
  - Remove `[[bench]] name = "preserve" harness = false`.
- `miniextendr-bench/benches/gc_protect.rs` (covers PROTECT stack +
  builders + accumulator + scope.collect + etc. — much more than
  preserve): remove `use miniextendr_api::preserve;`, plus the
  preserve-only divan benches (`preserve_insert_release`,
  `preserve_insert_release_unchecked`, `preserve_multiple`,
  `preserve_release_arbitrary_order`, `preserve_count`,
  `preserve_ppsize_scale`, `preserve_ppsize_arbitrary_order`).
- `miniextendr-bench/benches/gc_protection_compare.rs` (head-to-head
  bench): remove `use miniextendr_api::preserve;`, drop the
  `dll_preserve` / `dll_with_stack_depth` / `dll_with_work_allocs` /
  `dll_hold_n` / `dll_reinsert` functions across all sub-modules, plus
  any `mod dll_*` standalone modules that only contain DLL preserve
  benches. Keep the comparison file (it still benches stack, precious
  list, vec_pool, slotmap, BTreeMap, etc.) — it's a major file with
  long-term value.
- `miniextendr-bench/src/bench_plan.rs` — drop `pub mod preserve;` and
  the `preserve` doc-only references in the bench-plan module list.
- `miniextendr-bench/src/bench_plan/gc_protect.rs` — drop the
  `- 'preserve': preserve::insert/release vs ...` doc line.

**End-user / repo READMEs and site**
- `miniextendr-api/README.md` line 260: drop `debug-preserve` row.
- `README.md` (root) line 60: drop `debug-preserve` from feature list.
- `site/content/features.md` line 43: drop `debug-preserve` token.

**Docs (under `docs/`, source for site)**
- `docs/GC_PROTECT.md` lines 362–384 (the "Preserve List" section) and
  the choosing-a-strategy decision-tree line 432 — drop both. The
  remaining doc covers protect stack + refcount arenas + ExternalPtr.
- `docs/ALLOCATOR.md` rewrite the "preserve list" claims to refer to
  `R_PreserveObject` (which is what the code actually does — `rg`
  confirmed). Lines 8–11 ("via the preserve list"), 38–40 (Header
  description), 48 + 55 (the explicit `preserve::insert/release`
  steps). This is a stale-doc fix not a deprecation knock-on, but
  carrying preserve references after the module is gone would be a
  broken link.

**Analysis (kept-for-history)**
- `analysis/gc-protection-benchmarks-results.md` — KEEP, prepend a
  one-line header note `> Note: the DLL preserve list mechanism was
  removed in PR #N; results here remain as historical comparison
  vs ProtectPool / R_PreserveObject / refcount arenas.`
- `analysis/gc-protection-strategies.md` — KEEP, prepend a similar
  one-line historical note at the top. The "Mechanism 3" section is
  valuable design rationale even though the mechanism is removed.

**rust-analyzer / IDE**
- `.vscode/settings.json` line 57: remove `"debug-preserve",` from
  `rust-analyzer.cargo.features`.

### CI

`rg "debug-preserve" .github/` → no matches. Nothing to update in CI
workflows.

## Order

1. Remove `debug-preserve` feature from `miniextendr-api/Cargo.toml`
   first (downstream `#[cfg(feature = "debug-preserve")]` would otherwise
   fail to compile once the module they hide in is gone).
2. `git rm` the 4 doomed files.
3. Remove `pub mod preserve;` from `lib.rs`, and edit the surrounding
   doc table + code block.
4. Update `gc_protect.rs` and `externalptr.rs` doc tables.
5. Update bench Cargo.toml, bench files, bench_plan.
6. Update READMEs, site features, GC_PROTECT.md, ALLOCATOR.md.
7. Prepend historical notes to the two analysis files.
8. Update `.vscode/settings.json`.
9. `rg "preserve::|pub mod preserve|debug-preserve|mod preserve;"` to
   confirm zero matches (allowing `R_PreserveObject` /
   `Rf_PreserveObject` / English-prose "preserve" / "preserved").

## Verification

Pure subtraction with zero live callers — no gctorture pass required.

- `just check 2>&1 > /tmp/check.log`
- `just clippy 2>&1 > /tmp/clippy.log`
- `just test 2>&1 > /tmp/test.log`
- CI parity: read `.github/workflows/ci.yml` for `clippy_all` features,
  reproduce locally.
- `just configure && just rcmdinstall && just force-document` — R
  wrapper layer should be unaffected (no `#[miniextendr]` in
  `preserve.rs`).
- `just devtools-test`.

## Risks

Low. Pure subtraction. The only thing that could go subtly wrong is
the bench files — if I miss a `preserve::` call site inside the giant
`gc_protection_compare.rs`, the bench crate won't compile. Mitigated by
final `rg "preserve::"` sweep.

## PR

- Branch: `cleanup/delete-deprecated-preserve`
- Base: `refactor/api-sys-split` (or `main` if #675 merges first).
- Title: `cleanup(api): delete deprecated preserve.rs module`
- References: #509, #510 (and PR #675 if base is unmerged).
