# #441 — Stop the `inst/vendor.tar.xz` latch from masking bootstrap regressions

Closes #441.

## Background (one paragraph for the implementer)

`rpkg/inst/vendor.tar.xz` is the **single signal** that flips `configure` into
tarball mode (`[ -f inst/vendor.tar.xz ]`, see `rpkg/configure.ac` and
`CLAUDE.md` "Install modes"). Once the tarball exists in the source tree,
every subsequent `configure` honours it — including iterations that were
supposed to verify the bootstrap pipeline produces it from a clean state.
This bit us twice in 48h: #439 merged a broken in-tree bootstrap path because
the smoke run reused a leftover tarball; #440's verification only succeeded
after manual `rm rpkg/inst/vendor.tar.xz && just configure`. Recipes that
*produce* the tarball already trap-clean (`r-cmd-build`, `r-cmd-check`,
`devtools-build` — `justfile:506,548,561`); the gap is recipes that *consume*
state and recipes/tests that need to assert "from clean". This plan closes
both gaps.

## Out-of-scope (explicit non-goals)

- Don't change configure mode-detection itself. The latch is the design;
  we're adding hygiene around it.
- Don't gate `pkgbuild::build` upstream — that's `r-lib/pkgbuild`.
- Don't reshape the dev-iteration loop. The latch is fine for intentional
  iteration; this is about verification/regression workflows.

## Files to change

1. `justfile`
2. `rpkg/tests/testthat/test-bootstrap-vendor.R` (new) **or**
   `tests/bootstrap-produces-vendor.sh` (new) — see decision in §3
3. `CLAUDE.md`
4. `docs/CRAN_COMPATIBILITY.md` (cross-reference)
5. `minirextendr/inst/templates/rpkg/justfile` — port the same guard if/where
   the recipe exists in the template; check `patches/templates.patch` after
   and run `just templates-approve` if the delta is intentional

No vendoring changes (no `just vendor` needed).

## Plan (flat, priority order)

### 1. Add a `_assert-no-vendor-leak` private recipe in `justfile`

Drop near the existing `clean-vendor-leak` recipe (around `justfile:430`).
Hidden recipe (leading `_`) so it doesn't show in `just --list`.

```just
# Internal: abort if rpkg/inst/vendor.tar.xz is present.
# Used as a dep by recipes that consume configure state and would silently
# do the wrong thing in tarball mode.
[private]
[script("bash")]
_assert-no-vendor-leak:
    set -euo pipefail
    if [ -f rpkg/inst/vendor.tar.xz ]; then
        cat >&2 <<'EOF'
    error: rpkg/inst/vendor.tar.xz is present but not committed.

    This usually means a previous build/smoke leaked it into the source
    tree. configure now flips into tarball mode and ignores monorepo
    [patch."git+url"] propagation, so further dev iteration is unreliable.

    Fix:
      just clean-vendor-leak

    Or directly:
      rm rpkg/inst/vendor.tar.xz && just configure

    See CLAUDE.md "Vendor tarball is a latch" for context.
    EOF
        exit 1
    fi
```

Wire it as a dependency on the consume-side recipes that would otherwise
silently switch mode:

- `configure` — `configure: _assert-no-vendor-leak` (rejects accidental
  flip into tarball mode during dev). **Caveat:** `r-cmd-build` /
  `r-cmd-check` / `devtools-build` invoke `configure` as a dep *and* later
  produce the tarball in the same recipe via `vendor`. Use a different
  recipe name for the dev-mode assertion (e.g., `_assert-dev-mode`) **or**
  set an env var `MINIEXTENDR_VENDOR_OK=1` inside `r-cmd-build`/`r-cmd-check`
  before invoking `configure`, and skip the assertion when set. Pick whichever
  is less invasive once you read the recipe graph.
- `rcmdinstall` (= `r-cmd-install`) — same rule (consumes configure output).
- `devtools-test` — same.
- `devtools-load` — same.
- `devtools-install` — same.

**Implementer judgment**: read `justfile` end-to-end before wiring. Any recipe
that ends up running `cargo build` / `R CMD INSTALL` / `devtools::load_all`
against the source tree without first producing its own `vendor.tar.xz`
should get the guard. Recipes that *produce* the tarball (already trap-clean)
should NOT get the guard.

### 2. Reuse the existing `clean-vendor-leak` recipe — don't add a wrapper

The original issue mentioned `just smoke-build`. After reading the recipe
graph, this is unnecessary: `r-cmd-build`, `r-cmd-check`, `devtools-build`
already trap-clean. The only smoke path that doesn't is *raw*
`Rscript -e 'pkgbuild::build("rpkg")'` invocations from agent prompts /
ad-hoc shells. Document the leak risk (§4) rather than adding a wrapper —
agents who copy the prompt directly should see the documented hygiene step.

If the implementer disagrees after reading the graph, add a `smoke-build`
recipe that wraps `Rscript -e 'pkgbuild::build("rpkg")'` with the same
`trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT` pattern used by `r-cmd-build`.
Don't introduce both.

### 3. Add a regression test that bootstrap.R produces inst/vendor.tar.xz from clean

The bug in #439 was: `pkgbuild::build` smoke "passed" because the leftover
tarball was bundled regardless. We want a test that fails loudly under that
exact condition.

Pick the **shell-script** form:

`tests/bootstrap-produces-vendor.sh` (new, `chmod +x`):

