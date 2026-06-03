# Trait impl inventory

Source: `target/doc/miniextendr_lint.json`

Traits with impls: 28

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Freeze` | 12 | 0 |
| `From` | 12 | 0 |
| `UnwindSafe` | 12 | 0 |
| `Unpin` | 12 | 0 |
| `Any` | 12 | 0 |
| `Into` | 12 | 0 |
| `Sync` | 12 | 0 |
| `UnsafeUnpin` | 12 | 0 |
| `BorrowMut` | 12 | 0 |
| `TryFrom` | 12 | 0 |
| `Send` | 12 | 0 |
| `RefUnwindSafe` | 12 | 0 |
| `Borrow` | 12 | 0 |
| `TryInto` | 12 | 0 |
| `Debug` | 11 | 11 |
| `ToOwned` | 8 | 0 |
| `CloneToUninit` | 8 | 0 |
| `Clone` | 8 | 8 |
| `Eq` | 5 | 5 |
| `PartialEq` | 5 | 5 |
| `StructuralPartialEq` | 4 | 4 |
| `Copy` | 4 | 4 |
| `Default` | 3 | 3 |
| `Display` | 3 | 3 |
| `Hash` | 3 | 3 |
| `ToString` | 3 | 0 |
| `Ord` | 2 | 2 |
| `PartialOrd` | 2 | 2 |

## `From` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Diagnostic` | `<T>` | blanket | 1 | (no span) |
| `LintKind` | `<T>` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T>` | blanket | 1 | (no span) |
| `LintReport` | `<T>` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T>` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T>` | blanket | 1 | (no span) |
| `MiniextendrImplAttrs` | `<T>` | blanket | 1 | (no span) |
| `LintItem` | `<T>` | blanket | 1 | (no span) |
| `CrateIndex` | `<T>` | blanket | 1 | (no span) |
| `FileData` | `<T>` | blanket | 1 | (no span) |
| `LintCode` | `<T>` | blanket | 1 | (no span) |
| `Severity` | `<T>` | blanket | 1 | (no span) |

### `From` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `Diagnostic`, `LintKind`, `ImplMethodEntry`, `LintReport`, `MethodReceiverKind`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `LintCode`, `Severity`

## `Any` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintReport` | `<T> +1wc` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T> +1wc` | blanket | 1 | (no span) |
| `MiniextendrImplAttrs` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintItem` | `<T> +1wc` | blanket | 1 | (no span) |
| `CrateIndex` | `<T> +1wc` | blanket | 1 | (no span) |
| `FileData` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintCode` | `<T> +1wc` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 1 | (no span) |

### `Any` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `LintKind`, `ImplMethodEntry`, `LintReport`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `LintCode`, `MethodReceiverKind`, `Severity`, `Diagnostic`

## `Into` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `LintKind` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `LintReport` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `MiniextendrImplAttrs` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `LintItem` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CrateIndex` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FileData` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `LintCode` | `<T, U> +1wc` | blanket | 1 | (no span) |

### `Into` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `Severity`, `Diagnostic`, `LintKind`, `ImplMethodEntry`, `LintReport`, `MethodReceiverKind`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `LintCode`

## `BorrowMut` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintCode` | `<T> +1wc` | blanket | 1 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintReport` | `<T> +1wc` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T> +1wc` | blanket | 1 | (no span) |
| `MiniextendrImplAttrs` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintItem` | `<T> +1wc` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `CrateIndex` | `<T> +1wc` | blanket | 1 | (no span) |
| `FileData` | `<T> +1wc` | blanket | 1 | (no span) |

### `BorrowMut` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `LintCode`, `Severity`, `Diagnostic`, `LintKind`, `ImplMethodEntry`, `LintReport`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `MethodReceiverKind`, `CrateIndex`, `FileData`

## `TryFrom` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintKind` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ImplMethodEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintReport` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AttributedTraitImpl` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MiniextendrImplAttrs` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintItem` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CrateIndex` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FileData` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintCode` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Severity` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Diagnostic` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MethodReceiverKind` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryFrom` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `LintKind`, `ImplMethodEntry`, `LintReport`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `LintCode`, `Severity`, `Diagnostic`, `MethodReceiverKind`

## `Borrow` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintCode` | `<T> +1wc` | blanket | 1 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintReport` | `<T> +1wc` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T> +1wc` | blanket | 1 | (no span) |
| `MiniextendrImplAttrs` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintItem` | `<T> +1wc` | blanket | 1 | (no span) |
| `CrateIndex` | `<T> +1wc` | blanket | 1 | (no span) |
| `FileData` | `<T> +1wc` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T> +1wc` | blanket | 1 | (no span) |

