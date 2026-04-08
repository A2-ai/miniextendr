+++
title = "Full Project Tutorial"
weight = 4
description = "Walk through the pigworld model project from package skeleton to Rust runtime, configure glue, and generated R wrappers."
+++

This guide uses `tests/model_project/` as a concrete end-to-end example. It is a much better tutorial reference than a minimal hello-world because it shows a full package with configure logic, a nested Rust crate, generated R wrappers, and a nontrivial Rust type exported to R.

## Why this project is useful

The model project is a scaffolded package called `pigworld`. It includes:

- package metadata in `DESCRIPTION`, `NAMESPACE`, and `R/`
- a nested Rust crate in `src/rust/`
- configure/bootstrap/build glue for the R package build
- generated R wrappers and roxygen output
- a real exported Rust-backed `World` type instead of a single toy function

In other words: it shows the whole shape of a package, not just one boundary crossing.

## Step 1: the package shell

Start with the R package files:

- `tests/model_project/DESCRIPTION`
- `tests/model_project/NAMESPACE`
- `tests/model_project/R/pigworld-package.R`

These establish the installed package name (`pigworld`), register the dynamic library, and provide the minimal R-side package shell that the generated wrappers plug into.

This is the layer that should still look familiar to an R package author. miniextendr does not replace the package shell; it fills in the Rust side and the wrapper generation workflow.

If the extra `DESCRIPTION` entries look unfamiliar, read [DESCRIPTION Fields for miniextendr Packages](/manual/description-fields/). That page explains what the `Config/build/*`, `SystemRequirements`, roxygen, and testthat lines are doing there.

## Step 2: the Rust crate inside the package

The Rust code lives in:

- `tests/model_project/src/rust/Cargo.toml`
- `tests/model_project/src/rust/lib.rs`

`Cargo.toml` shows the standard shape of a nested package-local Rust crate:

- `crate-type = ["staticlib"]` for linking into the R package
- a package-local standalone `[workspace]`
- dependency on vendored `miniextendr-api`
- optional features forwarded into the runtime crate

`lib.rs` shows a more realistic exported type than a trivial function. The project defines a `World` struct, derives `ExternalPtr`, and exports methods through `#[miniextendr]`:

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct World {
    pigs: Vec<Pig>,
    tick: u32,
    next_id: u64,
    rng: rand::rngs::StdRng,
    food_per_tick: f64,
    move_cost: f64,
    reproduction_threshold: f64,
    max_age: u32,
    interaction_radius: f64,
}

#[miniextendr]
impl World {
    pub fn new(...) -> Self { ... }
    pub fn step(&mut self) { ... }
    pub fn run(&mut self, steps: i32) { ... }
    pub fn summary(&self) -> String { ... }
}
```

That is the key teaching point: the Rust side still looks like an ordinary type with methods. The R-facing API is derived from it, rather than hand-written separately.

## Step 3: the build and configure glue

The files that make this a real R package build are:

- `tests/model_project/bootstrap.R`
- `tests/model_project/configure.ac`
- `tests/model_project/configure`
- `tests/model_project/src/Makevars.in`
- `tests/model_project/src/rust/cargo-config.toml.in`
- `tests/model_project/src/stub.c`

This layer answers the questions that a toy example usually skips:

- how `R CMD INSTALL` finds Cargo and Rust
- how dev and CRAN-like modes are selected
- where cargo outputs go
- how package-local Rust config is generated
- how the static library gets linked into the final package library

`bootstrap.R` is especially useful to study because it shows how devtools workflows force a predictable configure mode before the package build starts.

## Step 4: wrapper generation and R-facing methods

The generated R API lives in:

- `tests/model_project/R/pigworld-wrappers.R`
- `tests/model_project/man/World.Rd`

This is what the user eventually interacts with from R:

```r
world <- World$new(
  n_initial = 50L,
  food_per_tick = 4,
  move_cost = 1,
  reproduction_threshold = 80,
  max_age = 100L,
  interaction_radius = 2,
  seed = 42L
)

world$run(100L)
world$summary()
```

The important thing to notice is that the wrappers are generated from the Rust impl block and Rust doc comments. The R methods, Rd entries, and method dispatch shape all come out of the Rust source.

## Step 5: how to read this project as a tutorial

If you want to follow it as a step-by-step walkthrough, read it in this order:

1. `DESCRIPTION` and `NAMESPACE` to understand the package shell.
2. `src/rust/Cargo.toml` to understand how the nested Rust crate is configured.
3. `src/rust/lib.rs` to see the actual exported Rust API.
4. `bootstrap.R` and `configure.ac` to understand how package builds are wired.
5. `R/pigworld-wrappers.R` to see what miniextendr generates for R users.
6. `man/World.Rd` to see the final user-facing documentation shape.

That sequence gives you a full-project mental model without having to infer the missing pieces from smaller examples.

## What this tutorial teaches better than a minimal example

Compared with a simple `add(a, b)` tutorial, the model project shows:

- an exported Rust-backed class instead of just free functions
- package bootstrap and configure behavior
- Cargo feature wiring in a package-local crate
- generated R method wrappers
- generated Rd output
- the actual shape of a scaffolded package you can compare against your own

## Where to go next

- Read [Getting Started](/getting-started/) for the minimal first package flow.
- Read [Packages](/packages/) if you want the repository-wide map of crates and R packages.
- Read [Package Map](/manual/packages/) for the detailed relationship between workspace crates, R packages, and fixtures.
