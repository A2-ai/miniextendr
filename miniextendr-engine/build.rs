// Build script: link against R for `miniextendr-engine`.
//
// This crate is used by benchmarks / embedding binaries, so we resolve `R_HOME`
// and emit the appropriate `-L` / `-lR` flags.
use std::env;
use std::process::Command;

fn main() {
    link_to_r();
}

fn link_to_r() {
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
    let r_arch = env::var("R_ARCH").unwrap_or_default();
    let r_libdir = format!("{}/lib{}", r_home, r_arch);
    println!("cargo:rustc-link-search=native={}", r_libdir);
    println!("cargo:rustc-link-lib=R");

    // Mirror `R CMD LINK` behavior: add runtime search path for libR.
    // Only emit rpath for non-library targets (bins/tests/benches/examples).
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" && should_emit_rpath() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", r_libdir);
    }
}

fn should_emit_rpath() -> bool {
    // These env vars are set for specific target types.
    if env::var_os("CARGO_BIN_NAME").is_some()
        || env::var_os("CARGO_TEST_NAME").is_some()
        || env::var_os("CARGO_BENCH_NAME").is_some()
        || env::var_os("CARGO_EXAMPLE_NAME").is_some()
    {
        return true;
    }

    // Fallback to crate-type check if available.
    if let Ok(crate_types) = env::var("CARGO_CRATE_TYPE") {
        return crate_types.split(',').any(|t| t.trim() == "bin");
    }

    false
}
