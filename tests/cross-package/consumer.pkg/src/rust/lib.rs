// Consumer package: Works with objects from producer package
//
// This demonstrates cross-package interoperability:
// - Receives objects with various class systems from producer
// - Tests that objects can pass through without type knowledge
// - Verifies trait dispatch works across package boundaries
// - Implements its own Counter (DoubleCounter) for bidirectional testing

use miniextendr_api::{ExternalPtr, SEXP, SexpExt, miniextendr, trait_abi::ccall};

miniextendr_api::miniextendr_init!();

// Import the shared Counter trait and its generated ABI types
pub use shared_traits::{__Counter_build_vtable, Counter, CounterVTable, CounterView, TAG_Counter};

// Import the shared Resettable trait and its generated ABI types
pub use shared_traits::{
    __Resettable_build_vtable, Resettable, ResettableVTable, ResettableView, TAG_Resettable,
};

// Import the View of the shared Tagged trait (owner receivers)
pub use shared_traits::TaggedView;

// region: Cross-package ExternalPtr pass-through utilities

/// Pass an ExternalPtr through consumer without knowing its type.
/// This tests that ExternalPtr can cross package boundaries as opaque SEXP.
/// @param ptr An ExternalPtr from any package
/// @return The same ExternalPtr (pass-through)
/// @export
#[miniextendr]
pub fn passthrough_ptr(ptr: SEXP) -> SEXP {
    ptr
}

/// Check if a SEXP is an ExternalPtr (type-agnostic check)
/// @param sexp Any R object
/// @return TRUE if it's an ExternalPtr
/// @export
#[miniextendr]
pub fn is_external_ptr(sexp: SEXP) -> bool {
    use miniextendr_api::prelude::SexpExt;
    sexp.is_external_ptr()
}

/// Get class of any R object (for cross-package testing)
/// @param x Any R object
/// @return Character vector of class names
/// @export
#[miniextendr]
pub fn consumer_get_class(x: SEXP) -> SEXP {
    x.get_class()
}

/// Check if an object has a specific class
/// @param x Any R object
/// @param class_name Class name to check for
/// @return TRUE if object has the specified class
/// @export
#[miniextendr]
pub fn has_class(x: SEXP, class_name: String) -> bool {
    let class_attr = x.get_class();
    if !class_attr.is_character() {
        return false;
    }
    let len = class_attr.len();
    for i in 0..len {
        if let Some(name) = class_attr.string_elt_str(i as miniextendr_api::R_xlen_t)
            && name == class_name
        {
            return true;
        }
    }
    false
}
// endregion

// region: Consumer's own Counter implementation: DoubleCounter

/// A counter that increments by 2 (consumer's own implementation)
#[derive(ExternalPtr)]
pub struct DoubleCounter {
    value: i32,
}

#[miniextendr]
impl DoubleCounter {
    fn create(initial: i32) -> Self {
        Self { value: initial }
    }

    /// Get the current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

#[miniextendr]
impl Counter for DoubleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 2; // Increments by 2, not 1!
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }

    fn panic_plain(&self) {
        panic!(
            "DoubleCounter::panic_plain triggered (value={})",
            self.value
        );
    }

    fn error_with_class(&self, class_name: String) {
        miniextendr_api::error!(
            class = class_name,
            "DoubleCounter::error_with_class triggered (value={})",
            self.value
        );
    }

    fn raise_error(&self, msg: String) {
        miniextendr_api::error!("{}", msg);
    }

    fn raise_warning(&self, msg: String) {
        miniextendr_api::warning!("{}", msg);
    }

    fn raise_warning_without_call(&self, msg: String) {
        miniextendr_api::warning!(call = none, class = "data_warning", "{}", msg);
    }

    fn raise_message(&self, msg: String) {
        miniextendr_api::message!("{}", msg);
    }

    fn raise_condition_classed(&self, class_name: String, msg: String) {
        miniextendr_api::condition!(class = class_name, "{}", msg);
    }

    fn raise_error_with_data(&self) {
        miniextendr_api::error!(
            class = "data_bearing_error",
            data = [
                ("field_a", self.value),
                ("field_b", "hello from consumer"),
                ("flag", true),
                ("score", 1.23_f64)
            ],
            "double counter error with structured data (value={})",
            self.value
        );
    }

    fn raise_arg_error(&self, mode: String) {
        miniextendr_api::arg_error!(param = "mode", "'mode' is not a counter mode: {mode:?}");
    }
}

