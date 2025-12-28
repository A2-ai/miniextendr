// Producer package: Creates Counter objects and exposes them to R
//
// This demonstrates cross-package trait dispatch:
// - Uses shared Counter trait from shared-traits crate
// - Implements Counter for SimpleCounter
// - Objects can be passed to consumer package

use miniextendr_api::{miniextendr, miniextendr_module, ffi::SEXP, trait_abi::ccall, ExternalPtr};

// Import the shared Counter trait and its generated ABI types
// The #[miniextendr] macro on the trait generates these items in shared-traits
pub use shared_traits::{
    Counter,
    CounterView,
    CounterVTable,
    TAG_COUNTER,
    __counter_build_vtable,
};

// ============================================================================
// SharedData: A simple type for cross-package ExternalPtr testing
// ============================================================================

/// A simple data container for testing cross-package ExternalPtr dispatch.
/// Consumer.pkg will define the SAME struct, and if the TAG matches,
/// an ExternalPtr created here can be read by consumer.pkg.
#[derive(ExternalPtr)]
pub struct SharedData {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

#[miniextendr]
impl SharedData {
    /// Create a new SharedData
    fn create(x: f64, y: f64, label: String) -> Self {
        Self { x, y, label }
    }

    /// Get the x coordinate
    fn get_x(&self) -> f64 {
        self.x
    }

    /// Get the y coordinate
    fn get_y(&self) -> f64 {
        self.y
    }

    /// Get the label
    fn get_label(&self) -> String {
        self.label.clone()
    }
}


// ============================================================================
// SimpleCounter implementation
// ============================================================================

#[derive(ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn create(initial: i32) -> Self {
        Self { value: initial }
    }
}

#[miniextendr]
impl SimpleCounter {
    /// Get current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }
}

/// Create a new counter with cross-package trait dispatch support
///
/// This wraps the SimpleCounter in an mx_erased wrapper so it can be
/// used with consumer.pkg's trait-generic functions.
///
/// @param initial Initial counter value
/// @return An external pointer to the wrapped counter
/// @export
#[miniextendr]
fn new_counter(initial: i32) -> SEXP {
    let counter = SimpleCounter::create(initial);
    // Use the generated mx_wrap function to wrap with trait dispatch support
    let erased = __mx_wrap_simplecounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}

/// Get the value from a trait-wrapped counter
///
/// @param counter_sexp An external pointer from new_counter()
/// @return The counter's current value
/// @export
#[miniextendr]
fn counter_get_value(counter_sexp: SEXP) -> i32 {
    let view = unsafe { CounterView::try_from_sexp(counter_sexp) }
        .expect("Object does not implement Counter trait");
    view.value()
}

/// Debug: Get TAG_COUNTER as hex string
/// @export
#[miniextendr]
fn debug_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_COUNTER.hi, TAG_COUNTER.lo)
}

/// Debug: Get the ExternalPtr type name for SharedData
/// @export
#[miniextendr]
fn debug_shared_data_type_name() -> String {
    use miniextendr_api::externalptr::TypedExternal;
    SharedData::TYPE_NAME.to_string()
}

miniextendr_module! {
    mod producer_pkg;
    impl SharedData;
    fn debug_shared_data_type_name;
    impl SimpleCounter;
    impl Counter for SimpleCounter;
    fn new_counter;
    fn counter_get_value;
    fn debug_tag_counter;
}
