# `#[miniextendr]` Attribute Reference

Complete reference for `#[miniextendr]` on every Rust item type.

## Dispatch Overview

`#[miniextendr]` adapts its behavior based on the item it's applied to:

| Item | Default Behavior | Example |
|------|-----------------|---------|
| `fn` | Generate C wrapper + R wrapper | `#[miniextendr] pub fn add(x: i32, y: i32) -> i32` |
| `impl` (inherent) | Env-class methods | `#[miniextendr] impl Counter { ... }` |
| `impl Trait for Type` | Trait ABI shims | `#[miniextendr(s3)] impl Display for Counter { ... }` |
| `trait` | Vtable + ABI generation | `#[miniextendr] pub trait Counter { ... }` |
| 1-field `struct` | ALTREP class | `#[miniextendr] struct MyVec(Vec<i32>)` |
| multi-field `struct` | ExternalPtr | `#[miniextendr] struct Point { x: f64, y: f64 }` |
| fieldless `enum` | RFactor | `#[miniextendr] enum Color { Red, Green, Blue }` |

---

## Functions

```rust
#[miniextendr]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

Generates:
- C wrapper (`C_greet`) handling SEXP conversion
- R wrapper (`greet <- function(name) { .Call(C_greet, name) }`)
- `pub` functions get `@export`; non-pub get `@noRd`

### Function Attributes

#### Visibility & Export

| Attribute | Effect |
|-----------|--------|
| `internal` | Add `@keywords internal`, suppress `@export` |
| `noexport` | Suppress `@export` only |
| `invisible` | Wrap R return in `invisible()` |
| `visible` | Force visible return (override default) |
| `doc = "..."` | Custom roxygen block (replaces auto-generated) |

```rust
#[miniextendr(internal)]
pub fn helper() -> i32 { 42 }

#[miniextendr(invisible)]
pub fn set_option(key: String, value: i32) { /* ... */ }
```

#### Threading

| Attribute | Effect |
|-----------|--------|
| `worker` | Run on worker thread (default if `default-worker` feature) |
| `no_worker` | Run on main R thread |
| `unsafe(main_thread)` | Main thread, no worker overhead (for SEXP params) |

Functions taking `SEXP` parameters automatically run on main thread.

```rust
#[miniextendr(no_worker)]
pub fn fast_add(x: i32, y: i32) -> i32 { x + y }

#[miniextendr(unsafe(main_thread))]
pub fn inspect_sexp(x: miniextendr_api::ffi::SEXP) -> i32 { /* ... */ }
```

#### Type Conversion

| Attribute | Effect |
|-----------|--------|
| `coerce` | Auto-coerce R types (e.g., double → int) |
| `no_coerce` | Reject type mismatches |
| `strict` | Panic on lossy conversions (i64/u64 overflow) |
| `no_strict` | Allow lossy conversions |
| `prefer = "..."` | Return type preference: `"list"`, `"externalptr"`, `"vector"`, `"native"`, `"auto"` |

```rust
#[miniextendr(strict)]
pub fn exact_value(x: i64) -> i64 { x }

#[miniextendr(prefer = "list")]
pub fn get_record() -> MyStruct { /* ... */ }
```

#### Error Handling

| Attribute | Effect |
|-----------|--------|
| `error_in_r` | Transport `Result::Err` as R condition (structured error) |
| `unwrap_in_r` | Transport `Result::Err` as R `stop()` message |

```rust
#[miniextendr(error_in_r)]
pub fn parse_data(s: String) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| e.to_string())
}
```

#### Miscellaneous

| Attribute | Effect |
|-----------|--------|
| `check_interrupt` | Insert `R_CheckUserInterrupt()` before call |
| `rng` | Manage R's RNG state (`GetRNGstate`/`PutRNGstate`) |
| `c_symbol = "..."` | Custom C function name |
| `lifecycle = "..."` | Mark as deprecated/experimental/superseded |
| `dots = typed_list!(...)` | Validate `...` arguments (see [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md)) |

```rust
#[miniextendr(check_interrupt, rng)]
pub fn long_simulation(n: i32) -> Vec<f64> { /* ... */ }

