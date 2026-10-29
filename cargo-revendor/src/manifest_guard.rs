//! RAII guard for transient `Cargo.toml` mutations.
//!
//! `run_cargo_vendor` (vendor.rs) temporarily appends a `[patch.crates-io]`
//! block to the workspace manifest, and `package_local_crates` (package.rs)
//! adds `version` keys to a crate's path dependencies; both shell out to
//! cargo, then restore the original. Without a guard, a panic,
//! `?`-propagated error, or SIGINT between the mutation and the restore
//! leaves the user's `Cargo.toml` pointing at paths that don't exist yet —
//! a confusing state that requires manual `git checkout` to recover.
//!
//! `ManifestGuard` captures the original bytes at construction and restores
//! them in `Drop`, so both the success path and any unwinding path (panic,
//! `?` return) end with the manifest back to its pre-mutation state.
//!
//! **Limitation**: `Drop` does NOT run on `std::process::abort()` or on
//! SIGKILL. That residual gap is accepted — Rust's drop model cannot cover
//! uncatchable signals, and the common cases (Ctrl-C via SIGINT, cargo
//! process crash, panic!) do trigger unwinding and thus the restore.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Snapshot + auto-restore of a single file.
///
/// Construct before any mutation to the file; the snapshotted bytes are
/// restored unconditionally when the guard is dropped — including on panic
/// unwind or early `?` return.
///
/// Use `finish()` to dismiss the guard when the mutation is intentional and
/// should persist (rare — typical cargo-revendor usage is always transient).
pub struct ManifestGuard {
    path: PathBuf,
    original: Vec<u8>,
    active: bool,
}

impl ManifestGuard {
    /// Capture the file's current bytes. The guard arms immediately — any
    /// subsequent mutation to the file will be reverted on drop.
    pub fn snapshot(path: &Path) -> Result<Self> {
        let original = std::fs::read(path)
            .with_context(|| format!("failed to snapshot {}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            original,
            active: true,
        })
    }

    /// Dismiss the guard so Drop does nothing. Use when the mutation is
    /// intended to persist (cargo-revendor doesn't currently do this, but
    /// the option is here for future callers).
    #[allow(dead_code)]
    pub fn finish(mut self) {
        self.active = false;
    }
}

impl Drop for ManifestGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        // Best-effort restore. If writing back fails (disk full, permission
        // issue), there's not much we can do from inside Drop — panicking
        // here would abort the process. Print a diagnostic so the user at
        // least knows their manifest needs manual recovery.
        if let Err(e) = std::fs::write(&self.path, &self.original) {
            eprintln!(
                "warning: ManifestGuard failed to restore {}: {e} — \
                 recover with `git checkout {}`",
                self.path.display(),
                self.path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests;
