# Serialize return attribute (#1521)

The attribute must change both the value conversion and return-shape analysis.
Otherwise a serialized Self or class return would still receive an R constructor
wrapper, and Result/Option would keep their ordinary boundary handling instead of
matching AsSerialize<Result/Option>. The shared helpers model the converted type
and prepare the call result after unwrapping any outer visibility marker.

The first compile identified a missed TraitMethod initializer in the adapter path;
that initializer and the unit-test helper now explicitly retain serialize = false.

The class-system unit probe initially used an instance receiver for vctrs, which
correctly rejects instance methods because its R values are base vectors. The
probe now uses static methods for that system, while retaining instance methods
for the other five.

The first installed-package run called an inherent Env method as
`SerializeHost$snapshot(obj)` and failed with an unused argument. Existing
Env wrappers bind `self` through the instance `$` dispatcher; only trait
namespace methods accept an explicit receiver. Corrected the tests and
examples to `obj$snapshot()` after inspecting the emitted signatures.

The doc lint caught two mistakes in the new examples: an explicit title that
differed from the first prose line, and examples attached to the impl instead
of a method. Used the prose title and moved class examples onto `new`.

The initial Rust View probe received an ordinary `ExternalPtr`, which does
not carry the trait ABI header queried by `TraitView::from_sexp`. The fixture
now constructs the derive-generated `__mx_wrap_serializehost` object and uses
`ccall::mx_wrap`, matching the producer-package tests, and roots it for the
whole View call. This exercises the concrete serialized vtable shim.
The rooting helper lives in `gc_protect`, not `gc`; corrected the initial fixture import after Clippy caught it.

The adjacent trait tests use internal fixtures; running `test_file` without
`package = "miniextendr"` hid those names. The targeted runner now requests the
same package test environment as the normal suite.

The built-tarball check found a real Env documentation generator bug: when
`@examples` was the last method tag, the appended `\describe` parameter prose
stayed inside the code block, producing an invalid Rd file and R examples.
`MethodDocBuilder` now starts `@details` before appended parameters after
`@examples` or `@examplesIf`. A shared-builder regression and the installed
constructor example cover the fix.
