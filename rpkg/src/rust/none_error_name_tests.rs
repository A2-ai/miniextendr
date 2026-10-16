//! None diagnostics use the R entry point's name (#2063, #2077).
//!
//! Standalone and inherent fixtures use `Option<()>` to exercise the None
//! error. Standalone `Option<scalar>` returns typed NA (#1415); trait fixtures
//! below use their existing scalar-Option unwrapping path.
//!
//! The worker fixtures need a worker feature, which rpkg's default build
//! (`just test-rpkg`) leaves off; enable one explicitly to test these paths.

// Exercises the deprecated `worker` option (#2117).
#![allow(deprecated)]

use miniextendr_api::{ExternalPtr, miniextendr};

/// @noRd
#[miniextendr(noexport, no_worker, r_name = "none_named_main")]
pub fn none_rust_main() -> Option<()> {
    None
}

/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker, r_name = "none_named_worker")]
pub fn none_rust_worker() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, no_worker, postfix = "_entry")]
pub fn none_postfix_main() -> Option<()> {
    None
}

/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker, postfix = "_entry")]
pub fn none_postfix_worker() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, r_name = "none_named_scalar")]
pub fn none_rust_scalar() -> Option<i32> {
    None
}

#[derive(ExternalPtr)]
pub struct NoneNameLookup;

/// @noRd
#[miniextendr(env, noexport)]
impl NoneNameLookup {
    pub fn new() -> Self {
        Self
    }

    #[miniextendr(r_name = "find_value")]
    pub fn rust_find(&self) -> Option<()> {
        None
    }

    #[miniextendr(postfix = "_entry")]
    pub fn find(&self) -> Option<()> {
        None
    }
}

// The worker methods, in a second impl block of the same class (#1776),
// which repeats the class settings.
/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(env, noexport)]
impl NoneNameLookup {
    #[miniextendr(env(worker), r_name = "load_value")]
    pub fn rust_load() -> Option<()> {
        #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
        assert!(!miniextendr_api::worker::is_r_main_thread());
        None
    }

    #[miniextendr(env(worker), postfix = "_entry")]
    pub fn load() -> Option<()> {
        #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
        assert!(!miniextendr_api::worker::is_r_main_thread());
        None
    }
}

// Keep these internal; S3 registration still exercises real generic dispatch.
macro_rules! none_name_class {
    ($name:ident, $system:ident, $visibility:ident, $entry:literal $(, $extra:item)*) => {
        #[derive(ExternalPtr)]
        pub struct $name;

        /// @noRd
        #[miniextendr($system, $visibility)]
        impl $name {
            pub fn new() -> Self {
                Self
            }

            #[miniextendr(r_name = $entry)]
            pub fn rust_read(&self) -> Option<()> {
                None
            }

            #[miniextendr(postfix = "_entry")]
            pub fn load() -> Option<()> {
                None
            }

            $($extra)*
        }
    };
}

none_name_class!(NoneNameR6, r6, noexport, "none_r6_value");
none_name_class!(NoneNameS3, s3, internal, "none_s3_value");
none_name_class!(NoneNameS7, s7, noexport, "none_s7_value");

none_name_class!(
    NoneNameS4,
    s4,
    noexport,
    "none_s4_value",
    #[miniextendr(s4(generic = "none_s4_lookup"))]
    pub fn rust_lookup(&self) -> Option<()> {
        None
    }
);

// The ABI needs a round-trippable return type. Trait method wrappers raise
// on None of Option<i32>; standalone Option<i32> retains its NA conversion.
// Empty annotated impls expose an existing Rust impl (TPIE).
#[miniextendr]
pub trait NoneNameMethods {
    #[miniextendr(r_name = "none_trait_value")]
    fn rust_value(&self) -> Option<i32> {
        None
    }

    #[miniextendr(r_name = "none_trait_load")]
    fn rust_load() -> Option<i32> {
        None
    }
}

impl NoneNameMethods for NoneNameLookup {}

/// @noRd
#[miniextendr(env, noexport)]
impl NoneNameMethods for NoneNameLookup {}

impl NoneNameMethods for NoneNameR6 {}

/// @noRd
#[miniextendr(r6, noexport)]
impl NoneNameMethods for NoneNameR6 {}

impl NoneNameMethods for NoneNameS3 {}

/// @noRd
#[miniextendr(s3, internal)]
impl NoneNameMethods for NoneNameS3 {}

// Populated impls and declaration-provided empty impls take different paths.
/// @noRd
#[miniextendr(s4, noexport)]
impl NoneNameMethods for NoneNameS4 {
    fn rust_value(&self) -> Option<i32> {
        None
    }
    fn rust_load() -> Option<i32> {
        None
    }
}

impl NoneNameMethods for NoneNameS7 {}

/// @noRd
#[miniextendr(s7, noexport)]
impl NoneNameMethods for NoneNameS7 {}

#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;
    pub struct NoneNameVctrs;

    /// @noRd
    #[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "none"), noexport)]
    impl NoneNameVctrs {
        #[allow(clippy::new_ret_no_self)]
        pub fn new(values: Vec<f64>) -> Vec<f64> {
            values
        }

        #[miniextendr(postfix = "_entry")]
        pub fn load() -> Option<()> {
            None
        }

        #[miniextendr(vctrs(format))]
        pub fn render(x: Vec<f64>) -> Option<()> {
            let _ = x;
            None
        }
    }
}
