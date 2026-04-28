# Cross-Package Trait Dispatch Test

This directory contains example code demonstrating cross-package
interoperability with miniextendr's trait ABI system.

## Overview

Two R packages interact via trait dispatch:

- **producer** - Defines the `Counter` trait and implements it for
  `SimpleCounter`
- **consumer** - Accepts any `Counter` implementation through the erased trait
  ABI

The key point: objects created in producer can be passed to consumer's generic
functions, which dispatch via vtables without consumer depending on producer's
concrete types.

## Building the packages

This directory contains two complete R packages: `producer.pkg` and
`consumer.pkg`.

### Quick start

```bash
cd tests/cross-package
just build-all
just test-all
```

Common helpers from `tests/cross-package/justfile`:

- `just configure-all` - run `autoconf` when available and generate build files
- `just document-all` - regenerate wrapper files and roxygen output
- `just build-all` - install both packages in dependency order
- `just test-all` - run both test suites
- `just check-all` - run `devtools::check()` for both packages
- `just clean` - remove build artifacts

### Manual build

If you are not using `just`, the equivalent manual flow is:

```bash
# Build producer.pkg
cd producer.pkg
if command -v autoconf >/dev/null 2>&1; then autoconf; fi
bash ./configure
Rscript -e 'devtools::install(".", upgrade=FALSE, quick=TRUE)'
cd ..

# Build consumer.pkg
cd consumer.pkg
if command -v autoconf >/dev/null 2>&1; then autoconf; fi
bash ./configure
Rscript -e 'devtools::install(".", upgrade=FALSE, quick=TRUE)'
cd ..
```

If you change exported Rust functions or methods, regenerate wrappers and
roxygen output before reinstalling:

```bash
cd tests/cross-package
just document-all
```

## Package structure

Each package is a complete R package with a Rust backend:

```text
producer.pkg/
├── DESCRIPTION
├── NAMESPACE
├── configure.ac
├── configure
├── bootstrap.R
├── cleanup*
├── R/
├── src/
│   ├── Makevars.in
│   ├── stub.c
│   └── rust/
│       ├── Cargo.toml.in
│       ├── lib.rs
│       └── build.rs
└── tests/testthat/
```

## How it works

### 1. Producer package (`producer/lib.rs`)

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
```

Generated infrastructure includes:

- `TAG_COUNTER` - 128-bit type tag for runtime identification
- `CounterVTable` - function pointer table for trait methods
- `__VTABLE_COUNTER_FOR_SIMPLECOUNTER` - concrete vtable for `SimpleCounter`
- `__MxWrapperSimpleCounter` - type-erased wrapper with `mx_erased` header

### 2. Consumer package (`consumer/lib.rs`)

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}

#[miniextendr]
fn increment_twice(counter_sexp: SEXP) -> i32 {
    unsafe {
        let mut counter_ptr = ErasedExternalPtr::from_sexp(counter_sexp);

        let counter = counter_ptr
            .downcast_trait_mut::<dyn Counter>()
            .expect("Not a Counter");

        counter.increment();
        counter.increment();
        counter.value()
    }
}
```

Key mechanisms:

- `ErasedExternalPtr` - type-erased pointer with `mx_erased` header
- `downcast_trait_mut` / `downcast_trait_ref` - vtable lookup via type tags
- Type-tag matching ensures ABI compatibility between packages

### 3. R usage

```r
library(producer.pkg)
library(consumer.pkg)

counter <- SimpleCounter$new(10L)

consumer.pkg::increment_twice(counter)  # 12
consumer.pkg::add_and_get(counter, 5L)  # 17

counter$get_value()  # 17
```

## ABI compatibility requirements

For cross-package trait dispatch to work:

1. Both packages must use the same trait definition.
2. `TAG_COUNTER` must hash to the same value.
3. Method signatures and method order must match.
4. The producer side must register `#[miniextendr]` on `impl Trait for Type`.

## What gets generated

### Producer (`impl Counter for SimpleCounter`)

```rust
struct __MxWrapperSimpleCounter {
    erased: mx_erased,
    data: SimpleCounter,
}

static __MX_BASE_VTABLE_SIMPLECOUNTER: mx_base_vtable = ...;
static __VTABLE_COUNTER_FOR_SIMPLECOUNTER: CounterVTable =
    __counter_build_vtable::<SimpleCounter>();

#[no_mangle]
extern "C-unwind" fn __mx_wrap_simplecounter(data: *mut SimpleCounter) -> *mut mx_erased;

#[no_mangle]
extern "C-unwind" fn __mx_query_counter_simplecounter(ptr: *const mx_erased) -> i32;
```

### Consumer

Consumer only needs the trait interface. At runtime it:

1. reads the `mx_erased` header with `ErasedExternalPtr::from_sexp()`
2. checks the tag against `TAG_COUNTER`
3. casts the vtable pointer to `*const CounterVTable`
4. calls methods through the function pointers

## Testing cross-package scenarios

### Scenario 1: Plain `ExternalPtr` passing

```r
obj <- SimpleCounter$new(5L)
```

### Scenario 2: Trait dispatch

```r
counter <- SimpleCounter$new(5L)
consumer.pkg::increment_twice(counter)
```

### Scenario 3: Multiple implementations

```r
consumer.pkg::increment_twice(fast)
consumer.pkg::increment_twice(atomic)
```

## Implementation notes

- Trait definitions must be exactly identical.
- Type tags use `mx_tag_from_path(concat!(module_path!(), "::Counter"))`.
- Vtables are statically initialized at compile time.
- No runtime codegen or dynamic linking trickery is required.
- Dispatch crosses package boundaries through R's `.Call` interface.
