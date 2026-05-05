# Comprehensive condition system tests + tail of #344/#349 cleanup

PR #344 landed `error!` / `warning!` / `message!` / `condition!` macros with
`rust_*` class layering and a tagged-SEXP transport. PR #349 extended layering
to ALTREP `RUnwind` callbacks for the `Error` variant. PR #382 hoisted the
per-wrapper switch into `.miniextendr_raise_condition`. This plan closes the
remaining test surface gaps and the one known correctness gap (#366), and
renames the transport class now that it carries non-error conditions too.

## Scope

Single sequenced PR off `refactor/raise-condition-helper` (the existing branch).
No worktree fan-out — every change touches `miniextendr-api` and triggers full
rebuild + R reinstall, which kills the parallelism win.

Three commits:

1. Test fixtures + R test file (failures expected from #366 + the rename audit)
2. #366 fix (route ALTREP non-error degradation through `raise_rust_condition_via_stop`)
3. Rename `rust_error_value` → `rust_condition_value` (class) and
   `__rust_error__` → `__rust_condition__` (attr)

## Coverage gap matrix

Existing `test-conditions.R` covers free-function path. `test-altrep-conditions.R`
covers ALTREP-error. Everything else is uncovered.

| Surface | error! | warning! | message! | condition! | custom class | conditionCall | e$kind / e$class |
|---|---|---|---|---|---|---|---|
| free fn | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| R6 method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| S3 method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| S4 method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| S7 method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Env method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Vctrs method | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Sidecar accessor (panic only) | n/a | n/a | n/a | n/a | n/a | ❌ | ❌ |
| ALTREP RUnwind elt | ✅ | ⚠️ #366 | ⚠️ #366 | ⚠️ #366 | ✅ | n/a | n/a |
| Trait-ABI shim (cross-pkg) | partial | ❌ | ❌ | ❌ | partial | ❌ | ❌ |

**Edge axes (cross-cutting, will fold into a separate `# region: edge cases`):**

- empty string message
- multibyte / unicode message (e.g. `"日本語 🦀"`)
- embedded `\n` in message
- very long message (≥ 4 KB) — exercises `CString` path + STRSXP encoding
- Result<T, RErrorAdapter<E>> with chained source — already covered for plain
  Result<T,E>; add custom-class variant
- panic with non-`String`/non-`&str`/non-`RCondition` payload
  (e.g. `panic_any(42_i32)`) — falls through to generic panic→string fallback
- nested handler that re-throws — class identity preserved across re-raise
- `withRestarts` + `invokeRestart` for `condition!` — user-defined restart
- `e$kind` matches the macro that raised it
- `e$class` exposes the user-supplied class slot (or `NULL` if none)

**Surface-specific `conditionCall` notes:**

- R6 methods, S7 getter/setter/validator, S7 finalizer/deep_clone — these go
  through lambda contexts where `match.call()` would capture an internal
  dispatch frame; #344 deliberately uses `.call = NULL` and the helper falls
  back to `sys.call()`. Tests must allow either `NULL` or the caller's frame
  here, not assert a specific call form.
- Standalone fns + R6/S3/S4/S7/Env/Vctrs **non-lambda** methods — these set
  `.call = match.call()`, so `conditionCall(e)` should equal the user's call
  expression (`fn(x)` / `obj$method(x)`).
- Sidecar accessors — C wrappers don't carry `__miniextendr_call`, so
  `.val$call` is `NULL` and the helper falls back to `sys.call()` of the R
  wrapper itself (`SidecarR6_get_value(x)`). Test asserts call inherits the
  expected accessor name.

## Fixture design

### `rpkg/src/rust/condition_class_system_tests.rs` (new)

One small struct per class system, each with four methods that raise each kind:

```rust
#[derive(ExternalPtr)]
pub struct R6ConditionRaiser { id: i32 }

#[miniextendr(r6)]
impl R6ConditionRaiser {
    pub fn new(id: i32) -> Self { ... }
    pub fn raise_error(&self, msg: &str) { error!("{msg}"); }
    pub fn raise_error_classed(&self, class: &str, msg: &str) { panic_any(RCondition::Error{...}); }
    pub fn raise_warning(&self, msg: &str) { warning!("{msg}"); }
    pub fn raise_warning_classed(&self, class: &str, msg: &str) { panic_any(...); }
    pub fn raise_message(&self, msg: &str) { message!("{msg}"); }
    pub fn raise_condition(&self, msg: &str) { condition!("{msg}"); }
    pub fn raise_condition_classed(&self, class: &str, msg: &str) { panic_any(...); }
}
```

Repeat for S3 / S4 / S7 / Env / Vctrs. Methods take `&self` — keeps the surface
narrow; we're testing the wrapper, not the method body.

### Sidecar accessor: extend an existing fixture

`rpkg/src/rust/rdata_sidecar_tests.rs` already has `SidecarR6` with `value`
and `label` fields. Add a `panicking_field` (custom Setter that panics) — or
construct one with bad value via constructor and access. Single test asserts
`tryCatch(get_panicking(x), rust_error = h)` matches.

### ALTREP non-error degradation

Extend `altrep_condition_tests.rs`:

```rust
#[derive(AltrepInteger)]
#[altrep(class = "WarnAltrep", manual)]
pub struct WarnAltrepData { len: usize, message: String }

impl AltIntegerData for WarnAltrepData {
    fn elt(&self, _i: usize) -> i32 { warning!("{}", self.message); 0 }
}
// + altrep_warn_on_elt, altrep_message_on_elt, altrep_condition_on_elt fns
```

### Tests file: `rpkg/tests/testthat/test-conditions-comprehensive.R`

Sections:

1. `# region: free fn — conditionCall + e$kind + e$class fields`
2. `# region: R6 methods — error / warning / message / condition × bare / classed`
3. `# region: S3 methods` (same matrix)
4. `# region: S4 methods`
5. `# region: S7 methods`
6. `# region: Env methods`
7. `# region: Vctrs methods`
8. `# region: sidecar accessor panic`
9. `# region: ALTREP non-error degradation` (xfail before #366 fix; passing after)
10. `# region: edge cases — empty / unicode / long / non-RCondition panic / nested re-raise`

Skip-on-CRAN flag: none — all are pure synchronous unit tests, fast.

## #366 fix

`miniextendr-api/src/unwind_protect.rs:415-426` — replace the
`panic_payload_to_r_error` call with `raise_rust_condition_via_stop`:

```rust
crate::condition::RCondition::Warning { .. }
| crate::condition::RCondition::Message { .. }
| crate::condition::RCondition::Condition { .. } => {
    let msg = "warning!/message!/condition! from ALTREP callback context \
               cannot be raised as non-fatal signals; use error!() instead. \
               This context has no R wrapper to handle signal restart.";
    crate::panic_telemetry::fire(msg, source);
    unsafe { raise_rust_condition_via_stop(msg, None, call) }
}
```

The diagnostic message stays the same; only the class layering changes
(plain `simpleError` → `c("rust_error", "simpleError", "error", "condition")`).
This makes the degradation consistent with the generic-panic path 10 lines
below it.

## Rename `rust_error_value` → `rust_condition_value`

The class string on the tagged transport SEXP, plus the `__rust_error__`
marker attribute, are purely internal — never observed by user `tryCatch`
because the wrapper translates them into `rust_error` / `rust_warning` /
`rust_message` / `rust_condition` before signalling. Now that the same
transport carries warning/message/condition payloads, the original "error_value"
naming is misleading.

Sites:

- `miniextendr-api/src/cached_class.rs:137,175` — the cached SEXP and symbol
- `miniextendr-api/src/condition.rs:301,313` — `from_tagged_sexp` reader
- `miniextendr-api/src/error_value.rs:16,17,26,27` — doc comments
- `miniextendr-macros/src/method_return_builder.rs:26,41` — codegen string
- `miniextendr-api/src/registry.rs:533` — comment in helper preamble
- `miniextendr-macros/src/miniextendr_impl_trait.rs:387` — doc comment
- The function name `make_rust_error_value` itself: leave alone for now —
  issue #363 already tracks consolidating into `make_rust_condition_value`,
  and that's a bigger refactor than the rename here.

Changes:

- `cached_class.rs`: `rust_error_class_sexp` → `rust_condition_class_sexp`
  emitting `c"rust_condition_value"`; `rust_error_attr_symbol` →
  `rust_condition_attr_symbol` emitting `c"__rust_condition__"`.
- `condition.rs::from_tagged_sexp`: read class `c"rust_condition_value"` and
  attr `c"__rust_condition__"`. **Backward-compat for in-flight `.so`'s built
  against the old class is unnecessary** — internal-only rename, no external
  ABI consumers.
- `method_return_builder.rs`: emit
  `inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))`
  in both the inline and standalone branches.
- Regenerate `rpkg/R/miniextendr-wrappers.R` via `just devtools-document`.
- `R/condition_helper.R` (if extracted) or doc strings: update references.

## Verification

- `just configure && just rcmdinstall && just devtools-document` — must succeed
- `just devtools-test` — all current tests + new comprehensive suite green
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo clippy ... --features <full feature set per CLAUDE.md>` for `clippy_all`
- Snapshot tests in `miniextendr-macros/tests/snapshots/` will require
  `cargo insta review` for the rename — accept the diffs.
- `just lint` for MXL static analysis.

## Out of scope (file as separate issues if not already tracked)

- #346 — structured `data = list(...)` payloads (large feature, deferred)
- #361 — magic kind strings → constants (separate refactor)
- #363 — unify `make_rust_error_value` into `make_rust_condition_value`
  (~15 sites, separate PR)
- #347 — discussion: remove non-`error_in_r` path entirely (no decision yet)
- #348 — sidecar accessors carrying `__miniextendr_call` (deferred)
