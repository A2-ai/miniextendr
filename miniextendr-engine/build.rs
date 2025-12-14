// Build script to link against R and set appropriate stack size
use std::env;
use std::process::Command;

fn main() {
    link_to_r();
    set_stack_size_flags();
}

fn link_to_r() {
    // Get R home
    let r_home = if let Ok(val) = env::var("R_HOME") {
        val
    } else {
        // Try to get from R itself
        let output = Command::new("R")
            .args(["RHOME"])
            .output()
            .expect("Failed to run R RHOME");

        String::from_utf8(output.stdout)
            .expect("R RHOME output not UTF-8")
            .trim()
            .to_string()
    };

    println!("cargo:rerun-if-env-changed=R_HOME");

    // Link to R
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search={}/lib", r_home);
        println!("cargo:rustc-link-lib=dylib=R");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-search={}/lib", r_home);
        println!("cargo:rustc-link-lib=dylib=R");
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search={}/bin/x64", r_home);
        println!("cargo:rustc-link-lib=dylib=R");
    }
}

fn set_stack_size_flags() {
    // R requires larger stacks than Rust's default 2 MiB:
    // - Unix: typically 8 MiB
    // - Windows: 64 MiB since R 4.2
    //
    // We set 8 MiB as a reasonable default that works on all platforms.
    const STACK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match (target_os.as_str(), target_env.as_str()) {
        // Windows MSVC: /STACK:size
        ("windows", "msvc") => {
            println!("cargo:rustc-link-arg=/STACK:{STACK_SIZE}");
        }
        // Windows GNU (MinGW): --stack,size
        ("windows", "gnu") => {
            println!("cargo:rustc-link-arg=-Wl,--stack,{STACK_SIZE}");
        }
        // macOS: -stack_size (requires hex value)
        ("macos", _) => {
            println!("cargo:rustc-link-arg=-Wl,-stack_size,{STACK_SIZE:x}");
        }
        // Linux and other Unix: -z stack-size
        ("linux", _) | ("freebsd", _) | ("netbsd", _) | ("openbsd", _) => {
            println!("cargo:rustc-link-arg=-Wl,-z,stack-size={STACK_SIZE}");
        }
        // Unknown platform - skip
        _ => {}
    }
}
