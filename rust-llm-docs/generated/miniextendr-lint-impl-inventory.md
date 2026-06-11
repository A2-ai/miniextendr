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
