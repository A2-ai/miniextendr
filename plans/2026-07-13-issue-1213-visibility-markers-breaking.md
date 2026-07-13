# Plan: #1213 — breaking `Invisible<T>`/`Visible<T>` markers; remove implicit invisibility

Date: 2026-07-13. Anchors verified against main @ 175674a7.
Branch: `feat/1213-visibility-markers`.

Decision baked in (maintainer, 2026-07-13): **#1213 design 1 in its breaking
form** — return visibility becomes marker-controlled. The implicit
`invisible(self)` / `invisible(x)` / `invisible(.val)` defaults are REMOVED;
nothing is invisible unless marked, via either spelling (type marker or
attribute). No back-compat shim (repo principle). `WrapAs*` (design 2) stays
demoted to optional-override and is NOT in this wave; designs 2b
(`ConvertTo`/`ConvertFrom`) and 2c (`#[miniextendr(serialize)]`) are separate
follow-ups. Context: #1219 (Self-return re-wrapping, closed #1141) and #1232
(cross-class auto-wrap, extended by #1251) already solved design 2's original
motivation without markers — this plan does not touch that machinery.

## What already exists (do not rebuild)

- `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` **already exist for
  bare fns**: parsed into `force_invisible: Option<bool>`
  (`miniextendr-macros/src/miniextendr_fn.rs:1118`, bare-ident arms
  `:1331-1334`, `name = bool` arms `:1392-1395`), consumed at
  `miniextendr-macros/src/lib.rs:892` (override) and `:1116-1118`
  (`"invisible(.val)"` vs `".val"`). Fixtures:
  `rpkg/src/rust/visibility_tests.rs` (`force_invisible_i32`,
  `force_visible_unit`), pinned by
  `rpkg/tests/testthat/test-visibility-more.R`.
- The attribute is **absent from the impl-method grammar**: the method-option
  list at `miniextendr-macros/src/miniextendr_impl.rs:1441` has no
  `invisible`/`visible`.
- Detection precedent: last-path-segment matching via `type_ends_with`
  (`miniextendr_fn.rs:125-136`), reused by `is_dots_type`/`is_missing_type` —
  new markers are new call sites of the same helper, not a new scheme (per the
  unified-design comment on #1213, 2026-07-11).

## Implicit-invisibility inventory (every emission site, verified)

1. **Bare fns, shape-derived**: `analyze_return_type`
   (`miniextendr-macros/src/return_type_analysis.rs`) sets
   `is_invisible = true` for: no return arrow (`:93`), `-> ()` (`:99`),
   `Option<()>` (`:170`), `Result<(), ()>` (`:232`), `Result<(), E>` (`:261`).
   Emitted as `invisible(.val)` at `lib.rs:1116-1118`.
2. **Inherent R6 chainable tail**: `ReturnStrategy::for_method`
   (`method_return_builder.rs:117-131`) classifies `&mut self -> ()` and
   self-ref builders as `ChainableMutation`; only the R6 tail wraps —
   `invisible(self)` at `method_return_builder.rs:319`. Env/S3/S7/S4 chain
   tails already return the bare receiver (`:301`, `:338`, `:362`, `:383`,
   `:402`, `:422`) — the asymmetry this plan removes.
3. **Trait-impl void instance methods**, all five generators in
   `miniextendr-macros/src/miniextendr_impl_trait/r_wrappers.rs`:
   `invisible(x)` at `:232` (env), `:364` (s3), `:542` (s4), `:701` (s7
   generic), `:942` (r6); `invisible(self)` at `:774` (s7 fast-path shortcut).
4. **EXEMPT — sidecar/active-binding setters**:
   `externalptr_derive.rs:724-727` (standalone `Type_set_field` →
   `invisible(x)`), `:817` (R6 active-binding setter → `invisible(self)`),
   and the inherent R6 active-binding setter at
   `miniextendr_impl/r6_class.rs:527`. These are derive-/property-generated
   accessors with **no marker surface** (no user-written return type), and R
   replacement/assignment semantics make `x$f <- v` invisible regardless.
   Keep them invisible this wave; file a `gh issue create` ("visibility
   markers for derive-generated accessors") and reference it in the PR body.
5. **EXEMPT — condition transport**: the two `invisible(NULL)` in the shared
   `.miniextendr_raise_condition` helper (emitted from
   `miniextendr-api/src/registry.rs`) are warning/message propagation, not
   return-visibility codegen. Untouched.

Scale, measured on main's regenerated `rpkg/R/miniextendr-wrappers.R`:
`invisible(self)` ×17, `invisible(x)` ×49, `invisible(.val)` ×113,
`invisible(NULL)` ×2 (exempt) — **181 emitted sites, ~179 governed by this
change**. Regenerate and re-grep for the authoritative list during the sweep.

## Design

1. **New marker types** in a new file `miniextendr-api/src/visibility.rs`
   (mod from `lib.rs`, re-export from `prelude.rs`):
   ```rust
   #[repr(transparent)]
   pub struct Invisible<T>(pub T);   // + new(), into_inner(), Deref<Target=T>
   #[repr(transparent)]
   pub struct Visible<T>(pub T);     // symmetric no-op marker (parity with the attr)
   ```
   Both get `impl<T: IntoR> IntoR` forwarding to the inner value, so an
   alias-hidden or non-macro use still converts (fail-safe direction =
   visible, matching the new default). Names collision-checked: zero
   `struct|enum|trait|type (Invisible|Visible)` and zero `Invisible<`/
   `Visible<` hits across `miniextendr-api/src`, `miniextendr-macros/src`,
   `rpkg/src/rust` (verified 2026-07-13).
2. **One shared peel helper** in `miniextendr-macros/src/type_inspect.rs`
   (beside `first_type_argument`, `:20`):
   `peel_visibility_marker(ty) -> (Option<bool /*invisible*/>, &syn::Type)` —
   last-segment match on `Invisible`/`Visible`, returns the inner type.
   `Invisible<Invisible<T>>`, `Invisible<Visible<T>>`, and either marker in
   **argument** position are `compile_error!` (mirror `Missing<T>` validation
   style, `miniextendr_fn.rs:289-305`).
3. **Bare fns**: at the top of `analyze_return_type`
   (`return_type_analysis.rs:76`), peel the marker; analyze the inner type
   exactly as today (so `Invisible<Option<i64>>` keeps Option semantics);
   delete every shape-derived `is_invisible = true` (the five sites in
   inventory item 1 — all become `false`). Visibility resolution:
   marker > `force_invisible` attr if both present and they disagree →
   `compile_error!` (agreeing duplicates are fine); attr alone keeps working.
   The C-wrapper conversion works on the peeled inner type; the generated
   call-glue unwraps the newtype (`.0`) before conversion —
   `c_wrapper_builder.rs` return handling must see the inner type (thread the
   peeled type through, same spot `first_type_argument` is consulted).
4. **Inherent methods**: peel in `ReturnStrategy::for_method`
   (`method_return_builder.rs:117-131`) BEFORE the `returns_self`/
   `returns_result_self`/`returns_option_self`/`returns_self_ref`/
   `returns_unit`/`returns_other_class` predicates
   (`miniextendr_impl.rs:2143-2314`) so `Invisible<Self>`,
   `Invisible<()>` on `&mut self`, and `Invisible<OtherClass>` all classify
   as today's inner strategy. Add an `invisible: bool` field to
   `MethodReturnBuilder`; `build_with_tails` (`:259-281`) wraps the tail's
   final expression in `invisible(...)` when set — one spot, all six class
   systems (the #362 shared-tail architecture is exactly why this is one
   spot). Change the R6 `chain_tail` at `:319` from `invisible(self)` to
   `self`. Add `invisible` / `visible` to the method-option grammar
   (`miniextendr_impl.rs`, extend the option list at `:1441` and the parse
   arms near `:1323`), routed into the same builder field — attribute and
   marker are two front doors to one decision.
5. **Trait methods**: the marker lives in the `#[miniextendr_trait]`
   declaration's method signature (impls must match it, so behavior is
   uniform across implementing types — desirable). Replace the six
   void-method emissions (inventory item 3) with the bare receiver
   (`x`/`self`), and emit `invisible(<receiver>)` only when the parsed trait
   method signature carries `Invisible<...>` or the method attr says
   `invisible`. Update the decision doc at `miniextendr_impl_trait.rs:207`.
6. **Semantics that do NOT change**: `ChainableMutation` still returns the
   receiver (pipes and `|>` chains keep working — only REPL auto-print
   changes); `ReturnSelf`/`ReturnOtherClass` wrapping (#1219/#1232/#1251) is
   untouched; `Result<(), E>` still raises on `Err` (only the Ok-path
   visibility changes).

## R-visible behavior change table (goes in the PR body + docs)

| Signature (unmarked) | Today | After |
|---|---|---|
| bare fn `-> ()` / no arrow | `invisible(NULL)` | visible `NULL` (prints at REPL) |
| bare fn `-> Option<()>` / `Result<(), E>` | invisible on success | visible `NULL` on success |
| R6 `&mut self -> ()` | `invisible(self)` | visible `self` (still chainable) |
| R6/Env `&self -> Self` self-ref builder | `invisible(self)` | visible `self` |
| Env/S3/S7/S4 `&mut self -> ()` | bare receiver (already visible) | unchanged |
| trait-impl void instance method (all 5 systems) | `invisible(x)`/`invisible(self)` | visible receiver |
| any of the above marked `Invisible<...>` or `#[miniextendr(invisible)]` | n/a | today's invisible behavior, now explicit |
| sidecar `Type_set_field` / active-binding setters | `invisible(x)`/`invisible(self)` | **unchanged (exempt)** |

Syntactic limits (document, don't solve — same as `Dots`/`Missing`): type
aliases and `use Invisible as I` renames defeat last-segment detection; the
miss is fail-safe (visible + inner `IntoR` conversion still correct). Document
once in `docs/MINIEXTENDR_ATTRIBUTE.md`'s new marker section and the rustdoc
of `visibility.rs`, not per call site.

## Fixture sweep (the bulk of the PR)

7. `rpkg/src/rust/`: ~47 `&mut self` void methods across fixture files plus
   the 113 unit-returning bare fns currently emitting `invisible(.val)`.
   Policy: fixtures accept the new visible default EXCEPT (a) fixtures whose
   testthat assertions pin invisibility, and (b) a representative set that
   must exercise every explicit spelling. Extend
   `rpkg/src/rust/visibility_tests.rs` with: `-> Invisible<i32>`,
   `-> Invisible<()>`, `-> Visible<()>`, an R6 method `-> Invisible<()>`
   (chainable+silent, the old default made explicit), an R6 method
   `#[miniextendr(invisible)]`, a trait with an `Invisible<()>` method, and
   `pipe_builder_tests.rs` builders re-marked to stay silent-chainable.
8. testthat: update the 7 `withVisible`/`expect_invisible` pins —
   `test-visibility-more.R` (:2, :8), `test-coerce.R` (:108),
   `test-sidecar.R` (:321, :332, :343 — these pin EXEMPT setter sites and
   must keep asserting invisible). Add new pins: unmarked unit fn is now
   visible-NULL; unmarked R6 void method returns visible receiver; each new
   marker fixture is invisible.
9. Macro unit tests + snapshots: `miniextendr_impl/tests.rs:1617-1670` and
   `miniextendr_impl_trait/tests.rs:486-514` assert `invisible(self)`/
   `invisible(x)` — rewrite to the new default + add marker-driven
   assertions. Insta: `snapshot_r6_basic.snap` (`:45` chainable site changes)
   and `snapshot_r6_active_bindings.snap` (`:43`, `:63` are exempt setter
   sites — should NOT change; if they do, the exemption is broken). Re-bless
   via `.snap.new` → `mv`, never blind.
10. trybuild UI tests (`miniextendr-macros/tests/ui/`): new cases — marker in
    argument position, nested markers, `Invisible<T>` + `#[miniextendr(visible)]`
    disagreement. `TRYBUILD=overwrite cargo test -p miniextendr-macros`, review
    the diff (CI is authoritative if rust-src spans differ — #1239).
11. Docs/skills sweep: `lib.rs:129` return-type table row (`()` →
    `invisible(NULL)`) and the attr docs at `lib.rs:527`; 22 `invisible`
    mentions across 8 `docs/*.md` (notably `docs/CLASS_SYSTEMS.md`,
    `docs/MINIEXTENDR_ATTRIBUTE.md`, `docs/TROUBLESHOOTING.md`); 3 skills
    (`.claude/skills/miniextendr-{class-systems,connections,macros}/SKILL.md`).
    Migration note for scaffolded packages goes in
    `docs/MINIEXTENDR_ATTRIBUTE.md` + the PR body: "chainable R6 builders now
    auto-print; add `#[miniextendr(invisible)]` or return `Invisible<()>`".
    Template exposure: `minirextendr/inst/templates/rpkg/lib.rs:65` has a
    commented `&mut self` example — update its comment; run
    `just templates-check`, and `just templates-approve` only if the template
    delta is intended.
12. gc surface: no new SEXP storage anywhere in this change → **no gc_stress
    fixture needed** (state this in the PR body so the reviewer check is
    answered).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
cargo test -p miniextendr-macros 2>&1 > /tmp/1213-macros.log   # Read it
just configure && just rcmdinstall && just force-document && just rcmdinstall
# double install: new marker fixtures are new exports (#CLAUDE.md rule)
just test 2>&1 > /tmp/1213-rust.log
just devtools-test 2>&1 > /tmp/1213-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1213-devtools.log  # devtools::test always exits 0
grep -c 'invisible(' rpkg/R/miniextendr-wrappers.R   # expect ONLY marked + exempt sites
cargo clippy --workspace --all-targets --locked -- -D warnings  # + clippy_all/_s7 per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE` + `man/*.Rd` in sync (pre-commit hook enforces;
`git config core.hooksPath .githooks`).

## Must NOT touch

- `ReturnSelf`/`ReturnOtherClass` wrapping and the `.__MX_WRAP_RETURN_*__`
  write-time resolver (#1219/#1232/#1251 machinery).
- `WrapAs*`/`ConvertTo`/`ConvertFrom`/`#[miniextendr(serialize)]` (demoted /
  separate follow-ups per the maintainer decision).
- Sidecar + active-binding setter emission (exempt; follow-up issue instead).
- `.miniextendr_raise_condition`'s `invisible(NULL)` (condition transport).
- `#1214`'s `Strict`/`Lax` surfaces (separate branch
  `feat/1214-per-param-strictness`; the only shared file is
  `type_inspect.rs`'s peel helper — coordinate merge order, either first).

## Done criteria

- No shape-derived invisibility anywhere: unmarked returns are visible in all
  six class systems, bare fns, and trait impls; markers + attributes (both
  spellings, fn/method/trait) produce `invisible(...)`; exempt sites
  unchanged; behavior table pinned by testthat; suites + three clippy legs +
  fmt green; regenerated artifacts committed; `Fixes #1213` with the
  follow-up issues (accessor markers; 2b; 2c if not already filed) linked.

## Escalation rule

If reality diverges — a generator's void-detection doesn't route through the
sites inventoried here, the exempt setter sites turn out to share an emission
path with method returns, `Invisible<Self>` can't classify cleanly before the
`returns_self` predicates, or the wrappers.R re-grep finds an emission site
this plan missed — **stop, commit nothing further, and report back. Do not
improvise.**
