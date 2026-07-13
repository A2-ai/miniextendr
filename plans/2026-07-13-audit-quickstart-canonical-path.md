# Plan: audit 2026-07-12 P0 — Quick Start contradicts the generated scaffold

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `docs/audit-quickstart-canonical-path`.

Covers 2026-07-12 audit worklist item 4 (+ the `create.R` message half of
item 6). The *behavioral* half (git-init in the standalone scaffold, CI
e2e smoke, sccache-hang investigation) is
`plans/2026-07-13-audit-scaffold-first-run-contract.md` — the two plans are
independent PRs; this one is docs + one message string.

## Verified defects (docs/GETTING_STARTED.md, 451 lines)

Every claim re-verified on 656e5cdd:

- `:25` — `install.packages("minirextendr")` (not on CRAN) with a literal
  `devtools::install_github("...")` placeholder (three dots, not a repo).
- `:31-45` — the "This creates a package structure" tree shows
  `R/mypackage_wrappers.R` (wrong name — generated file is
  `<pkg>-wrappers.R` with a hyphen — and it does NOT exist in a fresh
  scaffold; it appears on first build) and `src/rust/vendor/` (does not
  exist in a fresh standalone scaffold; vendoring is the tarball-mode
  latch, not a scaffold artifact).
- `:76-89` — Step 3 recommends `devtools::document()` +
  `devtools::install()`, and "Or manually": `./configure` +
  `R CMD INSTALL .`.
- `:336-346` — "Iteration Cycle" repeats the `devtools::document()` /
  `devtools::install()` loop and claims (:340-344) document() "handles
  ./configure, compilation, and wrapper generation automatically".
- `:375-382` — dots example declares BOTH an explicit param and dots:
  `pub fn count_args(_dots: &Dots, ...) -> i32`. Per
  `docs/DOTS_TYPED_LIST.md:7-33` the supported forms are: bare `...`
  (auto-creates `_dots: &Dots`), `name @ ...` (custom name), or an explicit
  typed `&Dots` parameter — never both together. (The rpkg fixture
  `rpkg/src/rust/impl_dots_tests.rs:18` uses `dots: ...` — implementer:
  verify the exact accepted surface against the macro parser before
  writing the corrected example.)
- `:425-437` — Troubleshooting tells users to rebuild with
  `./configure && R CMD INSTALL .` (twice: :427, :437).

All of the recommended paths contradict the scaffold's own generated
source: `minirextendr/inst/templates/rpkg/lib.rs:5-27` says in explicit
terms **do NOT** use bare `devtools::install()` / `R CMD INSTALL .` /
`devtools::document()` on a fresh build (auto-vendor latch → tarball mode →
skipped wrapper generation → empty namespace); use
`minirextendr::miniextendr_build()`.

And the scaffolder's final console message contradicts the template again:
`minirextendr/R/create.R:513` — "use `miniextendr_build()` **or
`devtools::document()`** so `library(...)` sees your functions."

The audit's dogfood run proved the practical consequence: following the
guide on a fresh (non-git) scaffold auto-vendored, latched to tarball mode,
and left a NAMESPACE with `useDynLib` but zero exports.

## Work items (flat order)

1. **Rewrite the Quick Start** (GETTING_STARTED.md Steps 1-4) around the
   one canonical path:
   - install from the actual distribution source:
     `devtools::install_github("A2-ai/miniextendr", subdir = "minirextendr")`
     (or `pak::pak("A2-ai/miniextendr/minirextendr")`) — verify the
     org/repo spelling against the repo remote before committing;
   - `minirextendr::create_miniextendr_package("mypackage")`;
   - a numbered "make sure the package is a git repo" step (drop it if/when
     `plans/2026-07-13-audit-scaffold-first-run-contract.md` lands git-init
     in the standalone scaffold — coordinate, whichever lands second
     adjusts);
   - edit `src/rust/lib.rs`;
   - `minirextendr::miniextendr_build()`;
   - `library(mypackage)` + call the two scaffold example functions
     (verify their names from `minirextendr/inst/templates/rpkg/lib.rs` —
     the webR smoke asserts `add()` and `hello()`).
2. **Fix the scaffold tree** (:31-45): show what a fresh scaffold actually
   contains (no wrappers file, no vendor/; correct `-wrappers.R` naming in
   the prose that explains what the first build will generate).
3. **Fix the Iteration Cycle** (:336-346): the loop is edit lib.rs →
   `miniextendr_build()` → test. Delete the :340-344 claim about
   `devtools::document()`.
4. **Fix Troubleshooting** (:425-437): replace both
   `./configure && R CMD INSTALL .` rebuild instructions with
   `miniextendr_build()`; keep the autoconf hint but route it through the
   canonical builder.
5. **Fix the dots example** (:375-382) per DOTS_TYPED_LIST.md forms.
6. **Align the scaffolder's parting message**: `minirextendr/R/create.R:513`
   — drop the "or `devtools::document()`" clause; the message recommends
   `miniextendr_build()` only, matching the template it just wrote.
   **Check for pinned strings first**: minirextendr tests grep source
   literals (known lesson) — `grep -rn "skips wrapper generation\|devtools::document"
   minirextendr/tests/` and update any expectation in the same commit.
7. **Sweep the rest of GETTING_STARTED.md** for further
   `devtools::document|install`/`R CMD INSTALL` recommendations
   (`grep -n` — the six ranges above are the verified ones; :79-80, :88,
   :340-344, :427, :437 must all be gone or reframed as
   "what miniextendr_build() runs under the hood").

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1     # must print 4.6.x
just worktree-sync                            # FIRST
just minirextendr-test 2>&1 > /tmp/audit-qs-minir-test.log   # Read it; devtools::test always exits 0 — grep '\[ FAIL'
grep -E '\[ FAIL [0-9]+' /tmp/audit-qs-minir-test.log
just templates-check                          # expect green — templates untouched
just site-docs && just site-build             # docs render (do not commit site output)
```

No Rust compilation is needed unless a test pin forces a scaffold
round-trip; `create.R` is pure R.

## Must NOT touch

- `minirextendr/inst/templates/**` and `patches/templates.patch` (the
  template lib.rs text is already correct — it is the *docs* that are
  wrong).
- `rpkg/**`, generated wrappers/NAMESPACE/man.
- Scaffold *behavior* (git-init, guards, CI e2e) — that is the sibling
  plan.

## Done criteria

- GETTING_STARTED.md contains exactly one build/install path
  (`miniextendr_build()`), a real install command, an accurate fresh-tree
  listing, and a dots example that compiles.
- `create.R`'s final message no longer recommends `devtools::document()`.
- `just minirextendr-test` green (with any string-pin updates);
  `just templates-check` green with zero template churn.

## Escalation rule

If reality diverges from this plan — the install command's repo/subdir
doesn't resolve, the dots forms in the macro don't match
DOTS_TYPED_LIST.md, or a test pins the old message in a way that needs
more than a string update — **stop, commit nothing further, and report
back. Do not improvise.**
