//! Build script for miniextendr-api
//!
//! Sets appropriate stack size linker flags for R-compatible binaries.
//! This affects tests, examples, and cdylib crates that depend on miniextendr-api.

fn main() {
    // Only set stack size flags when nonapi feature is enabled
    // (since that's where thread utilities live)
    #[cfg(feature = "nonapi")]
    set_stack_size_flags();

    // Ensure rebuild on feature changes
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_NONAPI");
}

#[cfg(feature = "nonapi")]
fn set_stack_size_flags() {
    // R requires larger stacks than Rust's default 2 MiB:
    // - Unix: typically 8 MiB
    // - Windows: 64 MiB since R 4.2
    //
    // We set 8 MiB as a reasonable default that works on all platforms.
    // Users needing Windows R's full 64 MiB can override via .cargo/config.toml.
    const STACK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match (target_os.as_str(), target_env.as_str()) {
        // Windows MSVC: /STACK:size
        ("windows", "msvc") => {
            println!("cargo::rustc-link-arg=/STACK:{STACK_SIZE}");
        }
        // Windows GNU (MinGW): --stack,size
        ("windows", "gnu") => {
            println!("cargo::rustc-link-arg=-Wl,--stack,{STACK_SIZE}");
        }
        // macOS: -stack_size (requires hex value)
        ("macos", _) => {
            println!("cargo::rustc-link-arg=-Wl,-stack_size,{STACK_SIZE:x}");
        }
        // Linux and other Unix: -z stack-size
        ("linux", _) | ("freebsd", _) | ("netbsd", _) | ("openbsd", _) => {
            println!("cargo::rustc-link-arg=-Wl,-z,stack-size={STACK_SIZE}");
        }
        // Unknown platform - skip
        _ => {}
    }
}