#[miniextendr(lifecycle = "deprecated")]
pub fn old_api() -> i32 { 0 }
```

#### R Wrapper Customization

These attributes inject custom R code into the generated wrapper function, giving
fine-grained control over the R-side behavior without touching the Rust logic.

| Attribute | Effect |
|-----------|--------|
| `r_name = "..."` | Override R function name (e.g., `"is.widget"`) |
| `r_entry = "..."` | Inject R code at function entry (before all checks) |
| `r_post_checks = "..."` | Inject R code after checks (before `.Call()`) |
| `r_on_exit = "..."` | Register `on.exit()` cleanup (short form, `add = TRUE`) |
| `r_on_exit(expr = "...", add = bool, after = bool)` | Long form with full `on.exit()` control |

Generated wrapper layout:

```r
fn_name <- function(formals) {
  # r_entry code
  on.exit(...)         # r_on_exit
  # missing defaults, lifecycle, stopifnot, match.arg
  # r_post_checks code
  .Call(C_fn_name, ...)
}
```

```rust
// Rename R function (C symbol still derived from Rust name)
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool { x > 0 }

// Coerce input before Rust sees it
#[miniextendr(r_entry = "x <- as.integer(x)")]
pub fn process(x: i32) -> i32 { x * 2 }

// Validate after built-in checks
#[miniextendr(r_post_checks = "stopifnot(x > 0L)")]
pub fn positive_only(x: i32) -> i32 { x }

// Register cleanup code
#[miniextendr(r_on_exit = "message(\"done\")")]
pub fn with_cleanup(x: i32) -> i32 { x + 1 }

// Full on.exit control (LIFO order)
#[miniextendr(r_on_exit(expr = "close(con)", after = false))]
pub fn with_connection(x: i32) -> i32 { x }

// Combine all four
#[miniextendr(
    r_name = "widget.create",
    r_entry = "n <- as.integer(n)",
    r_on_exit = "message(\"cleanup\")",
    r_post_checks = "stopifnot(n > 0L)",
)]
pub fn create_widget(n: i32) -> i32 { n * 10 }
```

`r_on_exit` defaults: `add = TRUE`, `after = TRUE` (composable, FIFO — standard R convention).
When `add = FALSE`: omits both `add` and `after` (R ignores `after` when `add = FALSE`).

#### S3 Standalone Functions

Functions can be standalone S3 methods without an impl block:

```rust
#[miniextendr(s3(generic = "format", class = "percent"))]
pub fn format_percent(x: SEXP, _dots: ...) -> Vec<String> { /* ... */ }
```

---

## Impl Blocks (Inherent)

```rust
#[derive(ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]       // default: env-class
impl Counter {
    pub fn new(initial: i32) -> Self { Counter { value: initial } }
    pub fn value(&self) -> i32 { self.value }
    pub fn increment(&mut self) { self.value += 1; }
}
```

### Class System Selection

| Syntax | System | R Pattern |
|--------|--------|-----------|
| `#[miniextendr]` | Env (default) | `obj$method()` environment dispatch |
| `#[miniextendr(r6)]` | R6 | `R6Class` with `$new()` |
| `#[miniextendr(s3)]` | S3 | `generic.Class(x, ...)` |
| `#[miniextendr(s4)]` | S4 | `setClass`/`setMethod` formal OOP |
| `#[miniextendr(s7)]` | S7 | `new_class`/`new_generic` modern OOP |
| `#[miniextendr(vctrs)]` | vctrs | vctrs-compatible S3 vector class |

### Impl-Level Attributes

| Attribute | Applies To | Effect |
|-----------|-----------|--------|
| `class = "..."` | All systems | Custom R class name |
| `label = "..."` | All systems | Distinguish multiple impl blocks on same type |
| `strict` / `no_strict` | All systems | Strict type conversion for all methods |
| `internal` | All systems | `@keywords internal` on class |
| `noexport` | All systems | Suppress `@export` on class |
| `blanket` | Trait impls | Skip trait ABI (for blanket impls) |

```rust
// Two impl blocks need labels
#[miniextendr(s3, label = "core")]
impl Counter { /* constructors + getters */ }

#[miniextendr(s3, label = "mutations")]
impl Counter { /* mutating methods */ }
```

