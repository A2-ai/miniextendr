//! Fixture for functional (native-pipe) builder support.
//!
//! Demonstrates that an in-place Rust builder — `&mut self -> &mut Self`
//! methods plus a terminal `build(&self) -> T` — maps to pipe-friendly S3 free
//! functions in R, so the idiom
//!
//! ```r
//! greeting <- new_greetingbuilder() |>
//!   builder_set_name("World") |>
//!   builder_set_punctuation("!") |>
//!   builder_set_loud(TRUE)
//! builder_build(greeting)   # "HELLO, WORLD!"
//! ```
//!
//! works end-to-end. Each `&mut self -> &mut Self` step mutates the underlying
//! Rust value in place and the C wrapper returns the *same* ExternalPtr handle
//! (no clone), so the S3 object the user piped in flows unchanged through the
//! chain. The terminal `builder_build()` reads `&self` and returns a `String`,
//! converted to R via the usual `IntoR` path.

use miniextendr_api::miniextendr;

/// A small greeting builder used to exercise native-pipe (`|>`) chaining.
#[derive(miniextendr_api::ExternalPtr)]
pub struct GreetingBuilder {
    name: String,
    punctuation: String,
    loud: bool,
}

/// Builder for a greeting string, demonstrating functional pipe chaining.
///
/// The `builder_set_*` methods return `&mut Self`, so they compose under R's
/// native pipe operator `|>` as free functions taking the object first.
/// @param x A `GreetingBuilder` object.
/// @param ... Additional arguments.
#[allow(clippy::new_without_default)]
#[miniextendr(s3)]
impl GreetingBuilder {
    /// Create a new greeting builder with empty defaults.
    pub fn new() -> Self {
        GreetingBuilder {
            name: String::new(),
            punctuation: String::from("."),
            loud: false,
        }
    }

    /// Set the name to greet. Returns the builder for chaining.
    /// @param name The name to greet.
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    /// Set the trailing punctuation. Returns the builder for chaining.
    /// @param punctuation The trailing punctuation string.
    pub fn set_punctuation(&mut self, punctuation: String) -> &mut Self {
        self.punctuation = punctuation;
        self
    }

    /// Toggle whether the greeting is shouted (upper-cased). Returns the builder.
    /// @param loud Whether to upper-case the greeting.
    pub fn set_loud(&mut self, loud: bool) -> &mut Self {
        self.loud = loud;
        self
    }

    /// Terminal step: render the configured greeting as a string.
    ///
    /// Takes `&self` (not `self`) so the R object remains valid afterwards, and
    /// returns a different type (`String`) converted to R via `IntoR`.
    pub fn build(&self) -> String {
        let name = if self.name.is_empty() {
            "world"
        } else {
            &self.name
        };
        let greeting = format!("Hello, {}{}", name, self.punctuation);
        if self.loud {
            greeting.to_uppercase()
        } else {
            greeting
        }
    }
}

/// An in-place counter demonstrating `&mut self -> &mut Self` on an integer
/// payload (no terminal type-change), so the chain returns the object itself.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PipeCounter {
    value: i32,
}

/// Counter with pipe-friendly mutators returning `&mut Self`.
/// @param x A `PipeCounter` object.
/// @param ... Additional arguments.
#[miniextendr(s3)]
impl PipeCounter {
    /// Create a counter starting at the given value.
    /// @param initial Initial counter value.
    pub fn new(initial: i32) -> Self {
        PipeCounter { value: initial }
    }

    /// Add `amount` to the counter in place. Returns the counter for chaining.
    /// @param amount Amount to add.
    pub fn bump(&mut self, amount: i32) -> &mut Self {
        self.value += amount;
        self
    }

    /// Double the counter in place. Returns the counter for chaining.
    pub fn twice(&mut self) -> &mut Self {
        self.value *= 2;
        self
    }

    /// Read the current value (terminal accessor).
    pub fn peek(&self) -> i32 {
        self.value
    }
}

// region: R6 self-ref builder fixture
//
// R6 is reference-semantic: a `&mut self -> &mut Self` builder must chain via
// `invisible(self)` so `obj$step(1)$step(2)$total()` reads through the *same*
// R6 wrapper environment. The codegen must NOT re-wrap via
// `R6PipeBuilder$new(.ptr = .val)` — that would mint a new R6 environment
// around the same pointer and break object identity. See issue #769.

/// A small accumulating builder exposed as an R6 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6PipeBuilder {
    total: i32,
}

/// R6 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains as `b$add(1L)$add(2L)$total()`.
#[miniextendr(r6)]
impl R6PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        R6PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: S4 self-ref builder fixture
//
// S4 self-ref builders generate free generics whose body returns the receiver
// `x` for chaining (S4 `ExternalPtr` objects are reference-semantic). Method
// names are auto-prefixed with `s4_`, so `add` -> `s4_add` (MXL111: never name
// the Rust method `s4_*`).

/// A small accumulating builder exposed as an S4 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4PipeBuilder {
    total: i32,
}

/// S4 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains under the native pipe as `b |> s4_add(1L) |> s4_add(2L) |> s4_total()`.
/// @aliases s4_add,S4PipeBuilder-method s4_total,S4PipeBuilder-method
/// @param x A `S4PipeBuilder` object.
#[miniextendr(s4, internal)]
impl S4PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        S4PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: S7 self-ref builder fixture
//
// S7 self-ref builders generate free generics whose body returns the receiver
// `x` for chaining. S7 method names are NOT auto-prefixed, so we name them
// `s7_*` explicitly to keep the user-facing generics in a clean namespace.

