# MXL111: redundant `s4_` prefix on `#[miniextendr(s4)]` method names

Tracks: [#387](https://github.com/A2-ai/miniextendr/issues/387). Found while
writing S4 condition fixtures in PR #385: a method named `s4_raise_error` on
an `#[miniextendr(s4)]` impl produced an R generic named `s4_s4_raise_error`,
because the S4 codegen unconditionally prepends `s4_` to method names.

This is a silent footgun — Rust compiles fine, the package installs, the only
symptom is users calling `s4_raise_error(obj, ...)` and getting `could not
find function`. If their tests happen to call the double-prefixed name (or
no tests cover the method), the bug ships.

## Goal

Detect this at `cargo build` time and tell the user to drop the prefix.

## Scope

Single PR adding rule `MXL111` plus the `crate_index` extension it needs (the
same extension `MXL110` needs — coordinate or land in either order).

## Behaviour to detect

```rust
#[miniextendr(s4)]
impl Foo {
    pub fn s4_method(&self) {}  // ← FIRES — generates s4_s4_method
    pub fn method(&self) {}      // ← OK — generates s4_method
    pub fn new(...) -> Self {}  // ← OK — constructors aren't prefixed
}
```

The rule must NOT fire on:
- `s4_*` methods inside `#[miniextendr(r6)]` / `(s3)` / `(s7)` / `(env)` impls — only S4 auto-prepends.
- Standalone `pub fn s4_helper(...)` outside any impl block — the `s4_` is then a regular naming choice.
- Constructors (`pub fn new(...) -> Self`) — these don't get the prefix even on S4. Verify against the codegen.

## Implementation

### 1. Extend `crate_index.rs`

Reuse the `fn_param_names` extension from MXL110 if it landed first; otherwise
add a separate field `pub impl_methods: HashMap<String, Vec<(String, usize, ClassSystem)>>`
keyed by impl-type-name, where the value is `(method_name, line, class_system)`.

`class_system` should be `String` (matches `inherent_impl_class_systems`).

### 2. New rule module

`miniextendr-lint/src/rules/s4_method_prefix.rs`:

```rust
use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (impl_type, methods) in &data.impl_methods {
            for (method_name, line, class_system) in methods {
                if class_system != "s4" {
                    continue;
                }
                if method_name == "new" {
                    continue;  // constructors aren't auto-prefixed
                }
                if let Some(rest) = method_name.strip_prefix("s4_") {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL111,
                            path,
                            *line,
                            format!(
                                "method `{impl_type}::{method_name}` will produce R generic \
                                 `s4_{method_name}` (s4 codegen auto-prepends `s4_`); \
                                 rename to `{rest}` to avoid the double prefix",
                            ),
                        )
                        .with_help(
                            "the macro prepends `s4_` to every S4 method name; \
                             keep the Rust method name unprefixed",
                        ),
                    );
                }
            }
        }
    }
}
```

### 3. Register

- `lint_code.rs`: add `MXL111`. Severity = `Warning` initially (heuristic;
  could conceivably be intentional). After one release with the warning,
  promote to `Error` if no false positives surface.
- `rules.rs`: `pub mod s4_method_prefix;` + invoke.

### 4. Test the rule

Mirror the existing rule-test pattern. Three fixtures:

**Positive (should fire):**
```rust
#[miniextendr(s4)]
impl Foo {
    pub fn s4_compute(&self) -> i32 { 0 }
}
```

**Negative — non-S4 class system:**
```rust
#[miniextendr(r6)]
impl Bar {
    pub fn s4_compute(&self) -> i32 { 0 }  // OK — R6 doesn't auto-prefix
}
```

**Negative — standalone fn:**
```rust
#[miniextendr]
pub fn s4_helper() {}  // OK — not in an impl
```

**Negative — constructor:**
```rust
#[miniextendr(s4)]
impl Foo {
    pub fn new() -> Self { Foo {} }
}
```

### 5. Documentation

Add to `CLAUDE.md` `miniextendr-lint` section:
```
- **MXL111**: `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends, you'll get `s4_s4_*`
```

Also worth a one-paragraph note in whichever doc covers the s4 class system,
documenting the auto-prefix behaviour explicitly. Search `docs/` for s4
references.

## Verification

- `just lint` — MXL111 fires on the positive fixture only.
- `just clippy` clean.
- `just test` — including any miniextendr-lint-specific tests.
- Reproduce CI clippy_default + clippy_all.
- Run `just rcmdinstall && just devtools-document` against the rpkg fixture
  with the offending pattern reverted (just to confirm: untouched fixture
  produces `s4_s4_*` codegen). The fixture committed in PR #385 already does
  the right thing — don't reintroduce the bug.

## Out of scope (file separately if needed)

- Coverage for the same bug shape on other class systems if any of them
  also auto-prepend (verify in `c_wrapper_builder.rs`/r_class_formatter): R6,
  S3, S7, Env all forward the verbatim method name as far as I (Mossa) can
  tell. Confirm during implementation; add to the rule's negative fixtures
  if any other system has its own auto-prefix.
- Doc improvement to make the auto-prefix behaviour discoverable (separate
  small docs PR).

## PR shape

- Title: `feat(lint): MXL111 — flag s4_-prefixed methods on #[miniextendr(s4)] impls`
- Closes #387
- Single commit (lint rule + test + doc) is fine.

## Coordination with #386

Both lints need `FileData` extensions to track function/method information
that isn't currently indexed. If both PRs are open simultaneously, the second
to merge should rebase on the first. The crate-index extension is the only
overlap; the rule modules are independent.
