# Trait impl inventory

Source: `target/doc/miniextendr.json`

Traits with impls: 33

## Summary (impl count per trait)

| Trait | # impls | # non-blanket non-synthetic |
|---|---|---|
| `Any` | 21 | 0 |
| `Borrow` | 21 | 0 |
| `BorrowMut` | 21 | 0 |
| `Freeze` | 21 | 0 |
| `From` | 21 | 0 |
| `Into` | 21 | 0 |
| `RefUnwindSafe` | 21 | 0 |
| `Send` | 21 | 0 |
| `Sync` | 21 | 0 |
| `TryFrom` | 21 | 0 |
| `TryInto` | 21 | 0 |
| `Unpin` | 21 | 0 |
| `UnsafeUnpin` | 21 | 0 |
| `UnwindSafe` | 21 | 0 |
| `FromArgMatches` | 13 | 13 |
| `Subcommand` | 11 | 11 |
| `Clone` | 5 | 5 |
| `CloneToUninit` | 5 | 0 |
| `Debug` | 5 | 5 |
| `ToOwned` | 5 | 0 |
| `Copy` | 3 | 3 |
| `Args` | 2 | 2 |
| `Deserialize` | 2 | 2 |
| `DeserializeOwned` | 2 | 0 |
| `Equivalent` | 2 | 0 |
| `CommandFactory` | 1 | 1 |
| `Display` | 1 | 1 |
| `Eq` | 1 | 1 |
| `Parser` | 1 | 1 |
| `PartialEq` | 1 | 1 |
| `Serialize` | 1 | 1 |
| `StructuralPartialEq` | 1 | 1 |
| `ToString` | 1 | 0 |

## `FromArgMatches` — 13 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:112 |
| `WorkflowCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:167 |
| `StatusCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:225 |
| `CargoBuildOpts` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:239 |
| `CargoCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:261 |
| `Command` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:28 |
| `Cli` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:4 |
| `VendorCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:425 |
| `FeatureCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:475 |
| `FeatureDetectCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:496 |
| `FeatureRuleCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:504 |
| `RenderCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:531 |
| `RustCmd` | `` | concrete | 4 | miniextendr-cli/src/cli.rs:552 |

## `Subcommand` — 11 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `InitCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:112 |
| `WorkflowCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:167 |
| `StatusCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:225 |
| `CargoCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:261 |
| `Command` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:28 |
| `VendorCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:425 |
| `FeatureCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:475 |
| `FeatureDetectCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:496 |
| `FeatureRuleCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:504 |
| `RenderCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:531 |
| `RustCmd` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:552 |

## `Clone` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:239 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:236 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:340 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:351 |

## `Debug` — 5 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:239 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:236 |
| `Dest` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:340 |
| `PlanEntry` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:351 |

## `Copy` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:236 |
| `Dest` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:340 |
| `PlanEntry` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:351 |

## `Args` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:239 |
| `Cli` | `` | concrete | 3 | miniextendr-cli/src/cli.rs:4 |

## `Deserialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `WorkspaceLibrary` | `<'de>` | concrete | 1 | miniextendr-cli/src/commands/init.rs:246 |
| `LibraryTarget` | `<'de>` | concrete | 1 | miniextendr-cli/src/commands/init.rs:254 |

## `CommandFactory` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 2 | miniextendr-cli/src/cli.rs:4 |

## `Display` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:18 |

## `Eq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:236 |

## `Parser` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `` | concrete | 0 | miniextendr-cli/src/cli.rs:4 |

## `PartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 1 | miniextendr-cli/src/scaffold.rs:236 |

## `Serialize` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:9 |

## `StructuralPartialEq` — 1 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Render` | `` | concrete | 0 | miniextendr-cli/src/scaffold.rs:236 |