/// Create a new DoubleCounter (consumer's own Counter implementation)
/// DoubleCounter increments by 2, demonstrating a different implementation.
/// @param initial Initial counter value
/// @return An external pointer to the wrapped DoubleCounter
/// @export
#[miniextendr]
pub fn new_double_counter(initial: i32) -> SEXP {
    let counter = DoubleCounter::create(initial);
    let erased = __mx_wrap_DoubleCounter(counter);
    unsafe { ccall::mx_wrap(erased) }
}
// endregion

// region: Generic functions working with Counter trait

/// Increment a counter twice (generic over Counter trait)
/// This function works with Counter objects from ANY package.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's value after incrementing twice
/// @export
#[miniextendr]
pub fn increment_twice(counter_sexp: SEXP) -> i32 {
    let mut counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.increment();
    counter.increment();
    counter.value()
}

/// Add a value to a counter and return the new value
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param n Value to add
/// @return The counter's value after adding n
/// @export
#[miniextendr]
pub fn add_and_get(counter_sexp: SEXP, n: i32) -> i32 {
    let mut counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.add(n);
    counter.value()
}

/// Get value without modifying (uses immutable reference)
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @return The counter's current value
/// @export
#[miniextendr]
pub fn peek_value(counter_sexp: SEXP) -> i32 {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.value()
}

/// Call `Counter::add` through the vtable with an R value, as a caller that
/// skips the View would: nothing converts the argument on this side, so the
/// implementing package's shim converts whatever it is given.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param n Any R value, passed to the shim as is
/// @export
#[miniextendr]
pub fn counter_add_raw(counter_sexp: SEXP, n: SEXP) {
    // SAFETY: both inputs are `.Call()` arguments, rooted for the whole
    // call. Query checks the erased-object tag and Counter implementation
    // before we read the header and its alignment-correct data offset.
    unsafe {
        let vtable = ccall::mx_query(counter_sexp, TAG_Counter).cast::<CounterVTable>();
        assert!(!vtable.is_null(), "object does not implement Counter");
        let erased = ccall::mx_get(counter_sexp);
        let offset = (*(*erased).base).data_offset;
        let data = erased.cast::<u8>().add(offset).cast();
        let result = ((*vtable).add)(data, 1, &n);
        miniextendr_api::trait_abi::repanic_if_rust_error(result);
    }
}

/// Check if an object implements the Counter trait
/// @param sexp Any R object
/// @return TRUE if the object implements Counter trait
/// @export
#[miniextendr]
pub fn is_counter(sexp: SEXP) -> bool {
    unsafe { CounterView::try_from_sexp(sexp).is_some() }
}

/// Make a Counter, keep it only through a View, collect, then read the View
///
/// The factory's result is reachable from nothing in R, so once the stack
/// roots used to build the View end, the View's own root is the only one.
/// A full collection then finalizes the Counter unless the View keeps it.
/// @param make_counter A function of no arguments that returns a Counter
/// @return The counter's value, or `NA` if the collection finalized it
/// @export
#[miniextendr]
pub fn view_value_after_gc(make_counter: SEXP) -> Option<i32> {
    use miniextendr_api::expression::RCall;
    use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
    use miniextendr_api::{Rboolean, sys};

    // SAFETY: on R's main thread inside `.Call`. Every SEXP is protected
    // while anything allocates; the weak reference is the only one carried
    // across the scope's end, and it is protected again before the next
    // allocation.
    unsafe {
        let (view, weak) = {
            let scope = ProtectScope::new();
            let counter = RCall::from_sexp(make_counter)
                .eval(sys::R_GlobalEnv)
                .expect("make_counter() failed");
            let counter = scope.protect(counter).get();
            // The weak reference reports the collection without reading the
            // object after it is freed.
            let weak = sys::R_MakeWeakRef(counter, SEXP::nil(), SEXP::nil(), Rboolean::FALSE);
            let weak = scope.protect(weak).get();
            (CounterView::from_sexp(counter), weak)
        };
        let weak = OwnedProtect::new(weak);
        sys::R_gc();
        if sys::R_WeakRefKey(weak.get()).is_nil() {
            return None;
        }
        Some(view.value())
    }
}
// endregion

