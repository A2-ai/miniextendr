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
| `Freeze` | 17 | 0 |
| `TryInto` | 17 | 0 |
| `TryFrom` | 17 | 0 |
| `Unpin` | 17 | 0 |
| `Into` | 17 | 0 |
| `From` | 17 | 0 |
| `Borrow` | 17 | 0 |
| `FromArgMatches` | 14 | 14 |
| `Subcommand` | 12 | 12 |
| `ToOwned` | 3 | 0 |
| `Clone` | 3 | 3 |
| `CloneToUninit` | 3 | 0 |
| `ToString` | 2 | 0 |
| `Serialize` | 2 | 2 |
| `Debug` | 2 | 2 |
| `Display` | 2 | 2 |
| `Args` | 2 | 2 |
| `CommandFactory` | 1 | 1 |
| `Parser` | 1 | 1 |

## `Any` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VendorCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProjectContext` | `<T> +1wc` | blanket | 1 | (no span) |
| `RustCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T> +1wc` | blanket | 1 | (no span) |
| `ConfigCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureRuleCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkflowCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureDetectCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Command` | `<T> +1wc` | blanket | 1 | (no span) |
| `HasResult` | `<T> +1wc` | blanket | 1 | (no span) |
| `InitCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `RenderCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Cli` | `<T> +1wc` | blanket | 1 | (no span) |
| `StatusCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoCmd` | `<T> +1wc` | blanket | 1 | (no span) |

### `Any` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `VendorCmd`, `ProjectContext`, `RustCmd`, `CargoBuildOpts`, `ConfigCmd`, `FeatureRuleCmd`, `WorkflowCmd`, `Config`, `FeatureCmd`, `FeatureDetectCmd`, `Command`, `HasResult`, `InitCmd`, `RenderCmd`, `Cli`, `StatusCmd`, `CargoCmd`

## `BorrowMut` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Command` | `<T> +1wc` | blanket | 1 | (no span) |
| `HasResult` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProjectContext` | `<T> +1wc` | blanket | 1 | (no span) |
| `Cli` | `<T> +1wc` | blanket | 1 | (no span) |
| `InitCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `RenderCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `StatusCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T> +1wc` | blanket | 1 | (no span) |
| `VendorCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `RustCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `ConfigCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureRuleCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkflowCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureDetectCmd` | `<T> +1wc` | blanket | 1 | (no span) |

### `BorrowMut` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `Command`, `HasResult`, `ProjectContext`, `Cli`, `InitCmd`, `RenderCmd`, `StatusCmd`, `CargoCmd`, `Config`, `VendorCmd`, `RustCmd`, `ConfigCmd`, `CargoBuildOpts`, `FeatureRuleCmd`, `WorkflowCmd`, `FeatureCmd`, `FeatureDetectCmd`

## `TryInto` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VendorCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProjectContext` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RustCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CargoBuildOpts` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ConfigCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureRuleCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkflowCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Config` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureDetectCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Command` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `HasResult` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `InitCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RenderCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StatusCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CargoCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Cli` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryInto` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `VendorCmd`, `ProjectContext`, `RustCmd`, `CargoBuildOpts`, `ConfigCmd`, `FeatureRuleCmd`, `WorkflowCmd`, `Config`, `FeatureCmd`, `FeatureDetectCmd`, `Command`, `HasResult`, `InitCmd`, `RenderCmd`, `StatusCmd`, `CargoCmd`, `Cli`

## `TryFrom` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `HasResult` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `InitCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RenderCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `StatusCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Cli` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CargoCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `VendorCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ProjectContext` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `RustCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `CargoBuildOpts` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `ConfigCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureRuleCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `WorkflowCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Config` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `FeatureDetectCmd` | `<T, U> +1wc` | blanket | 2 | (no span) |
| `Command` | `<T, U> +1wc` | blanket | 2 | (no span) |

### `TryFrom` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `HasResult`, `InitCmd`, `RenderCmd`, `StatusCmd`, `Cli`, `CargoCmd`, `VendorCmd`, `ProjectContext`, `RustCmd`, `CargoBuildOpts`, `ConfigCmd`, `FeatureRuleCmd`, `WorkflowCmd`, `Config`, `FeatureCmd`, `FeatureDetectCmd`, `Command`

## `Into` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Cli` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `VendorCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ProjectContext` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RustCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `ConfigCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FeatureRuleCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `WorkflowCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FeatureCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `FeatureDetectCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `Command` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `HasResult` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `InitCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `RenderCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `StatusCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |
| `CargoCmd` | `<T, U> +1wc` | blanket | 1 | (no span) |