### R6-Specific Options

```rust
#[miniextendr(r6(
    inherit = "BaseClass",
    portable = true,
    cloneable = true,
    lock_objects = false,
    lock_class = false,
    r_data_accessors,
))]
impl MyClass { /* ... */ }
```

### S7-Specific Options

```rust
#[miniextendr(s7(
    parent = "ParentClass",
    abstract = true,
    r_data_accessors,
))]
impl MyClass { /* ... */ }
```

### vctrs-Specific Options

```rust
#[miniextendr(vctrs(
    kind = "vctr",          // vctr | rcrd | list_of
    base = "double",        // underlying R type
    inherit_base_type = true,
    ptype = "double(0)",    // prototype R expression
    abbr = "pct",           // vec_ptype_abbr
))]
impl Percent { /* ... */ }
```

### Method-Level Attributes

Methods inside impl blocks can have per-method attributes nested under the class
system keyword:

```rust
#[miniextendr(s3)]
impl Person {
    // Override the S3 generic name
    #[miniextendr(generic = "print")]
    pub fn show(&mut self) { println!("{}", self.name); }

    // Override the class suffix for double-dispatch
    #[miniextendr(generic = "vec_ptype2", class = "my_vctr.my_vctr")]
    pub fn ptype2_self(&self) -> SEXP { /* ... */ }

    // Generate as.data.frame.Person
    #[miniextendr(as = "data.frame")]
    pub fn as_df(&self) -> List { /* ... */ }

    // Skip this method
    #[miniextendr(ignore)]
    pub fn internal_helper(&self) { /* ... */ }

    // Parameter defaults
    #[miniextendr(defaults(n = "1"))]
    pub fn increment_by(&mut self, n: i32) { self.value += n; }
}
```

#### Shared Method Attributes (all class systems)

| Attribute | Effect |
|-----------|--------|
| `ignore` | Don't generate R wrapper for this method |
| `constructor` | Mark as constructor (factory method) |
| `generic = "..."` | Override S3/S4/S7 generic name |
| `class = "..."` | Override S3 class suffix |
| `as = "..."` | Generate `as.<target>()` coercion method |
| `defaults(p = "val")` | R-side parameter defaults |
| `worker` / `no_worker` | Thread override |
| `check_interrupt` | Insert interrupt check |
| `coerce` / `no_coerce` | Type coercion override |
| `rng` | RNG state management |
| `error_in_r` / `unwrap_in_r` | Error handling override |
| `r_name = "..."` | Override R method name |
| `r_entry = "..."` | Inject R code at method entry |
| `r_post_checks = "..."` | Inject R code after checks |
| `r_on_exit = "..."` | Register `on.exit()` cleanup |

Valid `as = "..."` targets: `data.frame`, `list`, `character`, `numeric`, `double`,
`integer`, `logical`, `matrix`, `vector`, `factor`, `Date`, `POSIXct`, `complex`,
`raw`, `environment`, `function`.

#### R6-Specific Method Attributes

```rust
#[miniextendr(r6)]
impl MyClass {
    // Private method (R6 convention: prefixed with .)
    #[miniextendr(private)]
    pub fn internal_calc(&self) -> f64 { /* ... */ }

    // Active binding getter
    #[miniextendr(active, prop = "count")]
    pub fn get_count(&self) -> i32 { self.count }

    // Active binding setter
    #[miniextendr(setter, prop = "count")]
    pub fn set_count(&mut self, value: i32) { self.count = value; }

    // Destructor
    #[miniextendr(finalize)]
    pub fn cleanup(&mut self) { /* ... */ }
}
```

#### S7-Specific Method Attributes

