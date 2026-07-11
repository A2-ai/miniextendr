# Plan: #1273 — crate-prefix macro-emitted C symbols (webR cross-package collision)

Date: 2026-07-10. Anchors re-verified against main @ 17f634d8 (lib.rs cites
shifted +2 lines vs the original 14b2688e pass; updated below).
Branch: `fix/1273-c-symbol-crate-prefix`.

Related: #1259/PR #1272 (the CI leg that observed the bug — carries a
temporary rename workaround this fix must revert), #1271/#1270/#1255 (webR CI
cluster siblings), #495 (wasm cross-crate trait dispatch — adjacent design,
untouched here), #1242 (E0428 on *internal* statics across labeled impl
blocks — same naming neighborhood, different mechanism, no overlap).

**Sequencing**: ~~land after PR #1272 merges~~ **UNBLOCKED — #1272 merged
2026-07-11**; the scaffold leg with the `mxsmoke` rename workaround is on main
(`.github/workflows/webr.yml:364-366`), so step 15's revert applies directly.
Coordinate with the webR coverage batch
(`plans/2026-07-11-webr-ci-coverage-batch.md`) — both edit webr.yml;
whichever lands second rebases.

## The bug (recap)

Every `#[miniextendr]` wrapper is emitted `#[unsafe(no_mangle)]` with a
package-agnostic name (`C_add`, `C_Board__new`, …). Native `dyn.load` uses
`RTLD_LOCAL`, so identical names across package `.so`s never meet. Under webR,
package shared objects are Emscripten SIDE_MODULEs sharing one GOT: the
first-loaded module's definition of an exported name wins for **every** later
module, so a second package's `R_registerRoutines` table captures function
pointers into the first package's code. Observed empirically in the #1259 CI
leg: `mxsmoke::add(2, 3)` dispatched into rpkg's `add(left: i32, right: i32)`
(`rpkg/src/rust/panic_tests.rs:63`) and failed with rpkg's parameter names in
the error. Details + ruled-out alternatives in issue #1273.

## Design decision

