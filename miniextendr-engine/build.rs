//! Build script for linking `miniextendr-engine` against R.
//!
//! Resolves `R_HOME`, emits link search flags, and mirrors key parts of
//! `R CMD LINK` behavior for runtime library lookup.
use std::env;
use std::process::Command;

/// Build-script entrypoint.
fn main() {
    link_to_r();
}

/// Resolves `R_HOME` and emits linker flags for libR.
fn link_to_r() {
    // Skip libR resolution on wasm32 targets (webR / wasm32-unknown-emscripten
    // and friends). cargo check never links and the toolchain doesn't ship a
    // host R, so the rest of this function is pure overhead/breakage there.
    // Must use the env var, not `cfg!(target_arch = "wasm32")`: the cfg refers
    // to the build script binary's host arch, not the requested cross-compile
    // target. See issue #482.
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        return;
    }

    // Resolve R home directory.
    let r_home = if let Ok(val) = env::var("R_HOME") {
        val
    } else {
        let output = Command::new("R")
            .args(["RHOME"])
            .output()
            .expect("Failed to run `R RHOME` (set R_HOME or put `R` on PATH)");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "`R RHOME` failed with exit code {:?}.\n\
                 Ensure R is installed and on PATH, or set R_HOME.\n\
                 stderr: {}",
                output.status.code(),
                stderr
            );
        }

        let r_home = String::from_utf8(output.stdout)
            .expect("`R RHOME` output not UTF-8")
            .trim()
            .to_string();

        if r_home.is_empty() {
            panic!("`R RHOME` returned empty output. Set R_HOME explicitly.");
        }

        r_home
    };

    // Verify R_HOME directory exists
    if !std::path::Path::new(&r_home).is_dir() {
        panic!("R_HOME directory does not exist: {}", r_home);
    }

    println!("cargo:rerun-if-env-changed=R_HOME");

    // Link to libR.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let r_libdir = r_libdir(&r_home, &target_os);
    println!("cargo:rustc-link-search=native={}", r_libdir);
    println!("cargo:rustc-link-lib=R");

    // Mirror `R CMD LINK` behavior: add a runtime search path for libR so this
    // crate's own test/bin/example binaries find it. No-op for the R-package
    // staticlib build (staticlibs don't link) and skipped on Windows (no rpath).
    if target_os != "windows" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", r_libdir);
    }
}

/// Determines the directory containing R's shared library.
///
/// On Windows, R.dll lives in `bin/x64/` (or `bin/` for single-arch installs).
/// On Unix, libR.so lives in `lib/`.
/// Uses `R_ARCH` if set, otherwise probes the filesystem under `r_home`.
fn r_libdir(r_home: &str, target_os: &str) -> String {
    let r_arch = env::var("R_ARCH").unwrap_or_default();

    if target_os == "windows" {
        // Try R_ARCH first (e.g. "/x64")
        if !r_arch.is_empty() {
            return format!("{}/bin{}", r_home, r_arch);
        }
        // Probe for R.dll: bin/x64 (multi-arch) then bin/ (single-arch)
        let bin_x64 = format!("{}/bin/x64", r_home);
        if std::path::Path::new(&bin_x64).join("R.dll").exists() {
            return bin_x64;
        }
        format!("{}/bin", r_home)
    } else {
        format!("{}/lib{}", r_home, r_arch)
    }
}
