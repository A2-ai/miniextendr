+++
title = "roxygen2 8.0.0 adoption"
description = "Migrate DESCRIPTION fields, add @prop S7 emission, and add R6 @field opt-out for noexport active bindings"
+++

# roxygen2 8.0.0 adoption

roxygen2 8.0.0 was released 2026-05-01.
Blog post: https://opensource.posit.co/blog/2026-05-01_roxygen2-8-0-0/

---

## Background

Key 8.0.0 changes affecting miniextendr:

- **DESCRIPTION migration**: `Config/roxygen2/markdown: TRUE` replaces `Roxygen: list(markdown = TRUE)`. `Config/roxygen2/version` replaces `RoxygenNote` (auto-migrated on first run).
- **`@prop name description`**: new tag for S7 non-constructor properties (replaces putting them in `@param` on the constructor).
- **`@R6method Class$method`**: new tag for documenting R6 methods from outside the class block.
- **`@field name NULL`**: opt-out for fields/active bindings; omits the field from the rendered Rd.
- **R6 `$set()` block docs**: method docs now also work inside `Counter$set("public", "name", function(...) {...})` blocks.
- **R6 superclass parameter inheritance**: roxygen2 can now pull `@param` from parent classes.
- **`@inheritParams` filter syntax**: `@inheritParams f x y` (allowlist) and `@inheritParams f -z` (denylist).
- **`@inheritDotParams` reads documented params**, not formals (avoids phantom param docs).
- **`needs_roxygenize()` cheap up-to-date check**: avoids full re-run when nothing changed.
- **Min R 4.1** (already satisfied by miniextendr's `Depends: R (>= 4.1.0)`).

---

## Current state — gaps

- `rpkg/DESCRIPTION`: has `Config/roxygen2/version: 8.0.0` (set by PR #349) but still has legacy `Roxygen: list(markdown = TRUE)` and `RoxygenNote:` field. Needs `Config/roxygen2/markdown: TRUE` and `Config/roxygen2/version: 8.0.0`; legacy fields dropped.
- `tests/cross-package/producer.pkg/DESCRIPTION` and `tests/cross-package/consumer.pkg/DESCRIPTION`: still have legacy `Roxygen: list(markdown = TRUE)` and `RoxygenNote: 7.3.2`.
- `minirextendr/DESCRIPTION`: still has legacy `Roxygen: list(markdown = TRUE)` and `RoxygenNote: 7.3.3`.
- `minirextendr/R/create.R:187`: scaffolding template emits both legacy fields with stale `RoxygenNote: 7.3.2`. Newly scaffolded packages will be born out of date.
- `miniextendr-macros/src/miniextendr_impl/s7_class.rs`: no `@prop` emission for getter/setter properties that are not constructor parameters. They currently land in constructor `@param` or get no docs at all.
- `miniextendr-macros/src/miniextendr_impl/r6_class.rs:420`: always emits `#' @field <name> Active binding.` regardless of `#[miniextendr(noexport)]`. There is no opt-out path for active bindings that should be hidden from the Rd.

---

## In-scope work

Three independent PRs; no ordering dependency between them.

### PR A — DESCRIPTION migration (purely textual)

**Files to touch:**

- `rpkg/DESCRIPTION` — replace `Roxygen: list(markdown = TRUE)` with `Config/roxygen2/markdown: TRUE`; remove `RoxygenNote:` (version already in `Config/roxygen2/version: 8.0.0`).
- `tests/cross-package/producer.pkg/DESCRIPTION` — same migration; update `RoxygenNote: 7.3.2` → `Config/roxygen2/version: 8.0.0`.
- `tests/cross-package/consumer.pkg/DESCRIPTION` — same.
- `minirextendr/DESCRIPTION` — same; drop `RoxygenNote: 7.3.3`, add `Config/roxygen2/version: 8.0.0`.
- `minirextendr/R/create.R:187` — update the template string that scaffolds a new package's DESCRIPTION to emit `Config/roxygen2/markdown: TRUE` and `Config/roxygen2/version: 8.0.0` instead of the legacy fields.

**Acceptance criteria:**

- `grep -r "RoxygenNote" rpkg/ tests/cross-package/ minirextendr/` returns nothing.
- `grep -r "Roxygen: list" rpkg/ tests/cross-package/ minirextendr/` returns nothing.
- All four DESCRIPTION files have `Config/roxygen2/markdown: TRUE` and `Config/roxygen2/version: 8.0.0`.
- `just devtools-document` produces no `.Rd` churn (roxygen2 8.0.0 auto-migrates, so the first run is idempotent once DESCRIPTION is corrected).
- `just minirextendr-test` passes (scaffolding test verifies DESCRIPTION fields).
- `just cross-test` passes.

---

### PR B — S7 `@prop` emission

**Problem:** S7 properties defined via `#[miniextendr(s7(getter = ...))]`, `#[miniextendr(s7(setter = ...))]`, `#[miniextendr(s7(validator = ...))]`, or `#[miniextendr(s7(prop = "..."))]` that are NOT constructor parameters have no `@prop` tag in the generated class-level roxygen block. roxygen2 8.0.0 introduced `@prop name description` for this purpose.

**Files to touch:**

- `miniextendr-macros/src/miniextendr_impl/s7_class.rs` — in the class-level roxygen block emitter, for each S7 property that is not a constructor parameter, emit `#' @prop <prop_name> <description>`. Source the description from the getter's doc comment; fall back to setter's, then validator's, then an empty string. Constructor parameters keep `@param` as before.

**Fixture and verification:**

- Add an rpkg fixture struct (e.g., in `rpkg/src/rust/s7_tests.rs`) with at least one getter-only property (no constructor parameter).
- After `just configure && just rcmdinstall && just devtools-document`, verify the generated `.Rd` contains `\section{Properties}{` with the property name (roxygen2 8.0.0 renders `@prop` there).
- Commit the updated `rpkg/R/miniextendr-wrappers.R`, `NAMESPACE`, and affected `man/*.Rd` alongside the macro change.

**Acceptance criteria:**

- `grep "@prop" rpkg/man/<fixture_class>.Rd` succeeds.
- Constructor parameters still render as `\arguments{...}` not `\section{Properties}{`.
- `just devtools-test` passes.
- `just clippy` clean.

---

### PR C — R6 `@field name NULL` opt-out

**Problem:** `miniextendr-macros/src/miniextendr_impl/r6_class.rs:420` always emits `#' @field <name> Active binding.` for every active-binding method. When a method is marked `#[miniextendr(noexport)]`, that field still appears in the rendered Rd, leaking internal implementation details into the package docs.

**Files to touch:**

- `miniextendr-macros/src/miniextendr_impl/r6_class.rs` — at the site that emits `#' @field <name> Active binding.`, check whether `noexport` is set on the method. If yes, emit `#' @field <name> NULL` instead (roxygen2 8.0.0 `NULL` opt-out omits the field from rendered Rd).

**Fixture and verification:**

- Add or update an rpkg R6 fixture with one active binding annotated `#[miniextendr(noexport)]`.
- After `just configure && just rcmdinstall && just devtools-document`, verify the generated `.Rd` for that class does NOT contain the field name in `\section{Active bindings}{}`.
- Commit the updated wrappers, NAMESPACE, and Rd files.

**Acceptance criteria:**

- The noexport active binding does not appear in the rendered `man/<class>.Rd`.
- A non-noexport active binding on the same class still appears with `Active binding.` text.
- `just devtools-test` passes.
- `just clippy` clean.

---

## Out-of-scope items (tracked as issues)

These are lower priority or require more design work. Each has a dedicated issue.

- **#369** — R6 `$set()` block codegen migration: emit method docs from `Counter$set("public", "name", function(...) {...})` form so roxygen2 8.0.0 auto-detects them.
- **#370** — `@R6method` external-doc support: allow documenting an R6 method from outside the class block using the new `@R6method Class$method` tag.
- **#371** — doclisting for S7 generics: explore whether `@prop` + `@family` or doclisting integration improves S7 generic discoverability.
- **#372** — `@inheritParams` filter codegen: use the new allowlist/denylist filter syntax in generated wrappers where appropriate.
- **#373** — `needs_roxygenize()` short-circuit: integrate the cheap up-to-date check into `just devtools-document` or the build pipeline to skip full re-runs.
- **#374** — R6 superclass param inheritance: propagate `@param` from parent classes when R6 classes extend a base that is also defined via `#[miniextendr]`.

---

## Sequencing / dependencies

- PR A, PR B, and PR C touch disjoint files and can be opened, reviewed, and merged in any order.
- PR A is purely textual (no Rust compilation required for the DESCRIPTION changes, though `just devtools-document` must be run to verify no `.Rd` churn).
- PR B and PR C both require `just configure && just rcmdinstall && just devtools-document` to regenerate `R/miniextendr-wrappers.R`, NAMESPACE, and `man/*.Rd` after the macro changes — those generated artifacts must be committed in sync with the Rust change per the CLAUDE.md pre-commit hook rule.
- Merging PR A first has no effect on PR B or C; no rebase churn expected.
