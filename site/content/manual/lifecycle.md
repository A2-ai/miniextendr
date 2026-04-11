+++
title = "Lifecycle Integration"
weight = 40
description = "miniextendr integrates with the lifecycle R package to mark functions as experimental, deprecated, or defunct. The proc macro generates lifecycle badges, runtime warnings, and roxygen tags automatically."
+++

miniextendr integrates with the [lifecycle](https://lifecycle.r-lib.org/) R package to mark functions as experimental, deprecated, or defunct. The proc macro generates lifecycle badges, runtime warnings, and roxygen tags automatically.

## Quick Start

```rust
/// @title My old function
#[miniextendr(lifecycle = "deprecated")]
pub fn old_fn(x: i32) -> i32 {
    x + 1
}
```

This generates an R wrapper that:
1. Displays a lifecycle badge in the documentation
2. Calls `lifecycle::deprecate_warn()` at runtime
3. Adds `@importFrom lifecycle badge deprecate_warn` to roxygen

## Lifecycle Stages

| Stage | Attribute | Runtime | Badge |
|-------|-----------|---------|-------|
| Experimental | `lifecycle = "experimental"` | `signal_stage("experimental", ...)` | experimental |
| Stable | `lifecycle = "stable"` | (none) | (none) |
| Superseded | `lifecycle = "superseded"` | `signal_stage("superseded", ...)` | superseded |
| Soft-deprecated | `lifecycle = "soft-deprecated"` | `deprecate_soft(...)` | deprecated |
| Deprecated | `lifecycle = "deprecated"` | `deprecate_warn(...)` | deprecated |
| Defunct | `lifecycle = "defunct"` | `deprecate_stop(...)` | deprecated |

## Full Specification

For more control, use the expanded form:

```rust
#[miniextendr(lifecycle(
    stage = "deprecated",
    when = "0.4.0",        // version when deprecated
    with = "new_fn()",     // replacement function
    details = "Use new_fn() for better performance.",
    id = "old_fn-deprecation"  // unique ID for lifecycle tracking
))]
pub fn old_fn(x: i32) -> i32 {
    x + 1
}
```

### Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `stage` | Yes | One of the lifecycle stages above |
| `when` | No | Version string (e.g., `"1.0.0"`) |
| `what` | No | What is deprecated (defaults to function name) |
| `with` | No | Replacement function name |
| `details` | No | Additional context message |
| `id` | No | Unique deprecation ID for lifecycle tracking |

## Rust `#[deprecated]` Bridge

If your Rust code already has `#[deprecated]`, miniextendr picks it up automatically:

```rust
#[deprecated(since = "0.3.0", note = "Use new_fn() instead")]
#[miniextendr]
pub fn old_fn(x: i32) -> i32 {
    x + 1
}
```

This maps to `lifecycle(stage = "deprecated", when = "0.3.0", details = "Use new_fn() instead")`.

## Class Methods

Lifecycle works on impl methods too. Imports are aggregated at the class level:

```rust
#[miniextendr(r6)]
impl MyClass {
    #[miniextendr(lifecycle = "deprecated")]
    pub fn old_method(&self) -> i32 { 0 }

    #[miniextendr(lifecycle = "experimental")]
    pub fn new_method(&self) -> i32 { 1 }
}
```

The class-level roxygen block gets a single `@importFrom lifecycle badge deprecate_warn signal_stage` tag covering all methods.

## Requirements

The generated R code depends on the `lifecycle` package. Add it to your package's `DESCRIPTION`:

```text
Imports: lifecycle
```

Or in `Suggests` if lifecycle signals are optional.

## See Also

- [FEATURES.md](FEATURES.md) -- Feature flags reference
- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) -- How lifecycle interacts with class generators
