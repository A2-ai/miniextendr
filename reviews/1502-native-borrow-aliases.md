# Native scalar and boxed borrow preflight (#1502)

The borrowed-list guard from #1252 walked only syntactically visible slices.
Native scalar references, boxed borrowed elements, and type aliases could bypass
it and construct overlapping mutable/shared Rust references before user code.

A safe macro regression checked for guard emission without running an aliased
conversion; it failed before the fix. `TryFromSexp::NATIVE_BORROW` now describes
native leaves and list depth. The wrapper queries the actual selected converter,
then a protected SEXP walk batches conflicts before any reference conversion.
Scalar leaves require the native SEXPTYPE and length one, including NA values.
Forwarding conversions retain metadata from their actual delegate.

Compile coverage caught the named-lifetime scalar query requiring `'static`.
Native reference impls now accept the caller's lifetime, preserving the same
rooting contract and storage behavior. Special conversion paths skip metadata
queries for types that need no `TryFromSexp` impl. The rejected mutable SEXP
slice diagnostic gains a second trait-bound span from the new metadata query;
its snapshot is regenerated with the minimal toolchain used by `just test-ui`.

An attempted focused UI filter used `TRYBUILD` incorrectly (it accepts snapshot
modes, not a filename); rerunning the normal recipe exercised the full suite.

Branch-dependent `Either<L, R>` borrow selection remains tracked in #1504;
its first-success semantics cannot be represented by one unconditional leaf.

Verification passed: `just check`, all six `just test` legs, all-manifest Clippy
with `-D warnings`, and the three exact CI Clippy configurations. Formatting,
template sync, AGENTS structure, and the regenerated LLM corpus checks passed.
R 4.6.1 debug and release installs each passed 940 selected assertions with no
R test warnings or skips: 172 alias assertions plus conversion, match.arg,
scalar/slice, and worker-condition regressions. Both profiles ran the existing
allocating-list and RNG-cleanup GC-stress cases. R documentation was regenerated
and installed again before testing the new exports.

The all-integration macOS debug link exceeded the compact-unwind DWARF offset
limit (22.48 MiB `__eh_frame` versus the 24-bit offset field). The runtime
regressions passed; the same release build linked without the warning. This
separate build-profile issue is tracked in #1505 with unwind-preserving criteria.
`tools::undoc()` returns four category vectors even when all are empty; the
verification checks every vector length, rather than the outer list length.