```rust
#[miniextendr(s7)]
impl MyClass {
    // Computed property getter
    #[miniextendr(getter, prop = "area")]
    pub fn get_area(&self) -> f64 { self.width * self.height }

    // Property setter
    #[miniextendr(setter, prop = "area")]
    pub fn set_area(&mut self, value: f64) { /* ... */ }

    // Property validator
    #[miniextendr(validate, prop = "width")]
    pub fn validate_width(value: f64) -> Result<(), String> {
        if value < 0.0 { Err("width must be non-negative".into()) } else { Ok(()) }
    }

    // Property with defaults + constraints
    #[miniextendr(required)]      // no default, must be provided
    pub fn name(&self) -> String { self.name.clone() }

    #[miniextendr(frozen)]        // immutable after creation
    pub fn id(&self) -> i32 { self.id }

    #[miniextendr(default = "0")] // R expression for default
    pub fn score(&self) -> f64 { self.score }

    // Remove ... from generic signature
    #[miniextendr(no_dots)]
    pub fn length(&self) -> i32 { self.len }

    // Multiple dispatch
    #[miniextendr(dispatch = "x,y")]
    pub fn combine(&self, other: &Self) -> Self { /* ... */ }

    // Type conversion methods
    #[miniextendr(convert_from = "OtherClass")]
    pub fn from_other(other: OtherClass) -> Self { /* ... */ }

    #[miniextendr(convert_to = "OtherClass")]
    pub fn to_other(&self) -> OtherClass { /* ... */ }
}
```

---

## Trait Definitions

```rust
#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn default_initial() -> i32 { 0 }  // static method
}
```

Generates cross-package ABI:
- `TAG_COUNTER` — Type tag constant
- `CounterVTable` — Function pointer table
- Method shims — `extern "C"` trampolines with `with_r_unwind_protect`
- `CounterView` — Runtime dispatch wrapper

**No attributes accepted** on the `#[miniextendr]` for traits (the attr parameter is unused).

**Constraints:**
- Methods must take `&self` or `&mut self` (not `self` by value)
- No async methods
- No generic method parameters (trait-level generics are OK)
- Static methods work (resolved at compile time, not in vtable)

### Trait Impl Blocks

```rust
#[miniextendr(s3)]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.val }
    fn increment(&mut self) { self.val += 1; }
}
```

Use `blanket` to skip ABI emission for blanket impls:

```rust
#[miniextendr(blanket)]
impl<T: AsRef<str>> Display for T { /* ... */ }
```

---

## Structs

### 1-Field Struct → ALTREP (default)

```rust
#[miniextendr]
pub struct LazyInts(Vec<i32>);
```

Generates ALTREP class registration + `IntoR` + `TryFromSexp`. The struct wraps
a single data field and presents it as an R vector via ALTREP's lazy evaluation.

Override with ALTREP options:

```rust
#[miniextendr(class = "MyInts", base = "integer")]
pub struct LazyInts(Vec<i32>);
```

Override to use a different representation:

```rust
#[miniextendr(externalptr)]
pub struct Wrapper(Vec<i32>);   // ExternalPtr instead of ALTREP

#[miniextendr(list)]
pub struct Wrapper(Vec<i32>);   // List conversion instead of ALTREP
```

### Multi-Field Struct → ExternalPtr (default)

```rust
#[miniextendr]
pub struct Point { x: f64, y: f64 }
```

Generates `ExternalPtr` + `TypedExternal` derives. The struct lives as an opaque
R external pointer.

### Struct Mode Overrides

| Syntax | Result |
|--------|--------|
| `#[miniextendr]` on 1-field | ALTREP |
| `#[miniextendr]` on multi-field | ExternalPtr |
| `#[miniextendr(list)]` | `IntoList` + `TryFromList` + `PreferList` |
| `#[miniextendr(dataframe)]` | `IntoList` + `DataFrameRow` + companion type |
| `#[miniextendr(externalptr)]` | `ExternalPtr` + `TypedExternal` |
| `#[miniextendr(prefer = "list")]` | ExternalPtr + `PreferList` marker |
| `#[miniextendr(prefer = "dataframe")]` | ExternalPtr + `PreferDataFrame` marker |
| `#[miniextendr(prefer = "externalptr")]` | ExternalPtr (explicit) |
| `#[miniextendr(prefer = "native")]` | ExternalPtr + `PreferRNativeType` marker |

#### List Mode

```rust
#[miniextendr(list)]
pub struct Record {
    pub name: String,
    pub value: i32,
}

#[miniextendr]
pub fn make_record() -> Record {
    Record { name: "test".into(), value: 42 }
}
// R: list(name = "test", value = 42L)
```

