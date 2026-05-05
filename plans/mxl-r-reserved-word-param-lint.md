# MXL110: R reserved words as `#[miniextendr]` parameter names

Tracks: [#386](https://github.com/A2-ai/miniextendr/issues/386). Found while
writing comprehensive condition tests in PR #385: a `pub fn foo(repeat: i32)`
fixture compiled cleanly via `cargo build` but produced
`condition_error_long_message <- function(repeat` in the generated wrappers
file, which `devtools::document()` rejected with a syntax error several minutes
into the build.

The proc macro forwards parameter names verbatim into the R wrapper. R has a
fixed list of reserved words; using any of them as a Rust parameter name is
guaranteed to break codegen with no compile-time signal.

## Goal

Detect this at `cargo build` time (when `miniextendr-lint`'s `build.rs` runs)
so the user gets a clear error pointing at the offending parameter, not a
deparse-context message ten layers deep in `roxygen2::update_collate()`.

## Scope

Single PR adding rule `MXL110` plus the `crate_index` extension it needs.
No proc-macro change, no naming-convention change — purely a lint.

## R reserved words (the rule's blocklist)

Source: [R Language Definition §10.3.1, `?Reserved`](https://stat.ethz.ch/R-manual/R-devel/library/base/html/Reserved.html).

```
if  else  repeat  while  function  for  in  next  break
TRUE  FALSE  NULL  Inf  NaN  NA  NA_integer_  NA_real_  NA_complex_  NA_character_
```

Plus `...` and `..1` / `..2` / ... (dotted-numeric). These are rare as Rust
idents but worth covering since `..1` is a valid Rust ident only in unusual
edge cases (it's actually not a valid ident; safe to ignore).

Also worth flagging the *quasi-reserved* identifiers that aren't strictly
reserved but breaks downstream R tools when used as bare formal names:

- `T`, `F` — work as identifiers but shadow `TRUE`/`FALSE`. Skip these — too
  many false positives, and they don't actually break codegen.
- `c`, `t`, `q` — base R functions. Same reasoning, skip.

Stick to the strictly-reserved list. Quasi-reserved is a separate concern.

## Implementation

### 1. Extend `crate_index.rs`

Add `pub fn_param_names: HashMap<String, Vec<(String, usize)>>` to `FileData`
where the value is `(param_name, line)`. Populated by walking
`syn::ItemFn::sig.inputs` and `syn::ImplItemFn::sig.inputs`.

For impl methods, key by `format!("{}::{}", impl_type, method_name)` so the
diagnostic can name them precisely.

Two parsing locations:
- `Item::Fn` with `#[miniextendr]` — direct function path
- `ImplItem::Fn` inside `Item::Impl` whose impl block has `#[miniextendr(...)]`
  — only flag if the impl is registered (skip plain inherent impls)

The `#[miniextendr]`-detection helpers already exist in `helpers.rs`
(`has_miniextendr_attr`).

### 2. New rule module

`miniextendr-lint/src/rules/r_reserved_params.rs`:

```rust
use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

const R_RESERVED: &[&str] = &[
    "if", "else", "repeat", "while", "function", "for", "in", "next", "break",
    "TRUE", "FALSE", "NULL", "Inf", "NaN",
    "NA", "NA_integer_", "NA_real_", "NA_complex_", "NA_character_",
];

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, params) in &data.fn_param_names {
            for (param, line) in params {
                if R_RESERVED.contains(&param.as_str()) {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL110,
                            path,
                            *line,
                            format!(
                                "parameter `{param}` of `{fn_name}` is an R reserved word; \
                                 the generated R wrapper will be syntactically invalid",
                            ),
                        )
                        .with_help(
                            "rename the parameter (e.g., `n_chunks` instead of `repeat`)",
                        ),
                    );
                }
            }
        }
    }
}
```

### 3. Register the rule

- `miniextendr-lint/src/lint_code.rs`: add `MXL110` variant; severity = `Error`
  (broken codegen is unrecoverable).
- `miniextendr-lint/src/rules.rs`: `pub mod r_reserved_params;` + invoke in
  `run_all_rules`.

### 4. Test the rule

Existing rules' tests live near them (check the codebase — likely
`miniextendr-lint/tests/` or `#[cfg(test)]` modules). Mirror the existing
pattern. Minimal fixture:

```rust
#[miniextendr]
pub fn bad(repeat: i32) {}
```

Asserts: rule fires once with `MXL110`.

Negative fixture:
```rust
#[miniextendr]
pub fn good(n: i32) {}
```

Asserts: no `MXL110` diagnostic.

Method case:
```rust
#[miniextendr(r6)]
impl Foo {
    pub fn m(&self, while_: i32) {}  // OK — Rust trailing underscore avoids the keyword issue
    pub fn n(&self, repeat: i32) {}  // FIRES
}
```

### 5. Documentation

Add to `CLAUDE.md`'s `miniextendr-lint` section:
```
- **MXL110**: parameter name is an R reserved word → codegen will break
```

## Verification

- `just lint` shows `MXL110` firing on the negative test fixture only.
- `just clippy` clean.
- `just test` — including any `miniextendr-lint`-specific tests.
- Reproduce CI clippy_default + clippy_all per CLAUDE.md.

## Out of scope (file separately if needed)

- Quasi-reserved (`T`, `F`, `c`, `t`, `q`, `body`, etc.) — false-positive risk.
- Reserved words in field names of `#[derive(ExternalPtr)]` `#[r_data]` slots
  (sidecar accessors). Same bug shape but separate code path. Worth a
  follow-up issue if it bites.
- Same check at proc-macro emit time (would catch it earlier than `cargo build`
  but adds complexity and only saves one cargo invocation).

## PR shape

- Title: `feat(lint): MXL110 — flag R reserved words as parameter names`
- Closes #386
- Single commit (lint rule + test + doc) is fine.