### `Into` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `Cli`, `VendorCmd`, `ProjectContext`, `RustCmd`, `CargoBuildOpts`, `ConfigCmd`, `FeatureRuleCmd`, `WorkflowCmd`, `Config`, `FeatureCmd`, `FeatureDetectCmd`, `Command`, `HasResult`, `InitCmd`, `RenderCmd`, `StatusCmd`, `CargoCmd`

## `From` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `HasResult` | `<T>` | blanket | 1 | (no span) |
| `InitCmd` | `<T>` | blanket | 1 | (no span) |
| `RenderCmd` | `<T>` | blanket | 1 | (no span) |
| `StatusCmd` | `<T>` | blanket | 1 | (no span) |
| `CargoCmd` | `<T>` | blanket | 1 | (no span) |
| `Cli` | `<T>` | blanket | 1 | (no span) |
| `VendorCmd` | `<T>` | blanket | 1 | (no span) |
| `ProjectContext` | `<T>` | blanket | 1 | (no span) |
| `RustCmd` | `<T>` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T>` | blanket | 1 | (no span) |
| `ConfigCmd` | `<T>` | blanket | 1 | (no span) |
| `FeatureRuleCmd` | `<T>` | blanket | 1 | (no span) |
| `WorkflowCmd` | `<T>` | blanket | 1 | (no span) |
| `Config` | `<T>` | blanket | 1 | (no span) |
| `FeatureCmd` | `<T>` | blanket | 1 | (no span) |
| `FeatureDetectCmd` | `<T>` | blanket | 1 | (no span) |
| `Command` | `<T>` | blanket | 1 | (no span) |

### `From` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `HasResult`, `InitCmd`, `RenderCmd`, `StatusCmd`, `CargoCmd`, `Cli`, `VendorCmd`, `ProjectContext`, `RustCmd`, `CargoBuildOpts`, `ConfigCmd`, `FeatureRuleCmd`, `WorkflowCmd`, `Config`, `FeatureCmd`, `FeatureDetectCmd`, `Command`

## `Borrow` — 17 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `VendorCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `RustCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `ConfigCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Cli` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureRuleCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `WorkflowCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `FeatureDetectCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Command` | `<T> +1wc` | blanket | 1 | (no span) |
| `HasResult` | `<T> +1wc` | blanket | 1 | (no span) |
| `ProjectContext` | `<T> +1wc` | blanket | 1 | (no span) |
| `InitCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `RenderCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `StatusCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoCmd` | `<T> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T> +1wc` | blanket | 1 | (no span) |

### `Borrow` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (17 impls): `VendorCmd`, `RustCmd`, `ConfigCmd`, `Cli`, `CargoBuildOpts`, `FeatureRuleCmd`, `WorkflowCmd`, `FeatureCmd`, `FeatureDetectCmd`, `Command`, `HasResult`, `ProjectContext`, `InitCmd`, `RenderCmd`, `StatusCmd`, `CargoCmd`, `Config`

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

## `ToOwned` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `<T> +1wc` | blanket | 3 | (no span) |
| `ProjectContext` | `<T> +1wc` | blanket | 3 | (no span) |
| `Config` | `<T> +1wc` | blanket | 3 | (no span) |

### `ToOwned` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `CargoBuildOpts`, `ProjectContext`, `Config`

## `Clone` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:236 |
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |

## `CloneToUninit` — 3 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `ProjectContext` | `<T> +1wc` | blanket | 1 | (no span) |
| `CargoBuildOpts` | `<T> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T> +1wc` | blanket | 1 | (no span) |

### `CloneToUninit` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (3 impls): `ProjectContext`, `CargoBuildOpts`, `Config`

## `ToString` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `HasResult` | `<T> +1wc` | blanket | 1 | (no span) |
| `Config` | `<T> +1wc` | blanket | 1 | (no span) |

### `ToString` — for-types sharing a source span (likely macro-expanded / co-located)

- **(no span)** (2 impls): `HasResult`, `Config`

## `Serialize` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:11 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:10 |

## `Debug` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `CargoBuildOpts` | `` | concrete | 1 | miniextendr-cli/src/cli.rs:236 |
| `ProjectContext` | `` | concrete | 1 | miniextendr-cli/src/project.rs:62 |

## `Display` — 2 impls

| for-type | generics | kind | #items | span |
|---|---|---|---|---|
| `Config` | `` | concrete | 1 | miniextendr-cli/src/commands/config.rs:21 |
| `HasResult` | `` | concrete | 1 | miniextendr-cli/src/commands/status.rs:19 |

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
