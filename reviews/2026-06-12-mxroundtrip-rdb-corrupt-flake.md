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

**Fix**: none in this PR (out of scope for #502); tracked as a follow-up issue.
If it bites again: check whether `mxroundtrip.rdb` is truncated (size 0 / short)
vs genuinely mis-compressed, and whether retrying `document()` in a fresh R
session passes.
