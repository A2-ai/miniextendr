//! A disabled method must never participate in class analysis or code generation.
#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

macro_rules! gated_class {
    ($name:ident, $system:ident) => {
        #[derive(ExternalPtr)]
        pub struct $name;

        #[miniextendr($system)]
        impl $name {
            #[cfg(all())]
            pub fn new() -> Self { Self }

            // These names conflict only before cfg selection.
            #[cfg(any())]
            pub fn new(_value: MissingType) -> Self { unreachable!() }

            #[cfg(all())]
            pub fn enabled(&self) -> i32 { 42 }

            // A disabled method's options are never read.
            #[cfg(any())]
            #[miniextendr($system(check_interrupt))]
            pub fn disabled_method(_value: MissingType) -> MissingType { unreachable!() }

            // Multiple gates are conjunctive; cfg_attr can add a gate.
            #[cfg(all())]
            #[cfg_attr(all(), cfg(any()))]
            pub fn disabled_nested(&self) -> MissingType { unreachable!() }

            #[cfg_attr(any(), miniextendr(skip))]
            #[cfg_attr(all(), cfg_attr(all(), miniextendr(r_name = "renamed")))]
            pub fn conditional_name(&self) -> i32 { 7 }
        }
    };
}

gated_class!(GatedEnv, env);
gated_class!(GatedR6, r6);
gated_class!(GatedS3, s3);
gated_class!(GatedS4, s4);
gated_class!(GatedS7, s7);

fn main() {}
