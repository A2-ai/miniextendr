//! Class System Matrix Tests
//!
//! This module tests all combinations of class systems for trait impls:
//! - Inherent impl: Always Env (standard pattern using ExternalPtr)
//! - Trait impls: Env, S3, S4, S7, R6
//!
//! Each type is named: Counter_Trait{Style}
//! For example: Counter_TraitS3 has Env inherent impl and S3 trait impl.
//!
//! Note: The inherent impl class system (S3, S4, S7, R6) controls how the
//! constructor and methods are exposed. The trait impl class system controls
//! how trait methods are exposed. Not all combinations are compatible:
//! - S4/S7/R6 inherent impls don't create environments, so Env-style trait impls
//!   can't attach Type$Trait$method() patterns to them.
//! - For maximum compatibility, use Env inherent impl with any trait impl style.

use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Shared trait for all counter types
// =============================================================================

#[miniextendr]
pub trait MatrixCounter {
    fn custom_get(&self) -> i32;
    fn custom_add(&mut self, n: i32);
    fn default_value() -> i32;
}

// =============================================================================
// Env inherent impl × Env trait impl
// =============================================================================

#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter_TraitEnv { value: i32 }

#[miniextendr]
impl Counter_TraitEnv {
    fn new(v: i32) -> Self { Self { value: v } }
    fn get_value(&self) -> i32 { self.value }
}

#[miniextendr]
impl MatrixCounter for Counter_TraitEnv {
    fn custom_get(&self) -> i32 { self.value }
    fn custom_add(&mut self, n: i32) { self.value += n; }
    fn default_value() -> i32 { 1 }
}

// =============================================================================
// Env inherent impl × S3 trait impl
// =============================================================================

#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter_TraitS3 { value: i32 }

#[miniextendr]
impl Counter_TraitS3 {
    fn new(v: i32) -> Self { Self { value: v } }
    fn get_value(&self) -> i32 { self.value }
}

#[miniextendr(s3)]
impl MatrixCounter for Counter_TraitS3 {
    fn custom_get(&self) -> i32 { self.value }
    fn custom_add(&mut self, n: i32) { self.value += n; }
    fn default_value() -> i32 { 2 }
}

// =============================================================================
// Env inherent impl × S4 trait impl
// =============================================================================

#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter_TraitS4 { value: i32 }

#[miniextendr]
impl Counter_TraitS4 {
    fn new(v: i32) -> Self { Self { value: v } }
    fn get_value(&self) -> i32 { self.value }
}

#[miniextendr(s4)]
impl MatrixCounter for Counter_TraitS4 {
    fn custom_get(&self) -> i32 { self.value }
    fn custom_add(&mut self, n: i32) { self.value += n; }
    fn default_value() -> i32 { 3 }
}

// =============================================================================
// Env inherent impl × S7 trait impl
// =============================================================================

#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter_TraitS7 { value: i32 }

#[miniextendr]
impl Counter_TraitS7 {
    fn new(v: i32) -> Self { Self { value: v } }
    fn get_value(&self) -> i32 { self.value }
}

#[miniextendr(s7)]
impl MatrixCounter for Counter_TraitS7 {
    fn custom_get(&self) -> i32 { self.value }
    fn custom_add(&mut self, n: i32) { self.value += n; }
    fn default_value() -> i32 { 4 }
}

// =============================================================================
// Env inherent impl × R6 trait impl
// =============================================================================

#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter_TraitR6 { value: i32 }

#[miniextendr]
impl Counter_TraitR6 {
    fn new(v: i32) -> Self { Self { value: v } }
    fn get_value(&self) -> i32 { self.value }
}

#[miniextendr(r6)]
impl MatrixCounter for Counter_TraitR6 {
    fn custom_get(&self) -> i32 { self.value }
    fn custom_add(&mut self, n: i32) { self.value += n; }
    fn default_value() -> i32 { 5 }
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod class_system_matrix;

    // Inherent impls (all Env style)
    impl Counter_TraitEnv;
    impl Counter_TraitS3;
    impl Counter_TraitS4;
    impl Counter_TraitS7;
    impl Counter_TraitR6;

    // Trait implementations (different styles)
    impl MatrixCounter for Counter_TraitEnv;
    impl MatrixCounter for Counter_TraitS3;
    impl MatrixCounter for Counter_TraitS4;
    impl MatrixCounter for Counter_TraitS7;
    impl MatrixCounter for Counter_TraitR6;
}