// region: Owner receivers across packages

/// Add a label through a `Tagged` View, collect, then read the labels back
///
/// The View passes the owner it roots to the implementing package's
/// owner-receiver methods, which reach the labels in that owner's sidecar.
/// @param obj An object implementing Tagged, from any package
/// @param label The label to add
/// @return The labels after the addition
/// @export
#[miniextendr]
pub fn tagged_add_and_read(obj: SEXP, label: String) -> Vec<String> {
    let mut view =
        unsafe { TaggedView::try_from_sexp(obj) }.expect("object does not implement Tagged");
    view.add_label(label);
    unsafe { miniextendr_api::sys::R_gc() };
    view.labels()
}
// endregion

// region: Generic functions working with Resettable trait

/// Reset an object and check if it's in default state
/// This function works with Resettable objects from ANY package.
/// @param sexp An ExternalPtr to any type implementing Resettable
/// @return TRUE if the object is in default state after reset
/// @export
#[miniextendr]
pub fn reset_and_check(sexp: SEXP) -> bool {
    let mut view = unsafe { ResettableView::from_sexp(sexp) };
    view.reset();
    view.is_default()
}

/// Check if an object is in its default state (without resetting)
/// @param sexp An ExternalPtr to any type implementing Resettable
/// @return TRUE if the object is in default state
/// @export
#[miniextendr]
pub fn check_is_default(sexp: SEXP) -> bool {
    let view = unsafe { ResettableView::from_sexp(sexp) };
    view.is_default()
}

/// Check if an object implements the Resettable trait
/// @param sexp Any R object
/// @return TRUE if the object implements Resettable trait
/// @export
#[miniextendr]
pub fn is_resettable(sexp: SEXP) -> bool {
    unsafe { ResettableView::try_from_sexp(sexp).is_some() }
}
// endregion

// region: Combined trait usage (Counter + Resettable on same object)

/// Increment a Counter twice, then reset it via Resettable, return is_default
/// Tests combined trait usage on the same object across packages.
/// @param sexp An ExternalPtr to a type implementing BOTH Counter and Resettable
/// @return TRUE if the object is in default state after increment+reset
/// @export
#[miniextendr]
pub fn increment_then_reset(sexp: SEXP) -> bool {
    // First use it as a Counter
    let mut counter = unsafe { CounterView::from_sexp(sexp) };
    counter.increment();
    counter.increment();

    // Then use it as a Resettable (same underlying object)
    let mut resettable = unsafe { ResettableView::from_sexp(sexp) };
    resettable.reset();
    resettable.is_default()
}

/// Get the value of a Counter, reset it, then get the value again
/// Returns the value after reset.
/// @param sexp An ExternalPtr to a type implementing BOTH Counter and Resettable
/// @return The counter value after reset
/// @export
#[miniextendr]
pub fn get_reset_get(sexp: SEXP) -> i32 {
    // Reset via Resettable
    let mut resettable = unsafe { ResettableView::from_sexp(sexp) };
    resettable.reset();

    // Read via Counter
    let counter = unsafe { CounterView::from_sexp(sexp) };
    counter.value()
}

/// Debug: Get TAG_Resettable as hex string
/// @export
#[miniextendr]
pub fn debug_consumer_tag_resettable() -> String {
    format!("{:016x}{:016x}", TAG_Resettable.hi, TAG_Resettable.lo)
}
// endregion

// region: Error-class layering test helpers (issue #345 spike)

/// Call panic_plain() on any Counter object via trait dispatch.
///
/// Used by spike tests to verify rust_error class layering across the
/// trait-ABI boundary.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @export
#[miniextendr]
pub fn counter_panic_plain(counter_sexp: SEXP) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.panic_plain();
}

/// Call error_with_class() on any Counter object via trait dispatch.
///
/// Used by spike tests to verify user-class layering across the trait-ABI
/// boundary.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param class_name The class name to use with error!()
/// @export
#[miniextendr]
pub fn counter_error_with_class(counter_sexp: SEXP, class_name: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.error_with_class(class_name);
}

