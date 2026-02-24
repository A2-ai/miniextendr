# TODO: Future Feature Ideas

Remaining items from plan 05 that haven't been implemented.

## Streaming ALTREP Connections (Medium priority)

Bridge R connections with ALTREP vectors — a connection-backed ALTREP
vector that reads from a stream as elements are accessed.

```rust
struct StreamingVector {
    connection: RConnectionHandle,
    buffer: Vec<f64>,
    len: usize,
}

impl AltRealData for StreamingVector {
    fn len(&self) -> usize { self.len }
    fn elt(&self, i: usize) -> f64 {
        self.ensure_read_to(i);
        self.buffer[i]
    }
}
```

Use case: reading large CSV/binary files lazily.

## Serde → DataFrame Direct Path (Medium priority)

Bypass row-to-column transpose by serializing directly into columnar
output via a custom serde serializer.

Currently: `Vec<MyStruct>` → row List → transpose → DataFrame (3 allocs/col)
Proposed: `Vec<MyStruct>` → columnar serializer → DataFrame (1 alloc/col)

The serde module already has `RSerializeNative`. Extend with a "columnar"
mode that accumulates struct fields into column vectors.

## Windowed Iterator ALTREP (Low priority)

Current iterator-backed ALTREP eagerly materializes into Vec on
`dataptr()`. Add a windowed mode that materializes only the requested
region, keeping iterator state for sequential access.

Falls back to full materialization on random access.

## Factor/MatchArg Unification (Low priority)

`#[derive(RFactor)]` and `#[derive(MatchArg)]` both convert fieldless
enums to/from string sets. Could share an `EnumChoices` trait. Both
work independently — low urgency.

## List Builder Improvements (Low priority)

Add conditional insertion and iteration support to `ListBuilder`:

```rust
let list = List::builder()
    .push(1)
    .push_named("label", "hello")
    .push_if(condition, || expensive_value())
    .extend(other_list)
    .build();
```

Current API works fine for most cases.