#### DataFrame Mode

```rust
#[miniextendr(dataframe)]
pub struct Obs {
    pub id: i32,
    pub score: f64,
}

#[miniextendr]
pub fn make_obs() -> ObsDataFrame {  // companion type
    Obs::to_dataframe(vec![
        Obs { id: 1, score: 0.5 },
        Obs { id: 2, score: 0.8 },
    ])
}
// R: data.frame(id = c(1L, 2L), score = c(0.5, 0.8))
```

---

## Enums (Fieldless)

### Default → RFactor

```rust
#[miniextendr]
#[derive(Copy, Clone)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}
```

Generates `MatchArg` + `RFactor` + `IntoR` + `TryFromSexp`. The enum maps to
an R factor with levels matching variant names.

```r
s <- get_season()  # factor("Summer", levels = c("Spring", "Summer", "Autumn", "Winter"))
```

### MatchArg Mode

```rust
#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}
```

Like RFactor but uses R's `match.arg()` semantics with partial matching. The enum
maps to a character scalar in R, validated against the allowed choices.

### Explicit Factor

```rust
#[miniextendr(factor)]  // same as default, but explicit
pub enum Color { Red, Green, Blue }
```

---

## `miniextendr_module!`

Every `#[miniextendr]` item must be registered in a module declaration:

```rust
miniextendr_module! {
    mod my_module;

    // Functions
    fn greet;
    fn add;

    // Impl blocks (covers all methods in all impl blocks for this type)
    impl Counter;

    // ALTREP structs (needed for class registration)
    struct LazyInts;

    // Sub-modules
    use child_module;

    // Conditional compilation
    #[cfg(feature = "extras")]
    use extras_module;
}
```

**Rules:**
- Functions → `fn name;`
- Types with impl blocks → `impl TypeName;`
- ALTREP structs → `struct TypeName;`
- Sub-modules → `use module_name;`
- Structs/enums without impl blocks (list, dataframe, factor, match_arg) do NOT need entries
- `#[cfg(...)]` gates must match between `mod` declaration and `use` in parent

---

## Derive Macros

These are separate from `#[miniextendr]` but complementary:

| Derive | What It Generates |
|--------|-------------------|
| `ExternalPtr` | Type-safe external pointer wrapper |
| `Altrep` | Full ALTREP class (IntoR + TryFromSexp + registration) |
| `AltrepInteger` / `AltrepReal` / ... | ALTREP for specific vector types |
| `IntoList` | Struct → R list |
| `TryFromList` | R list → Struct |
| `DataFrameRow` | Struct → columnar data.frame + companion type |
| `RFactor` | Enum ↔ R factor |
| `MatchArg` | Enum ↔ R character with `match.arg()` |
| `RNativeType` | Newtype for native R types |
| `PreferList` | Marker: route `IntoR` through list |
| `PreferDataFrame` | Marker: route `IntoR` through data.frame |
| `PreferExternalPtr` | Marker: route `IntoR` through ExternalPtr |
| `PreferRNativeType` | Marker: route `IntoR` through native SEXP |
| `Vctrs` | vctrs-compatible S3 class (feature-gated) |

The `#[miniextendr]` attribute on structs/enums is a convenience that calls the
appropriate derives internally. Both paths produce identical code.

## See Also

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) — All 6 class systems with examples
- [S3_METHODS.md](S3_METHODS.md) — Detailed S3 print/format guide
- [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md) — Dots and typed_list validation
- [ALTREP.md](ALTREP.md) — ALTREP deep dive
- [DATAFRAME.md](DATAFRAME.md) — DataFrame conversion (derive + serde + columnar)
- [SERDE_R.md](SERDE_R.md) — serde integration for direct Rust-R serialization
- [ERROR_HANDLING.md](ERROR_HANDLING.md) — error_in_r, unwrap_in_r, panic handling
- [LIFECYCLE.md](LIFECYCLE.md) — Deprecation/experimental lifecycle attributes
- [TRAIT_ABI.md](TRAIT_ABI.md) — Cross-package trait dispatch
