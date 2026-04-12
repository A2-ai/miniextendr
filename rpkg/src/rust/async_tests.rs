//! Test fixtures for `#[miniextendr(background)]` async functions.

use miniextendr_api::prelude::*;

/// Simple background function: returns immediately, computes in background.
#[miniextendr(background)]
pub fn bg_add(a: i32, b: i32) -> i32 {
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(100));
    a + b
}

/// Background function returning a string.
#[miniextendr(background)]
pub fn bg_greeting(name: String) -> String {
    std::thread::sleep(std::time::Duration::from_millis(50));
    format!("Hello, {}!", name)
}

/// Background function returning f64.
#[miniextendr(background)]
pub fn bg_pi_approx(iterations: i32) -> f64 {
    let mut sum = 0.0_f64;
    for i in 0..iterations {
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        sum += sign / (2 * i + 1) as f64;
    }
    sum * 4.0
}

/// Background function that panics (for error handling test).
#[miniextendr(background)]
pub fn bg_panicking() -> i32 {
    panic!("intentional panic in background thread");
}

/// Background function with Result return type.
#[miniextendr(background)]
pub fn bg_result_ok(x: i32) -> Result<i32, String> {
    if x > 0 {
        Ok(x * 2)
    } else {
        Err(format!("x must be positive, got {}", x))
    }
}
