+++
title = "Full Project Tutorial"
weight = 4
description = "Walk through the pigworld model project from package skeleton to Rust runtime, configure glue, and generated R wrappers."
+++

This guide uses `tests/model_project/` as a concrete end-to-end example. It is a more instructive reference than a minimal hello-world because it shows a full package with configure logic, a nested Rust crate, generated R wrappers, and a nontrivial Rust type exported to R.

## Why this project is useful

The model project is a scaffolded package called `pigworld`. It includes:

- package metadata in `DESCRIPTION`, `NAMESPACE`, and `R/`
- a nested Rust crate in `src/rust/`
- configure/bootstrap/build glue for the R package build
- generated R wrappers and roxygen output
- a real exported Rust-backed `World` type instead of a single toy function

It shows the whole shape of a package, not just one boundary crossing.

## Step 1: the package shell

Start with the R package files:

- `tests/model_project/DESCRIPTION`
- `tests/model_project/NAMESPACE`
- `tests/model_project/R/pigworld-package.R`

These establish the installed package name (`pigworld`), register the dynamic library, and provide the minimal R-side package shell that the generated wrappers plug into.

This layer should look familiar to an R package author. miniextendr does not replace the package shell; it fills in the Rust side and the wrapper generation workflow.

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

`lib.rs` shows a more realistic exported type than a trivial function. The project defines a `World` struct, derives `ExternalPtr`, and exports methods through `#[miniextendr]`. The file starts with the package entry point, then the struct and impl:

```rust
use miniextendr_api::miniextendr;

// Required: generates the R_init_pigworld entry point that R calls on package load.
miniextendr_api::miniextendr_init!(pigworld);

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
    pub fn new(
        n_initial: i32,
        food_per_tick: f64,
        move_cost: f64,
        reproduction_threshold: f64,
        max_age: i32,
        interaction_radius: f64,
        seed: i32,
    ) -> Self { ... }

    pub fn step(&mut self) { ... }
    pub fn run(&mut self, steps: i32) { ... }
    pub fn summary(&self) -> String { ... }
    // ... plus get_tick, population, x_positions, y_positions, energies, ages, age_histogram
}
```

The key teaching point: the Rust side looks like an ordinary type with methods. The R-facing API is derived from it, not hand-written separately. Every `pub fn` in the `impl` block becomes an R method; every doc comment becomes an `@param`/`@return` entry in the generated `.Rd` file.

`#[distributed_slice]` registration is automatic. No `miniextendr_module!` macro or manual registration call is needed beyond `miniextendr_init!` at the top of `lib.rs`.

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

**Dev mode prerequisite.** In the miniextendr monorepo, always run `bash ./configure` (via `just configure`) before any R CMD operation. Configure generates `Makevars` from `.in` templates and, in dev mode, syncs workspace crates into `src/rust/vendor/`. Skipping this step causes stale or missing build artifacts.

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

The wrappers are generated from the Rust impl block and Rust doc comments. The R methods, Rd entries, and method dispatch shape all come out of the Rust source.

**Regenerating wrappers.** Whenever you change a `#[miniextendr]` function, add a parameter, or edit a doc comment, regenerate the wrappers:

```bash
just configure && just rcmdinstall && just devtools-document
```

The generated file (`R/pigworld-wrappers.R`) and the documentation (`man/World.Rd`, `NAMESPACE`) must be committed together with the Rust changes that produced them.

**Running tests.** The package ships R tests in `tests/testthat/`. Run them with:

```bash
just devtools-test
```

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
- Read [Class Systems](/class-systems/) for env, R6, S3, S4, S7, and vctrs class generation.
- Browse the [API reference](/api/) for per-crate rustdoc.
- See the [GitHub issues](https://github.com/A2-ai/miniextendr/issues) for open work.
