# Plan: consolidate the three `dev/run_bindgen_corpus*.sh` scripts (#131)

## Goal

Replace the three near-duplicate scripts in `dev/`:

- `run_bindgen_corpus.sh` (C-only, v1 — 108 lines)
- `run_bindgen_corpus_cpp.sh` (C+C++, v2 — 121 lines)
- `run_bindgen_corpus_v3.sh` (C+C++ with isysroot, LinkingTo, fallback — 227 lines)

with a single `dev/run_bindgen_corpus.sh` that accepts a `--mode` flag
(`c`, `cpp`, or `full`) and reproduces each original script's behaviour
byte-for-byte for a given mode. Delete the two legacy scripts once the
consolidated script demonstrably produces the same outputs.

## Constraints / key facts

- These scripts are dev-only, not shipped to users. No CLAUDE.md
  maintainer-vs-enduser concern, no vendor regen, no R wrapper regen.
- They are **not** called by any `just` recipe, `CI`, or `.github/workflows/`
  (verify with grep). This means the PR is purely a file-shuffle plus doc
  update.
- `set -uo pipefail` (no `-e`) is intentional — bindgen-on-corpus expects
  per-package failures; they're aggregated into the CSV. Preserve the same
  error-swallowing semantics in the consolidated script.
- Scripts read `dev/002_pkgs_with_inst_include.csv` for the input corpus
  and write a per-mode output directory with logs + a summary CSV.
  Keep both behaviours identical in the consolidated version.
- The v3 script has the most features (isysroot, LinkingTo resolution,
  c++14 fallback, better error categorisation). It's the correct **base**
  for the consolidated version — `c` and `cpp` become reduced-option
  variants of `full`.

## Work items (flat, prioritized)

1. **Read all three scripts end-to-end.** Diff them to identify what's
   actually different vs. what's copy-paste. Expected axes of difference:
   - C vs C++ bindgen mode (`-x c++`, `-std=c++17`, `--enable-cxx-namespaces`)
   - `-isysroot $(xcrun --show-sdk-path)` for macOS C++ stdlib
   - `--wrap-static-fns`
   - LinkingTo transitive resolution (v3 only)
   - c++14 fallback (v3 only)
   - `R_NO_REMAP` (v2 and v3, not v1)
   - Output directory name and CSV filename

2. **Design the `--mode` flag.** Three modes:
   - `--mode c` — reproduces v1 behaviour (C only, no `R_NO_REMAP`, no
     isysroot, no LinkingTo resolution).
   - `--mode cpp` — reproduces v2 behaviour (C++, `R_NO_REMAP`,
     `-std=c++17`, no isysroot, no LinkingTo resolution, no c++14 fallback).
   - `--mode full` — reproduces v3 behaviour (everything enabled).
   Default mode should be `full` (the most-recent, most-useful script).

3. **Write the consolidated `dev/run_bindgen_corpus.sh`.** Structure:

   ```bash
   #!/usr/bin/env bash
   set -uo pipefail

   MODE="full"
   OUTPUT_DIR=""
   while [[ $# -gt 0 ]]; do
     case "$1" in
       --mode) MODE="$2"; shift 2 ;;
       --mode=*) MODE="${1#*=}"; shift ;;
       -h|--help) print_usage; exit 0 ;;
       *) OUTPUT_DIR="$1"; shift ;;
     esac
   done

   case "$MODE" in
     c|cpp|full) ;;
     *) echo "Unknown --mode: $MODE (expected c|cpp|full)" >&2; exit 2 ;;
   esac

   OUTPUT_DIR="${OUTPUT_DIR:-dev/bindgen-corpus-${MODE}-results}"
   ```

   Then the main loop (the common 90%) with mode-gated blocks for the
   10% that differs. Keep flag sets in named variables
   (`COMMON_BINDGEN_FLAGS`, `CPP_FLAGS`, `FULL_EXTRA_FLAGS`) to make the
   mode gating readable.

