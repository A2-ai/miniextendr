//! Tests for adapter traits (RDebug, RDisplay, RHash, ROrd, etc.)
//!
//! These tests verify that the adapter trait blanket implementations work correctly
//! by exposing their methods through wrapper functions.

use miniextendr_api::adapter_traits::{RClone, RDebug, RDefault, RDisplay, RFromStr, RHash, ROrd};
use miniextendr_api::{ExternalPtr, miniextendr, miniextendr_module};
use std::fmt;
use std::str::FromStr;

// =============================================================================
// Test type: Point
// Implements Debug, Display, Hash, PartialOrd, Ord, Clone, Default, FromStr
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, ExternalPtr)]
pub struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl FromStr for Point {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if !s.starts_with('(') || !s.ends_with(')') {
            return Err("Expected format: (x, y)".to_string());
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() != 2 {
            return Err("Expected two comma-separated values".to_string());
        }
        let x: i32 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid x value: {}", parts[0]))?;
        let y: i32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid y value: {}", parts[1]))?;
        Ok(Point { x, y })
    }
}

/// Adapter Traits Test - Point
///
/// @name Point
/// @title Point struct for testing adapter traits
/// @description A 2D point that demonstrates adapter trait functionality.
/// @return An ExternalPtr to a Point.
/// @examples
/// p <- Point$new(3L, 4L)
/// p$debug_str()
/// p$as_r_string()
/// p$hash()
#[miniextendr]
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }

    // RDebug methods
    fn debug_str(&self) -> String {
        RDebug::debug_str(self)
    }

    fn debug_str_pretty(&self) -> String {
        RDebug::debug_str_pretty(self)
    }

    // RDisplay method
    fn as_r_string(&self) -> String {
        RDisplay::as_r_string(self)
    }

    // RHash method
    fn hash(&self) -> f64 {
        // Return as f64 to avoid type issues with R
        RHash::hash(self) as f64
    }

    // ROrd method - compare with another Point
    fn cmp_to(&self, other: &Point) -> i32 {
        ROrd::cmp(self, other)
    }

    // Comparison helpers
    fn is_less_than(&self, other: &Point) -> bool {
        ROrd::cmp(self, other) < 0
    }

    fn is_equal_to(&self, other: &Point) -> bool {
        ROrd::cmp(self, other) == 0
    }

    fn is_greater_than(&self, other: &Point) -> bool {
        ROrd::cmp(self, other) > 0
    }

    // RClone method
    fn clone_point(&self) -> Point {
        RClone::clone(self)
    }

    // RDefault (static method)
    fn default_point() -> Point {
        RDefault::default()
    }

    // RFromStr (static method) - tests &str parameter with worker thread
    fn from_str(s: &str) -> Option<Point> {
        <Point as RFromStr>::from_str(s)
    }
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod adapter_traits_tests;

    impl Point;
}
