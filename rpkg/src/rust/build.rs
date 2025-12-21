use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-env-changed=MINIEXTENDR_LINT");

    let enabled = match miniextendr_lint::lint_enabled("MINIEXTENDR_LINT") {
        Ok(enabled) => enabled,
        Err(message) => {
            {
                let message: &str = &message;
                let message = message.replace(['\n', '\r'], " ");
                println!("cargo::warning={}", message.trim());
            };
            return;
        }
    };

    if !enabled {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(err) => {
            {
                let message: &str = &format!("CARGO_MANIFEST_DIR: {err}");
                let message = message.replace(['\n', '\r'], " ");
                println!("cargo::warning={}", message.trim());
            };
            return;
        }
    };

    let report = match miniextendr_lint::run(&manifest_dir) {
        Ok(report) => report,
        Err(message) => {
            {
                let message: &str = &message;
                let message = message.replace(['\n', '\r'], " ");
                println!("cargo::warning={}", message.trim());
            };
            return;
        }
    };

    for path in &report.files {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    if !report.errors.is_empty() {
        {
            let message = "miniextendr-lint found issues".replace(['\n', '\r'], " ");
            println!("cargo::warning={}", message.trim());
        };
        for err in &report.errors {
            {
                let message = err.replace(['\n', '\r'], " ");
                println!("cargo::warning={}", message.trim());
            };
        }
    }
}
