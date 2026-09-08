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
