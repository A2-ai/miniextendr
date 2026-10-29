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
    assert!(
        result.is_err(),
        "expected snapshot of missing file to error"
    );
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
