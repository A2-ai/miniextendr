fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::warning={}", "Building the embedded rust crate");
}