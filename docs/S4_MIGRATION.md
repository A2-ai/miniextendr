# S4 to S7 Migration Guide

This guide covers when and how to migrate miniextendr `#[miniextendr(s4)]` code to `#[miniextendr(s7)]`.

## When to Use S4 vs S7 in miniextendr

### Keep S4 when:

- **Bioconductor integration** -- Bioconductor still requires S4 classes. If your package interoperates with Bioconductor objects (e.g., `SummarizedExperiment`, `GenomicRanges`), stay with S4.
- **Existing S4 ecosystem** -- If your users call `setMethod()` on your generics, or you extend S4 classes from other packages, switching to S7 would break downstream code.
- **Multiple dispatch** -- S4's `setMethod("fun", c("ClassA", "ClassB"))` dispatches on multiple argument types. S7 supports multi-dispatch too, but S4's is more battle-tested.
- **Formal validity** -- S4's `setValidity()` mechanism has no direct S7 equivalent (S7 uses property validators instead).

### Use S7 when:

- **Starting a new package** -- S7 is the modern successor and the recommended path forward.
- **Computed properties** -- S7 properties (via `@`) map naturally to Rust getters/setters. S4 slots are raw data only.
- **Cleaner API** -- S7 constructors look like functions (`Point(1, 2)`), not `new("Point", ...)`.
- **No Bioconductor dependency** -- If you don't need S4 interop, S7 is simpler.

## Conceptual Mapping

| S4 Concept | S7 Equivalent | miniextendr Attribute |
|---|---|---|
| `setClass("Foo", slots = ...)` | `S7::new_class("Foo", properties = ...)` | `#[miniextendr(s4)]` vs `#[miniextendr(s7)]` |
| `setGeneric("bar", ...)` | `bar <- S7::new_generic("bar", ...)` | Auto-generated for instance methods |
| `setMethod("bar", "Foo", ...)` | `S7::method(bar, Foo) <- ...` | Auto-generated for instance methods |
| `new("Foo", ...)` | `Foo(...)` | Constructor method (`fn new(...)`) |
| `obj@slot_name` | `obj@property_name` | `#[miniextendr(s7(getter))]` for computed props |
| `slot(obj, "name")` | `obj@name` | Direct property access |
| `slot(obj, "name") <- val` | `obj@name <- val` | `#[miniextendr(s7(setter, prop = "name"))]` |
| `setValidity("Foo", ...)` | Property validators in `new_property()` | Not yet supported by miniextendr |
| `contains = "ParentClass"` | `parent = ParentClass` | `#[miniextendr(s7, parent = "ParentClass")]` |
| `isVirtualClass("Foo")` | `abstract = TRUE` | `#[miniextendr(s7, abstract)]` |
| `is(obj, "Foo")` | `S7_inherits(obj, Foo)` | N/A (R-side check) |
| `showMethods("bar")` | N/A | N/A |

## How `#[miniextendr(s4)]` and `#[miniextendr(s7)]` Differ

### S4 generated R code

```r
# Class definition
methods::setClass("Gene", slots = c(ptr = "externalptr"))

# Constructor
Gene <- function(symbol, chromosome) {
  methods::new("Gene", ptr = .Call(C_Gene__new, symbol, chromosome))
}

# Instance methods become setGeneric + setMethod pairs
methods::setGeneric("symbol", function(x, ...) standardGeneric("symbol"))
methods::setMethod("symbol", "Gene", function(x, ...) .Call(C_Gene__symbol, x@ptr))
```

Key characteristics:
- Class defined with `setClass` + `externalptr` slot
- Constructor wraps result in `methods::new()`
- Each method creates a `setGeneric()` + `setMethod()` pair
- Generic signatures always include `...`
- Method names get an `s4_` prefix by default to avoid clashing with existing generics

### S7 generated R code

```r
# Class definition with constructor and properties
Gene <- S7::new_class("Gene",
  properties = list(
    .ptr = S7::class_any
  ),
  constructor = function(symbol, chromosome, .ptr = NULL) {
    if (!is.null(.ptr)) {
      S7::new_object(S7::S7_object(), .ptr = .ptr)
    } else {
      S7::new_object(S7::S7_object(),
        .ptr = .Call(C_Gene__new, symbol, chromosome))
    }
  }
)

# Instance methods use S7::new_generic + S7::method
symbol <- S7::new_generic("symbol", "x", function(x, ...) S7::S7_dispatch())
S7::method(symbol, Gene) <- function(x, ...) .Call(C_Gene__symbol, x@.ptr)
```

Key characteristics:
- Class defined with `S7::new_class()` + properties list
- Constructor is inline in the class definition
- `.ptr` property passed via internal constructor path
- Each method creates a `new_generic()` + `method()` pair
- Supports computed properties via `#[miniextendr(s7(getter))]`
- Supports dynamic (read-write) properties via getter + setter

### Key differences at a glance

