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

        String::from_utf8(output.stdout)
            .expect("`R RHOME` output not UTF-8")
            .trim()
            .to_string()
    };

    println!("cargo:rerun-if-env-changed=R_HOME");

    // Link to libR.
    println!("cargo:rustc-link-search=native={}/lib", r_home);
    println!("cargo:rustc-link-lib=R");
}
