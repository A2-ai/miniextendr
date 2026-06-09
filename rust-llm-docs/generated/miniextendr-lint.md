# miniextendr_lint v0.1.0

miniextendr-lint: internal build-time lint helpers for the workspace.

This crate scans Rust sources for miniextendr macro usage and emits
cargo warnings with actionable diagnostics. It is intended for local
development and CI, not as a public API.

## Usage in build.rs

```ignore
fn main() {
    miniextendr_lint::build_script();
}
```

## Configuration
- Controlled by the `MINIEXTENDR_LINT` env var (enabled by default).
- Set it to `0`, `false`, `no`, or `off` to disable.

## Lint Codes

Each diagnostic carries a stable `MXL###` code. See [`LintCode`] for the full catalog.

---

## Structs

### `AttributedTraitImpl`

**Fields:**

- `type_name`: `String`
- `trait_name`: `String`
- `class_system`: `Option<String>`
- `line`: `usize`

### `CrateIndex`

Shared parsed state for all lint rules.

**Fields:**

- `files`: `Vec<std::path::PathBuf>`
  - All scanned Rust source files.
- `file_data`: `std::collections::HashMap<std::path::PathBuf, FileData>`
  - Per-file parsed data.

**Methods:**

#### `build`

```rust
build(root: &Path) -> Result<Self, String>
```

Build the index from a crate root directory.

### `Diagnostic`

A single lint diagnostic with structured metadata.

**Fields:**

- `code`: `crate::lint_code::LintCode`
  - Stable rule code (e.g. `MXL101`).
- `severity`: `Severity`
  - Severity level.
- `path`: `std::path::PathBuf`
  - Source file path.
- `line`: `usize`
  - 1-based line number (0 if unknown).
- `message`: `String`
  - Primary diagnostic message.
- `help`: `Option<String>`
  - Optional fix guidance.

**Methods:**

#### `new`

```rust
new(code: LintCode, path: impl Into<PathBuf>, line: usize, message: String) -> Self
```

Create a new diagnostic with the rule's default severity.

#### `to_legacy_string`

```rust
to_legacy_string(self: &Self) -> String
```

Format as a legacy error string (for backward-compatible `LintReport::errors`).

#### `with_help`

```rust
with_help(self: Self, help: impl Into<String>) -> Self
```

Attach a help message.

### `FileData`

**Fields:**

- `miniextendr_items`: `Vec<LintItem>`
- `types_with_external_ptr`: `std::collections::HashSet<String>`
- `types_with_typed_external`: `std::collections::HashSet<String>`
- `inherent_impl_class_systems`: `std::collections::HashMap<String, (String, usize)>`
- `attributed_trait_impls`: `Vec<AttributedTraitImpl>`
- `impl_blocks_per_type`: `std::collections::HashMap<String, Vec<(Option<String>, usize)>>`
- `fn_visibility`: `std::collections::HashMap<String, bool>`
- `declared_child_mods`: `Vec<String>`
  - Simple `mod child;` declarations (by ident name).
- `path_redirected_mods`: `Vec<(String, String)>`
  - `#[path = "file.rs"] mod name;` declarations: (mod_name, file_path_str).
- `mod_decl_cfgs`: `std::collections::HashMap<String, Vec<String>>`
  - cfg attrs on `mod child;` declarations: mod_name -> cfg strings.
- `export_control`: `std::collections::HashMap<String, (bool, bool, usize)>`
  - (has_internal, has_noexport, line)
- `impl_methods`: `std::collections::HashMap<String, Vec<ImplMethodEntry>>`
  - Methods per inherent impl type: type_name → `Vec<ImplMethodEntry>`.
- `fn_doc_tags`: `std::collections::HashMap<String, Vec<String>>`
  - Known roxygen tags: "@noRd", "@export", "@keywords internal"
- `rf_error_calls`: `Vec<(String, usize)>`
  - Lines containing direct Rf_error/Rf_errorcall calls: (function_name, line_number).
- `ffi_unchecked_calls`: `Vec<(String, usize)>`
  - Lines containing `ffi::*_unchecked()` calls: (function_name, line_number).
- `fn_param_names`: `std::collections::HashMap<String, Vec<(String, usize)>>`
  - Maps fn/method name → list of (param_name, line) for params that are R reserved words.
- `lifetime_param_items`: `Vec<(String, usize)>`
  - `#[miniextendr]` functions or impl blocks that carry explicit lifetime params.
- `interleaved_doc_attrs`: `Vec<(String, usize)>`
  - `#[miniextendr]` items where a non-doc attribute interrupts a doc-comment stream.

### `ImplMethodEntry`

Per-method data collected during the crate-index pass for impl-method lint rules.

**Fields:**