### `Borrow` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `LintCode`, `Severity`, `Diagnostic`, `LintKind`, `ImplMethodEntry`, `LintReport`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `MethodReceiverKind`

## `TryInto` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintKind` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ImplMethodEntry` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MethodReceiverKind` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintReport` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `AttributedTraitImpl` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `MiniextendrImplAttrs` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintItem` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CrateIndex` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FileData` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `LintCode` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Severity` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Diagnostic` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryInto` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (12 impls): `LintKind`, `ImplMethodEntry`, `MethodReceiverKind`, `LintReport`, `AttributedTraitImpl`, `MiniextendrImplAttrs`, `LintItem`, `CrateIndex`, `FileData`, `LintCode`, `Severity`, `Diagnostic`

## `Debug` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:102 |
| `AttributedTraitImpl` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:149 |
| `FileData` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:160 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:24 |
| `ImplMethodEntry` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:76 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:93 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:30 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `MiniextendrImplAttrs` | `` | concrete | 1 | miniextendr-lint/src/helpers.rs:104 |
| `LintReport` | `` | concrete | 1 | miniextendr-lint/src/lib.rs:93 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `ToOwned` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ImplMethodEntry` | `<T> +1wc` | blanket | 3 | (no span) |
| `AttributedTraitImpl` | `<T> +1wc` | blanket | 3 | (no span) |
| `LintItem` | `<T> +1wc` | blanket | 3 | (no span) |
| `MethodReceiverKind` | `<T> +1wc` | blanket | 3 | (no span) |
| `LintCode` | `<T> +1wc` | blanket | 3 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 3 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 3 | (no span) |
| `LintKind` | `<T> +1wc` | blanket | 3 | (no span) |

### `ToOwned` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (8 impls): `ImplMethodEntry`, `AttributedTraitImpl`, `LintItem`, `MethodReceiverKind`, `LintCode`, `Severity`, `Diagnostic`, `LintKind`

## `CloneToUninit` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintCode` | `<T> +1wc` | blanket | 1 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `ImplMethodEntry` | `<T> +1wc` | blanket | 1 | (no span) |
| `AttributedTraitImpl` | `<T> +1wc` | blanket | 1 | (no span) |
| `MethodReceiverKind` | `<T> +1wc` | blanket | 1 | (no span) |
| `LintItem` | `<T> +1wc` | blanket | 1 | (no span) |

### `CloneToUninit` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (8 impls): `LintCode`, `Severity`, `Diagnostic`, `LintKind`, `ImplMethodEntry`, `AttributedTraitImpl`, `MethodReceiverKind`, `LintItem`

## `Clone` — 8 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:102 |
| `AttributedTraitImpl` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:149 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:24 |
| `ImplMethodEntry` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:76 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:93 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:30 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `Eq` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:116 |
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:24 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:93 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `PartialEq` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:110 |
| `MethodReceiverKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:24 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:93 |
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `StructuralPartialEq` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:24 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:93 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `Copy` — 4 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `MethodReceiverKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:24 |
| `LintKind` | `` | concrete | 0 | miniextendr-lint/src/crate_index.rs:93 |
| `Severity` | `` | concrete | 0 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 0 | miniextendr-lint/src/lint_code.rs:10 |

## `Default` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `FileData` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:160 |
| `MiniextendrImplAttrs` | `` | concrete | 1 | miniextendr-lint/src/helpers.rs:104 |
| `LintReport` | `` | concrete | 1 | miniextendr-lint/src/lib.rs:93 |

## `Display` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:19 |
| `Diagnostic` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:76 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:58 |

## `Hash` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintItem` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:118 |
| `LintKind` | `` | concrete | 1 | miniextendr-lint/src/crate_index.rs:93 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `ToString` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `LintCode` | `<T> +1wc` | blanket | 1 | (no span) |
| `Severity` | `<T> +1wc` | blanket | 1 | (no span) |
| `Diagnostic` | `<T> +1wc` | blanket | 1 | (no span) |

### `ToString` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `LintCode`, `Severity`, `Diagnostic`

## `Ord` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |

## `PartialOrd` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Severity` | `` | concrete | 1 | miniextendr-lint/src/diagnostic.rs:9 |
| `LintCode` | `` | concrete | 1 | miniextendr-lint/src/lint_code.rs:10 |