4. **Verify output parity.** For each of the three modes, run the
   consolidated script and diff its output directory against a run of
   the corresponding legacy script on the same corpus. Expect: identical
   per-package log file names, identical CSV header + row count,
   identical "parses / fails" counts. Small text drift (e.g., timestamp
   lines) is acceptable if the summary CSV matches.

   It is probably impractical to actually run the full corpus in CI —
   it needs a populated `rv/library/4.5/arm64` staging tree. **Do the
   verification manually** on the machine of the implementer; document
   the mode-by-mode parity results in the PR body rather than committing
   any generated output directory. If running the full corpus is not
   feasible, at minimum run each mode on a 3-package subset (pick
   packages with C-only headers, C++ headers, and LinkingTo dependencies
   respectively) and include that output in the PR description.

5. **Delete the legacy scripts** (`git rm`):
   - `dev/run_bindgen_corpus_cpp.sh`
   - `dev/run_bindgen_corpus_v3.sh`
   (The original `run_bindgen_corpus.sh` becomes the consolidated script.)

6. **Grep for references** to the removed script names across the repo:
   ```bash
   rg 'run_bindgen_corpus_(cpp|v3)' --hidden
   ```
   Update any matches (docs, issue references, comments) to point at
   `run_bindgen_corpus.sh --mode=cpp` / `--mode=full`. Known candidates:
   - `docs/NATIVE_R_PACKAGES.md` (referenced in #126, #130)
   - `README.md` or `dev/README.md` if any
   - `.github/workflows/` (verify no CI uses them)
   - Issue #126 references — those are historical, leave as-is

7. **Help output and top-of-file comment.** The consolidated script's
   header comment must explain: what it does, the three modes, the
   default output dir naming convention, and the input corpus CSV
   location. `--help` must print the same info.

8. **Delete `plans/consolidate-bindgen-corpus.md`** in the final commit.

## Non-goals

- Rewriting the corpus logic itself (caching, parallelism, better
  error categorisation). Keep the script behaviourally identical to
  v3 in `full` mode.
- Making the script Windows-compatible. It already depends on
  `xcrun --show-sdk-path` which is macOS-only; preserve that.
- Moving the corpus script anywhere other than `dev/`.
- Expanding the corpus CSV or changing its schema.
- Adding CI automation for the corpus run. (Out of scope — the corpus
  needs a staged `rv` tree which isn't reproducible in CI.)

## Validation

- `shellcheck dev/run_bindgen_corpus.sh` — clean (install with
  `brew install shellcheck` if missing; skip if the tool isn't available
  and note in the PR body).
- `bash -n dev/run_bindgen_corpus.sh` — passes syntactic check.
- `bash dev/run_bindgen_corpus.sh --help` — prints the expected usage.
- `bash dev/run_bindgen_corpus.sh --mode=c [subset-output-dir]` —
  produces the same summary CSV rows as the legacy `run_bindgen_corpus.sh`
  on the same subset.
- Repeat for `--mode=cpp` and `--mode=full` against the respective legacy
  scripts.
- `rg 'run_bindgen_corpus_(cpp|v3)'` — no stale references remaining
  (except perhaps in changelogs or closed-issue text).
- **No** `just` / Rust / R CMD commands needed. This PR does not touch
  any compiled code, any `.Rd` / wrapper, any workspace crate. Therefore
  **no vendor tarball regen is required** — the pre-commit hook should
  pass without triggering `cargo revendor`.

## Branch / PR

- Branch: `dev/consolidate-bindgen-corpus`
- Base: fresh `origin/main` (fetch + rebase immediately before push).
- PR title: `dev: consolidate bindgen corpus scripts into one (#131)`
- PR body: short summary, link to this plan, the parity evidence from
  step 4, and a checklist mirroring Validation. `Closes #131.`

## Expected diff shape

- `dev/run_bindgen_corpus.sh` — rewritten (~230 lines, similar to v3
  plus the mode switch).
- `dev/run_bindgen_corpus_cpp.sh` — deleted.
- `dev/run_bindgen_corpus_v3.sh` — deleted.
- Up to a handful of `docs/**.md` one-liner updates replacing old script
  names with `--mode=` invocations. If none are found, skip.
- **No** vendor/ / wrapper / NAMESPACE / man changes.

If the diff touches anything outside `dev/**` and `docs/**`, stop and
re-read the plan.
