# Cross-Package Trait Dispatch Test

This directory contains example code demonstrating cross-package interoperability with miniextendr's trait ABI system.

## Overview

Two R packages interact via trait dispatch:

- **producer** - Defines `Counter` trait and implements it for `SimpleCounter`
- **consumer** - Has generic functions that work with any `Counter` implementation

The key insight: Objects created in producer can be passed to consumer's generic functions, which dispatch via vtables without consumer needing to depend on producer's concrete types.

## How It Works

### 1. Producer Package (`producer/lib.rs`)

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}

#[derive(ExternalPtr)]
pub struct SimpleCounter { value: i32 }

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
    fn add(&mut self, n: i32) { self.value += n; }
}

miniextendr_module! {
    mod producer_pkg;
    impl SimpleCounter;               // Inherent impl registration
    impl Counter for SimpleCounter;   // Trait impl registration
}
```

**Generated infrastructure:**
- `TAG_COUNTER` - 128-bit type tag for runtime identification
- `CounterVTable` - Function pointer table for trait methods
- `__VTABLE_COUNTER_FOR_SIMPLECOUNTER` - Concrete vtable for SimpleCounter
- `__MxWrapperSimpleCounter` - Type-erased wrapper with mx_erased header

### 2. Consumer Package (`consumer/lib.rs`)

```rust
// Same trait definition (ABI-compatible via type tags)
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32;
}

#[miniextendr]
fn increment_twice(counter_sexp: SEXP) -> i32 {
    unsafe {
        let mut counter_ptr = ErasedExternalPtr::from_sexp(counter_sexp);

        // Downcast to trait object - works for ANY Counter impl
        let counter = counter_ptr.downcast_trait_mut::<dyn Counter>()
            .expect("Not a Counter");

        counter.increment();
        counter.increment();
        counter.value()
    }
}
```

**Key mechanisms:**
- `ErasedExternalPtr` - Type-erased pointer with `mx_erased` header
- `downcast_trait_mut/ref` - Queries vtable via type tag matching
- Type tag matching ensures ABI compatibility between packages

### 3. R Usage

```r
# Load both packages
library(producer.pkg)
library(consumer.pkg)

# Create counter in producer
counter <- SimpleCounter$new(10L)

# Pass to consumer's generic function
consumer.pkg::increment_twice(counter)  # Returns 12
consumer.pkg::add_and_get(counter, 5L)  # Returns 17

# Producer's methods still work
counter$get_value()  # Returns 17
```

## ABI Compatibility Requirements

For cross-package trait dispatch to work:

1. **Identical trait definitions** - Both packages must vendor the same trait definition
2. **Type tag matching** - `TAG_COUNTER` must hash to the same value
3. **Method signatures** - Same number/order of methods in trait
4. **Registration** - Producer must register `impl Trait for Type` in miniextendr_module!

## What Gets Generated

### Producer (`impl Counter for SimpleCounter;`)

```rust
// Type-erased wrapper
struct __MxWrapperSimpleCounter {
    erased: mx_erased,            // Header with tag + vtable ptr
    data: SimpleCounter,
}

// Base vtable (drop + query)
static __MX_BASE_VTABLE_SIMPLECOUNTER: mx_base_vtable = ...;

// Trait-specific vtable
static __VTABLE_COUNTER_FOR_SIMPLECOUNTER: CounterVTable = __counter_build_vtable::<SimpleCounter>();

// C-callable wrapper constructor
#[no_mangle]
extern "C-unwind" fn __mx_wrap_simplecounter(data: *mut SimpleCounter) -> *mut mx_erased;

// C-callable query function
#[no_mangle]
extern "C-unwind" fn __mx_query_counter_simplecounter(ptr: *const mx_erased) -> i32;
```

### Consumer (no specific type knowledge needed)

Consumer only knows about the `Counter` trait interface. The vtable dispatch happens at runtime:

1. `ErasedExternalPtr::from_sexp()` reads the `mx_erased` header
2. Type tag is compared against `TAG_COUNTER`
3. If match, vtable pointer is cast to `*const CounterVTable`
4. Methods are called through function pointers

## Testing Cross-Package Scenarios

### Scenario 1: Plain ExternalPtr Passing

```r
# Producer creates object
obj <- SimpleCounter$new(5L)

# Consumer receives as raw SEXP
# downcast_mut<SimpleCounter>() would work if consumer has the type definition
```

### Scenario 2: Trait Dispatch

```r
# Producer creates object (registered with trait vtable)
counter <- SimpleCounter$new(5L)

# Consumer uses via trait interface
consumer.pkg::increment_twice(counter)
# -> Dispatches via TAG_COUNTER match + vtable lookup
```

### Scenario 3: Multiple Implementations

```r
# Producer could have multiple Counter impls
struct FastCounter { value: i32 }
struct AtomicCounter { value: AtomicI32 }

# Consumer's generic functions work with all of them
consumer.pkg::increment_twice(fast)
consumer.pkg::increment_twice(atomic)
```

## Implementation Notes

- Trait definitions must be **exactly identical** (same module path for consistent hashing)
- Type tags use `mx_tag_from_path(concat!(module_path!(), "::Counter"))`
- Vtables are statically initialized at compile time
- No runtime codegen or dynamic linking required
- Works across package boundaries via R's .Call interface

## Future Enhancements

- Shared trait crate to avoid duplication
- Documentation generation for cross-package traits
- Helper functions for common patterns
- CI tests that build both packages and verify interop
