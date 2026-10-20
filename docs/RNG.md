# RNG (Random Number Generation)

Safe access to R's random number generators from Rust.

## Overview

R's RNG maintains internal state that must be synchronized with R's `.Random.seed`
variable. Before drawing, `GetRNGstate()` loads the state; after drawing,
`PutRNGstate()` saves it back -- even if an error occurs. miniextendr does both for
you, and hands you an `RngGuard`: the proof that the state is loaded on R's main
thread. Every draw is a safe method on the guard, named after R's C function.

There are two ways to get a guard: the `#[miniextendr(rng)]` attribute
(recommended for functions R calls) and `with_rng` / `RngGuard::new` for
everything else.

## Table of Contents

- [Quick Start](#quick-start)
- [The `#[miniextendr(rng)]` Attribute](#the-miniextendr-rng-attribute)
- [Seeded Simulation: `rng(seed = ..)`](#seeded-simulation-rng-seed)
- [Manual Control: with_rng and RngGuard](#manual-control-with-rng-and-rngguard)
- [Drawing from the Guard](#drawing-from-the-guard)
- [`rand` integration (`feature = "rand"`)](#rand-integration-feature-rand)
- [Combining with Other Attributes](#combining-with-other-attributes)
- [R Longjumps and Safety](#r-longjumps-and-safety)
- [Parallel Code (Rayon)](#parallel-code-rayon)

## Quick Start

<!-- docs-examples-check -->
```rust
use miniextendr_api::{RngGuard, miniextendr};

#[miniextendr(rng)]
pub fn random_sample(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}
```

From R, the guard is no argument:

```r
set.seed(42)
random_sample(5)
#> [1] 0.9148060 0.9370754 0.2861395 0.8304476 0.6417455
```

These are the values `runif(5)` gives after `set.seed(42)`.

## The `#[miniextendr(rng)]` Attribute

The recommended approach for a function R calls. Works on standalone functions and
on impl methods in every class system (env, R6, S3, S4, S7, vctrs).

The wrapper loads the state before the call and saves it after. A parameter of
type `&RngGuard`, at any position, receives the guard the wrapper holds. It is no R
argument: it is not in the R function's formals, its usage line or its `@param`
list. A `&RngGuard` parameter on a function without `rng` is a compile error,
since nothing has loaded the state for it.

### Standalone Functions

<!-- docs-examples-check -->
```rust
use miniextendr_api::{RngGuard, miniextendr};

#[miniextendr(rng)]
pub fn uniform_sample(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}

#[miniextendr(rng)]
pub fn normal_sample(rng: &RngGuard, n: i32) -> Vec<f64> {
    (0..n).map(|_| rng.norm_rand()).collect()
}
```

### Impl Methods

<!-- docs-examples-check -->
```rust
use miniextendr_api::{ExternalPtr, RngGuard, miniextendr};

#[derive(ExternalPtr)]
pub struct MySampler {
    scale: f64,
}

#[miniextendr(env)]
impl MySampler {
    pub fn new(scale: f64) -> Self {
        MySampler { scale }
    }

    #[miniextendr(rng)]
    pub fn sample_uniform(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| self.scale * rng.unif_rand()).collect()
    }

    #[miniextendr(rng)]
    pub fn static_sample(n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| rng.unif_rand()).collect()
    }
}
```

In R these are `sampler$sample_uniform(n)` and `MySampler$static_sample(n)`.

### Trait Methods

`rng` also works on a method of a `#[miniextendr] impl Trait for Type` block: the
wrapper R calls loads and saves the state around it. A trait method takes no
`&RngGuard` parameter (a compile error): its arguments cross the trait ABI as R
values, and the guard is none. Draw inside `with_rng` in the method body instead,
without `rng` on the method (#2196 tracks taking the guard there).
`rng(seed = ..)` is refused on trait and impl methods alike (a compile error):
see [Seeded Simulation](#seeded-simulation-rng-seed).

### Generated Code Pattern

The attribute generates code that:

1. Enters an outer `with_r_unwind_protect` guard and calls `GetRNGstate()`
2. Runs the body under an inner guard, returning tagged conditions on Rust panics
3. Protects that result while `PutRNGstate()` saves the state
4. Returns the value or tagged condition for the R wrapper to handle

Both RNG calls can raise R errors, so both run inside the outer guard. Saving runs
on successful returns and caught Rust panics. R-origin nonlocal exits propagate
through the guards; they do not run the remaining body or save step. The guard
passed to a `&RngGuard` parameter does not save the state when it drops: step 3
does.

A function or method that takes the guard always runs on R's main thread, whatever
`worker` or `worker-default` says: the guard is `!Send`.

**Inside an `rng` function, draw from the `&RngGuard` parameter; don't call
`with_rng`.** A nested `GetRNGstate()` reloads `.Random.seed`, which still holds
the state from before the call, so the draws would repeat.

## Seeded Simulation: `rng(seed = ..)`

A `simulate()` method takes a `seed`. With one, it draws from `set.seed(seed)`
and then puts the session's random stream back, so the caller's next `rnorm()`
continues where it was; the result records the seed in its `seed` attribute.
`#[miniextendr(rng(seed = <param>))]` writes that contract into the generated R
wrapper, in the order `stats:::simulate.lm` uses, so the body only draws:

<!-- docs-examples-check -->
```rust
use miniextendr_api::dots::Dots;
use miniextendr_api::{RngGuard, SEXP, miniextendr};

/// @param object A `my_fit`.
/// @param nsim Number of draws.
/// @param seed `NULL`, or a seed for `set.seed()`.
/// @param ... Unused.
#[miniextendr(rng(seed = _seed), s3(generic = "stats::simulate", class = "my_fit"))]
pub fn simulate_my_fit(
    _object: SEXP,
    #[miniextendr(default = "1")] nsim: usize,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    _dots: &Dots,
    rng: &RngGuard,
) -> Vec<f64> {
    (0..nsim).map(|_| rng.norm_rand()).collect()
}
```

```r
fit <- structure(list(), class = "my_fit")
set.seed(1)
before <- .Random.seed
x <- simulate(fit, nsim = 3, seed = 42)
identical(.Random.seed, before)  # TRUE: the session's stream is back
attr(x, "seed")                  # 42, with attribute kind = as.list(RNGkind())
```

The draws are what `set.seed(42); rnorm(3)` gives. `<param>` names the seed
parameter by its Rust name, and the wrapper uses its formal (`_seed` is
`seed`). The option implies `rng`.

A package that uses `rng(seed = ..)` lists `stats` in `Imports:`, because the
wrapper calls `stats::runif()`. Name the generic `stats::simulate`: the method
is then registered when stats' namespace loads, so the package's namespace
loads with only its stated dependencies (`R CMD check` loads it with
`R_DEFAULT_PACKAGES=NULL`, where a bare `simulate` is "not found"; see
[S3_METHODS.md](S3_METHODS.md#generics-from-a-suggests-package)).

After the argument checks, the wrapper runs:

```r
if (!exists(".Random.seed", envir = globalenv(), inherits = FALSE)) stats::runif(1)
if (is.null(seed)) {
  .mx_rng_state <- get(".Random.seed", envir = globalenv())
} else {
  .mx_r_seed <- get(".Random.seed", envir = globalenv())
  on.exit(assign(".Random.seed", .mx_r_seed, envir = globalenv()), add = TRUE)
  set.seed(seed)
  .mx_rng_state <- structure(seed, kind = as.list(RNGkind()))
}
.val <- .Call(...)
# ... the result's condition check ...
if (!is.null(.val) && (is.atomic(.val) || is.list(.val))) attr(.val, "seed") <- .mx_rng_state
.val
```

The contract, as `stats::simulate()` has it:

- `.Random.seed` is created when it is missing. A refused argument changes no
  RNG state: the argument checks come first.
- With a seed, the session's `.Random.seed` is put back on exit, also when the
  body returns an error or panics after drawing. The C wrapper loads the state
  after `set.seed()` and saves it inside `.Call()`, before `on.exit()` restores
  the session's state. With `seed = NULL` the draws continue the session's
  stream and stay drawn, as under plain `rng`.
- `set.seed()` validates the seed: the wrapper passes the value through, and a
  bad seed is R's own error. So the seed parameter has no type check of its own
  in the wrapper, and its Rust conversion runs after `set.seed()`. Give it a
  type that takes the seeds callers pass: `Option<i64>` takes `NULL`, `42` and
  `42L`, while `Option<i32>` takes integers only, so `seed = 42` would fail its
  conversion.
- The `seed` attribute is the seed with attribute `kind = as.list(RNGkind())`,
  or with `seed = NULL` the `.Random.seed` from before the draws. A result that
  can't carry attributes (`NULL`, an environment, an external pointer, a
  function) is left as it is.

The body gets the seed parameter like any other, and draws through
`&RngGuard`. `rng(seed = ..)` is for standalone functions and their `s3(...)`
methods; impl and trait methods refuse it with a compile error. It is also a
compile error when the name is no parameter, when the parameter is no R argument
the wrapper evaluates (`...`, `NArgs`, a `Call` marker, the `&RngGuard`, a
`Quoted` / `Quosure` argument), and when the function returns `()` (or
`Result<(), E>`, `Option<()>`), whose `NULL` can't carry the attribute.

Use it for an R-facing function whose caller passes a seed, as `simulate()`
methods do. A function without a seed argument needs only `rng`: the caller's
own `set.seed()` decides its draws.

## Manual Control: with_rng and RngGuard

For internal helper functions or code that needs finer control over RNG scope.

### with_rng

Runs a closure with the state loaded and passes it the guard:

<!-- docs-examples-check -->
```rust
use miniextendr_api::with_rng;

fn ten_uniforms() -> Vec<f64> {
    with_rng(|rng| (0..10).map(|_| rng.unif_rand()).collect())
}
```

`with_rng` loads the state when it is called, not earlier, so a `set.seed()` that
ran in R before the call decides the draws. It saves the state when the closure
returns or panics, and panics itself when called off R's main thread.

### RngGuard

`RngGuard::new()` calls `GetRNGstate()`, and dropping the guard calls
`PutRNGstate()`:

<!-- docs-examples-check -->
```rust
use miniextendr_api::RngGuard;

fn generate_random() -> f64 {
    // SAFETY: this helper is called on R's main thread.
    let rng = unsafe { RngGuard::new() };
    rng.unif_rand()
    // PutRNGstate() called automatically when `rng` drops
}
```

`RngGuard::new()` is `unsafe` because only the caller can prove it is running on
R's main thread. `with_rng` performs that check at runtime. Both forms still rely
on Rust destructors, so neither is a replacement for `#[miniextendr(rng)]` at an
R-facing boundary.

### When to Use Manual vs Attribute

| Scenario | Use |
|----------|-----|
| Function exposed to R | `#[miniextendr(rng)]` with a `&RngGuard` parameter |
| Internal helper (not `#[miniextendr]`) | `with_rng` or `RngGuard` |
| Code already inside `with_r_unwind_protect` | `with_rng` or `RngGuard` |
| Scoped RNG within a larger function | `RngGuard` |

## Drawing from the Guard

Each draw borrows the guard, so it can't run outside the guard's scope or on
another thread:

| Method | R's C function | Distribution | Same draw as |
|--------|----------------|--------------|--------------|
| `rng.unif_rand()` | `unif_rand()` | Uniform on `(0, 1)` | `runif(1)` |
| `rng.norm_rand()` | `norm_rand()` | Standard normal | `rnorm(1)` |
| `rng.exp_rand()` | `exp_rand()` | Standard exponential | `rexp(1)` |
| `rng.unif_index(n)` | `R_unif_index(n)` | Uniform integer on `[0, n)`, as a double | `sample.int(n, 1) - 1` |

<!-- docs-examples-check -->
```rust
use miniextendr_api::{RngGuard, miniextendr};

#[miniextendr(rng)]
pub fn rng_demo(rng: &RngGuard) -> Vec<f64> {
    vec![
        rng.unif_rand(),        // Uniform on (0, 1)
        rng.norm_rand(),        // Standard normal
        rng.exp_rand(),         // Standard exponential
        rng.unif_index(100.0),  // Uniform integer on [0, 100)
    ]
}
```

After `set.seed(1)`, `rng_demo()` returns
`c(runif(1), rnorm(1), rexp(1), sample.int(100, 1) - 1)` as drawn after the same
`set.seed(1)`.

The raw functions are still in `miniextendr_api::sys` (`unsafe`, and they assume
the state is loaded); the guard methods make them safe.

## `rand` integration (`feature = "rand"`)

Enable `rand` when downstream Rust code expects a `rand` RNG, or when you want
typed access to R's native distributions through a trait:

```toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["rand"] }
```

`RRng` draws from R's RNG, so `set.seed()` remains reproducible. It borrows a
guard: `guard.rng()` makes one, and it can't outlive the guard.

```rust
use miniextendr_api::{RDistributions, RngGuard, miniextendr};
use miniextendr_api::rand::RngExt;

#[miniextendr(rng)]
pub fn rand_sample(n: i32, guard: &RngGuard) -> Vec<f64> {
    let mut rng = guard.rng();
    (0..n)
        .map(|_| rng.random::<f64>())
        .chain([rng.standard_normal(), rng.standard_exp()])
        .collect()
}
```

`RRng` supplies the generic `rand` interface; `RDistributions` calls R's
`norm_rand`, `exp_rand`, `R_unif_index`, and `unif_rand` implementations directly,
so `standard_normal()` draws what `rnorm(1)` draws. `rand`'s own
`random::<f64>()` takes 64 bits, two `unif_rand()` draws, so it does not match
`runif()`.

`RRng` is `!Send` and `!Sync`, like the guard it borrows: it can't move into a
worker closure or a rayon task. Outside an `rng` function, make one inside
`with_rng`: `with_rng(|guard| guard.rng().standard_normal())`.

The same feature exposes `RRngOps` and `RDistributionOps` for exporting custom
RNG and distribution objects through the trait ABI. `rand_distr` additionally
re-exports the `rand_distr` crate for its probability distributions.

## Combining with Other Attributes

`rng` composes with other `#[miniextendr]` options:

<!-- docs-examples-check -->
```rust
use miniextendr_api::{RngGuard, miniextendr};

// RNG + interrupt checking
#[miniextendr(rng, check_interrupt)]
pub fn interruptible_random(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}
```

Under the deprecated `worker` option the body runs on the worker thread, where no
guard can go. A function that takes `&RngGuard` stays on R's main thread
regardless.

## R Longjumps and Safety

`RngGuard` and `with_rng` rely on Rust's drop semantics. If R triggers a longjmp
(via `Rf_error`, etc.), the guard's destructor will NOT run unless the code is
wrapped in `with_r_unwind_protect`.

This is why **`#[miniextendr(rng)]` is preferred for R-exposed functions** -- the
outer unwind guard covers loading and saving the state, and `PutRNGstate()` runs
after the body's own guard has turned a panic into a value.

Summary:

| Approach | Handles panics | Handles R longjumps |
|----------|---------------|-------------------|
| `#[miniextendr(rng)]` | Yes | Yes |
| `RngGuard` / `with_rng` | Yes (drop runs) | Only inside `with_r_unwind_protect` |

## Parallel Code (Rayon)

R's RNG is **not thread-safe**. A guard (and an `RRng` borrowing one) is `!Send`
and `!Sync`, so a rayon closure that draws from it does not compile. For parallel
random number generation, use a Rust RNG with deterministic per-chunk seeding:

```rust
use miniextendr_api::rand::{RngExt, SeedableRng, rngs::StdRng};
use miniextendr_api::rayon_bridge::with_r_vec;

#[miniextendr]
fn parallel_random(len: usize, seed: u64) -> SEXP {
    with_r_vec(len, move |chunk: &mut [f64], offset| {
        let offset = u64::try_from(offset).expect("R vector offset fits u64");
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(offset));
        for slot in chunk.iter_mut() {
            *slot = rng.random();
        }
    })
}
```

See [Rayon](RAYON.md#rng-reproducibility) for full details on reproducible parallel RNG.

## See Also

- [Rayon Integration](RAYON.md) -- Parallel computation (including RNG reproducibility)
- [`#[miniextendr]` Attribute](MINIEXTENDR_ATTRIBUTE.md) -- Complete attribute reference
- [Threads](THREADS.md) -- Worker thread architecture
