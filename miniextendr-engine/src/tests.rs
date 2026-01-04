use super::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

// Changing environment variables is process‑global; run these serially.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn sentinel_is_false_before_initialization() {
    assert!(
        !r_initialized_sentinel(),
        "sentinel should be false before R is initialized"
    );
}

#[cfg(unix)]
#[test]
fn ensure_r_home_env_sets_env_with_fake_r() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Save and clear the current R_HOME/PATH so we can restore them after the test.
    let original_r_home = std::env::var_os("R_HOME");
    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::remove_var("R_HOME") };

    // Create a fake `R` executable that responds to `R RHOME`.
    let tmp = tempfile::tempdir().expect("tempdir");
    let fake_r_path: PathBuf = tmp.path().join("R");
    let mut script = File::create(&fake_r_path).expect("create fake R script");
    writeln!(script, "#!/bin/sh\necho /fake/r/home").unwrap();
    // Make it executable.
    let mut perms = script.metadata().unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&fake_r_path, perms).unwrap();
    }

    // Prepend the temp dir to PATH so Command::new("R") finds our stub.
    let new_path = format!("{}:{}", tmp.path().display(), original_path);
    unsafe { std::env::set_var("PATH", &new_path) };

    ensure_r_home_env(None).expect("fake R should set R_HOME");
    assert_eq!(std::env::var("R_HOME").unwrap(), "/fake/r/home");

    // Restore environment
    match original_r_home {
        Some(val) => unsafe { std::env::set_var("R_HOME", val) },
        None => unsafe { std::env::remove_var("R_HOME") },
    }
    unsafe { std::env::set_var("PATH", original_path) };
}

#[test]
fn ensure_r_home_env_errors_when_r_missing() {
    let _guard = ENV_LOCK.lock().unwrap();

    let original_r_home = std::env::var_os("R_HOME");
    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::remove_var("R_HOME") };

    // Use an empty temp dir so `R` cannot be found.
    let tmp = tempfile::tempdir().expect("tempdir");
    unsafe { std::env::set_var("PATH", tmp.path()) };

    let err = ensure_r_home_env(None).expect_err("should fail when R is missing");
    assert!(matches!(err, REngineError::RHomeNotFound { stderr: _ }));

    match original_r_home {
        Some(val) => unsafe { std::env::set_var("R_HOME", val) },
        None => unsafe { std::env::remove_var("R_HOME") },
    }
    unsafe { std::env::set_var("PATH", original_path) };
}

#[test]
fn ensure_r_home_env_uses_explicit_path() {
    let _guard = ENV_LOCK.lock().unwrap();

    let original_r_home = std::env::var_os("R_HOME");
    unsafe { std::env::remove_var("R_HOME") };

    let explicit_path = PathBuf::from("/explicit/r/home");
    ensure_r_home_env(Some(&explicit_path)).expect("explicit path should succeed");

    assert_eq!(std::env::var("R_HOME").unwrap(), "/explicit/r/home");

    // Restore environment
    match original_r_home {
        Some(val) => unsafe { std::env::set_var("R_HOME", val) },
        None => unsafe { std::env::remove_var("R_HOME") },
    }
}
