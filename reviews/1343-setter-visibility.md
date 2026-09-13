# Generated setter visibility

Added field-level setter visibility and routed inherent R6 method markers into
the active-binding setter branch, preserving the existing defaults and guarded
condition transport. Direct calls must be tested with `withVisible()` because
R assignment stays invisible regardless of the underlying setter's return.

The first compile-fail fixture imported `RSidecar` from the crate root, which
added an unrelated unresolved-import error alongside the intended diagnostics.
The type lives in `miniextendr_api::externalptr`; corrected the import and
regenerated the snapshot through `just test-ui` under the verified minimal
1.98.1 toolchain. The fixture now covers only invalid values, duplicate options,
private fields, and the selector field.

The first lint run caught the new R example attached to the `SidecarR6` impl
block, where method-only `@examples` tags are rejected. Moved it to the existing
`rdata_sidecar_r6_new` free-function documentation. The verification script checks
that the generated module Rd page actually contains examples and executes them,
in addition to package-wide usage and alias checks.

A verification run overlapped `just test` with API corpus generation, which
shares the global Cargo target directory and rebuilds dependency metadata with
`RUSTC_BOOTSTRAP=1`. Two rustdoc test targets then reported E0463 for `syn`
while unit tests and UI tests passed. Run the full test suite, lint gates, and
corpus generation sequentially when they share that target directory. The final
verification uses this ordering, with separate logs for each attempt.
