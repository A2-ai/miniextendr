//! Shared utilities for handling method return values in R wrapper generation.
//!
//! This module provides helpers for generating consistent return value handling
//! across all class systems (Env, R6, S7, S3, S4).

use crate::miniextendr_impl::{ParsedMethod, ReceiverKind};

/// Return handling strategy for methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnStrategy {
    /// Return Self: wrap result with class attribute or create new object
    ReturnSelf,
    /// Mutable method returning unit: return self/x for chaining
    ChainableMutation,
    /// Default: return result directly
    Direct,
}

impl ReturnStrategy {
    /// Determine the return strategy for a method.
    pub fn for_method(method: &ParsedMethod) -> Self {
        if method.returns_self() {
            ReturnStrategy::ReturnSelf
        } else if method.env == ReceiverKind::RefMut && method.returns_unit() {
            ReturnStrategy::ChainableMutation
        } else {
            ReturnStrategy::Direct
        }
    }
}

/// Builder for generating R method body with appropriate return handling.
pub struct MethodReturnBuilder {
    /// The .Call expression (e.g., ".Call(C_Counter__inc, self)")
    call_expr: String,
    /// Return handling strategy
    strategy: ReturnStrategy,
    /// Class name (for wrapping Self returns)
    class_name: Option<String>,
    /// Variable name to return for chaining (e.g., "self" or "x")
    chain_var: Option<String>,
    /// Indentation level (number of spaces)
    indent: usize,
}

impl MethodReturnBuilder {
    /// Create a new builder with the given .Call expression.
    pub fn new(call_expr: String) -> Self {
        Self {
            call_expr,
            strategy: ReturnStrategy::Direct,
            class_name: None,
            chain_var: None,
            indent: 4,
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

    /// Build R code lines for the method body.
    ///
    /// Returns a vector of strings, one per line (without trailing newlines).
    pub fn build(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
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

/// Specialized builders for different class systems.
impl MethodReturnBuilder {
    /// Build R6-style return (uses invisible(self) for chaining).
    pub fn build_r6_body(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
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

    /// Build S3-style return (uses structure() for Self returns).
    pub fn build_s3_body(&self) -> Vec<String> {
        let indent = " ".repeat(self.indent);
        let chain_var = self.chain_var.as_deref().unwrap_or("x");

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

    /// Build S7-style return (creates new S7 object with .ptr).
    pub fn build_s7_inline(&self) -> String {
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

    /// Build S4-style return (uses methods::new()).
    pub fn build_s4_inline(&self) -> String {
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

#[cfg(test)]
mod tests;
