# Trait impl inventory

Source: `target/doc/miniextendr.json`

Traits with impls: 26

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `UnwindSafe` | 17 | 0 |
| `Sync` | 17 | 0 |
| `UnsafeUnpin` | 17 | 0 |
| `Any` | 17 | 0 |
| `BorrowMut` | 17 | 0 |
| `RefUnwindSafe` | 17 | 0 |
| `Send` | 17 | 0 |
| `From` | 17 | 0 |
| `Freeze` | 17 | 0 |
| `TryInto` | 17 | 0 |
| `Unpin` | 17 | 0 |
| `Into` | 17 | 0 |
| `Borrow` | 17 | 0 |
| `TryFrom` | 17 | 0 |
| `FromArgMatches` | 14 | 14 |
| `Subcommand` | 12 | 12 |
| `ToOwned` | 3 | 0 |
| `CloneToUninit` | 3 | 0 |
| `Clone` | 3 | 3 |
| `Serialize` | 2 | 2 |
| `Display` | 2 | 2 |
| `ToString` | 2 | 0 |
| `Debug` | 2 | 2 |
| `Args` | 2 | 2 |
| `CommandFactory` | 1 | 1 |
| `Parser` | 1 | 1 |

## `FromArgMatches` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:164 |
| `StatusCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:222 |
| `CargoBuildOpts` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:236 |
| `CargoCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:258 |
| `Command` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:28 |
| `Cli` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:4 |
| `VendorCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:422 |
| `FeatureCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:472 |
| `FeatureDetectCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:493 |
| `FeatureRuleCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:501 |
| `RenderCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:528 |
| `RustCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:549 |
| `ConfigCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:568 |

## `Subcommand` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:164 |
| `StatusCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:222 |
| `CargoCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:258 |
| `Command` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:28 |
| `VendorCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:422 |
| `FeatureCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:472 |
| `FeatureDetectCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:493 |
| `FeatureRuleCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:501 |
| `RenderCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:528 |
| `RustCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:549 |
| `ConfigCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:568 |

## `Clone` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:236 |
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |

## `Serialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:9 |

## `Display` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:21 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:18 |

## `Debug` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:236 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |

## `Args` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:236 |
| `Cli` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:4 |

## `CommandFactory` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 2 | miniextendr-cli/src/cli.rs:4 |

## `Parser` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 0 | miniextendr-cli/src/cli.rs:4 |
