# Trait impl inventory

Source: `target/doc/miniextendr_engine.json`

Traits with impls: 20

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Sync` | 3 | 0 |
| `UnwindSafe` | 3 | 0 |
| `Borrow` | 3 | 0 |
| `TryInto` | 3 | 0 |
| `Any` | 3 | 0 |
| `BorrowMut` | 3 | 0 |
| `Send` | 3 | 0 |
| `From` | 3 | 0 |
| `UnsafeUnpin` | 3 | 0 |
| `Unpin` | 3 | 0 |
| `RefUnwindSafe` | 3 | 0 |
| `TryFrom` | 3 | 0 |
| `Into` | 3 | 0 |
| `Freeze` | 3 | 0 |
| `Error` | 1 | 1 |
| `Display` | 1 | 1 |
| `Drop` | 1 | 1 |
| `Default` | 1 | 1 |
| `ToString` | 1 | 0 |
| `Debug` | 1 | 1 |

## `Borrow` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngine` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngineBuilder` | `<T> +1wc` | blanket | 1 | (no span) |

### `Borrow` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngineError`, `REngine`, `REngineBuilder`

## `TryInto` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngine` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REngineError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REngineBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryInto` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngine`, `REngineError`, `REngineBuilder`

## `Any` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngineBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngine` | `<T> +1wc` | blanket | 1 | (no span) |

### `Any` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngineError`, `REngineBuilder`, `REngine`

## `BorrowMut` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineBuilder` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngine` | `<T> +1wc` | blanket | 1 | (no span) |
| `REngineError` | `<T> +1wc` | blanket | 1 | (no span) |

### `BorrowMut` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngineBuilder`, `REngine`, `REngineError`

## `From` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineBuilder` | `<T>` | blanket | 1 | (no span) |
| `REngine` | `<T>` | blanket | 1 | (no span) |
| `REngineError` | `<T>` | blanket | 1 | (no span) |

### `From` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngineBuilder`, `REngine`, `REngineError`

## `TryFrom` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REngineBuilder` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `REngine` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryFrom` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngineError`, `REngineBuilder`, `REngine`

## `Into` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngine` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `REngineBuilder` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `REngineError` | `<T, U> +1wc` | blanket | 1 | (no span) |

### `Into` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `REngine`, `REngineBuilder`, `REngineError`

## `Error` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `` | concrete | 0 | miniextendr-engine/src/lib.rs:383 |

## `Display` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `` | concrete | 1 | miniextendr-engine/src/lib.rs:360 |

## `Drop` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngine` | `` | concrete | 1 | miniextendr-engine/src/lib.rs:265 |

## `Default` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineBuilder` | `` | concrete | 1 | miniextendr-engine/src/lib.rs:132 |

## `ToString` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `<T> +1wc` | blanket | 1 | (no span) |

## `Debug` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `REngineError` | `` | concrete | 1 | miniextendr-engine/src/lib.rs:347 |
