//! Shared utilities for handling method return values in R wrapper generation.
//!
//! This module provides helpers for generating consistent return value handling
//! across all class systems (Env, R6, S7, S3, S4).

use crate::miniextendr_impl::ParsedMethod;

// region: Shared R error-check code for error_in_r mode

/// Generate the R `if` block that checks for a tagged error value and raises a condition.
///
/// Expects `.val` to already be assigned (e.g., `.val <- .Call(...)`). Each line
/// is indented by `indent`. The check tests `inherits(.val, "rust_error_value")`
/// and `isTRUE(attr(.val, "__rust_error__"))`, then calls `stop()` with a
/// structured condition of class `c("rust_error", "simpleError", "error", "condition")`.
pub fn error_in_r_check_lines(indent: &str) -> Vec<String> {
    vec![
        format!(
            "{}if (inherits(.val, \"rust_error_value\") && isTRUE(attr(.val, \"__rust_error__\"))) {{",
            indent
        ),
        format!("{}  stop(structure(", indent),
        format!(
            "{}    class = c(\"rust_error\", \"simpleError\", \"error\", \"condition\"),",
            indent
        ),
        format!(
            "{}    list(message = .val$error, call = .val$call %||% sys.call(), kind = .val$kind)",
            indent
        ),
        format!("{}  ))", indent),
        format!("{}}}", indent),
    ]
}

/// Generate an inline R error-check block for single-expression contexts (S7, S4).
///
/// Returns a multi-line block string: `{ .val <- <call_expr>; if (...) stop(...); <inner> }`.
/// Used where the class system requires a single expression rather than separate lines
/// (e.g., S7 property definitions, S4 method bodies).
///
/// - `call_expr`: The `.Call()` expression to evaluate
/// - `inner`: The final expression to return after the error check passes
pub fn error_in_r_inline_block(call_expr: &str, inner: &str) -> String {
    format!(
        "{{\n    .val <- {call_expr}\n    \
         if (inherits(.val, \"rust_error_value\") && isTRUE(attr(.val, \"__rust_error__\"))) {{\n      \
           stop(structure(\n        \
             class = c(\"rust_error\", \"simpleError\", \"error\", \"condition\"),\n        \
             list(message = .val$error, call = .val$call %||% sys.call(), kind = .val$kind)\n      \
           ))\n    \
         }}\n    \
         {inner}\n  \
         }}"
    )
}

/// Generate a standalone-function R wrapper body for error_in_r mode.
///
/// Returns the full body string: `.val <- <call_expr>; if (...) stop(...); <final_return>`.
/// Used for top-level `#[miniextendr]` functions (not class methods).
///
/// - `call_expr`: The `.Call()` expression to evaluate
/// - `final_return`: The expression to return (typically `".val"` or `"invisible(.val)"`)
pub fn error_in_r_standalone_body(call_expr: &str, final_return: &str) -> String {
    format!(
        ".val <- {call_expr}\n  \
         if (inherits(.val, \"rust_error_value\") && isTRUE(attr(.val, \"__rust_error__\"))) {{\n    \
           stop(structure(\n      \
             class = c(\"rust_error\", \"simpleError\", \"error\", \"condition\"),\n      \
             list(message = .val$error, call = .val$call %||% sys.call(), kind = .val$kind)\n    \
           ))\n  \
         }}\n  \
         {final_return}"
    )
}
// endregion

// region: Return strategy

/// Return handling strategy for class methods.
///
/// Determines how the R wrapper function processes and returns the `.Call()` result.
/// Each class system generator uses this to produce idiomatic R return code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnStrategy {
    /// The method returns `Self`. The wrapper wraps the raw pointer result with
    /// the appropriate class attribute or creates a new class object (e.g.,
    /// `R6Class$new(.ptr = result)` or `structure(result, class = "...")`).
    ReturnSelf,
    /// The method is a `&mut self` method returning `()`. The wrapper calls the
    /// `.Call()` for its side effect and returns the receiver (`self`/`x`) for
    /// method chaining (e.g., `invisible(self)` for R6).
    ChainableMutation,
    /// Default strategy: return the `.Call()` result directly without wrapping.
    Direct,
}