/// A small accumulating builder exposed as an S7 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7PipeBuilder {
    total: i32,
}

/// S7 builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains under the native pipe as `b |> s7_add(1L) |> s7_add(2L) |> s7_total()`.
#[miniextendr(s7, internal)]
impl S7PipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        S7PipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn s7_add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn s7_total(&self) -> i32 {
        self.total
    }
}
// endregion

// region: Cross-class return builder fixtures
//
// Terminal builder methods that materialize a different ExternalPtr-backed
// class should return a ready-to-use R wrapper, not a bare externalptr.

/// Board materialized by `R6CrossPlan::build`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6CrossBoard {
    width: i32,
    height: i32,
    seed: i32,
}

/// R6 board target for cross-class builder returns.
#[miniextendr(r6)]
impl R6CrossBoard {
    /// Create a board directly.
    /// @param width Board width.
    /// @param height Board height.
    /// @param seed Seed carried from the plan.
    pub fn new(width: i32, height: i32, seed: i32) -> Self {
        R6CrossBoard {
            width,
            height,
            seed,
        }
    }

    /// Count the cells in the board.
    pub fn cells(&self) -> i32 {
        self.width * self.height
    }

    /// Summarize the board dimensions and seed.
    pub fn signature(&self) -> String {
        format!("{}x{}@{}", self.width, self.height, self.seed)
    }
}

/// R6 builder whose terminal `build()` returns a different R6 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6CrossPlan {
    seed: i32,
}

/// R6 builder fixture for cross-class return wrapping.
#[miniextendr(r6)]
impl R6CrossPlan {
    /// Create a cross-class plan.
    /// @param seed Seed to carry into built boards.
    pub fn new(seed: i32) -> Self {
        R6CrossPlan { seed }
    }

    /// Materialize this plan into an `R6CrossBoard`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn build(&self, width: i32, height: i32) -> R6CrossBoard {
        R6CrossBoard {
            width,
            height,
            seed: self.seed,
        }
    }

    /// Materialize this plan into an `S7CrossBoard` (R6 source, S7 target).
    ///
    /// The write-time resolver keys off the *returned* class, so the emitted
    /// wrapper uses S7's `S7CrossBoard(.ptr = .val)` constructor even though
    /// the enclosing method lives on an R6 class.
    /// @param width Board width.
    /// @param height Board height.
    pub fn build_s7(&self, width: i32, height: i32) -> S7CrossBoard {
        S7CrossBoard {
            cells: self.seed + (width * height),
        }
    }
}

/// Board materialized by `S7CrossPlan::s7_cross_build`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7CrossBoard {
    cells: i32,
}

/// S7 board target for cross-class builder returns.
#[miniextendr(s7)]
impl S7CrossBoard {
    /// Create a board directly.
    /// @param cells Number of cells.
    pub fn new(cells: i32) -> Self {
        S7CrossBoard { cells }
    }

    /// Count the cells in the board.
    pub fn s7_cross_cells(&self) -> i32 {
        self.cells
    }
}

/// S7 builder whose terminal method returns a different S7 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7CrossPlan {
    seed: i32,
}

/// S7 builder fixture for cross-class return wrapping.
#[miniextendr(s7)]
impl S7CrossPlan {
    /// Create a cross-class plan.
    /// @param seed Seed added to built board cells.
    pub fn new(seed: i32) -> Self {
        S7CrossPlan { seed }
    }

    /// Materialize this plan into an `S7CrossBoard`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn s7_cross_build(&self, width: i32, height: i32) -> S7CrossBoard {
        S7CrossBoard {
            cells: self.seed + (width * height),
        }
    }

    /// Materialize this plan into an `R6CrossBoard` (S7 source, R6 target).
    ///
    /// Mirror of [`R6CrossPlan::build_s7`]: the returned class drives the
    /// constructor choice, so this S7 method's wrapper builds the R6 target via
    /// `R6CrossBoard$new(.ptr = .val)`.
    /// @param width Board width.
    /// @param height Board height.
    pub fn s7_build_r6(&self, width: i32, height: i32) -> R6CrossBoard {
        R6CrossBoard {
            width,
            height,
            seed: self.seed,
        }
    }
}
// endregion

// region: Env self-ref builder fixture
//
// Env classes share `ChainableMutation` semantics with R6: the method body
// returns `self` (the environment). Chains as `b$add(1L)$add(2L)$total()`
// through the `$` re-parenting dispatch.

/// A small accumulating builder exposed as an env-style class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EnvPipeBuilder {
    total: i32,
}

/// Env builder with a `&mut self -> &mut Self` step and a terminal accessor.
/// Chains as `b$add(1L)$add(2L)$total()`.
#[miniextendr(env)]
impl EnvPipeBuilder {
    /// Create a new builder starting at zero.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EnvPipeBuilder { total: 0 }
    }

    /// Add `amount` in place and return the builder for chaining.
    /// @param amount Amount to add.
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.total += amount;
        self
    }

    /// Terminal accessor: read the accumulated total.
    pub fn total(&self) -> i32 {
        self.total
    }
}
// endregion
