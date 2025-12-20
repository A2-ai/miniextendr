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
    println!("cargo:rustc-link-search=native={}/lib", r_home);
    println!("cargo:rustc-link-lib=R");
}