/// Raise a bare error!() through a Counter trait method via trait dispatch.
///
/// Verifies that a producer-side `error!()` round-trips to `rust_error` /
/// `e$kind == "error"` across the trait-ABI boundary, matching the in-process
/// matrix's free-fn / R6 error rows.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param msg The error message
/// @export
#[miniextendr]
pub fn counter_raise_error(counter_sexp: SEXP, msg: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_error(msg);
}

/// Raise a warning!() through a Counter trait method via trait dispatch.
///
/// Verifies that a producer-side `warning!()` round-trips to `rust_warning` /
/// `e$kind == "warning"` across the trait-ABI boundary, matching the in-process
/// matrix's R6/S3/S4/S7 warning rows.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param msg The warning message
/// @export
#[miniextendr]
pub fn counter_raise_warning(counter_sexp: SEXP, msg: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_warning(msg);
}

/// Raise a call-less warning!() through a Counter trait method via trait
/// dispatch.
///
/// Verifies that the producer's `warning!(call = none, ...)` keeps no call
/// across the trait-ABI boundary: the "no call" marker in the tagged value's
/// call slot survives the re-panic, so the consumer's wrapper does not attach
/// its own call (#1725).
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param msg The warning message
/// @export
#[miniextendr]
pub fn counter_raise_warning_without_call(counter_sexp: SEXP, msg: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_warning_without_call(msg);
}

/// Raise an arg_error!() through a Counter trait method via trait dispatch.
///
/// Verifies that a producer-side `arg_error!()` reaches the consumer as an
/// argument error: `e$kind == "conversion"`, `e$param`, the consumer wrapper's
/// call, and the consumer crate's `conversion_error_class` (none here), since
/// the consumer's wrapper raises it (#1740).
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param mode The value reported as a bad `mode` argument
/// @export
#[miniextendr]
pub fn counter_raise_arg_error(counter_sexp: SEXP, mode: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_arg_error(mode);
}

/// Raise a message!() through a Counter trait method via trait dispatch.
///
/// Verifies that a producer-side `message!()` round-trips to `rust_message` /
/// `e$kind == "message"` across the trait-ABI boundary, matching the in-process
/// matrix's message rows.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param msg The message body
/// @export
#[miniextendr]
pub fn counter_raise_message(counter_sexp: SEXP, msg: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_message(msg);
}

/// Raise a classed condition!() through a Counter trait method via trait
/// dispatch.
///
/// Verifies that a producer-side `condition!(class = …)` is catchable by the
/// user class and round-trips to `e$kind == "condition"` across the trait-ABI
/// boundary, matching the in-process matrix's condition!() rows.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @param class_name The class name to use with condition!()
/// @param msg The condition message
/// @export
#[miniextendr]
pub fn counter_raise_condition_classed(counter_sexp: SEXP, class_name: String, msg: String) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_condition_classed(class_name, msg);
}

/// Raise an error!() with structured `data =` fields through a Counter trait
/// method via trait dispatch.
///
/// Exercises the `from_tagged_sexp` slot [4] round-trip path (issue #996 path-1):
/// producer raises `error!(class = "data_bearing_error", data = [...], "msg")`,
/// the vtable shim returns a tagged SEXP, `repanic_if_rust_error` reconstructs
/// the `RCondition` (including data), and the consumer's `with_r_unwind_protect`
/// guard re-emits a new tagged SEXP with the data intact.
///
/// R handlers can then access `e$field_a`, `e$field_b`, `e$flag`, `e$score`.
/// @param counter_sexp An ExternalPtr to any type implementing Counter
/// @export
#[miniextendr]
pub fn counter_raise_error_with_data(counter_sexp: SEXP) {
    let counter = unsafe { CounterView::from_sexp(counter_sexp) };
    counter.raise_error_with_data();
}

// endregion

// region: Simple utility functions for testing

/// Greet with a message demonstrating consumer package is working
/// @param name Name to greet
/// @return A greeting string
/// @export
#[miniextendr]
pub fn consumer_greet(name: String) -> String {
    format!("Hello {} from consumer.pkg!", name)
}

/// Return a constant to verify the package is loaded
/// @return The number 42
/// @export
#[miniextendr]
pub fn consumer_magic_number() -> i32 {
    42
}

/// Debug: Get TAG_Counter as hex string
/// @export
#[miniextendr]
pub fn debug_consumer_tag_counter() -> String {
    format!("{:016x}{:016x}", TAG_Counter.hi, TAG_Counter.lo)
}
// endregion
