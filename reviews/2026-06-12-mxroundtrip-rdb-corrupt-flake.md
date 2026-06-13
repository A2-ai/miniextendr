# test-templates.R:565 — mxroundtrip lazy-load .rdb "corrupt" failure (pre-existing)

**What was attempted**: full `just minirextendr-test` run while implementing #502
(tools/detect-features.R). One failure in the standalone e2e test that scaffolds,
builds, and installs a throwaway `mxroundtrip` package.

**What went wrong**: `miniextendr_build(pkg_path, install = TRUE)` →
`devtools::document` → `pkgload::load_all` → `pkgload::unregister` fails with:

```
internal error 1 in R_decompress1 with libdeflate
Error in env_get_list(...): lazy-load database '.../library/mxroundtrip/R/mxroundtrip.rdb' is corrupt
```

First seen alongside a genuine disk-full event (`ld: write() failed, errno=28`,
disk at 98% / 256Mi free from parallel agent worktrees). But the .rdb failure
**reproduced twice more with 19Gi free**, including once on **clean origin/main
with all working-tree changes stashed** — so it is a pre-existing flake/regression
on main, unrelated to #502.

**Root cause**: not fully diagnosed. The install completes (`* DONE (mxroundtrip)`),
then the *subsequent* `devtools::document` in the same session unloads the
namespace and hits the corrupt lazy-load db. Suspects: R 4.6.0's libdeflate-backed
`R_decompress1` reading an .rdb written by a staged install moments earlier,
or pkgload unregister racing the freshly-installed lazy-load db. The remaining
55 templates tests (and 519/520 of the full suite) pass.

**Fix (deferred at the time)**: none in #502 (out of scope); tracked as #1000.

---

## Addendum (2026-06-12, #1000): root cause is in our workflow, not upstream

Re-reading `minirextendr/R/workflow.R` made the mechanism concrete — this is the
textbook *reinstall-over-a-loaded-package* lazy-load corruption, not (primarily)
an R 4.6.0 libdeflate bug:

1. The bootstrap path installs the package (`install()` at workflow.R:312) with
   devtools' **default `reload = TRUE`** → pkgload loads the just-installed,
   `.rdb`-backed namespace into the running session, with lazy promises pointing
   at byte offsets in `mxroundtrip.rdb`.
2. `devtools::document()` (workflow.R:334) runs `pkgload::load_all` →
   `unregister`, which forces those lazy bindings (`env_get_list` in the
   traceback) — i.e. *reads* the `.rdb`.
3. The Step-4 reinstall (workflow.R:336, again `reload = TRUE`) **rewrites the
   same `.rdb`** while the earlier-loaded namespace still holds promises at the
   old offsets → `R_decompress1` reads new bytes at stale offsets → "corrupt".

R 4.6.0's libdeflate backend changed the *message* ("internal error 1 in
R_decompress1 with libdeflate") and tightened detection, which is why the
latent bug surfaced now rather than silently returning garbage.

**Fix shipped**: `reload = FALSE` at all three `devtools::install()` sites
(workflow.R:224 / :312 / :336) so we never hold a loaded namespace across a
reinstall; plus a defensive `pkgload::unload()` before the test's `library()`
(test-templates.R) so it resolves the *installed* copy rather than a leftover
`load_all` dev namespace. No retry loop — the failure was reproducible, not
transient, so a retry would be noise. If the repro survives this (it should
not), the fallback is `skip_if(...)` citing #1000 + an upstream report with a
minimal `install→reload→reinstall→unregister` repro.
