//! #1677: generated bindings stay hygienic when `#[miniextendr]` sits inside a
//! `macro_rules!` body, whether the methods are written there or spliced in by
//! the caller.
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, miniextendr};

macro_rules! define_host {
    ($name:ident, { $($methods:tt)* }) => {
        #[derive(Clone, ExternalPtr)]
        pub struct $name { value: i32 }

        #[miniextendr(env)]
        impl $name {
            pub fn new(value: i32) -> Self { Self { value } }
            pub fn generated(&mut self, arg_1: i32, value: i32) -> &mut Self {
                self.value = arg_1 + value;
                self
            }
            pub fn generated_receiver(&self, self_ref: i32, self_ptr: i32, self_sexp: i32) -> i32 {
                self.value + self_ref + self_ptr + self_sexp
            }
            $($methods)*
        }
    };
}

define_host!(SplicedHost, {
    pub fn numbered(&self, arg_1: i32, value: i32) -> i32 {
        self.value + arg_1 + value
    }
    pub fn set(&mut self, self_sexp: i32) -> &mut Self {
        self.value = self_sexp;
        self
    }
    pub fn checked(&mut self, self_sexp: i32) -> Result<&mut Self, String> {
        self.value = self_sexp;
        Ok(self)
    }
    pub fn finish(self, self_ptr: i32) -> Self {
        Self { value: self.value + self_ptr }
    }
    pub fn try_finish(self, self_slot: i32) -> Result<Self, String> {
        Ok(Self { value: self.value + self_slot })
    }
    pub fn terminal(self, __mx_taken: i32) -> i32 {
        self.value + __mx_taken
    }
});

// The trait and its impl written in the macro body.
macro_rules! define_trait {
    ($trait_name:ident, $host:ident) => {
        #[miniextendr]
        pub trait $trait_name {
            fn probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32;
            fn transfer(&mut self, data: &mut Self, argc: i32, self_ptr: i32) -> i32;
        }

        #[miniextendr(env)]
        impl $trait_name for $host {
            fn probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32 {
                self.value + data + argc + argv + self_ref
            }
            fn transfer(&mut self, data: &mut Self, argc: i32, self_ptr: i32) -> i32 {
                data.value += self.value;
                data.value + argc + self_ptr
            }
        }
    };
}

define_trait!(SplicedProbe, SplicedHost);

// The trait's methods spliced in by the caller.
macro_rules! define_trait_from {
    ($trait_name:ident, $host:ident, { $($decl:tt)* }, { $($body:tt)* }) => {
        #[miniextendr]
        pub trait $trait_name {
            $($decl)*
        }

        #[miniextendr(env)]
        impl $trait_name for $host {
            $($body)*
        }
    };
}

define_trait_from!(CallerProbe, SplicedHost, {
    fn caller_probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32;
}, {
    fn caller_probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32 {
        self.value + data + argc + argv + self_ref
    }
});

fn main() {}
