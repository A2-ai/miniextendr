# #443 — Stop EXTRA_PKG_CFLAGS from producing an `R CMD check --as-cran` WARNING

Closes #443.

## Background (one paragraph for the implementer)

`rpkg/configure.ac:284-298` and `rpkg/src/Makevars.in:21,29` ship a clang
probe that adds `-Wno-unknown-warning-option` to `PKG_CFLAGS` when the
detected `CC` is clang. The intent is to silence the cosmetic clang 21+
warning emitted for upstream R's `Boolean.h` `#pragma clang diagnostic
ignored "-Wfixed-enum-extension"`. The trade-off:
`R CMD check --as-cran`'s "checking compilation flags used" step flags
`-Wno-unknown-warning-option` itself as a non-portable warning-suppressor
flag, producing a CI-blocking WARNING on clang platforms (currently macOS).
miniextendr's own CI pins `error-on: '"error"'` so the WARNING doesn't
fail the build — but downstream packages that adopt the upstream block
without also pinning `error-on` (default `"warning"`) trip the WARNING and
fail CI. Real incident: `a2-ai-tech-training/astra` PR #9, macOS R 4.6 job.
Goal: keep the suppression working downstream **without** burdening every
consumer with an `error-on` policy choice.

## Out-of-scope (explicit non-goals)

- Don't remove `EXTRA_PKG_CFLAGS` outright — the warning it suppresses
  is real and affects every clang 21+ developer.
- Don't change miniextendr's CI (`ci.yml`) — `error-on: '"error"'` is the
  right policy for this repo regardless of how this issue is fixed.
- Don't try to upstream a fix to R's `Boolean.h` — out of scope here;
  R Core may patch their pragma but we can't depend on that.

## Decision: shim-header approach

Of the three options in the issue:

1. Document the dependency on `error-on='error'` — burdens every downstream.
2. Wrap the `Rinternals.h` include in a per-TU shim with `#pragma clang
   diagnostic push/ignored "-Wunknown-warning-option"/pop`. Keeps the
   suppression scoped to one TU, **out of `PKG_CFLAGS`**, so R CMD check
   sees no flag to flag.
3. Skip suppression entirely — every clang 21+ user sees the cosmetic
   warning forever.

Pick **(2)**. It's the only option that delivers the suppression *and*
keeps downstream CI clean *and* doesn't require any policy choice from
the consumer. (1) is documented as a *fallback* in case (2) doesn't apply
to all C TUs; (3) is a fallback if a future R version requires Boolean.h
in headers we can't shim.

## Files to change

1. `rpkg/src/r_shim.h` (new) — shim header that wraps `<Rinternals.h>`
2. `rpkg/src/cli_wrapper.h` — replace `#include <Rinternals.h>` with
   `#include "r_shim.h"` (or remove the include if `cli/progress.h`
   already pulls Rinternals via the shim — verify, don't assume)
3. `rpkg/src/stub.c` — does not include `Rinternals.h` today; leave
   alone unless the implementer finds a missing case
4. `rpkg/configure.ac` — strip the EXTRA_PKG_CFLAGS clang probe
5. `rpkg/src/Makevars.in` — remove `EXTRA_PKG_CFLAGS` substitution and
   `PKG_CFLAGS = $(EXTRA_PKG_CFLAGS)`
6. `rpkg/configure` — regenerate via `autoconf` after editing `.ac`
7. `minirextendr/inst/templates/rpkg/configure.ac` — port the strip
8. `minirextendr/inst/templates/rpkg/src/Makevars.in` — port the strip
9. `minirextendr/inst/templates/rpkg/src/r_shim.h` (new) — port the shim
   so scaffolded packages get it
10. `patches/templates.patch` — refresh via `just templates-approve`
11. Bindgen-generated wrapper headers (e.g.
    `minirextendr/R/use_native_package.R` and any tooling that emits
    `*_wrapper.h` files) — update the emitted template to include
    `r_shim.h`. Find via `grep -rn "include <Rinternals" minirextendr/`.

No vendor regen needed (no Rust source changes; only C headers + autoconf).

## Plan (flat, priority order)

### 1. Author `rpkg/src/r_shim.h`

```c
/* r_shim.h — local include that locally suppresses clang 21+'s
 * "-Wunknown-warning-option" warning emitted for upstream R's
 *   #pragma clang diagnostic ignored "-Wfixed-enum-extension"
 * in Boolean.h (R 4.5+). The pragma is upstream R, not us.
 *
 * Why local push/pop instead of -Wno-unknown-warning-option in PKG_CFLAGS:
 * R CMD check --as-cran flags any non-portable "warning suppressor" flag
 * in PKG_CFLAGS, producing a CI-blocking WARNING for downstream packages
 * that don't pin error-on='"error"'. Scoped pragma keeps the flag out of
 * PKG_CFLAGS — see issue #443.
 *
 * Use this header *instead of* including <Rinternals.h> directly in
 * package C sources.
 */
#ifndef MINIEXTENDR_R_SHIM_H
#define MINIEXTENDR_R_SHIM_H

#if defined(__clang__)
#  pragma clang diagnostic push
#  pragma clang diagnostic ignored "-Wunknown-warning-option"
#endif

#include <Rinternals.h>

#if defined(__clang__)
#  pragma clang diagnostic pop
#endif

#endif /* MINIEXTENDR_R_SHIM_H */
```

A few subtle points the implementer must validate:

- **Does the pragma actually pop after the include?** Yes — `pop` only
  reverts state for *subsequent* code in the same TU. Diagnostics emitted
  *during* `#include <Rinternals.h>` (lexer/parser warnings emitted as the
  preprocessor expands Boolean.h) are governed by the *push* state,
  which is what we want.
- **Use `-Wunknown-warning-option`, not `-Wfixed-enum-extension`.** The
  warning we're hitting is the *meta* warning ("clang doesn't know
  `-Wfixed-enum-extension`"); we suppress that specifically. If we
  suppressed `-Wfixed-enum-extension` directly, clang 20 (which doesn't
  know the flag) would produce the same meta-warning. Push/pop pattern
  with `-Wunknown-warning-option` is the surgical fix.
- **gcc**: `__clang__` guards everything; gcc and other non-clang
  compilers see only the bare `#include <Rinternals.h>`. No-op on gcc.

### 2. Update C TUs to include the shim

Find every C TU that pulls `Rinternals.h`:

```bash
grep -rn 'include\s*<Rinternals\.h>\|include\s*"Rinternals\.h"' rpkg/src minirextendr/inst/templates 2>/dev/null
```

Currently in this repo: only `rpkg/src/cli_wrapper.h:4`. Replace with
`#include "r_shim.h"`. Verify `cli_static_wrappers.c` (its only consumer)
still compiles — it should, since the shim re-exports the same symbols
plus the suppression. Add the shim to the bindgen-generated header
template in minirextendr too (search `minirextendr/R/use_native_package.R`
for `Rinternals.h` and adjust the emitter).

### 3. Remove the now-unused `EXTRA_PKG_CFLAGS` machinery

`rpkg/configure.ac:284-298` — delete the entire `dnl ---- Clang-only
flag: silence unknown-warning-option ----` block including
`AC_SUBST([EXTRA_PKG_CFLAGS])`.

`rpkg/src/Makevars.in:21,29` — delete `EXTRA_PKG_CFLAGS = @EXTRA_PKG_CFLAGS@`
and the `PKG_CFLAGS = $(EXTRA_PKG_CFLAGS)` line. If `PKG_CFLAGS` ends up
empty, remove the line entirely (R's build system handles a missing
`PKG_CFLAGS` fine — verify by checking `R CMD config CFLAGS`).

Regenerate `rpkg/configure`:

```bash
cd rpkg && autoconf
```

Commit the regenerated `configure` alongside the `configure.ac` edit (per
CLAUDE.md "Edit `.in` templates, not generated files" — the `.ac` is the
source, `configure` is generated; both are tracked and must be in sync).

### 4. Port to minirextendr templates

`minirextendr/inst/templates/rpkg/configure.ac:314-322` — same delete.
`minirextendr/inst/templates/rpkg/src/Makevars.in` — same delete.
Add `minirextendr/inst/templates/rpkg/src/r_shim.h` with the same content
as §1. Update the bindgen-wrapper emitter so newly-scaffolded packages
include `r_shim.h` instead of `Rinternals.h` directly.

Run `just templates-approve` to refresh `patches/templates.patch`.

### 5. Cross-reference in CLAUDE.md

Under "Common Issues" or "Rust/FFI gotchas":

```
- **R CMD check `compilation flags used` WARNING**: `-W*` flags in
  `PKG_CFLAGS` trigger a non-portable-flag WARNING under
  `R CMD check --as-cran`. Use scoped `#pragma clang diagnostic` in a
  shim header (`rpkg/src/r_shim.h`) for clang-specific suppressions, not
  `PKG_CFLAGS`. See #443.
```

## Verification (before opening PR)

1. `bash ./configure` (in `rpkg/`) — succeeds; `Makevars` no longer
   references `EXTRA_PKG_CFLAGS`.
2. `just rcmdinstall` — installs cleanly. On macOS clang specifically,
   confirm no `-Wfixed-enum-extension` warnings hit stdout (the shim
   suppresses them).
3. `just r-cmd-build` — produces a tarball.
4. `just r-cmd-check ERROR_ON=warning` — must pass (this is the reverse
   condition the issue describes; without `EXTRA_PKG_CFLAGS` the
   "compilation flags used" check has nothing to flag).
5. `just devtools-check` — clean.
6. `just templates-check` — clean (or run `just templates-approve` first
   if the diff is intentional).
7. `just minirextendr-test` — passes (no regressions in scaffolding tests).
8. Manual: `cd rpkg/src && clang -c -I"$(R RHOME)/include" cli_static_wrappers.c -o /tmp/out.o`
   — compiles without unknown-warning-option meta-warnings (when clang
   ≥ 21 is available; otherwise note this in the PR body).
9. `just configure && just devtools-document` — wrappers regenerate.

## PR notes

- Title: `fix(rpkg): scope clang warning suppression to a shim header (#443)`
- Body: explain the trade-off (WARNING in PKG_CFLAGS vs. local pragma),
  cite astra PR #9 as the downstream incident, link the issue.
- File any deferred items (e.g., "Audit other C TUs that may add R-version-
  specific suppressions") as separate issues *before* PR per
  "Concessions → issues".

## Common pitfalls

- **Don't confuse `-Wunknown-warning-option` with `-Wfixed-enum-extension`.**
  The former is the meta-warning (the one we want to suppress); the latter
  is what R's pragma asks clang to silence (and clang 21 doesn't recognise).
  Suppressing the wrong one re-introduces the issue on either clang 20
  or 21+.
- **Don't put the pragma in `Makevars.in`** (e.g., as a `-Xclang` arg).
  That defeats the whole point — anything in `PKG_CFLAGS` re-exposes the
  WARNING.
- **Forgetting `autoconf`** after editing `configure.ac` leaves the old
  `configure` script in place; CI will run the regenerated configure on
  fresh checkouts but local `bash ./configure` may use the cached one
  silently. Always regen and commit both.
- **Missing TUs**: if a C TU includes `Rinternals.h` *transitively* (e.g.,
  `Rdefines.h` → `Rinternals.h`), the shim still needs to wrap whichever
  header is the entry point. Audit `rpkg/src/*.c` and `rpkg/src/*.h` end
  to end before declaring done.
- **Bindgen-emitted wrappers**: when a user scaffolds a new native-package
  binding via `minirextendr::use_native_package()`, the emitted
  `<pkg>_wrapper.h` must include `r_shim.h`, not `<Rinternals.h>` directly.
  Check the codepath in `minirextendr/R/use_native_package.R` (or
  wherever the template lives) and update the emitter.