**Prefix the identifier with the consuming crate's name at macro expansion,
keeping the invariant `registration name == linker symbol name`** ("Option A",
the issue's sketch): `C_add` → `C_<crate>_add`.

Why the invariant matters: the wasm snapshot writer
(`miniextendr-api/src/wasm_registry_writer.rs`) reconstructs `extern "C" { fn
<name>(...); }` declarations **from `R_CallMethodDef.name`** (`CallDefRow.name`,
see the module doc: "Every fn / static referenced from a slice gets a matching
`extern` decl — the WASM linker resolves them against the user crate's
`#[no_mangle]` exports"). Renaming only the symbol while keeping the short
registration name ("Option B") would require carrying a second, symbol-only
string beside every `R_CallMethodDef` — a new wrapper struct in
`registry.rs`, a writer change, and a `GENERATOR_VERSION` bump — to save churn
in files that are gitignored and regenerated anyway (`wrappers.R`,
`wasm_registry.rs`). Rejected.

Under Option A nothing shape-changes: `GENERATOR_VERSION` stays 1, the
generated file just carries longer names and a new content-hash. R-level
`.Call(C_<crate>_<fn>, ...)` objects come from `useDynLib(.registration=TRUE)`
and are namespace-local, so the rename is invisible to package users;
`NAMESPACE` and `man/` are untouched (export names don't change).

**Prefix source**: `std::env::var("CARGO_CRATE_NAME")` read by the proc macro
at expansion time — exactly the precedent `miniextendr_init!()` already uses,
including the missing-var and invalid-ident error paths
(`miniextendr-macros/src/lib.rs:2929-2959`). Cargo normalizes hyphens to
underscores, so the value is a valid C/R identifier; rpkg's crate is named
`miniextendr` (`rpkg/src/rust/Cargo.toml:2`), so its symbols become
`C_miniextendr_add`, `C_miniextendr_Board__new`, etc.

## Verified symbol inventory (what gets prefixed)

| Emission site | Current shape | New shape |
|---|---|---|
| `miniextendr_fn.rs:989-997` (`c_wrapper_ident`) | `C_<fn>` | `C_<crate>_<fn>` |
| `miniextendr_impl.rs:2124-2130` | `C_<Type>__<method>`, `C_<Type>_<label>_<method>` | `C_<crate>_<Type>__<method>`, `C_<crate>_<Type>_<label>_<method>` |
| `miniextendr_impl_trait.rs:187-198` (TraitMethod, ident + string) | `C_<Type>__<Trait>__<method>` | `C_<crate>_<Type>__<Trait>__<method>` |
| `miniextendr_impl_trait.rs:296-305` (TraitConst, ident + string) | `C_<Type>__<Trait>__<CONST>` | `C_<crate>_<Type>__<Trait>__<CONST>` |
| `externalptr_derive.rs:939-940` (sidecar accessors) | `C__mx_rdata_get_<Type>_<field>` / `..._set_...` | `C_<crate>__mx_rdata_get_<Type>_<field>` / … |
| `altrep.rs:75-79` (ALTREP registration fns) | `__mx_altrep_reg_<Ident>` | `__mx_altrep_reg_<crate>_<Ident>` |
| `vtable.rs:96` (vtable static, no_mangle at :233) | `__VTABLE_<TRAIT>_FOR_<TYPE>` | `__VTABLE_<crate:upper>_<TRAIT>_FOR_<TYPE>` |
| `vtable.rs:254,381` (vtable shims, no_mangle at :260) | `__vtshim_<Type>__<Trait>__<method>` | `__vtshim_<crate>_<Type>__<Trait>__<method>` |

Deliberately **unchanged**:
- `R_init_<pkg>` / `R_unload_<pkg>` — already package-unique (`lib.rs:2963-2964`).
  `rpkg/src/win.def.in` exports only `R_init_@PACKAGE_NAME@`; no def change.
- `miniextendr_force_link` — package-independent **by design** (stub.c takes
  its address without configure substitution, `lib.rs:3017-3029`). It's a
  value-irrelevant linker anchor; GOT interposition of a marker byte is
  harmless. Leave it, note why in a comment.
- Explicit `extern "C-unwind"` fns and `#[export_name = "..."]` passthroughs
  (`miniextendr_fn.rs:993-996`) — the user owns those symbols; document that
  they also own cross-package uniqueness on webR (docs step below).
- api-crate fixed names `miniextendr_write_wrappers` /
  `miniextendr_write_wasm_registry` — looked up per-DLL via
  `getNativeSymbolInfo(..., lib)` on **native only** (Makevars wrapper-gen
  step); wasm never calls them. In scope for the follow-up issue (below), not
  this PR.

## Work items (flat order)

1. **`naming.rs`: one canonical helper.** Add `crate_prefix() -> String`
   reading `CARGO_CRATE_NAME` (error text mirroring `lib.rs:2936` if unset in
   a cargo build; deterministic fallback for direct-call unit tests — see
   caveat below) and thin formatters the table above routes through. Per
   macros CLAUDE.md, `naming.rs` is the single source of truth — the point of
   this item is that steps 2–7 become one-line call-site edits.
   *Caveat to verify empirically*: `CARGO_CRATE_NAME` is guaranteed at
   **expansion** time (inside rustc), but the macros crate's own insta tests
   call codegen fns at **test runtime** — check whether cargo sets it there;
   if not, fall back to normalized `CARGO_PKG_NAME`, else literal `"crate"`,
   and pin snapshots to whichever value is stable on both local and CI.
2. **Bare fns**: `miniextendr_fn.rs:989-997`. Internal-wrapper arm only;
   extern/export_name arms pass through untouched.
3. **Impl methods**: `miniextendr_impl.rs:2124-2130` (both label arms).
4. **Trait methods + consts**: `miniextendr_impl_trait.rs:187-198,296-305`.
   The ident and string variants must share one formatter so they can't drift
   (today the format string is duplicated four times).
5. **Sidecar accessors**: `externalptr_derive.rs:939-940`. The `\0`-suffixed
   registration cstrings at `:1005-1010` derive from the same strings — no
   second edit, but confirm.
6. **ALTREP registration fns**: `altrep.rs:75` (+ the paired
   `__MX_ALTREP_REG_ENTRY_<Ident>` linkme static at `:76` may stay unprefixed —
   it's internal, not `no_mangle`).
7. **Vtable statics + shims**: `vtable.rs:96,254,381`. While there, check the
   `TAG_<TRAIT>` static (`vtable.rs:191`) and `__MX_BASE_VTABLE_/__MX_TAG_`
   statics (`externalptr_derive.rs:1241-1242`) for `no_mangle`; grep says they
   aren't exported (tags travel via `R_RegisterCCallable` strings, which are
   already package-scoped) — confirm and leave.
8. **Stray-format audit.** Grep the macros crate for inline `"C_{`/`C_%`-style
   reconstructions that bypass the helpers: known consumers
   `r_class_formatter.rs:287`, `method_context.rs:97`,
   `miniextendr_impl_trait/r_wrappers.rs:258,427,590,836,988`,
   `r_wrapper_builder.rs` (`DotCallBuilder` takes the ident as a string — fine
   if callers pass the helper output). All must route through steps 2–7's
   functions, not re-derive.
9. **Lint**: `miniextendr-lint/src/rules/trait_tag_collision.rs:107-118`
   reconstructs `__VTABLE_{TRAIT}_FOR_{TYPE}`. The rule compares symbols
   *within one crate*, so a constant crate prefix can't change its verdicts —
   but update the reconstruction + rule docs (`lint_code.rs:48`) to the new
   shape anyway so the cited symbol matches reality (lint runs in the user
   crate's `build.rs`, which has no `CARGO_CRATE_NAME`; since the check is
   prefix-invariant, compare on the unprefixed suffix rather than trying to
   reconstruct the prefix from `CARGO_PKG_NAME`).
10. **rpkg literal**: `rpkg/tests/testthat/test-rng.R:65` —
    `getNativeSymbolInfo("C_rng_worker_uniform", "miniextendr")` →
    `"C_miniextendr_rng_worker_uniform"`. This is the only raw registration
    name in R sources/tests (`test-worker.R`'s `unsafe_C_*` are R-level
    wrapper *function* names derived from Rust fn idents — unaffected).
11. **Snapshots**: rebaseline macros insta snapshots (`.snap.new` → review →
    move); trybuild `.stderr` only if any fixture output embeds symbol names —
    regenerate via CI, **not** local `TRYBUILD=overwrite` (rust-src span
    lesson, #1239).
12. **Docs sweep**: 6 files cite `C_…` shapes — `docs/CALL_ATTRIBUTION.md`,
    `CLASS_SYSTEMS.md`, `R_COERCE.md`, `S3_METHODS.md`, `TRAIT_AS_R.md`,
    `VISIBILITY.md`. Update shapes; add a `docs/WEBR.md` gotcha: symbols are
    crate-prefixed for GOT-interposition safety, and hand-written
    `extern`/`export_name` fns must be made package-unique by the author.
    Also touch the shape mentions in `miniextendr-macros/CLAUDE.md` (MXL111
    text unaffected) and rustdoc comments quoting `C_<name>` in
    `miniextendr_fn.rs:966-988`.
13. **Regen + native verification**: `just configure && just rcmdinstall &&
    just force-document && just rcmdinstall` (new-export double-install rule —
    here names change, same discipline), then `just test`, `just
    devtools-test`, `just cross-install && just cross-test` (cross-package
    producer/consumer regenerate their wrappers with their own crate
    prefixes — this is itself a mini collision test: producer.pkg and
    consumer.pkg now can't share wrapper symbols even in principle). Three
    clippy legs per CLAUDE.md. `NAMESPACE`/`man` diffs should be empty; if
    force-document flips S3method/export lines, apply the known
    revert-NAMESPACE lesson.
14. **Templates**: `minirextendr/inst/templates/**` contains no generated
    wrapper code and no `C_` literals (verified — only the api-level
    `miniextendr_write_wrappers` Makevars lookups, unchanged). Expect
    `templates-check` green with no patch churn; run it to confirm.
15. **wasm end-to-end proof** (after #1272 merges): revert the workaround that
    renames the scaffolded package's stock `add`/`hello` in the #1259 CI leg
    (issue #1273 "Notes" pins this). The tier-3 session then loads rpkg +
    `mxsmoke` with an *intentional* `add` name collision and asserts
    `mxsmoke::add(2, 3) == 5` via mxsmoke's own f64 path AND rpkg's
    `add(2L, 3L)` still works. Optionally add a cheap belt-and-braces check to
    the smoke script: no `C_`-prefixed export name appears in more than one
    side module (`wasm-nm`/`llvm-nm` over the two `.so`s).
16. **Follow-up issue (file, don't fix here)**: miniextendr-api's own
    `#[unsafe(no_mangle)]` exports are byte-identical names in every
    miniextendr package (`mx_abi.rs:61,81,104`, `worker.rs:230,304`,
    `encoding.rs:52,99`, `registry.rs:383,1826,1854`, `backtrace.rs:67`, and
    — most concerning — `altrep_impl/builtins.rs`, whose registration fns are
    extern-declared in every package's `wasm_registry.rs`). Under the same
    GOT interposition, package B's builtin-ALTREP registration can execute
    package A's copy against A's module-local statics. Not observed yet (the
    #1259 leg doesn't exercise builtin ALTREP cross-package); needs an
    `nm`-based inventory + a repro attempt before deciding on a
    crate-disambiguation story for api-internal exports. Reference this plan.

## Residual risks (accepted, stated in PR body)

- Two R packages whose **crates** share a name (custom `[lib] name` diverging
  from the package name) still collide. R itself forbids two same-named
  *packages* in a session, and the scaffold derives crate name from package
  name, so this needs deliberate misconfiguration. Optional cheap guard later:
  `minirextendr_doctor()` warns when crate name ≠ package name.
- Longer symbol/registration names are cosmetic in `getDLLRegisteredRoutines()`
  output and backtraces.

## Done criteria

- Fresh scaffold + rpkg loaded into one webR session with colliding Rust fn
  names dispatch to their own implementations (the reverted #1259 leg is the
  regression test).
- All macro-emitted `no_mangle` symbols carry the crate prefix; `just test`,
  `just devtools-test`, `just cross-test`, three clippy legs, snapshots green.
- Follow-up issue for api-crate exports filed and cross-referenced from
  #1273 and the PR body.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1      # must print 4.6.x
just worktree-sync                             # FIRST (rv sync prunes dev pkgs)
just configure && just rcmdinstall && just force-document && just rcmdinstall
just test 2>&1 > /tmp/1273-rust-test.log       # Read the log, don't tail
just devtools-test 2>&1 > /tmp/1273-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1273-devtools.log   # devtools::test always exits 0
just cross-install && just cross-test 2>&1 > /tmp/1273-cross.log
just templates-check                           # expect green, no patch churn
# Three clippy legs (read the feature list from .github/workflows/ci.yml clippy_all):
cargo clippy --workspace --all-targets --locked -- -D warnings   # + all/all_s7 legs
cargo fmt --all
```

Snapshot rebaselining: insta `.snap.new` → diff → `mv` over `.snap`, re-run
`just test`. trybuild `.stderr`: never `TRYBUILD=overwrite` locally (#1239) —
if stderr output changes, stop and report per the escalation rule.

## Must NOT touch

- `rpkg/R/miniextendr-wrappers.R` / `rpkg/src/rust/wasm_registry.rs`
  (generated, gitignored). `NAMESPACE`/`man` expected UNCHANGED — if
  force-document flips S3method↔export lines, revert NAMESPACE (known
  load_all drift lesson) and keep only intended diffs.
- `minirextendr/inst/templates/**` and `patches/templates.patch` (verified: no
  C_ literals there; templates-check must stay green without edits).
- api-crate `no_mangle` exports (work item 16 files an issue instead).
- `miniextendr_force_link`, `R_init_/R_unload_` shapes.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, the
CARGO_CRATE_NAME caveat in item 1 resolves differently than either fallback,
a test fails in a way the plan doesn't predict — **stop, commit nothing
further, and report back. Do not improvise.**
