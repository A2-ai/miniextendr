# CI cargo tests: two five-month-old latent bugs (empty LD_LIBRARY_PATH, R stack check off-main-thread)

## What was attempted

Plan `plans/2026-07-01-ci-cargo-test-job.md` (audit A1): add a dedicated
`rust-tests` CI job running `cargo test --workspace --locked` +
cargo-revendor's standalone workspace. `cargo test` had been disabled in CI
since commit `952f78b5` (2026-01-15) with the note "stack overflow issues on
Linux CI" — never diagnosed, so the entire Rust test suite (including the
trybuild UI snapshots and PR #1095's `ReprotectSlot` tests) ran only on dev
machines for ~5.5 months. Two distinct latent bugs surfaced, one per CI run.

## Bug 1: silently-empty LD_LIBRARY_PATH

First run failed at the very first test binary:

```
target/debug/deps/miniextendr_api-…: error while loading shared libraries:
libR.so: cannot open shared object file: No such file or directory
```

The job's env dump showed `LD_LIBRARY_PATH:` **empty**, though the setup step
(copied verbatim from the R-check job's "Configure Linux") reported success.

**Root cause.** The idiom used in every workflow was

```bash
echo "LD_LIBRARY_PATH=$(R -s -e 'cat(R.home(\"lib\"))')" >> $GITHUB_ENV
```

Inside bash single quotes, `\"` is two literal characters. R receives
`cat(R.home(\"lib\"))`, whose parser rejects `\"` ("unexpected end of
input"). The command substitution swallows R's non-zero exit and stderr,
yields the empty string, `echo` succeeds, and the step goes green having
written an empty `LD_LIBRARY_PATH`. Verified against the latest successful
main run: **every** job that "set" it this way had it silently empty. Nobody
noticed because `R CMD check` / `Rscript` re-exec R through the launcher
script, which computes its own `LD_LIBRARY_PATH`; `cargo test` binaries are
the first consumers that link `libR.so` directly.

**Fix.** All five occurrences (ci.yml ×4, gctorture-nightly.yml ×1) replaced
with a form that has no nested quoting to get wrong:

```bash
echo "LD_LIBRARY_PATH=$(R RHOME)/lib" >> "$GITHUB_ENV"
```

(`R.home("lib")` is `${R_HOME}/lib${R_ARCH}`; `R_ARCH` is empty on the Linux
runners these jobs use.)

## Bug 2: the original "stack overflow" — R's C-stack check off the main thread

With `LD_LIBRARY_PATH` fixed, the 2026-01 failure reproduced exactly, in the
first test binary that actually boots embedded R (`condition_roundtrip`):

```
Error: C stack usage  159026301492 is too close to the limit
Execution halted
```

**Root cause.** The usage figure (~148 GB) is the giveaway: on Linux/glibc,
`Rf_initialize_R` calibrates `R_CStackStart` from the **process main
thread** (`__libc_stack_end`), but the test harness
(`miniextendr-api/tests/r_test_utils.rs`) initializes R on a dedicated 16 MB
`r-test-main` thread. R's stack-usage computation then measures the distance
between two different threads' stacks — garbage — and the check fires during
`setup_Rmainloop()`'s startup evaluation, before any test runs. R suicides
("Execution halted"), the process exits 1.

Two reasons nobody ever saw it locally:

1. macOS calibrates per-thread (`pthread_get_stackaddr_np(pthread_self())`),
   so the computation is sane there. Dev machines here are macOS; the
   failure is Linux-only, exactly matching the "stack overflow issues on
   Linux CI" note.
2. The harness *does* disable R's stack check (`R_CStackLimit = usize::MAX`)
   — but only **after** `REngine::init()` returns, and the check fires
   *inside* init, during `setup_Rmainloop()`. The disable was correct and
   too late.

**Fix.** `miniextendr-engine`'s `REngine::init()` now sets
`R_CStackLimit = usize::MAX` between `Rf_initialize_R` and
`setup_Rmainloop()` — the only window where it helps. This is standard
embedded-R-on-a-thread practice (Writing R Extensions §8); the OS guard page
still catches real overflows. The engine crate is already wholly non-API
(Rembedded.h) and never ships in an R package, so touching `R_CStackLimit`
there is in-policy.

None of the plan's anticipated levers (`RUST_MIN_STACK`, `--test-threads`,
proptest case counts) were needed — the failure was never a Rust stack
overflow at all.

## Lessons

- A green setup step is not a set variable: `$(...)` substitution failures
  inside `echo … >> $GITHUB_ENV` are invisible. Prefer substitutions that
  cannot fail silently, or verify the value in the same step.
- "Disabled due to X" comments without a captured log or issue rot: five
  months later the disable was dead weight and X turned out to be
  misdiagnosed ("stack overflow" was R's stack *checker* malfunctioning, not
  a stack overflow).
- Copying a step from a working job proves nothing if that job never
  consumed the step's output.
- R error messages with absurd magnitudes ("C stack usage 159026301492")
  point at cross-thread miscalibration, not at actual resource exhaustion —
  glibc's `__libc_stack_end` calibration only matches the thread that
  spawned the process.
