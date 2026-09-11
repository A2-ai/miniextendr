# Interrupted monorepo build recovery (#1429)

A monorepo path-dependency build can freeze src/rust/Cargo.toml to vendor
paths. Removing only inst/vendor.tar.xz leaves those paths behind, so the
next metadata/configure operation fails if vendor is absent. The scaffolder
also inferred package names from arbitrary name lines, assumed every core
crate lived in a child directory, and omitted crate_name_rs. Starting R from
the suggested package subdirectory could select an older helper installation.

The freeze writer also emitted an empty [patch] parent heading. Make that
parent implicit while retaining [patch.crates-io], with a serialized-output
regression assertion.

An initial edit embedded quote-producing code inside a quoted CLI message and
failed R parsing. Construct the displayed build call separately, then interpolate
that value into the message.

The existing-project scaffold now asks Cargo for its workspace packages and
selects a library member (the root library by default; an explicit `crate_name`
resolves ambiguous virtual workspaces). It derives the dependency path from the
selected manifest, names the binding package separately from its library, and
uses a `core_library` dependency alias so a package named `core` does not shadow
Rust's standard library. Existing libraries have arbitrary APIs, so their
scaffold does not emit a call to a presumed `hello()` function. The newly created
library template still provides and demonstrates that function.

The first regression run passed Cargo metadata assertions for both layouts but
failed two message assertions: CLI wrapped “from this workspace root” across
lines. The assertion now normalizes display whitespace before checking the
workspace-root build instructions.

The end-to-end root-package build then reached a second assumption in
`cargo-revendor`: workspace-root discovery walked parents looking for the
literal `[workspace]` table and aborted on a standalone library. It now uses
`cargo locate-project --workspace`, which recognizes Cargo's implicit
single-package workspaces as well as explicit workspace membership.

The complete Rust suite also caught the CLI's embedded-template consumer: it
did not supply the newly required core path. The CLI now supplies the same
workspace-derived dependency path, distinct binding package name and commented
existing-API example as the R scaffolder. Its `init use --crate-name` selects
an explicit virtual-workspace member. Both template consumers retain simple
string substitution; a comment-prefix variable controls whether the known
new-library `hello()` example is enabled.

The new Cargo workspace-root regression caught macOS's `/var` versus
`/private/var` spelling: `cargo locate-project` preserves the input spelling.
The helper canonicalizes the returned directory, retaining its previous path
identity contract.

With standalone packaging working, the real tarball install exposed a second
vendoring bug: renamed path dependencies were looked up by the dependency key
rather than their `package` field, leaving `core_library` pointed at the
workspace root inside the tarball. Freeze, vendored-path repair and package
version insertion now resolve the actual package name while preserving the
alias and dependency options. Regression coverage exercises inline and table
forms, including build dependencies.

The successful local-framework builds revealed one last source-tree side
effect: `cargo package` appended `[[patch.unused]] core` to the framework
workspace's existing `Cargo.lock` while using transient patches. Packaging now
snapshots/restores that existing lock alongside its manifests; a real
standalone-package regression checks that the lock's bytes remain unchanged.

An unrelated parallel telemetry test emitted a swallowed assertion panic during
validation. It is tracked separately in #1492. The R package check's existing
AGENTS.md and miniextendr Rd cross-reference notes are tracked by #1409/#1410
and already fixed in open PR #1487.

Validation after rebasing onto main's #1485 merge: all Rust test legs and
three CI clippy configurations passed; the R CI suite passed 836 assertions.
R configure/install/force-document, template synchronization and LLM corpus
checks passed with no generated drift. The two-layout integration test runs
four real builds and verifies installed calls plus manifest restoration; its
initial and rebased runs each passed all 40 assertions. The final run also
left the framework workspace manifest and lockfile unchanged.

The first webR CI run exposed a Mustache rendering difference in the new
core-library example. R's whisker consumes the fourth closing brace in
`{{{core_example_prefix}}}}`, removing the Rust function's closing delimiter;
the CLI's literal substitution preserved it. An intervening space keeps the
Rust brace separate. A regression now renders both the enabled and commented
examples through the actual R scaffolder and parses them with rustfmt. It
failed with the same unclosed-delimiter error before the fix.

The first CRAN-like CI run also reproduced #1326 verbatim: `compare_proxy`
segfaulted at address `0xa1` while checking `result$total` in the DataFusion
SUM test. PR #1486 already fixes that Arrow buffer provenance bug and has
passed its complete CI suite; this PR does not duplicate that runtime change.

The new-monorepo smoke initially selected the June 24 system installation of
cargo-revendor, which lacks this PR's aliased-dependency freeze fix. Its
native build passed but its tarball install retained the sibling path.
Rebuilt with `CARGO_TARGET_DIR="$PWD/target" just revendor-build` and prepended
`$PWD/target/debug` to PATH for the smoke run; the user's global tool was not
replaced. The R renderer regression and full standard scaffolder suite pass
(840 assertions, zero failures/warnings), as do `just fmt`, `just check`,
`just test -- monorepo` (including the UI leg), `just clippy -- -D warnings`
and all three CI Clippy configurations.

With the worktree-built cargo-revendor, a newly scaffolded monorepo completed
`miniextendr_build()` through the native bootstrap, vendored tarball install,
and roxygen pass. Its installed `core_greeting()`, `add(2, 3)` and `hello("R")`
returned the expected values. The source manifest and framework lockfile
remained unchanged.