| Aspect | S4 | S7 |
|---|---|---|
| Slot/property access | `obj@ptr` (raw slot) | `obj@.ptr` (property) |
| Constructor | `methods::new("Class", ...)` | `S7::new_object(...)` |
| Method prefix | `s4_` by default | None |
| Generic definition | `setGeneric()` (idempotent) | `new_generic()` |
| Computed properties | Not supported | `#[miniextendr(s7(getter))]` |
| Dynamic properties | Not supported | Getter + setter pairs |
| Package dependency | `methods` (base R) | `S7` package |

## Step-by-Step Migration Checklist

### 1. Change the class system attribute

```rust
// Before
#[miniextendr(s4)]
impl Gene { ... }

// After
#[miniextendr(s7)]
impl Gene { ... }
```

### 2. Remove `s4_` method name prefixes

S4 methods default to `s4_<method>` names. S7 uses the Rust method name directly. If you manually named S4 methods with `#[miniextendr(generic = "...")]`, check whether the names still make sense.

```rust
// S4: generates setMethod("s4_symbol", "Gene", ...)
pub fn symbol(&self) -> String { ... }

// S7: generates method(symbol, Gene) <- ...
pub fn symbol(&self) -> String { ... }
```

### 3. Convert slot access to properties (if applicable)

If you were accessing S4 slots from R code using `obj@slot_name`, those become S7 properties accessed the same way (`obj@property_name`). The syntax is identical, but the underlying mechanism changes from raw S4 slots to S7 properties with optional getters/setters.

### 4. Add computed properties where appropriate

S7 supports computed properties that S4 cannot express:

```rust
// New in S7: read-only computed property
#[miniextendr(s7(getter))]
pub fn length(&self) -> f64 {
    self.end - self.start
}

// New in S7: read-write dynamic property
#[miniextendr(s7(getter, prop = "midpoint"))]
pub fn get_midpoint(&self) -> f64 { ... }

#[miniextendr(s7(setter, prop = "midpoint"))]
pub fn set_midpoint(&mut self, value: f64) { ... }
```

### 5. Update R-side code

| S4 pattern | S7 replacement |
|---|---|
| `methods::new("Gene", ...)` | `Gene(...)` |
| `is(obj, "Gene")` | `S7::S7_inherits(obj, Gene)` |
| `isGeneric("symbol")` | N/A (generics are first-class objects) |
| `setMethod("show", "Gene", ...)` | `S7::method(print, Gene) <- ...` |

### 6. Update DESCRIPTION

```
# Before
Imports: methods

# After
Imports: S7
```

### 7. Update NAMESPACE

```r
# Before
importFrom(methods, setClass, setGeneric, setMethod, new)

# After
# (S7 imports are auto-generated by miniextendr)
```

### 8. Run tests and verify

```bash
just configure
just rcmdinstall
just devtools-test
```

## When Migration ISN'T Worth It

Do **not** migrate if:

1. **Your package is on Bioconductor** -- Bioconductor requires S4. Even if S7 interoperates with S4, the tooling and review process expects S4.

2. **Downstream packages extend your classes** -- If other packages call `setMethod()` on your generics or `contains = "YourClass"` in their `setClass()`, migration breaks them.

3. **You use multiple dispatch heavily** -- While S7 supports multi-dispatch, if your codebase relies on complex S4 method signatures like `setMethod("combine", c("TypeA", "TypeB"))`, the migration requires careful testing.

4. **Your S4 code is stable and working** -- If it ain't broke, don't fix it. S4 will continue to work indefinitely. S7 is the future, but S4 isn't going away.

5. **You need `setValidity()`** -- S4's validity checking mechanism has no exact equivalent in S7. S7 uses per-property validators, which are more granular but different in structure.

## S4 Helpers for Interop

When working with S4 objects from Rust code (e.g., receiving Bioconductor objects as function arguments), use the helpers in `miniextendr_api::s4_helpers`:

```rust
use miniextendr_api::s4_helpers;

#[miniextendr]
pub fn inspect_s4(obj: SEXP) -> String {
    unsafe {
        if !s4_helpers::s4_is(obj) {
            return "Not an S4 object".to_string();
        }

        let class = s4_helpers::s4_class_name(obj)
            .unwrap_or_else(|| "unknown".to_string());

        if s4_helpers::s4_has_slot(obj, "data") {
            let data = s4_helpers::s4_get_slot(obj, "data").unwrap();
            format!("S4 {class} with data slot")
        } else {
            format!("S4 {class} (no data slot)")
        }
    }
}
```

Available helpers:

| Function | Description |
|---|---|
| `s4_is(obj)` | Check if SEXP is an S4 object |
| `s4_class_name(obj)` | Get the S4 class name (first element of class attribute) |
| `s4_has_slot(obj, name)` | Check if a named slot exists |
| `s4_get_slot(obj, name)` | Read a slot value (returns `Result<SEXP, String>`) |
| `s4_set_slot(obj, name, value)` | Write a slot value (returns `Result<(), String>`) |
