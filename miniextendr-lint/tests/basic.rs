use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use miniextendr_lint::{lint_enabled, run};

// Tests that mutate env vars should not run in parallel.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests;
