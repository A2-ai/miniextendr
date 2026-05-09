+++
title = "Release Workflow: Platform Gotchas"
weight = 50
description = "Every miniextendr consumer eventually writes an r-release.yml GitHub Actions workflow that builds and checks their package on multiple platforms. AlmaLinux 8 containers and macOS arm64 runners surface four reproducible gotchas that downstream maintainers hit independently. This document explains each one, gives the canonical fix, and explains why miniextendr's own locale check exists — so you know not to file a bug upstream."
+++

Every miniextendr consumer eventually writes an `r-release.yml` GitHub Actions
workflow that builds and checks their package on multiple platforms. AlmaLinux 8
containers and macOS arm64 runners surface four reproducible gotchas that
downstream maintainers hit independently. This document explains each one, gives
the canonical fix, and explains why miniextendr's own locale check exists — so
you know not to file a bug upstream.

Use `minirextendr::use_release_workflow()` to scaffold a template that has all
four fixes already baked in.

## The four gotchas

### Gotcha 1: AlmaLinux 8 minimal defaults to the `C` locale

AlmaLinux 8's container image ships with only the `C`/`POSIX` locale installed.
When R starts, `l10n_info()[["UTF-8"]]` returns `FALSE`, which causes
`miniextendr_assert_utf8_locale()` to fire:

```
Error: miniextendr requires a UTF-8 locale (R >= 4.2.0 uses UTF-8 by default)
```

**Fix**: set `LANG=C.UTF-8` and `LC_ALL=C.UTF-8` at the **job** level (not the
workflow level — see Gotcha 2) for container jobs, or install a full locale pack
and use `en_US.UTF-8`:

```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
    container: almalinux:8
    env:
      LANG: C.UTF-8
      LC_ALL: C.UTF-8
    steps:
      - name: Install build tools
        run: |
          dnf install -y --setopt=install_weak_deps=False \
            git gh glibc-langpack-en
```

`glibc-langpack-en` is lightweight and also makes `en_US.UTF-8` available as an
alternative.

### Gotcha 2: macOS rejects `C.UTF-8`

`C.UTF-8` is a glibc extension. macOS's `setlocale(3)` does not recognise it;
the call silently falls back to `C`, which is non-UTF-8, and the same assertion
fires.

**Fix**: do **not** set `LANG` or `LC_ALL` at the workflow-level `env:` block.
macOS runners default to UTF-8 (`en_US.UTF-8` / `UTF-8`) and work correctly
without any locale override. Scope the locale env vars to the container jobs
only (as shown in Gotcha 1).

```yaml
# WRONG — breaks macOS:
env:
  LANG: C.UTF-8

# CORRECT — scoped to the AlmaLinux container job only:
jobs:
  build-linux:
    container: almalinux:8
    env:
      LANG: C.UTF-8
```

### Gotcha 3: AlmaLinux minimal lacks `git`; cargo libgit2 cannot auth private deps

The AlmaLinux 8 minimal image ships without `git`. Cargo's built-in libgit2
path cannot read git's credential helper, so private git dependencies fail to
fetch even if a token is available.

**Fix**: install `git` and `gh` before any cargo operation, then call
`gh auth setup-git` to configure git's credential helper:

```yaml
    steps:
      - name: Install build tools (AlmaLinux 8 minimal)
        run: |
          dnf install -y --setopt=install_weak_deps=False \
            git gh dnf-plugins-core which procps-ng glibc-langpack-en
      - uses: actions/checkout@v4
      - name: Configure git auth for cargo private deps
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh auth setup-git
```

`gh auth setup-git` requires `GH_TOKEN` (or `GITHUB_TOKEN`) in the step's
environment; without it `gh` exits with "no token". Also set
`CARGO_NET_GIT_FETCH_WITH_CLI=true` (see Gotcha 4) so cargo uses the CLI git
instead of libgit2.

### Gotcha 4: macOS needs auth + `CARGO_NET_GIT_FETCH_WITH_CLI`

macOS runners ship with `git` preinstalled, but cargo still uses its built-in
libgit2 by default. libgit2 does not read the macOS Keychain credential store
or the git credential helper configured by `gh auth setup-git`, so private git
deps fail.

**Fix**: call `gh auth setup-git` (same as Linux), and set
`CARGO_NET_GIT_FETCH_WITH_CLI=true` workflow-wide so cargo uses the system
`git` binary and inherits its credential config:

```yaml
env:
  CARGO_NET_GIT_FETCH_WITH_CLI: "true"

jobs:
  build-macos:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - name: Configure git auth for cargo private deps
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh auth setup-git
```

`CARGO_NET_GIT_FETCH_WITH_CLI=true` is safe to set at the workflow level —
it has no negative effect on AlmaLinux or any other platform.

## Why `package_init` checks for UTF-8

`miniextendr_assert_utf8_locale()` is called during R package initialization
(from `R_init_<package>`) via `miniextendr-api/src/encoding.rs`. It calls
`l10n_info()[["UTF-8"]]` (public R API) and aborts if the result is `FALSE`.

This check exists because miniextendr's string conversion layer
(`charsxp_to_str`) assumes all CHARSXP bytes are valid UTF-8. R >= 4.2.0
guarantees this when the locale is UTF-8, but makes no guarantees in a
non-UTF-8 locale. A silent wrong-locale scenario would produce corrupted string
data rather than a clear error, so the framework asserts up-front.

The check is not a bug in miniextendr — it is a guard against using the
framework in an environment where string correctness cannot be guaranteed. The
platform is wrong (or differently configured), not the framework. The fixes in
Gotchas 1–2 bring the platform into compliance; the assertion then passes and
package load succeeds normally.

R >= 4.2.0 itself defaulted to UTF-8 on all mainstream platforms (Windows via
UTF-8 code page, macOS and modern Linux always UTF-8), so the assertion fires
only on minimal container images (like AlmaLinux 8) that ship without any
UTF-8 locale installed.

## Reference template

`minirextendr/inst/templates/r-release.yml` is a drop-in baseline workflow
with all four fixes applied. Scaffold it into your package with:

```r
minirextendr::use_release_workflow()
```

The template targets AlmaLinux 8 (RHEL 8 family — common in enterprise release
pipelines) and macOS arm64 (`macos-14`). Extend it with additional steps for
your package's specific setup (R package installation, cargo caching, upload
steps, etc.).
