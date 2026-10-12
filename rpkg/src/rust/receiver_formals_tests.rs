//! Receiver formals of generated wrappers (#1677), driven by
//! `tests/testthat/test-receiver-formals.R`.
//!
//! Env methods, Env/R6 trait namespace functions and S7 fast-path shortcuts
//! bind their receiver as `.mx_self`, so a parameter may be named `self`, `x`,
//! `private` or `obj` (`_self` / `_private` in Rust). S3 and S4 methods on a
//! generic another package defines take the receiver formal that generic
//! names (`receiver = "..."`) and attach to that generic instead of defining
//! one.
//! Each `_self` beside `&self` allows clippy's `duplicate_underscore_argument`.

use miniextendr_api::{Invisible, miniextendr};

// region: Env and R6: `self`, `x` and `private` are free

/// Env class whose methods take parameters named like the old receivers.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FormalsEnv {
    value: i32,
}

#[miniextendr(env, noexport)]
impl FormalsEnv {
    /// Create an object holding `value`.
    /// @param value The initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The object's value, then `self`, `x` and `obj`.
    /// @param self,x,obj Returned in order.
    #[allow(clippy::duplicate_underscore_argument)]
    pub fn echo(&self, _self: i32, x: i32, obj: i32) -> Vec<i32> {
        vec![self.value, _self, x, obj]
    }

    /// Add `self` to the value and return the receiver.
    /// @param self The amount to add.
    #[allow(clippy::duplicate_underscore_argument)]
    pub fn add(&mut self, _self: i32) {
        self.value += _self;
    }
}

/// R6 class for the trait namespace case.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FormalsR6 {
    value: i32,
}

#[miniextendr(r6, noexport)]
impl FormalsR6 {
    /// Create an object holding `value`.
    /// @param value The initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// Trait whose parameters take the names Env and R6 trait methods used to
/// reserve.
#[miniextendr]
pub trait FormalNames {
    /// The receiver's value followed by each argument, in order.
    #[allow(clippy::duplicate_underscore_argument)]
    fn echo_formals(&self, _self: i32, x: i32, _private: i32, obj: i32) -> Vec<i32>;

    /// Add `x` to the value; returns the receiver.
    fn bump(&mut self, x: i32);
}

#[miniextendr(env, noexport)]
impl FormalNames for FormalsEnv {
    /// @param self,x,private,obj Returned in order.
    #[allow(clippy::duplicate_underscore_argument)]
    fn echo_formals(&self, _self: i32, x: i32, _private: i32, obj: i32) -> Vec<i32> {
        vec![self.value, _self, x, _private, obj]
    }

    /// @param x The amount to add.
    fn bump(&mut self, x: i32) {
        self.value += x;
    }
}

#[miniextendr(r6, noexport)]
impl FormalNames for FormalsR6 {
    /// @param self,x,private,obj Returned in order.
    #[allow(clippy::duplicate_underscore_argument)]
    fn echo_formals(&self, _self: i32, x: i32, _private: i32, obj: i32) -> Vec<i32> {
        vec![self.value, _self, x, _private, obj]
    }

    /// @param x The amount to add.
    fn bump(&mut self, x: i32) {
        self.value += x;
    }
}

// endregion

// region: S7 shortcuts: `self` is free

/// S7 class whose methods take a parameter named `self`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FormalsS7 {
    value: i32,
}

#[miniextendr(s7, internal)]
impl FormalsS7 {
    /// Create an object holding `value`.
    /// @param value The initial value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// The object's value, then `self`.
    /// @param self Returned after the value.
    #[allow(clippy::duplicate_underscore_argument)]
    pub fn formals_echo_self(&self, _self: i32) -> Vec<i32> {
        vec![self.value, _self]
    }

    /// Add `self` to the value and return the receiver.
    /// @param self The amount to add.
    #[allow(clippy::duplicate_underscore_argument)]
    pub fn formals_add_self(&mut self, _self: i32) {
        self.value += _self;
    }

    /// The object's value.
    pub fn formals_value(&self) -> i32 {
        self.value
    }
}

/// Trait whose S7 shortcut takes a parameter named `self`.
#[miniextendr]
pub trait ShortcutSelf {
    /// The receiver's value, then `self`.
    #[allow(clippy::duplicate_underscore_argument)]
    fn shortcut_self(&self, _self: i32) -> Vec<i32>;
}

#[miniextendr(s7, internal)]
impl ShortcutSelf for FormalsS7 {
    /// @param self Returned after the value.
    #[allow(clippy::duplicate_underscore_argument)]
    fn shortcut_self(&self, _self: i32) -> Vec<i32> {
        vec![self.value, _self]
    }
}

// endregion

// region: S3 and S4 methods on generics another package defines

/// Counts the calls its `summary()` / `show()` method receives.
#[miniextendr]
pub trait FormalReport {
    /// Count this call and return the count.
    fn report(&mut self) -> i32;
}

/// S3 class with methods for base's `all.equal(target, current, ...)` and
/// `summary(object, ...)`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FormalsS3 {
    value: i32,
    reports: i32,
}

#[miniextendr(s3, internal)]
impl FormalsS3 {
    /// Create an object holding `value`.
    /// @param value The initial value.
    pub fn new(value: i32) -> Self {
        Self { value, reports: 0 }
    }

    /// Whether the value equals `current`.
    /// @param current The value to compare with.
    #[miniextendr(s3(generic = "all.equal", receiver = "target"))]
    pub fn same_value(&self, current: i32) -> bool {
        self.value == current
    }
}

#[miniextendr(s3, internal)]
impl FormalReport for FormalsS3 {
    #[miniextendr(s3(generic = "summary", receiver = "object"))]
    fn report(&mut self) -> i32 {
        self.reports += 1;
        self.reports
    }
}

/// S4 class with methods for `methods::show(object)`, the primitive
/// `length(x)` and a generic the package defines under an explicit name.
#[derive(miniextendr_api::ExternalPtr, Default)]
pub struct FormalsS4 {
    shown: i32,
}

#[miniextendr(s4, internal)]
impl FormalsS4 {
    /// Create an object that has not been shown.
    pub fn new() -> Self {
        Self::default()
    }

    /// Count a `show()` call. Invisible, as `show()` methods are.
    #[miniextendr(s4(generic = "show", receiver = "object", no_dots))]
    pub fn display(&mut self) -> Invisible<()> {
        self.shown += 1;
        Invisible(())
    }

    /// The number of `show()` calls.
    pub fn shown(&self) -> i32 {
        self.shown
    }

    /// A fixed length.
    #[miniextendr(s4(generic = "length", no_dots))]
    pub fn fixed_length(&self) -> i32 {
        42
    }

    /// A generic no other package defines.
    #[miniextendr(s4(generic = "formals_fresh_generic"))]
    pub fn fresh(&self) -> i32 {
        7
    }
}

/// S4 class whose `show()` method comes from a trait impl.
#[derive(miniextendr_api::ExternalPtr, Default)]
pub struct FormalsS4Trait {
    reports: i32,
}

#[miniextendr(s4, internal)]
impl FormalsS4Trait {
    /// Create an object that has not been shown.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of `show()` calls.
    pub fn reports(&self) -> i32 {
        self.reports
    }
}

/// @rdname FormalsS4Trait
/// @aliases show,FormalsS4Trait-method
#[miniextendr(s4, internal)]
impl FormalReport for FormalsS4Trait {
    #[miniextendr(s4(generic = "show", receiver = "object", no_dots))]
    fn report(&mut self) -> i32 {
        self.reports += 1;
        self.reports
    }
}

// endregion
