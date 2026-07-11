# Plan: #1251 — cross-class return wrapping for `Option<Class>` / `Result<Class, E>`

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1251-container-cross-class-returns`.

Related: #1231/PR #1232 (the bare-type fix this extends — fixtures in
`rpkg/src/rust/pipe_builder_tests.rs:239-378`), #1219 (`-> Self` container
handling this mirrors), base plan
`plans/2026-07-08-golife-hiccup9-cross-class-return-wrapping.md` (mechanism
background — already merged, read-only context).

## Scope decision (baked in — no judgment left)

- **In scope**: `Option<Class>` and `Result<Class, E>` returns, where `Class`
  is a bare capitalized ident. The C wrapper ALREADY unwraps these and raises
  on `None`/`Err` (verified: `Option<non-Self>` →
  `ReturnHandling::OptionIntoRUnwrap`, raises `NONE_ERR` on None,
  `c_wrapper_builder.rs:1727` selection / `:800,1040` emission;
  `Result<non-Self,E>` → `ResultIntoR`). So the successful `.val` is a bare
  pointer — the R-side tail is IDENTICAL to the bare-class case. **No C-side
  changes.**
- **Out of scope**: `Vec<Class>`. Verified: `Vec<T>` has NO `IntoR` impl for
  ExternalPtr-derived element types (`IntoRVecElement` covers newtype/MatchArg
  only — `newtype.rs:83-97`; the only pointer-vector impl is
  `Vec<ExternalPtr<T>>`, `into_r/large_integers.rs:404`), so `-> Vec<Class>`
  does not even compile today; an R-side lapply resolver would be dead code.
  **Instead**: file a follow-up issue (`gh issue create`) titled
  "cross-class return wrapping for Vec<Class> / Vec<ExternalPtr<Class>>
  (list lapply resolver)" — body: the lapply-marker sketch (distinct
  `.__MX_WRAP_LIST_RETURN_` prefix; NOTE the scalar resolver rewrites
  unrecognized `.__MX_WRAP_RETURN_*` names to the bare expr, so a shared
  prefix would destroy list markers), the missing `IntoR` prerequisite, and a
  reference to #1251 + this plan. Link it in the PR body.

## Work items (flat order)

1. **Extend `ParsedMethod::returns_other_class()`**
   (`miniextendr-macros/src/miniextendr_impl.rs:2218-2242`). Today it returns
   `None` for any last segment with path arguments (`:2226-2228`). New logic:
   - Bare capitalized path (current behavior): unchanged.
   - Last segment `Option` or `Result` with `AngleBracketed` args: take the
     FIRST type argument (reuse the crate's `first_type_argument` /
     `second_type_argument` helpers — see `c_wrapper_builder.rs:1700,1745`);
     if it is a bare `syn::Type::Path` with NO path arguments whose ident
     passes the existing filter (not `Self`, not
     `is_builtin_return_type_name`, not `is_known_return_container_name`,
     first char ascii-uppercase), return that ident. Special case: keep
     `Result<T, ()>` (unit error) EXCLUDED — it maps to
     `ResultNullOnErr` (`c_wrapper_builder.rs:1743-1748`), where `.val` can
     be `NULL`; wrapping NULL into a constructor would break. Check the
     second type arg and return `None` for unit-error Results.
   - Anything else (incl. `Vec<...>`, nested containers like
     `Option<Vec<T>>`): `None`, as today.
   Update the method's rustdoc. Note the `for_method` ordering guarantee:
   `returns_result_self()`/`returns_option_self()` are checked BEFORE
   `returns_other_class()` (`method_return_builder.rs:118-131`), so
   `Option<Self>`/`Result<Self,_>` still take `ReturnSelf`; additionally the
   inner-`Self` filter above keeps them `None` here.
2. **No `ReturnStrategy` changes.** Scalar containers reuse the existing
   `ReturnOtherClass` variant, marker emission
   (`method_return_builder.rs:244-245` → `.__MX_WRAP_RETURN_<name>__(.val)`),
   and the write-time resolver in `registry.rs` (passes at `:1332-1343` and
   `:1933-1940`) — all untouched.
3. **Macro unit tests** (`miniextendr-macros/src/miniextendr_impl/tests.rs`
   — extend the existing block at `:1417-1426`):
   `-> Option<Board>` → `Some(Board)`; `-> Result<Board, String>` →
   `Some(Board)`; `-> Result<Board, ()>` → `None`; `-> Option<Self>` →
   `None`; `-> Option<i32>` → `None`; `-> Vec<Board>` → `None`;
   `-> Option<Vec<Board>>` → `None`.
4. **rpkg fixtures** — extend the cross-class region of
   `rpkg/src/rust/pipe_builder_tests.rs` (`:239-378`), matching its style:
   on `R6CrossPlan` (`impl` at `:286`) add
   `try_build(&self, width: i32, height: i32, fail: bool) -> Option<R6CrossBoard>`
   (None when `fail`) and
   `checked_build(&self, width: i32, height: i32, fail: bool) -> Result<R6CrossBoard, String>`
   (Err message when `fail`). On `S7CrossPlan` (`:347`) add one mixed-system
   container return, e.g.
   `s7_try_build_r6(&self, ...) -> Option<R6CrossBoard>` (proves the
   target-keyed resolver on the container path, mirroring commit 7563d60f's
   mixed-system rationale).
5. **testthat** — extend `rpkg/tests/testthat/test-pipe-builder.R` (match its
   existing cross-class assertions): the Some/Ok arms return a CLASSED object
   (`expect_s3_class(b, "R6")` / class name pin, then a method call on the
   result works — the #1231 "not subsettable" repro inverted); the None arm
   raises a `rust_error` (NONE_ERR path) — `expect_error(..., class = "rust_error")`;
   the Err arm raises with the fixture's message text. From-outside-package
   calls, per the suite's convention.
6. **Regen + snapshots**: full loop (below). Insta snapshots in
   `miniextendr-macros` that contain generated wrapper bodies will gain the
   marker for any fixture with container cross-class returns (likely only new
   tests). Rebaseline `.snap.new` after diff review. trybuild: no error-path
   changes expected.
7. **File the follow-up issue** from the scope decision, reference it + #1251
   in the PR body (`Fixes #1251` is CORRECT here — the issue's own sketch
   scopes Vec as optional follow-up; the new issue carries it).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2: new exported fixture methods