- `method_name`: `String`
- `line`: `usize`
- `class_system`: `String`
- `return_type_str`: `String`
  - Stringified return type tokens (empty string = `()` / no explicit return).
- `receiver_kind`: `MethodReceiverKind`
  - Receiver kind detected from the method signature.
- `has_constructor_attr`: `bool`
  - True when the method carries `#[miniextendr(constructor)]`.

### `LintItem`

**Fields:**

- `kind`: `LintKind`
- `name`: `String`
- `label`: `Option<String>`
- `line`: `usize`

**Methods:**

#### `new`

```rust
new(kind: LintKind, name: String, line: usize) -> Self
```

#### `with_label`

```rust
with_label(kind: LintKind, name: String, label: Option<String>, line: usize) -> Self
```

### `LintReport`

Result of running the lint over a crate source tree.

**Fields:**

- `files`: `Vec<std::path::PathBuf>`
  - Rust source files that were scanned.
- `diagnostics`: `Vec<Diagnostic>`
  - Structured diagnostics from all rules.
- `errors`: `Vec<String>`
  - Legacy string errors (derived from diagnostics, for backward compatibility).

### `MiniextendrImplAttrs`

Parsed miniextendr attribute information for an impl block.

**Fields:**

- `class_system`: `Option<String>`
  - Class system (e.g., "r6", "s3", "s4", "s7", or empty for env)
- `label`: `Option<String>`
  - Optional label for distinguishing multiple impl blocks of the same type
- `internal`: `bool`
  - Has `internal` flag
- `noexport`: `bool`
  - Has `noexport` flag
- `strict`: `bool`
  - Has `strict` flag

---

## Enums

### `LintCode`

Stable lint rule identifier.

Display format is `MXL###`, derived directly from the variant name.

**Variants:**

- `MXL008`
  - Trait impl class system incompatible with inherent impl class system.
- `MXL009`
  - Multiple impl blocks for one type without labels.
- `MXL010`
  - Duplicate labels on impl blocks for one type.
- `MXL106`
  - Registered top-level function is not `pub`.
- `MXL110`
  - Parameter name is an R reserved word; codegen will produce invalid R syntax.
- `MXL111`
  - `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends `s4_`.
- `MXL112`
  - Explicit lifetime parameter on `#[miniextendr]` fn or impl — use owned types instead.
- `MXL120`
  - vctrs constructor returns `Self` / named type, or impl has an instance-method receiver.
- `MXL203`
  - `internal` + `noexport` redundancy.
- `MXL300`
  - Direct `Rf_error`/`Rf_errorcall` call in user code.
- `MXL301`
  - `_unchecked` FFI call outside guard context.
- `MXL302`
  - Non-doc attribute interrupts a doc-comment stream on a `#[miniextendr]` item.

**Methods:**

#### `default_severity`

```rust
default_severity(self: Self) -> super::diagnostic::Severity
```

Default severity for this rule.

### `LintKind`

**Variants:**

- `Function`
- `Impl`
- `Struct`
- `TraitImpl`
- `Vctrs`

### `MethodReceiverKind`

Receiver kind for an impl method, mirroring `ReceiverKind` in `miniextendr-macros`.

Mirror: `miniextendr-macros/src/miniextendr_impl.rs` — `ReceiverKind`.
Keep both in sync: if the macro relaxes one receiver kind, update this enum too.

**Variants:**

- `None`
  - No self — static / associated function.
- `Ref`
  - `&self`
- `RefMut`
  - `&mut self`
- `Value`
  - `self` (consuming)
- `ExternalPtrRef`
  - `self: &ExternalPtr<Self>`
- `ExternalPtrRefMut`
  - `self: &mut ExternalPtr<Self>`
- `ExternalPtrValue`
  - `self: ExternalPtr<Self>`

**Methods:**

#### `is_instance`

```rust
is_instance(self: Self) -> bool
```

Returns true if this is an instance receiver (any form of `self`).

Mirrors `ReceiverKind::is_instance` in `miniextendr-macros/src/miniextendr_impl.rs`.
`Value` (consuming `self`) is **excluded** — the macro treats consuming-`self` methods
separately: they are either constructors (`returns Self` or `#[miniextendr(constructor)]`)
or finalizers, not ordinary instance calls.  Including `Value` here would produce a
false-positive for a vctrs method with `#[miniextendr(constructor)]` that consumes `self`.

#### `spelling`

```rust
spelling(self: Self) -> &''static str
```

Human-readable spelling used in diagnostic messages.

### `Severity`

Diagnostic severity level.

**Variants:**

- `Info`
  - Migration hints and informational notes.
- `Warning`
  - Default for new rules; non-blocking.
- `Error`
  - CI-blocking in strict mode.
