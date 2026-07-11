# Plan: #1248 — S3/vctrs generic guard silently skips dispatch on base non-generics

Date: 2026-07-11. Anchors re-verified against main @ 17f634d8.
Branch: `fix/1248-s3-nongeneric-shadow`.

Related: #1114 (the merged S7 sibling — pattern source, `s7_class.rs:880-935`),
#1261 item 1 (`.mx_gen` leaks into the package namespace from that very S7
emission — this plan fixes the leak in the mirrored line and back-ports the
fix to S7, see below), #1242/#1206 (other macro-static issues, no file
overlap), PR #1223 (the review that surfaced this).

## The bug (recap)

`emit_s3_generic_guard` (`r_class_formatter.rs:57-62`) emits a bare
existence check:

```r
if (!exists("name", mode = "function")) {
  name <- function(x, ...) UseMethod("name")
}
```

When `name` resolves to a **plain non-generic closure** (`var`, `get`, `row`,
`col`, `diag`, `reshape`, …), the guard sees it exists and never creates the
`UseMethod` dispatcher. The generator still emits `name.Class` +
`S3method(name, Class)`, but `name` stays bound to e.g. `stats::var`, which
never consults the S3 table — the registered method silently never fires.
Unlike the S7 case (#1114, loud load error), this installs clean and just
misbehaves.

Three call sites share the helper: `s3_class.rs:138`, `vctrs_class.rs:289`,
trait-ABI `miniextendr_impl_trait/r_wrappers.rs:332`. Fixing the helper fixes
all three; the trait-ABI names are namespace-qualified so the new classifier
is inert there (the `!exists` branch fires as today — verify one generated
sample in the snapshot diff).

## Design: mirror the #1114 classifier, with two corrections

Replace the helper body with the exists/classify/shadow chain, transliterated
from `s7_class.rs:880-935` to S3:

```r
if (!base::exists("name", mode = "function")) {
  name <- function(x, ...) UseMethod("name")
} else if (local({ .mx_gen <- base::get("name", mode = "function"); !(is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric("name") || inherits(.mx_gen, "S7_generic")) })) {
  # `name` is a plain closure that UseMethod dispatch will never consult.
  # Shadow it with a package-local generic; keep ordinary calls working by
  # delegating the default method to the masked closure.
  name.default <- local({
    .mx_masked <- base::get("name", mode = "function")
    function(x, ...) .mx_masked(x, ...)
  })
  name <- function(x, ...) UseMethod("name")
}
# else: existing usable generic (primitive/S3/S4/S7) — reuse as-is.
```

Decisions baked into that shape, in order of how much they can bite:

1. **`name.default` binds BEFORE `name` is rebound.** Both `local` blocks call
   `base::get("name", mode = "function")`; wrappers.R is sourced top-to-bottom,
   so if the generic were bound first, `get` would find *our own generic* and
   the default method would recurse forever. Same eager-force reasoning as the
   `.mx_masked` comment at `s7_class.rs:920-924` (assignment inside `local`
   forces the value now; an argument would stay an unforced promise).
2. **No `S3method(name, default)` registration, deliberately.** roxygen2
   cannot introspect a binding created inside an `else if` brace, and an
   unconditional `@rawNamespace S3method(name, default)` would break package
   *load* whenever the shadow branch doesn't fire (`name.default` wouldn't
   exist — and for a real generic like `print` we must never touch
   `print.default`). Registration is also unnecessary: `UseMethod` searches
   the generic's defining environment, and `name.default` sits beside the
   generic in the package namespace. Pin this with a from-outside-the-package
   testthat call (see Tests) rather than trusting the prose.
