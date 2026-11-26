//! Class lambdas raise what the Rust side refuses (#1675).
//!
//! S7 calls a property's setter without checking the value's class first, so a
//! setter lambda that dropped the `.Call()` result left `o@level <- 3.5`
//! unchanged without a word, and a getter lambda handed R the condition object
//! as the property's value. R6's private `deep_clone` stored it in the clone.

use miniextendr_api::{ExternalPtr, IntoR, SEXP, miniextendr};

/// An S7 class with an integer property and a fallible one.
#[derive(ExternalPtr)]
pub struct S7Gauge {
    level: i32,
}

/// @noRd
#[miniextendr(s7, noexport)]
impl S7Gauge {
    pub fn new(level: i32) -> Self {
        Self { level }
    }

    #[miniextendr(s7(getter))]
    pub fn level(&self) -> i32 {
        self.level
    }

    #[miniextendr(s7(setter, prop = "level"))]
    pub fn set_level(&mut self, value: i32) {
        self.level = value;
    }

    /// The level, refused while it is negative.
    #[miniextendr(s7(getter))]
    pub fn checked(&self) -> Result<i32, String> {
        if self.level < 0 {
            Err(format!("level {} is negative", self.level))
        } else {
            Ok(self.level)
        }
    }
}

/// An R6 class whose deep-clone handler refuses a negative value.
#[derive(ExternalPtr)]
pub struct R6CloneRefusal {
    value: i32,
}

/// @noRd
#[miniextendr(r6(cloneable), noexport)]
impl R6CloneRefusal {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    #[miniextendr(r6(deep_clone))]
    pub fn clone_field(&self, name: &str, value: SEXP) -> Result<SEXP, String> {
        if name != ".ptr" {
            Ok(value)
        } else if self.value < 0 {
            Err(format!("value {} is not cloned", self.value))
        } else {
            Ok(ExternalPtr::new(Self { value: self.value }).into_sexp())
        }
    }
}
