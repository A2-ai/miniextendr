# serde: the buffered `Content` has no 128-bit integers

- **Upstream**: [serde-rs/serde#1682](https://github.com/serde-rs/serde/issues/1682)
  ("Untagged enum does not play nice with u128"), open since 2019-11-26.
  [serde-rs/serde#1717](https://github.com/serde-rs/serde/issues/1717) was
  closed as its duplicate;
  [serde-rs/serde#2230](https://github.com/serde-rs/serde/issues/2230)
  ("Implement `visit_i128` and `visit_u128` for ContentVisitor") is open too.
- **Our issue**: #2257. We have not commented upstream.

## Impact

Untagged and internally tagged enums and `#[serde(flatten)]` decode through
serde's private buffered `Content`, which has no 128-bit variant:
`ContentVisitor` doesn't implement `visit_i128` / `visit_u128`, and its
deserializers keep the default `deserialize_i128` / `deserialize_u128`, which
error. So an `i128` or `u128` in one of those contexts fails to decode,
whatever the format's `deserialize_any` visits, small values included.

For us: an `ExternalPtr` snapshot (#2256) reads every other value back
exactly in those contexts, and an `i128` / `u128` only in a plain field, a
sequence or a map. `integers_come_back_unchanged` in
`miniextendr-api/tests/serde_exact.rs` pins the failure;
`miniextendr-api/src/serde/exact.rs` and `docs/EXTERNALPTR.md` document it.

## Workaround

A `#[serde(with = ...)]` module that writes the integer as a decimal string
and parses it back; the snapshot keeps strings exactly.

## Closes when

serde's `Content` carries 128-bit integers in a release, we bump serde, and
the pinned failure becomes a `check_everywhere` of `i128::MIN` / `u128::MAX`.

## Last checked

2026-10-29, serde 1.0.228: #1682 and #2230 open, no linked pull request.