cargo test -p miniextendr-macros 2>&1 > /tmp/1251-macros.log   # Read it
just test 2>&1 > /tmp/1251-rust-test.log
just devtools-test 2>&1 > /tmp/1251-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1251-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE` + `man/*.Rd` with the Rust changes (pre-commit
hook enforces pairing). Never `TRYBUILD=overwrite` locally (#1239).

## Must NOT touch

- `c_wrapper_builder.rs` — zero C-side changes (the whole point of the design).
- The `ReturnStrategy` enum, marker grammar, or `registry.rs` resolver.
- `#1219`'s `-> Self` paths (`returns_result_self`/`returns_option_self`
  predicates and `ReturnSelf` tails).
- No `Vec` recognition code, even dormant.
- Generated files (`wrappers.R`, `wasm_registry.rs`).

## Done criteria

- `plan$try_build(...)` returns a usable classed object on Some; raises
  `rust_error` on None; `checked_build` mirrors for Ok/Err; mixed-system
  container return wraps with the TARGET's constructor.
- Macro unit tests from item 3 all pass; suites + three clippy legs green;
  NAMESPACE/man committed in sync; follow-up Vec issue filed and linked.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, the C wrapper's
None/Err behavior differs from the verified description, a fixture fails to
compile for a reason not covered here — **stop, commit nothing further, and
report back. Do not improvise.**