3. **The classifier is wrapped in `local({...})`** — this is the correction to
   the #1114 pattern. S7's `else if ({ .mx_gen <- ... })` evaluates the braced
   block in the namespace env at source time, leaving a stray `.mx_gen`
   object in every installed package — that is exactly #1261 item 1's
   `R CMD check` WARNING (".mx_gen calls .Internal(get(...))": the leaked
   binding is a captured base closure). **Also apply the one-line `local()`
   fix to `s7_class.rs:902` in this PR** — it's the identical line being
   mirrored, and leaving it leaky while writing the corrected copy next door
   fails the "fix warnings you see" rule. Cross-reference #1261 in the PR
   body so item 1 is checked off there.
4. **Classifier accepts primitive / S3-std-generic / S4 generic / S7 generic**
   — the full #1114 usability list, not just the issue sketch's two-term
   version. An S7 or S4 generic binding can accept S3 method registration via
   the table; treating them as "plain closure" would shadow needlessly.
   `base::exists`/`base::get` qualification is load-bearing (a shadow generic
   named `get` would otherwise re-route later classifiers — s7_class.rs:888
   comment).
5. **Roxygen safety**: the chain still starts with `if (` — no top-level
   assignment for roxygen2 to latch onto (the `\alias{.mx_gen}` pollution
   trap, s7_class.rs:880-886). The explicit-target `#' @export name` handling
   at `s3_class.rs:126-137` / `vctrs_class.rs:280-287` is unchanged.
6. **S3 generic formals are always `function(x, ...)`** — masked closures with
   different first-arg names (`var`'s is `x`, but e.g. `reshape`'s is `data`)
   still work through the default method because delegation is positional +
   `...`. Note this in the helper's rustdoc; it's why S3 doesn't need S7's
   `dispatch_args`-mirroring `fallback_sig` machinery.

Repeated shadowing is self-stabilizing: a second class method on the same
shadowed generic re-runs the chain, finds our package-local `UseMethod`
generic, classifies it `isS3stdGeneric` → reuses. No double-`name.default`.

Macro-time denylist warning (floated in the issue sketch): #1114 did **not**
ship one for S7 (only the comment at `s7_class.rs:846` lists the known
names); the runtime classifier fully handles the situation, silently and
correctly. Don't invent one for S3 either — record that in the PR body as the
resolution of the issue's "keep the failure loud" note (there is no failure
left to keep loud; masking is safe in all S3 cases per point 6).

## Dependency plumbing (scaffolded packages)

The classifier references `utils::isS3stdGeneric` and `methods::isGeneric`
**in every S3/vctrs package** (the guard is emitted per generic, not only on
collision), so undeclared-Imports NOTEs follow without:

