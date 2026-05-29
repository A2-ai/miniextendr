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
