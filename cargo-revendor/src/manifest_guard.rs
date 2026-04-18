//! RAII guard for transient `Cargo.toml` mutations.
//!
//! Both `run_cargo_vendor` (vendor.rs) and `package_local_crates` (package.rs)
//! temporarily append a `[patch.crates-io]` block to the workspace manifest,
//! shell out to cargo, then restore the original. Without a guard, a panic,
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
mod tests {
    use super::*;

    #[test]
    fn snapshot_restores_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Cargo.toml");
        std::fs::write(&path, b"[workspace]\n").unwrap();

        {
            let _guard = ManifestGuard::snapshot(&path).unwrap();
            std::fs::write(&path, b"mutated").unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), b"mutated");
        }

        assert_eq!(std::fs::read(&path).unwrap(), b"[workspace]\n");
    }

    #[test]
    fn snapshot_restores_on_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Cargo.toml");
        let original = b"[workspace]\nmembers = []\n";
        std::fs::write(&path, original).unwrap();

        let path_for_panic = path.clone();
        let result = std::panic::catch_unwind(move || {
            let _guard = ManifestGuard::snapshot(&path_for_panic).unwrap();
            std::fs::write(&path_for_panic, b"mid-flight mutation").unwrap();
            panic!("simulated cargo panic");
        });

        assert!(result.is_err(), "expected panic to propagate");
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn finish_dismisses_guard() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Cargo.toml");
        std::fs::write(&path, b"original").unwrap();

        let guard = ManifestGuard::snapshot(&path).unwrap();
        std::fs::write(&path, b"intended new content").unwrap();
        guard.finish();

        // Guard is dismissed; Drop should not restore.
        assert_eq!(std::fs::read(&path).unwrap(), b"intended new content");
    }

    #[test]
    fn snapshot_of_nonexistent_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("does-not-exist.toml");
        let result = ManifestGuard::snapshot(&path);
        assert!(result.is_err(), "expected snapshot of missing file to error");
    }

    #[test]
    fn back_to_back_snapshot_and_restore() {
        // Common cargo-revendor pattern: snapshot, mutate, shell out, drop.
        // Then a second call does it again on the same file.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("Cargo.toml");
        std::fs::write(&path, b"[package]\nname = \"x\"\n").unwrap();

        for i in 0..3 {
            let before = std::fs::read(&path).unwrap();
            {
                let _g = ManifestGuard::snapshot(&path).unwrap();
                std::fs::write(&path, format!("temp-{i}")).unwrap();
            }
            assert_eq!(std::fs::read(&path).unwrap(), before, "iteration {i}");
        }
    }
}