```bash
#!/usr/bin/env bash
# Regression test for #441: bootstrap.R must produce inst/vendor.tar.xz
# from a clean source tree. If a previous run left a stale copy, this
# test would silently pass — so we delete first and assert the build
# regenerates it.
set -euo pipefail

cd "$(dirname "$0")/.."

# Pre: clean state
rm -f rpkg/inst/vendor.tar.xz

# Build (use a throwaway lib to avoid touching the user's library)
TMP_LIB=$(mktemp -d)
trap 'rm -rf "$TMP_LIB" rpkg/inst/vendor.tar.xz miniextendr_*.tar.gz' EXIT
R_LIBS_USER="$TMP_LIB" Rscript -e \
  '.libPaths(c("'"$TMP_LIB"'", .libPaths())); pkgbuild::build("rpkg", dest_path = ".")'

# Find the produced tarball
TARBALL=$(ls -t miniextendr_*.tar.gz | head -n1)
if [ -z "$TARBALL" ]; then
    echo "FAIL: pkgbuild::build did not produce a tarball" >&2
    exit 1
fi

# Assert: the tarball ships inst/vendor.tar.xz
if ! tar -tJf "$TARBALL" | grep -q 'inst/vendor.tar.xz$'; then
    echo "FAIL: $TARBALL does not contain inst/vendor.tar.xz" >&2
    echo "       Bootstrap pipeline regression — see #441/#440." >&2
    exit 1
fi

echo "OK: $TARBALL contains inst/vendor.tar.xz"
```

Plus a thin `just` recipe so CI/maintainers can invoke it:

```just
# Regression: bootstrap.R must produce inst/vendor.tar.xz from clean.
# See #441 / #440 — leftover tarballs masked the broken-bootstrap regression.
test-bootstrap-vendor:
    bash tests/bootstrap-produces-vendor.sh
```

(Don't auto-add to a CI workflow in this PR — keep the change scope tight;
a separate follow-up issue can wire it into ci.yml. File that follow-up
explicitly per "Concessions → issues" rule.)

**Why a shell test, not testthat:** `R CMD check` runs testthat *against
the built tarball*, which means `rpkg/inst/vendor.tar.xz` is already inside
the tarball before testthat runs — too late to assert anything about the
bootstrap pipeline. The test has to live outside the package.

### 4. Document the latch in CLAUDE.md

Add a short subsection to `CLAUDE.md` under "Common Issues" (or as a
dedicated subsection right after the existing "Stale `.cargo/config.toml`"
bullet). Title: **"Vendor tarball is a latch"**. Three sentences plus a
bullet list:

```
**Vendor tarball is a latch (#441).** `rpkg/inst/vendor.tar.xz` is the
single signal `configure` checks for tarball mode; once present, every
subsequent configure honours it. Recipes that *produce* the tarball
(`just r-cmd-build`, `just r-cmd-check`, `just devtools-build`) trap-clean
on exit. Recipes that *consume* configure state (`just rcmdinstall`,
`just devtools-test`) refuse to run if the tarball is present —
`just clean-vendor-leak` removes the leak.

Symptom of a leaked tarball: monorepo workspace-crate edits are silently
ignored (no `[patch."git+url"]`), or `Cargo.lock` mismatch errors during
build. Fix: `just clean-vendor-leak`. Regression test:
`just test-bootstrap-vendor` (#441).
```

Cross-reference from `docs/CRAN_COMPATIBILITY.md` if it discusses install
modes — keep one canonical source (`CLAUDE.md`) and link to it.

## Verification (before opening PR)

Run all of these — they're cheap and catch regressions:

1. `just clean-vendor-leak` — start from clean state.
2. `just configure` — succeeds (no leftover tarball).
3. `touch rpkg/inst/vendor.tar.xz && just devtools-test` — must fail
   with the new error message. Then `just clean-vendor-leak`.
4. `just test-bootstrap-vendor` — passes.
5. `touch rpkg/inst/vendor.tar.xz && just test-bootstrap-vendor` — must
   still pass (the test deletes the tarball at the start, demonstrating
   the "fail without the fix" property).
6. `just r-cmd-build` (or `just devtools-build`) — must still work, the
   guard must NOT fire here. Verify trap-clean removed the tarball at exit.
7. `just templates-check` — clean (or `just templates-approve` if you
   intentionally ported a recipe).

## PR notes

- Title: `fix(dev): guard against leaked rpkg/inst/vendor.tar.xz (#441)`
- Reference #441 in the PR body; explain the latch + the three guards
  (assert recipe, regression test, CLAUDE.md doc).
- File any follow-up issues *before* opening the PR (e.g., "wire
  test-bootstrap-vendor into ci.yml") and reference them in the body
  per the "Concessions → issues" rule.

## Common pitfalls

- **The `configure` recipe gating itself**: if `_assert-no-vendor-leak` is
  a dep on `configure`, `r-cmd-build` (which deps `configure` *then*
  `vendor`) will fail before vendor runs. Use the env-var skip pattern
  documented in §1 or split the assertion onto consume-only recipes that
  don't transitively chain into producer recipes. Test #6 above catches
  this.
- **`pkgbuild::build` `dest_path`**: by default `pkgbuild::build` writes
  the tarball to the parent dir; pin it to `.` (or a tmpdir) so cleanup is
  predictable.
- **Test running with system R**: don't assume devtools is on the user's
  default `.libPaths()`. Use `R_LIBS_USER=$TMP_LIB` and `.libPaths(...)`
  as in §3, or check `requireNamespace` and skip with a clear message.