- `rpkg/DESCRIPTION` already Imports `utils` (line 25) and `methods` (via
  #1114) — no change.
- **minirextendr: add `use_s3()`** mirroring `use_s7()`
  (`minirextendr/R/use-feature.R:158-166`) — `add_import("methods")` +
  `add_import("utils")` with a #1248-flavored version of the same
  explanatory comment. There is currently no S3 enabler at all (S3 needed no
  deps before this), so this is a new exported function: roxygen block,
  `minirextendr/NAMESPACE` regen, mention in the scaffolding docs.
- **Extend `use_vctrs()`** (`use-feature.R:111`) with the same two imports —
  the vctrs generator shares the helper.
- Update `docs/CLASS_SYSTEMS.md`'s S3 section: name the shadow behavior and
  the `use_s3()` requirement for packages whose method names collide with
  base non-generics.

## Work items (flat order)

1. Rewrite `emit_s3_generic_guard` (`r_class_formatter.rs:57-62`) per the
   design block; update its rustdoc (currently shows the bare-exists form and
   says "Do not use for S7" — keep that note).
2. Apply the `local()` classifier fix to `s7_class.rs:902` (#1261 item 1).
3. `use_s3()` + `use_vctrs()` imports in minirextendr; document; regen
   minirextendr NAMESPACE/man.
4. rpkg fixture: `#[miniextendr(s3)]` impl with `fn var(&self) -> f64` (plus
   a second S3 class also defining `var` to pin the reuse path). Mirror
   however #1114's S7 repro fixture is laid out in `rpkg/src/rust/`.
5. testthat: (a) `var(obj)` hits the Rust method; (b) `var(1:10)` still
   returns `stats::var`'s answer via the default delegation, called from the
   test env (outside the package namespace); (c) both S3 classes dispatch;
   (d) no `.mx_gen` object in `getNamespace("miniextendr")` (pins the leak
   fix for both S3 and S7 paths).
6. Rebaseline macro snapshots — every S3/vctrs/trait snapshot containing the
   old 3-line guard changes (mechanical, large; review one of each kind).
7. Regen loop ×2 (`configure && rcmdinstall && force-document &&
   rcmdinstall` — new fixture exports need the second install), commit
   NAMESPACE/man with the Rust changes. `just devtools-test`, `just
   cross-test` (trait-ABI wrappers regenerate), `just minirextendr-test`,
   three clippy legs.
8. Confirm the `checking R code for possible problems` WARNING line about
   `.mx_gen` is gone from `just r-cmd-check` output (log to
   `/tmp/rcmdcheck.log`, Read it) — partial credit toward #1261.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1      # must print 4.6.x
just worktree-sync                             # FIRST (rv sync prunes dev pkgs)
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                  ^ ×2 install: new fixture exports must be
#                                    runtime-callable for testthat/gctorture
just test 2>&1 > /tmp/1248-rust-test.log       # Rust suites (Read the log)
just devtools-test 2>&1 > /tmp/1248-devtools.log
# devtools::test ALWAYS exits 0 — grep the log for "[ FAIL" and require FAIL 0
grep -E '\[ FAIL [0-9]+' /tmp/1248-devtools.log
just cross-test 2>&1 > /tmp/1248-cross.log     # trait-ABI wrappers regenerate
just minirextendr-test 2>&1 > /tmp/1248-minir.log   # same FAIL-0 grep applies
just r-cmd-check 2>&1 > /tmp/1248-rcmdcheck.log     # then Read for .mx_gen WARNING absence
# Three clippy legs (feature list: read from .github/workflows/ci.yml clippy_all step):
cargo clippy --workspace --all-targets --locked -- -D warnings
# + clippy_all and clippy_all_s7 variants per ci.yml
cargo fmt --all
```

Snapshot rebaselining: for insta `.snap.new` files, diff vs `.snap`, and if the
only change is the new guard chain, `mv` over the old snapshot and re-run
`just test`. For trybuild `.stderr` mismatches: do NOT run `TRYBUILD=overwrite`
locally (CI toolchain is authoritative — #1239); if a UI test's stderr changes,
stop and report per the escalation rule.

## Must NOT touch

- `rpkg/R/miniextendr-wrappers.R`, `rpkg/src/rust/wasm_registry.rs` (generated,
  gitignored — never hand-edit, never commit).
- `NAMESPACE`/`man/*.Rd` are committed ONLY as regenerated output of the loop
  above, in the same commit as the Rust change that caused them.
- No edits to `s7_class.rs` beyond the single `local()` wrap at line 902.
- No macro-time denylist warning (see design note above — resolved as not-needed).
- Don't touch `rpkg/src/Makevars` or other generated build files.

## Done criteria

- An S3 method named `var`/`get`/`diag` dispatches; plain calls to the masked
  base function still work from outside the package; two classes sharing a
  shadowed generic install and both dispatch.
- No `.mx_gen` binding in any installed namespace (S3 or S7 path); the
  corresponding `R CMD check` WARNING is gone.
- `use_s3()` exists; `use_vctrs()` declares `methods`+`utils`; scaffolded
  S3/vctrs packages check clean of undeclared-import NOTEs.
- Snapshots, all test suites, three clippy legs green.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, a test fails for
a reason the plan doesn't predict, a step is impossible as written — **stop,
commit nothing further, and report back what you found. Do not improvise.**
