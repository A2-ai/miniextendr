//! Structured diagnostic output for lint rules.

use std::fmt;
use std::path::PathBuf;

use crate::lint_code::LintCode;

/// Diagnostic severity level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Migration hints and informational notes.
    Info,
    /// Default for new rules; non-blocking.
    Warning,
    /// CI-blocking in strict mode.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => f.write_str("info"),
            Self::Warning => f.write_str("warning"),
            Self::Error => f.write_str("error"),
        }
    }
}

/// A single lint diagnostic with structured metadata.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Stable rule code (e.g. `MXL101`).
    pub code: LintCode,
    /// Severity level.
    pub severity: Severity,
    /// Source file path.
    pub path: PathBuf,
    /// 1-based line number (0 if unknown).
    pub line: usize,
    /// Primary diagnostic message.
    pub message: String,
    /// Optional fix guidance.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new diagnostic with the rule's default severity.
    pub fn new(code: LintCode, path: impl Into<PathBuf>, line: usize, message: String) -> Self {
        Self {
            severity: code.default_severity(),
            code,
            path: path.into(),
            line,
            message,
            help: None,
        }
    }

    /// Attach a help message.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Format as a legacy error string (for backward-compatible `LintReport::errors`).
    pub fn to_legacy_string(&self) -> String {
        let mut s = format!("{}:{}: {}", self.path.display(), self.line, self.message);
        if let Some(ref help) = self.help {
            s.push(' ');
            s.push_str(help);
        }
        s
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}:{}: {}",
            self.code,
            self.path.display(),
            self.line,
            self.message,
        )?;
        if let Some(ref help) = self.help {
            write!(f, " Help: {}", help)?;
        }
        Ok(())
    }
}
