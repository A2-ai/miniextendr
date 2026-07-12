// Cross-package trait dispatch demonstration
//
// This tests the pattern for sharing objects between R packages via traits:
//
// REAL CROSS-PACKAGE SCENARIO:
// ============================
//
// producer.pkg/src/lib.rs:
//   #[miniextendr]
//   pub trait SharedCounter { fn value(&self) -> i32; ... }
//
//   #[derive(ExternalPtr)]
//   pub struct SimpleCounter { value: i32 }
//
//   #[miniextendr]
//   impl SharedCounter for SimpleCounter { ... }
//
//
// consumer.pkg/Cargo.toml:
//   [dependencies]
//   producer-pkg = { version = "0.1" }  # Rust-level dependency
//
// consumer.pkg/src/lib.rs:
//   use producer_pkg::Counter;  # Import trait, NOT concrete types
//
//   // Generic function works with ANY Counter, even from other packages
//   #[miniextendr]
//   fn increment_twice<T: Counter>(counter: &mut T) -> i32 {
//       counter.increment();
//       counter.increment();
//       counter.value()
//   }
//
// R usage:
//   library(producer.pkg)
//   library(consumer.pkg)
//
//   counter <- SimpleCounter$new(10)           # Created in producer
//   consumer.pkg::increment_twice(counter)     # Used in consumer
//
// The dispatch works via:
// 1. ExternalPtr preserves type identity across packages
// 2. Generated trait wrappers (Type$Trait$method) available in R
// 3. Consumer can call producer's trait methods on the object

use miniextendr_api::{ExternalPtr, miniextendr};
use std::sync::atomic::{AtomicI32, Ordering};

// region: Shared trait definition

#[miniextendr]
pub trait SharedCounter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
    fn reset(&mut self);
}
// endregion

// region: Producer package - SimpleCounter

#[derive(ExternalPtr)]
pub struct SharedSimpleCounter {
    value: i32,
}

// Bare (default class system): the inherent + trait impls flip together, so a
// feature-default leg gets a consistent class system for both.
#[miniextendr(env)]
impl SharedSimpleCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}

// Bare: #1115 is fixed — two impls of one trait on different types now emit
// class-scoped wrappers, so this + AtomicCounter no longer collide when both
// flip to r6 under r6-default. (The `Type$Trait$method(x)` call form these
// tests use works under env and r6; test-shared-trait.R runs in the default
// env build only.)
#[miniextendr(env)]
impl SharedCounter for SharedSimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }

    fn reset(&mut self) {
        self.value = 0;
    }
}
// endregion

// region: Producer package - AtomicCounter (alternative implementation)

#[derive(ExternalPtr)]
pub struct AtomicCounter {
    value: AtomicI32,
}

// Bare (default class system): flips together with its trait impl below.
#[miniextendr(env)]
impl AtomicCounter {
    fn new_atomic(initial: i32) -> Self {
        Self {
            value: AtomicI32::new(initial),
        }
    }
}

// Bare: see SharedSimpleCounter above — class-scoped r6 wrappers (#1115 fix)
// mean this and SharedSimpleCounter no longer collide under r6-default.
#[miniextendr(env)]
impl SharedCounter for AtomicCounter {
    fn value(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    fn increment(&mut self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }

    fn add(&mut self, n: i32) {
        self.value.fetch_add(n, Ordering::SeqCst);
    }

    fn reset(&mut self) {
        self.value.store(0, Ordering::SeqCst);
    }
}
// endregion

// region: Module registration
// endregion
