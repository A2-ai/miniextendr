//! Build script for miniextendr-bench: an rpath to R's library directory.
//!
//! `miniextendr-engine` links libR and adds an rpath, but `rustc-link-arg`
//! reaches only that crate's own targets. It exports R's library directory
//! as `links = "R"` metadata instead, which arrives here as `DEP_R_LIBDIR`,
//! so the benches and tests of this crate find libR (and R's libRblas /
//! libRlapack under the `blas-lapack` feature) at run time without
//! `LD_LIBRARY_PATH`. Any crate that depends on miniextendr-engine directly
//! can do the same.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if let Ok(libdir) = std::env::var("DEP_R_LIBDIR")
        && target_os != "windows"
    {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{libdir}");
    }
}