impl ReturnStrategy {
    /// Determine the return strategy for a parsed method.
    ///
    /// - Methods that return `Self` use `ReturnSelf`
    /// - `&mut self` methods returning `()` use `ChainableMutation`
    /// - All other methods use `Direct`
    pub fn for_method(method: &ParsedMethod) -> Self {
        if method.returns_self() {
            ReturnStrategy::ReturnSelf
        } else if method.env.is_mut() && method.returns_unit() {
            ReturnStrategy::ChainableMutation
        } else {
            ReturnStrategy::Direct
        }
    }
}

/// Builder for generating R method body lines with appropriate return handling.
///
/// Produces lines of R code for a method body, combining the `.Call()` expression
/// with the return strategy and optional error checking. Each class system has
/// specialized builder methods (`build_r6_body`, `build_s3_body`, etc.) that
/// produce idiomatic R code for that system.
pub struct MethodReturnBuilder {
    /// The `.Call()` expression string (e.g., `".Call(C_Counter__inc, .call = match.call(), self)"`).
    call_expr: String,
    /// How to handle the return value (direct, chaining, or Self wrapping).
    strategy: ReturnStrategy,
    /// R class name, required when `strategy` is `ReturnSelf` to construct
    /// the class wrapper (e.g., `"Counter"` for `Counter$new(.ptr = result)`).
    class_name: Option<String>,
    /// Variable name to return for `ChainableMutation` strategy (e.g., `"self"` for R6,
    /// `"x"` for S3). Defaults to `"self"` if not set.
    chain_var: Option<String>,
    /// Number of leading spaces for each generated line.
    indent: usize,
    /// When `true`, generates error_in_r checking: captures the `.Call()` result in `.val`,
    /// checks for `rust_error_value` class, and raises an R condition on error.
    error_in_r: bool,
}

impl MethodReturnBuilder {
    /// Create a new builder with the given .Call expression.
    pub fn new(call_expr: String) -> Self {
        Self {
            call_expr,
            strategy: ReturnStrategy::Direct,
            class_name: None,
            chain_var: None,
            indent: 2,
            error_in_r: false,
        }
    }

    /// Set the return strategy.
    pub fn with_strategy(mut self, strategy: ReturnStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the class name (for Self returns).
    pub fn with_class_name(mut self, class_name: String) -> Self {
        self.class_name = Some(class_name);
        self
    }

    /// Set the variable name to return for chaining (default: "self").
    pub fn with_chain_var(mut self, var: String) -> Self {
        self.chain_var = Some(var);
        self
    }

    /// Set indentation level (number of spaces).
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Enable error_in_r mode: capture .Call result, check for rust_error_value,
    /// raise R condition on error.
    pub fn with_error_in_r(mut self, error_in_r: bool) -> Self {
        self.error_in_r = error_in_r;
        self
    }

    /// Generate the `if (inherits(.val, "rust_error_value") ...)` check lines.
    ///
    /// Emits an R `inherits` guard that extracts and re-raises Rust errors
    /// transported as tagged SEXP values, using the current indentation.
    fn error_check_lines(&self, indent: &str) -> Vec<String> {
        error_in_r_check_lines(indent)
    }

    /// Build R code lines for the method body.
    ///
    /// Returns a vector of strings, one per line (without trailing newlines).
    pub fn build(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
        if self.error_in_r {
            let mut lines = vec![format!("{}.val <- {}", indent, self.call_expr)];
            lines.extend(self.error_check_lines(&indent));
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    lines.push(format!("{}class(.val) <- \"{}\"", indent, class_name));
                    lines.push(format!("{}.val", indent));
                }
                ReturnStrategy::ChainableMutation => {
                    let chain_var = self.chain_var.as_deref().unwrap_or("self");
                    lines.push(format!("{}{}", indent, chain_var));
                }
                ReturnStrategy::Direct => {
                    lines.push(format!("{}.val", indent));
                }
            }
            lines
        } else {
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    vec![
                        format!("{}result <- {}", indent, self.call_expr),
                        format!("{}class(result) <- \"{}\"", indent, class_name),
                        format!("{}result", indent),
                    ]
                }
                ReturnStrategy::ChainableMutation => {
                    let chain_var = self.chain_var.as_deref().unwrap_or("self");
                    vec![
                        format!("{}{}", indent, self.call_expr),
                        format!("{}{}", indent, chain_var),
                    ]
                }
                ReturnStrategy::Direct => {
                    vec![format!("{}{}", indent, self.call_expr)]
                }
            }
        }
    }
}

