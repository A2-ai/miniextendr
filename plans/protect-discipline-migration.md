# PROTECT discipline migration

## Base

- Branched off `origin/refactor/api-sys-split` (PR #675's branch).
- All target files now use `crate::sys::` (post-rename), so no rebase conflict expected against #675.

## Goal

Migrate ~80–100 raw `Rf_protect`/`Rf_unprotect` ladder sites in the
`miniextendr-api` crate to the existing RAII guards (`OwnedProtect`,
`ProtectScope`) and high-level builders (`StrVecBuilder`, `RCall`).

Serde is explicitly out of scope (tracked in #673).

## Target files (8 total)

| File | Sites | Strategy |
|---|---|---|
| `miniextendr-api/src/mx_abi.rs` | 1 (line 64–66) | `OwnedProtect` |
| `miniextendr-api/src/factor.rs` | 2 (lines 461–463, 574–576) + new helper | Strategy A: helper returns built-and-unprotected SEXP; helper handles full protect/build/unprotect internally |
| `miniextendr-api/src/match_arg.rs` | 2 (`choices_sexp`, `match_arg_vec_into_sexp`) | `ProtectScope` + per-elem loop preserving `R_BlankString` short-circuit |
| `miniextendr-api/src/named_vector.rs` | 1 helper (`set_names_on_sexp`) + 2 IntoR impls | `ProtectScope` for STRSXP build (add R_BlankString short-circuit); `OwnedProtect::new(sexp)` for IntoR impls |
| `miniextendr-api/src/expression.rs` | `RCall::build` + `RCall::eval` | `ProtectScope` + `OwnedProtect`. Add `RCall::dollar_extract(name)` helper for `$` lookup in lambda contexts |
| `miniextendr-api/src/connection.rs` | `check_connections_runtime` (lines 137–186) + 2 close paths | `OwnedProtect`s with early returns; `RCall::new("close").arg(sexp).eval_base()` for close paths |
| `miniextendr-api/src/txt_progress_bar.rs` | `build_inner` (6 protects), `set_txt_progress_bar_inner` (1), `kill_txt_progress_bar_inner` (4) | `OwnedProtect`s; rewrite `kill` via `RCall::dollar_extract("kill")` and `RCall::eval_base()` |
| `miniextendr-macros/src/dataframe_derive/enum_expansion.rs` | 1 site (~1818-1822) | Emit single call to factor helper |

## Locked decisions

1. **`build_levels_sexp` strategy = A.** Keep the low-level `build_levels_sexp` and `build_factor` public functions returning bare `SEXP` (callers respect protection contract). Introduce a new helper `build_factor_with_levels(indices, level_names) -> SEXP` in `factor.rs` that internally builds the levels STRSXP, protects it across the `build_factor` call, and unprotects on return. All three sites (factor.rs FactorVec impl, factor.rs FactorOptionVec impl, macro codegen) switch to this helper.

2. **`RCall::dollar_extract(name)`** added as new public API on `RCall`. Replaces the hand-rolled `Rf_install("$") + Rf_lang3 + R_tryEvalSilent` dance in `txt_progress_bar.rs` and `connection.rs::check_connections_runtime`.

3. **`R_BlankString` short-circuit preserved everywhere** STRSXP build loops handle user-supplied strings: `choices_sexp`, `match_arg_vec_into_sexp` (kept), `set_names_on_sexp` (added new — was missing before).

4. **No new public API beyond `RCall::dollar_extract` and `factor::build_factor_with_levels`** — both noted in PR body.

5. **Tests prove correctness.** Existing test suite + gctorture pass. New no-arg `gc_stress_*` fixtures added for any new SEXP-storage path that wasn't there before.

## Risks

- **gctorture sensitivity** — the factor codegen change in `enum_expansion.rs` is hot codegen and a regression here would silently corrupt unit-only enum factor returns. Test with `gctorture(TRUE)`.
- **Line-number drift** — none of these sites have moved since the spec was written; the ffi→sys rename is the only intervening change.
- **`_unchecked` validity inside ALTREP callbacks** — explicitly out of scope. `altrep_impl.rs:355-370` and `altrep_impl.rs:1196-1210` left untouched.
- **Macro codegen latch** — after editing `enum_expansion.rs`, must run `just configure && just rcmdinstall && just force-document` to regenerate wrappers. Pre-commit hook (`.githooks/pre-commit`) enforces wrappers.R/NAMESPACE pairing.
- **Pre-commit hook silent fail (#680)** — if `git commit` exit 1 with no stderr, run `bash -x .githooks/pre-commit` to diagnose.

## Implementation order

1. `mx_abi.rs` — 1 site, validates pattern.
2. `factor.rs` (new helper + 2 callers) + macro codegen edit — atomic contract.
3. `match_arg.rs` — 2 sites.
4. `named_vector.rs` — helper + 2 IntoR impls.
5. `expression.rs` — `RCall::build`/`eval` + add `dollar_extract`.
6. `connection.rs` — `check_connections_runtime` first, then close paths.
7. `txt_progress_bar.rs` — depends on `dollar_extract` from step 5.

## Verification

To `/tmp/<name>.log`:
- `just check`
- `just clippy -D warnings`
- `just test`
- CI clippy_all feature set parity (read from `.github/workflows/ci.yml`)
- `just configure && just rcmdinstall && just force-document`
- `just devtools-test`
- gctorture pass via no-arg fixtures (drive factor construction, STRSXP build paths)
- `TRYBUILD=overwrite cargo test -p miniextendr-macros`, review diff, re-run

## Final self-check

- `rg "Rf_protect\(" miniextendr-api/src/{connection,expression,match_arg,named_vector,factor,txt_progress_bar,mx_abi}.rs miniextendr-macros/src/dataframe_derive/enum_expansion.rs` — zero hits outside doc comments.
- Same for `Rf_unprotect\(`.
