# sccache: rustc command lines too long for Windows after expanding cargo's argfile

- **Upstream**: [mozilla/sccache#3026](https://github.com/mozilla/sccache/pull/3026)
  ("rust: spawn a long command line through a response file"), merged
  2026-11-14, not released yet. Reports:
  [mozilla/sccache#2397](https://github.com/mozilla/sccache/issues/2397) (open,
  filed 2025-05-22) and
  [mozilla/sccache#2787](https://github.com/mozilla/sccache/issues/2787)
  (closed 2026-11-14 by #3026). It started in v0.17.0 with
  [mozilla/sccache#2782](https://github.com/mozilla/sccache/pull/2782). What
  made it hit us: [rust-lang/cargo#17354](https://github.com/rust-lang/cargo/pull/17354),
  the new build-dir layout in Rust 1.100.0.
- **Our issue**: #2353. We have not commented upstream.

## Impact

From Rust 1.100.0, cargo puts each unit's output in its own directory and
passes rustc one `-L dependency=` per dependency. In the Windows R CMD check,
`datafusion-physical-plan`'s rustc command line had 163 of them, about 36000
characters with R CMD check's source path, past Windows' 32767-character
limit. Cargo alone retries such a command through an argfile. sccache 0.17.0
to 0.19.0 expands the argfile and spawns rustc with the whole command line,
which fails with `The filename or extension is too long. (os error 206)`.
Observed 2026-11-16 on #2352
([job](https://github.com/A2-ai/miniextendr/actions/runs/43932568892/job/133528746019)).

Linux and macOS jobs are not affected. A Windows user whose cargo
`rustc-wrapper` is `sccache` could hit it when a package has enough
dependencies. Not reproduced outside CI.

## Workaround

- CI: the Windows job does not set `RUSTC_WRAPPER` (step "Set RUSTC_WRAPPER
  if sccache available" in `.github/workflows/ci.yml`). With that, the same
  job installed the package
  ([job](https://github.com/A2-ai/miniextendr/actions/runs/43934761758/job/133538196239)).
- Users: `RUSTC_WRAPPER=""` for the install. mozilla/sccache#2787 reports
  that sccache 0.16.0 passes the argfile through to rustc (uncached).

## Closes when

CI runs an sccache release that includes #3026 (v0.20.0 or later), and the
Windows job sets `RUSTC_WRAPPER=sccache` again on that release. CI pins
v0.19.0 until #2396 is resolved, so this waits on #2396 too.

## Last checked

2026-11-23: v0.20.0 released 2026-11-23 and includes #3026. CI pins v0.19.0
because 0.20.0's server times out starting under GitHub cache rate limiting
(#2396). #2397 open; mozilla/sccache#3053 (2026-11-20) reports the same os
error 206.
