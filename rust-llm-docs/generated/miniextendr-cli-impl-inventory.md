# Trait impl inventory

Source: `target/doc/miniextendr.json`

Traits with impls: 33

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 23 | 0 |
| `Borrow` | 23 | 0 |
| `BorrowMut` | 23 | 0 |
| `Freeze` | 23 | 0 |
| `From` | 23 | 0 |
| `Into` | 23 | 0 |
| `RefUnwindSafe` | 23 | 0 |
| `Send` | 23 | 0 |
| `Sync` | 23 | 0 |
| `TryFrom` | 23 | 0 |
| `TryInto` | 23 | 0 |
| `Unpin` | 23 | 0 |
| `UnsafeUnpin` | 23 | 0 |
| `UnwindSafe` | 23 | 0 |
| `FromArgMatches` | 14 | 14 |
| `Subcommand` | 12 | 12 |
| `Clone` | 6 | 6 |
| `CloneToUninit` | 6 | 0 |
| `ToOwned` | 6 | 0 |
| `Debug` | 5 | 5 |
| `Copy` | 3 | 3 |
| `Args` | 2 | 2 |
| `Deserialize` | 2 | 2 |
| `DeserializeOwned` | 2 | 0 |
| `Display` | 2 | 2 |
| `Equivalent` | 2 | 0 |
| `Serialize` | 2 | 2 |
| `ToString` | 2 | 0 |
| `CommandFactory` | 1 | 1 |
| `Eq` | 1 | 1 |
| `Parser` | 1 | 1 |
| `PartialEq` | 1 | 1 |
| `StructuralPartialEq` | 1 | 1 |

## `FromArgMatches` — 14 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:172 |
| `StatusCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:230 |
| `CargoBuildOpts` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:244 |
| `CargoCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:266 |
| `Command` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:28 |
| `Cli` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:4 |
| `VendorCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:430 |
| `FeatureCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:480 |
| `FeatureDetectCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:501 |
| `FeatureRuleCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:509 |
| `RenderCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:536 |
| `RustCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:557 |
| `ConfigCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:576 |

## `Subcommand` — 12 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:117 |
| `WorkflowCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:172 |
| `StatusCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:230 |
| `CargoCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:266 |
| `Command` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:28 |
| `VendorCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:430 |
| `FeatureCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:480 |
| `FeatureDetectCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:501 |
| `FeatureRuleCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:509 |
| `RenderCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:536 |
| `RustCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:557 |
| `ConfigCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:576 |

## `Clone` — 6 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:244 |
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:226 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:330 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:341 |

## `Debug` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:244 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:226 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:330 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:341 |

## `Copy` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:226 |
| `Dest` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:330 |
| `PlanEntry` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:341 |

## `Args` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:244 |
| `Cli` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:4 |

## `Deserialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `WorkspaceLibrary` | `<'de>` | concrete | 1 | miniextendr-cli/src/commands/init.rs:241 |
| `LibraryTarget` | `<'de>` | concrete | 1 | miniextendr-cli/src/commands/init.rs:249 |

## `Display` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:21 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:18 |

## `Serialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:9 |

## `CommandFactory` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 2 | miniextendr-cli/src/cli.rs:4 |

## `Eq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:226 |

## `Parser` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 0 | miniextendr-cli/src/cli.rs:4 |

## `PartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:226 |

## `StructuralPartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:226 |
