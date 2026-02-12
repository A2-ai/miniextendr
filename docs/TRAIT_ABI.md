# Trait ABI

The trait ABI lets R (and other packages) call Rust trait methods without knowing the concrete Rust type at compile time. It does this by storing a tiny "header + vtable" next to the object and using R external pointers to carry it around.

## Key Concepts

### Type Tags (`mx_tag`)

A 128-bit identifier for a trait or concrete type, generated from the fully-qualified Rust path via hashing.

```rust
// Generated at compile time from "crate::Counter"
const TAG_COUNTER: mx_tag = mx_tag_from_path("crate::Counter");
```

### Erased Header (`mx_erased`)

A tiny header that points to a base vtable. This is the common prefix of all type-erased objects:

```c
typedef struct mx_erased {
    const mx_base_vtable *base;
} mx_erased;
```

### Base Vtable (`mx_base_vtable`)

Every erased object has a base vtable with:
- `drop`: Destructor for cleanup when R garbage collects the object
- `concrete_tag`: Type tag for downcasting to the concrete type
- `query`: Function that returns trait vtables by tag

```c
typedef struct mx_base_vtable {
    void (*drop)(mx_erased *ptr);
    mx_tag concrete_tag;
    const void *(*query)(mx_erased *ptr, mx_tag trait_tag);
} mx_base_vtable;
```

### Trait Vtables

Each trait gets its own vtable with function pointers for each method. All methods use a uniform ABI:

```c
typedef SEXP (*mx_meth)(void *data, int argc, const SEXP *argv);
```

## How It Works

### 1. Compile Time

The `#[miniextendr]` macro on a trait generates:
- Tag constant (`TAG_COUNTER`)
- Vtable struct (`CounterVTable`)
- View struct (`CounterView`) for calling methods
- Method shims that convert between R and Rust types

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}
```

### 2. Object Creation

When you create an object for trait dispatch, a wrapper struct is allocated with `mx_erased` at the front and the real Rust data after it:

```rust
#[repr(C)]
struct __MxWrapperMyCounter {
    erased: mx_erased,  // Must be first
    data: MyCounter,
}
```

### 3. Packaging with `mx_wrap`

`mx_wrap` turns `*mut mx_erased` into an R external pointer (`EXTPTRSXP`). The finalizer uses the base vtable's `drop` function to clean up when garbage collected.

```rust
// In a constructor
let wrapper = Box::new(__MxWrapperMyCounter {
    erased: mx_erased { base: &__MX_BASE_VTABLE_MYCOUNTER },
    data: my_counter,
});
let sexp = unsafe { mx_wrap(Box::into_raw(wrapper) as *mut mx_erased) };
```

### 4. Dispatch with `mx_query`

When you want to call a trait method, `mx_query(sexp, TAG_TRAIT)` asks the object for the trait vtable:

```rust
// Get the Counter vtable for this object
let vtable = mx_query(sexp, TAG_COUNTER);
if vtable.is_null() {
    // Object doesn't implement Counter
}
```

### 5. Method Call

If the vtable exists, the generated shim converts R args, calls the Rust method, and converts the result back to R:

```rust
// CounterView wraps the sexp and vtable for ergonomic calls
let view = CounterView::try_from_sexp(sexp)?;
let value = view.value();  // Calls through vtable
view.increment();
```

## Multiple Traits Per Type

A single type can implement multiple traits. Register them all in `miniextendr_module!`:

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[miniextendr]
pub trait Resettable {
    fn reset(&mut self);
}

pub struct MyCounter { value: i32 }

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}

#[miniextendr]
impl Resettable for MyCounter {
    fn reset(&mut self) { self.value = 0; }
}

miniextendr_module! {
    mod mymod;

    // Register both trait impls for the same type
    impl Counter for MyCounter;
    impl Resettable for MyCounter;
}
```

The macro groups trait impls by concrete type. `MyCounter` gets a single wrapper with a query function that handles both traits:

```rust
unsafe extern "C" fn __mx_query_mycounter(
    _ptr: *mut mx_erased,
    trait_tag: mx_tag,
) -> *const c_void {
    if trait_tag == TAG_COUNTER {
        return &__VTABLE_COUNTER_FOR_MYCOUNTER as *const _;
    }
    if trait_tag == TAG_RESETTABLE {
        return &__VTABLE_RESETTABLE_FOR_MYCOUNTER as *const _;
    }
    std::ptr::null()
}
```

From R or consumer packages, both views work on the same object:

```rust
// Get Counter view
let counter_view = CounterView::try_from_sexp(sexp)?;
counter_view.increment();

// Get Resettable view from same object
let reset_view = ResettableView::try_from_sexp(sexp)?;
reset_view.reset();
```

## Cross-Package Usage

The trait ABI enables cross-package dispatch where:
- **Producer package**: Defines traits and concrete types
- **Consumer package**: Uses trait views without knowing concrete types

### Architecture

```
Every package (rpkg, producer.pkg, consumer.pkg, ...)
────────────────────────────────────────────────────
mx_abi.c (compiled into each package's .so)
  mx_abi_register()                  ← called from entrypoint.c
    init_tag()                       ← Rf_install("miniextendr::mx_erased")
    R_RegisterCCallable(...)         ← registers for cross-package use
  mx_wrap() / mx_get() / mx_query() ← linked directly via extern "C"
```

- Each package compiles its own `mx_abi.c` into its `.so`
- Rust calls `mx_wrap`/`mx_get`/`mx_query` directly (no `R_GetCCallable` indirection)
- Cross-package dispatch works because all packages share the same `Rf_install("miniextendr::mx_erased")` tag symbol

### Requirements

1. **Call `mx_abi_register()` in `R_init_*`**: This initializes the tag symbol and registers C-callables.

```c
void R_init_mypkg(DllInfo *dll) {
    miniextendr_panic_hook();
    miniextendr_worker_init();
    mx_abi_register();  // Required for trait ABI
    // ...
}
```

2. **Main thread only**: All trait ABI operations must happen on R's main thread (which is where `.Call` runs).

3. **Null checks**: If a type doesn't implement a trait, `mx_query` returns null. The generated `TraitView::try_from_sexp` handles this gracefully.

## Source Files

- `miniextendr-api/src/trait_abi/mod.rs` - Core types and traits
- `miniextendr-api/src/trait_abi/ccall.rs` - C-callable wrappers and init
- `miniextendr-api/src/abi.rs` - FFI-compatible struct definitions
- `rpkg/src/mx_abi.c` - C implementation of mx_wrap/mx_get/mx_query
- `rpkg/inst/include/mx_abi.h` - C header for consumer packages