/// Specialized builders for different class systems.
impl MethodReturnBuilder {
    /// Build R6-style return (uses invisible(self) for chaining).
    pub fn build_r6_body(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
        if self.error_in_r {
            let mut lines = vec![format!("{}.val <- {}", indent, self.call_expr)];
            lines.extend(self.error_check_lines(&indent));
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    lines.push(format!("{}{}$new(.ptr = .val)", indent, class_name));
                }
                ReturnStrategy::ChainableMutation => {
                    lines.push(format!("{}invisible(self)", indent));
                }
                ReturnStrategy::Direct => {
                    lines.push(format!("{}.val", indent));
                }
            }
            lines
        } else {
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    vec![format!(
                        "{}{}$new(.ptr = {})",
                        indent, class_name, self.call_expr
                    )]
                }
                ReturnStrategy::ChainableMutation => {
                    vec![
                        format!("{}{}", indent, self.call_expr),
                        format!("{}invisible(self)", indent),
                    ]
                }
                ReturnStrategy::Direct => {
                    vec![format!("{}{}", indent, self.call_expr)]
                }
            }
        }
    }

    /// Build S3-style return (uses structure() for Self returns).
    pub fn build_s3_body(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
        let chain_var = self.chain_var.as_deref().unwrap_or("x");

        if self.error_in_r {
            let mut lines = vec![format!("{}.val <- {}", indent, self.call_expr)];
            lines.extend(self.error_check_lines(&indent));
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    lines.push(format!(
                        "{}structure(.val, class = \"{}\")",
                        indent, class_name
                    ));
                }
                ReturnStrategy::ChainableMutation => {
                    lines.push(format!("{}{}", indent, chain_var));
                }
                ReturnStrategy::Direct => {
                    lines.push(format!("{}.val", indent));
                }
            }
            lines
        } else {
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    vec![format!(
                        "{}structure({}, class = \"{}\")",
                        indent, self.call_expr, class_name
                    )]
                }
                ReturnStrategy::ChainableMutation => {
                    vec![
                        format!("{}{}", indent, self.call_expr),
                        format!("{}{}", indent, chain_var),
                    ]
                }
                ReturnStrategy::Direct => {
                    vec![format!("{}{}", indent, self.call_expr)]
                }
            }
        }
    }

    /// Build S7-style return (creates new S7 object with .ptr).
    ///
    /// In error_in_r mode, returns a multi-line block expression.
    pub fn build_s7_inline(&self) -> String {
        if self.error_in_r {
            let inner = match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    format!("{}(.ptr = .val)", class_name)
                }
                ReturnStrategy::ChainableMutation => "x".to_string(),
                ReturnStrategy::Direct => ".val".to_string(),
            };
            error_in_r_inline_block(&self.call_expr, &inner)
        } else {
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    format!("{}(.ptr = {})", class_name, self.call_expr)
                }
                ReturnStrategy::ChainableMutation => {
                    format!("{{ {}; x }}", self.call_expr)
                }
                ReturnStrategy::Direct => self.call_expr.clone(),
            }
        }
    }

    /// Build S4-style return (uses methods::new()).
    ///
    /// In error_in_r mode, returns a multi-line block expression.
    pub fn build_s4_inline(&self) -> String {
        if self.error_in_r {
            let inner = match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    format!("methods::new(\"{}\", ptr = .val)", class_name)
                }
                ReturnStrategy::ChainableMutation => "x".to_string(),
                ReturnStrategy::Direct => ".val".to_string(),
            };
            error_in_r_inline_block(&self.call_expr, &inner)
        } else {
            match self.strategy {
                ReturnStrategy::ReturnSelf => {
                    let class_name = self
                        .class_name
                        .as_ref()
                        .expect("class_name required for ReturnSelf strategy");
                    format!("methods::new(\"{}\", ptr = {})", class_name, self.call_expr)
                }
                ReturnStrategy::ChainableMutation => {
                    format!("{{ {}; x }}", self.call_expr)
                }
                ReturnStrategy::Direct => self.call_expr.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests;
// endregion
