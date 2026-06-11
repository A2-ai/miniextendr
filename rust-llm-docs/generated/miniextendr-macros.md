# miniextendr_macros v0.1.0

## miniextendr-macros - Procedural macros for Rust <-> R interop

This crate provides the procedural macros that power miniextendr's code
generation. Most users should depend on `miniextendr-api` and use its
re-exports, but this crate can be used directly when you only need macros.

Primary macros and derives:
- `#[miniextendr]` on functions, impl blocks, trait defs, and trait impls.
- `#[r_ffi_checked]` for main-thread routing of C-ABI wrappers.
- Derives: `ExternalPtr`, `RNativeType`, ALTREP derives, `RFactor`.
- Helpers: `typed_list` for typed list builders.

R wrapper generation is driven by Rust doc comments (roxygen tags are
extracted). During package build, the cdylib link pass loads the crate
into R and calls `miniextendr_write_wrappers`, which walks the linkme
`#[distributed_slice]` tables and writes `R/<pkg>-wrappers.R`.

### Quick start

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Macro expansion pipeline

#### Overview

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on fn                             │
│                                                                          │
│  1. Parse: syn::ItemFn → MiniextendrFunctionParsed                       │
│  2. Analyze return type (Result<T>, Option<T>, raw SEXP, etc.)           │
│  3. Generate:                                                            │
│     ├── C wrapper: extern "C-unwind" fn C_<name>(call: SEXP, ...) → SEXP │
│     ├── R wrapper: const R_WRAPPER_<NAME>: &str = "..."                  │
│     └── Registration: const call_method_def_<name>: R_CallMethodDef      │
│  4. Original function preserved (with added attributes)                  │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr(env|r6|s3|s4|s7)] on impl               │
│                                                                          │
│  1. Parse: syn::ItemImpl → extract methods                               │
│  2. For each method:                                                     │
│     ├── Generate C wrapper (handles self parameter)                      │
│     ├── Generate R method wrapper string                                 │
│     └── Generate registration entry                                      │
│  3. Generate class definition code per class system:                     │
│     ├── env: new.env() + method assignment                               │
│     ├── r6: R6Class() definition                                         │
│     ├── s3: S3 generics + methods                                        │
│     ├── s4: setClass() + setMethod()                                     │
│     └── s7: new_class() definition                                       │
│  4. Emit const with combined R code                                      │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                         #[miniextendr] on trait                          │
│                                                                          │
│  1. Parse: syn::ItemTrait → extract method signatures                    │
│  2. Generate:                                                            │
│     ├── Trait tag constant: const TAG_<TRAIT>: mx_tag = ...              │
│     ├── Vtable struct: struct __vtable_<Trait> { ... }                   │
│     └── CCalls table: static MX_CCALL_<TRAIT>: [...] = ...               │
│  3. Original trait preserved                                             │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    #[miniextendr] impl Trait for Type                    │
│                                                                          │
│  1. Parse: syn::ItemImpl (trait impl)                                    │
│  2. Generate:                                                            │
│     ├── Vtable instance: static __VTABLE_<TRAIT>_FOR_<TYPE>: ...         │
│     ├── Wrapper struct: struct __MxWrapper<Type> { erased, data }        │
│     ├── Query function: fn __mx_query_<type>(tag) → vtable ptr           │
│     └── Base vtable: static __MX_BASE_VTABLE_<TYPE>: ...                 │
│  3. Original impl preserved                                              │
└──────────────────────────────────────────────────────────────────────────┘

```

#### Key Modules

| Module | Purpose |
|--------|---------|
| `miniextendr_fn` | Function parsing and attribute handling |
| `c_wrapper_builder` | C wrapper generation (`extern "C-unwind"`) |
| `r_wrapper_builder` | R wrapper code generation |
| `rust_conversion_builder` | Rust→SEXP return value conversion |
| `miniextendr_impl` | `impl Type` block processing |
| `r_class_formatter` | Class system code generation (env/r6/s3/s4/s7) |
| `miniextendr_trait` | Trait ABI metadata generation |
| `miniextendr_impl_trait` | `impl Trait for Type` vtable generation |
| `altrep` / `altrep_derive` | ALTREP struct derivation |
| `externalptr_derive` | `#[derive(ExternalPtr)]` |
| `roxygen` | Roxygen doc comment handling |

#### Generated Symbol Naming

For a function `my_func`:
- C wrapper: `C_my_func`
- R wrapper const: `R_WRAPPER_MY_FUNC`
- Registration: `call_method_def_my_func`

For a type `MyType` with trait `Counter`:
- Vtable: `__VTABLE_COUNTER_FOR_MYTYPE`
- Wrapper: `__MxWrapperMyType`
- Query: `__mx_query_mytype`

### Return Type Handling

The `return_type_analysis` module determines how to convert Rust returns to SEXP:

| Rust Type | Strategy | R Result |
|-----------|----------|----------|
| `T: IntoR` | `.into_sexp()` | Converted value |
| `Result<T, E>` | Unwrap or R error | Value or error |
| `Option<T>` | `Some` → value, `None` → `NULL` | Value or NULL |
| `SEXP` | Pass through | Raw SEXP |
| `()` | Invisible NULL | `invisible(NULL)` |

Use `#[miniextendr(unwrap_in_r)]` to return `Result<T, E>` to R without unwrapping.

### Thread Strategy

By default, `#[miniextendr]` functions run on R's main thread. Opt into
worker-thread execution with `#[miniextendr(worker)]` (requires the
`worker-thread` feature on `miniextendr-api`). A worker opt-in is ignored
when the signature requires main-thread execution (returns/takes `SEXP`,
uses variadic dots, or sets `check_interrupt`).

**Note**: `ExternalPtr<T>` is `Send` — it can be returned from worker
thread functions. All R API operations on `ExternalPtr` are serialized
through `with_r_thread`.

### Class Systems

The `r_class_formatter` module generates R code for different class systems:

| System | Generated R Code | Self Parameter |
|--------|------------------|----------------|
| `env` | `new.env()` with methods | `self` environment |
| `r6` | `R6Class()` | `self` environment |
| `s3` | `structure()` + generics | First argument |
| `s4` | `setClass()` + `setMethod()` | First argument |
| `s7` | `new_class()` | `self` property |

---

## Macros

### `#[derive(Altrep)]`

Derive ALTREP registration for a data struct.

Generates `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`,
linkme registration entry, and `Ref`/`Mut` accessor types.

The struct must already have low-level ALTREP traits implemented.
For most use cases, prefer a family-specific derive:
`#[derive(AltrepInteger)]`, `#[derive(AltrepReal)]`, etc.
Use `#[altrep(manual)]` on a family derive to skip data trait generation
when you provide your own `AltrepLen` + `Alt*Data` impls.

#### Attributes

- `#[altrep(class = "Name")]` — custom ALTREP class name (defaults to struct name)

#### Example

```ignore
// Prefer family derives with manual:
#[derive(AltrepInteger)]
#[altrep(manual, class = "MyCustom", serialize)]
struct MyData { ... }

impl AltrepLen for MyData { ... }
impl AltIntegerData for MyData { ... }
```

### `#[derive(AltrepComplex)]`

Derive macro for ALTREP complex vector data types.

Auto-implements `AltrepLen` and `AltComplexData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepInteger)]`

Derive macro for ALTREP integer vector data types.

Auto-implements `AltrepLen`, `AltIntegerData`, and the low-level ALTREP
trait impls (`Altrep`, `AltVec`, `AltInteger`, `InferBase`).

#### Attributes

- `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
- `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
- `#[altrep(dataptr)]` - Enable direct data-pointer access
- `#[altrep(serialize)]` - Enable ALTREP serialization support
- `#[altrep(subset)]` - Enable `Extract_subset` optimization
- `#[altrep(no_lowlevel)]` - Skip the automatic low-level trait impls

#### Example (Constant Vector - Zero Boilerplate!)

```ignore
#[derive(ExternalPtr, AltrepInteger)]
#[altrep(elt = "value")]  // All elements return this field
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// That's it! 3 lines instead of 30!
// AltrepLen, AltIntegerData, and low-level impls are auto-generated

#[miniextendr(class = "ConstantInt")]
pub struct ConstantIntClass(pub ConstantIntData);
```

#### Example (Custom elt() - Override One Method)

```ignore
#[derive(ExternalPtr, AltrepInteger)]
pub struct ArithSeqData {
    start: i32,
    step: i32,
    len: usize,
}

// Auto-generates AltrepLen and stub AltIntegerData
// Just override elt() for custom logic:
impl AltIntegerData for ArithSeqData {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step
    }
}
```

### `#[derive(AltrepList)]`

Derive macro for ALTREP list vector data types.

Auto-implements `AltrepLen` and `AltListData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger),
except `dataptr` and `subset` which are not supported for list ALTREP.

### `#[derive(AltrepLogical)]`

Derive macro for ALTREP logical vector data types.

Auto-implements `AltrepLen` and `AltLogicalData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepRaw)]`

Derive macro for ALTREP raw vector data types.

Auto-implements `AltrepLen` and `AltRawData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepReal)]`

Derive macro for ALTREP real vector data types.

Auto-implements `AltrepLen` and `AltRealData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(AltrepString)]`

Derive macro for ALTREP string vector data types.

Auto-implements `AltrepLen` and `AltStringData` traits.
Supports the same `#[altrep(...)]` attributes as [`AltrepInteger`](derive@AltrepInteger).

### `#[derive(DataFrameRow)]`

Derive `DataFrameRow`: generates a companion `*DataFrame` type with collection fields,
plus `IntoR` / `TryFromSexp` / `IntoDataFrame` impls for seamless R data.frame conversion.

#### Example

```ignore
#[derive(DataFrameRow)]
struct Measurement {
    time: f64,
    value: f64,
}

// Generates MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }
// plus conversion impls
```

#### Struct-level attributes

- `#[dataframe(name = "CustomDf")]` — custom name for the generated DataFrame type
- `#[dataframe(align)]` — pad shorter columns with NA to match longest
- `#[dataframe(tag = "my_tag")]` — attach a tag attribute to the data.frame
- `#[dataframe(conflicts = "string")]` — resolve conflicting column types as strings

#### Field-level attributes

- `#[dataframe(skip)]` — omit this field from the DataFrame
- `#[dataframe(rename = "col")]` — custom column name
- `#[dataframe(as_list)]` — keep collection as single list column (no expansion)
- `#[dataframe(expand)]` / `#[dataframe(unnest)]` — expand collection into suffixed columns
- `#[dataframe(width = N)]` — pin expansion width (shorter rows get NA)

### `#[derive(ExternalPtr)]`

Derive macro for implementing `TypedExternal` on a type.

This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.

#### Basic Usage

```ignore
use miniextendr_api::TypedExternal;

#[derive(ExternalPtr)]
struct MyData {
    value: i32,
}

// Now you can use ExternalPtr<MyData>
let ptr = ExternalPtr::new(MyData { value: 42 });
```

#### Trait ABI

Trait dispatch wrappers are automatically generated:

```ignore
use miniextendr_api::miniextendr;

#[derive(ExternalPtr)]
struct MyCounter {
    value: i32,
}

#[miniextendr]
impl Counter for MyCounter {
    fn value(&self) -> i32 { self.value }
    fn increment(&mut self) { self.value += 1; }
}
```

This generates additional infrastructure for type-erased trait dispatch:
- `__MxWrapperMyCounter` - Type-erased wrapper struct
- `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
- `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`

#### Generated Code (Basic)

For a type `MyData` without traits:

```ignore
impl TypedExternal for MyData {
    const TYPE_NAME: &'static str = "MyData";
    const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
}
```

### `#[derive(IntoList)]`

Derive `IntoList` for a struct (Rust → R list).

- Named structs → named R list: `list(x = 1L, y = 2L)`
- Tuple structs → unnamed R list: `list(1L, 2L)`
- Fields annotated `#[into_list(ignore)]` are skipped

### `#[derive(IntoR)]`

Derive `IntoR` for a single-field newtype: forward the Rust → R conversion to
the inner type.

Generates a scalar `IntoR` impl that delegates to the inner type, plus an
`IntoRNewtype` marker (powering the `Option<T>` / `Vec<Option<T>>` container
blankets) and a concrete `IntoRVecElement` impl (powering `Vec<T>`). See
`#[derive(TryFromSexp)]` for usage.

Do not derive both `IntoR` and `MatchArg` on the same type: both feed the
single `IntoR for Vec<T>` blanket slot and would collide (E0119).

### `#[derive(MatchArg)]`

Derive `MatchArg`: enables conversion between Rust enums and R character strings
with `match.arg` semantics (partial matching, informative errors).

#### Usage

```ignore
#[derive(Copy, Clone, MatchArg)]
enum Mode {
    Fast,
    Safe,
    Debug,
}
```

#### Attributes

- `#[match_arg(rename = "name")]` - Rename a variant's choice string
- `#[match_arg(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)

#### Generated Implementations

- `MatchArg` - Choice metadata and bidirectional conversion
- `TryFromSexp` - Convert R STRSXP/factor to enum (with partial matching)
- `IntoR` - Convert enum to R character scalar

### `#[derive(PreferDataFrame)]`

Derive `PreferDataFrame`: when a type implements both `IntoDataFrame` (via `DataFrameRow`)
and other conversion paths, this selects data.frame as the default `IntoR` conversion.

#### Example

```ignore
#[derive(DataFrameRow, PreferDataFrame)]
struct Obs { time: f64, value: f64 }
// IntoR produces data.frame(time = ..., value = ...)
```

### `#[derive(PreferExternalPtr)]`

Derive `PreferExternalPtr`: when a type implements both `ExternalPtr` and
other conversion paths (e.g., `IntoList`), this selects `ExternalPtr` wrapping
as the default `IntoR` conversion.

#### Example

```ignore
#[derive(ExternalPtr, IntoList, PreferExternalPtr)]
struct Model { weights: Vec<f64> }
// IntoR wraps as ExternalPtr (opaque R object), not list
```

### `#[derive(PreferList)]`

Derive `PreferList`: emits an `IntoR` impl selecting list as the type's default
Rust→R conversion (via `IntoList::into_list`).

A type carries exactly one representation default: stacking two `Prefer*`
derives is a compile error. Each `Prefer*` derive emits a fixed-name marker
const, so a second one triggers a guided `duplicate definitions with name
__miniextendr_conflicting_Prefer_derives__keep_ONE_or_use_call_site_As_wrappers`
error (alongside the raw conflicting-`IntoR`-impl error) — keep one `Prefer*`,
or drop them all and choose a representation per return value at the call site
with an `As*` wrapper (`AsList`, `AsExternalPtr`, `AsDataFrame`, ...).

#### Example

```ignore
#[derive(IntoList, PreferList)]
struct Config { verbose: bool, threads: i32 }
// IntoR produces list(verbose = TRUE, threads = 4L)
```

### `#[derive(PreferRNativeType)]`

Derive `PreferRNativeType`: when a newtype wraps an `RNativeType` and also
implements other conversions, this selects the native R vector conversion
as the default `IntoR` path.

#### Example

```ignore
#[derive(Copy, Clone, RNativeType, PreferRNativeType)]
struct Meters(f64);
// IntoR produces a numeric scalar, not an ExternalPtr
```

### `#[derive(RFactor)]`

Derive `RFactor`: enables conversion between Rust enums and R factors.

#### Usage

```ignore
#[derive(Copy, Clone, RFactor)]
enum Color {
    Red,
    Green,
    Blue,
}
```

#### Attributes

- `#[r_factor(rename = "name")]` - Rename a variant's level string
- `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)

### `#[derive(RNativeType)]`

Derive macro for implementing `RNativeType` on a newtype wrapper.

This allows newtype wrappers around R native types to work with `Vec<T>`,
`&[T]` conversions and the `Coerce<R>` traits.
The inner type must implement `RNativeType`.

#### Supported Struct Forms

Both tuple structs and single-field named structs are supported:

```ignore
use miniextendr_api::RNativeType;

// Tuple struct (most common)
#[derive(Clone, Copy, RNativeType)]
struct UserId(i32);

// Named single-field struct
#[derive(Clone, Copy, RNativeType)]
struct Temperature { celsius: f64 }
```

#### Generated Code

For `struct UserId(i32)`, this generates:

```ignore
impl RNativeType for UserId {
    const SEXP_TYPE: SEXPTYPE = <i32 as RNativeType>::SEXP_TYPE;
    const R_NA: Self = UserId(<i32 as RNativeType>::R_NA);

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        <i32 as RNativeType>::dataptr_mut(sexp).cast()
    }
}
```

#### Using the Newtype with Coerce

Once `RNativeType` is derived, you can implement `Coerce` to/from the newtype:

```ignore
impl Coerce<UserId> for i32 {
    fn coerce(self) -> UserId { UserId(self) }
}

let id: UserId = 42.coerce();
```

#### Requirements

- Must be a newtype struct (exactly one field, tuple or named)
- The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
- Should also derive `Copy` (required by `RNativeType: Copy`)

### `#[derive(TryFromList)]`

Derive `TryFromList` for a struct (R list → Rust).

- Named structs: extract by field name
- Tuple structs: extract by position (0, 1, 2, ...)
- Fields annotated `#[into_list(ignore)]` are not read and are initialized with `Default::default()`

### `#[derive(TryFromSexp)]`

Derive `TryFromSexp` for a single-field newtype: forward the R → Rust
conversion to the inner type.

Generates a scalar `TryFromSexp` impl that delegates to the inner type (so the
newtype inherits its exact SEXPTYPE checks, NA policy, and error text), plus a
`FromRNewtype` marker impl. The marker lets `miniextendr-api`'s container
blankets light up `Vec<T>` / `Option<T>` / `Vec<Option<T>>` automatically.

#### Usage

```ignore
use uuid::Uuid;

#[derive(TryFromSexp)]            // R -> Rust only
struct Pattern(regex::Regex);

#[derive(TryFromSexp, IntoR)]     // round-trip; Vec/Option containers work too
struct UserId(Uuid);
```

Direction is chosen by which derive you list — derive only `TryFromSexp` for
inner types that read from R but cannot be written back (e.g. `regex::Regex`).

### `impl_typed_external!`

Generate `TypedExternal` and `IntoExternalPtr` impls for a concrete monomorphization
of a generic type.

Since `#[derive(ExternalPtr)]` rejects generic types, use this macro to generate
the necessary impls for a specific type instantiation.

#### Example

```ignore
struct Wrapper<T> { inner: T }

impl_typed_external!(Wrapper<i32>);
impl_typed_external!(Wrapper<String>);
```

### `list!`

Construct an R list from Rust values.

This macro provides a convenient way to create R lists in Rust code,
using R-like syntax. Values are converted to R objects via the [`IntoR`] trait.

#### Syntax

```ignore
// Named entries (like R's list())
list!(
    alpha = 1,
    beta = "hello",
    "my-name" = vec![1, 2, 3],
)

// Unnamed entries
list!(1, "hello", vec![1, 2, 3])

// Mixed (unnamed entries get empty string names)
list!(alpha = 1, 2, beta = "hello")

// Empty list
list!()
```

#### Examples

```ignore
use miniextendr_api::{list, IntoR};

// Create a named list
let my_list = list!(
    x = 42,
    y = "hello world",
    z = vec![1.0, 2.0, 3.0],
);

// In R this is equivalent to:
// list(x = 42L, y = "hello world", z = c(1, 2, 3))
```

[`IntoR`]: https://docs.rs/miniextendr-api/latest/miniextendr_api/into_r/trait.IntoR.html

### `#[miniextendr]`

Export Rust items to R.

`#[miniextendr]` can be applied to:
- `fn` items (generate C + R wrappers)
- `impl` blocks (generate R class methods)
- `trait` items (generate trait ABI metadata)
- ALTREP wrapper structs (generate `RegisterAltrep` impls)

#### Functions

```ignore
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 { a + b }
```

This produces a C wrapper `C_add` and an R wrapper `add()`.
Registration is automatic via linkme distributed slices.

##### `extern "C-unwind"`

If the function is declared `extern "C-unwind"` and exported with
`#[no_mangle]` (2021), `#[unsafe(no_mangle)]` (2024), or `#[export_name = "..."]`,
the function itself is the C symbol and the R wrapper is prefixed with
`unsafe_` to signal bypassed safety (no worker isolation or conversion).

##### Variadics (`...`)

Use `...` as the last argument. The Rust parameter becomes `_dots: &Dots`.
Use `name @ ...` to give it a custom name (e.g., `args @ ...` → `args: &Dots`).

###### Typed Dots Validation

Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
and create a `dots_typed` variable with typed accessors:

```ignore
#[miniextendr(dots = typed_list!(x => numeric(), y => integer(), z? => character()))]
pub fn my_func(...) -> String {
    let x: f64 = dots_typed.get("x").expect("x");
    let y: i32 = dots_typed.get("y").expect("y");
    let z: Option<String> = dots_typed.get_opt("z").expect("z");
    format!("x={}, y={}", x, y)
}
```

Type specs: `numeric()`, `integer()`, `logical()`, `character()`, `list()`,
`raw()`, `complex()`, or `"class_name"` for class inheritance checks.
Add `(n)` for exact length: `numeric(4)`. Use `?` suffix for optional fields.
Use `@exact;` prefix for strict mode (reject extra fields).

##### Attributes

- `#[miniextendr(worker)]` — opt into worker-thread execution
- `#[miniextendr(invisible)]` / `#[miniextendr(visible)]` — control return visibility
- `#[miniextendr(check_interrupt)]` — check for user interrupt after call
- `#[miniextendr(coerce)]` — coerce R type before conversion (also usable per-parameter)
- `#[miniextendr(strict)]` — reject lossy conversions for i64/u64/isize/usize
- `#[miniextendr(unwrap_in_r)]` — return `Result<T, E>` to R without unwrapping
- `#[miniextendr(dots = typed_list!(...))]` — validate dots, create `dots_typed`
- `#[miniextendr(internal)]` — adds `@keywords internal` to R wrapper
- `#[miniextendr(noexport)]` — suppresses `@export` from R wrapper

#### Impl blocks (class systems)

Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block.
Use `#[miniextendr(label = "...")]` to disambiguate multiple impl blocks
on the same type.
Registration is automatic.

##### R6 Active Bindings

For R6 classes, use `#[miniextendr(r6(active))]` on methods to create
active bindings (computed properties accessed without parentheses):

```ignore
use miniextendr_api::miniextendr;

pub struct Rectangle {
    width: f64,
    height: f64,
}

#[miniextendr(r6)]
impl Rectangle {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Returns the area (width * height).
    #[miniextendr(r6(active))]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Regular method (requires parentheses).
    pub fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }
}
```

In R:
```r
r <- Rectangle$new(3, 4)
r$area        # 12 (active binding - no parentheses!)
r$scale(2)    # Regular method call
r$area        # 24
```

Active bindings must be getter-only methods taking only `&self`.

##### S7 Properties

For S7 classes, use `#[miniextendr(s7(getter))]` and `#[miniextendr(s7(setter))]`
to create computed properties accessed via `@`:

```ignore
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Range {
    start: f64,
    end: f64,
}

#[miniextendr(s7)]
impl Range {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Computed property (read-only): length of the range.
    #[miniextendr(s7(getter))]
    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Dynamic property getter.
    #[miniextendr(s7(getter, prop = "midpoint"))]
    pub fn get_midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }

    /// Dynamic property setter.
    #[miniextendr(s7(setter, prop = "midpoint"))]
    pub fn set_midpoint(&mut self, value: f64) {
        let half = self.length() / 2.0;
        self.start = value - half;
        self.end = value + half;
    }
}
```

In R:
```r
r <- Range(0, 10)
r@length     # 10 (computed, read-only)
r@midpoint   # 5 (dynamic property)
r@midpoint <- 20  # Adjusts start/end to center at 20
```

###### Property Attributes

- `#[miniextendr(s7(getter))]` - Read-only computed property
- `#[miniextendr(s7(getter, prop = "name"))]` - Named property getter
- `#[miniextendr(s7(setter, prop = "name"))]` - Named property setter
- `#[miniextendr(s7(getter, default = "0.0"))]` - Property with default value
- `#[miniextendr(s7(getter, required))]` - Required property (error if not provided)
- `#[miniextendr(s7(getter, frozen))]` - Property that can only be set once
- `#[miniextendr(s7(getter, deprecated = "Use X instead"))]` - Deprecated property
- `#[miniextendr(s7(validate))]` - Validator function for property

##### S7 Generic Dispatch Control

Control how S7 generics are created:

- `#[miniextendr(s7(no_dots))]` - Create strict generic without `...`
- `#[miniextendr(s7(dispatch = "x,y"))]` - Multi-dispatch on multiple arguments
- `#[miniextendr(s7(fallback))]` - Register method for `class_any` (catch-all).
  The generated R wrapper uses `tryCatch(x@.ptr, error = function(e) x)` to
  safely extract the self argument, so non-miniextendr objects won't crash with
  a slot-access error. Instead, incompatible objects produce a Rust type-conversion
  error when the method tries to interpret the argument as `&Self`.

```ignore
#[miniextendr(s7)]
impl MyClass {
    /// Strict generic: function(x) instead of function(x, ...)
    #[miniextendr(s7(no_dots))]
    pub fn strict_method(&self) -> i32 { 42 }

    /// Fallback method dispatched on class_any.
    /// Calling this on a non-MyClass object produces a type-conversion error,
    /// not a slot-access crash.
    #[miniextendr(s7(fallback))]
    pub fn describe(&self) -> String { "generic description".into() }
}
```

##### S7 Type Conversion (`convert`)

Use `convert_from` and `convert_to` to enable S7's `convert()` for type coercion:

```ignore
use miniextendr_api::{miniextendr, ExternalPtr};

#[derive(ExternalPtr)]
pub struct Celsius { value: f64 }

#[derive(ExternalPtr)]
pub struct Fahrenheit { value: f64 }

#[miniextendr(s7)]
impl Fahrenheit {
    pub fn new(value: f64) -> Self { Self { value } }

    /// Convert FROM Celsius TO Fahrenheit.
    /// Usage: S7::convert(celsius_obj, Fahrenheit)
    #[miniextendr(s7(convert_from = "Celsius"))]
    pub fn from_celsius(c: ExternalPtr<Celsius>) -> Self {
        Fahrenheit { value: c.value * 9.0 / 5.0 + 32.0 }
    }

    /// Convert FROM Fahrenheit TO Celsius.
    /// Usage: S7::convert(fahrenheit_obj, Celsius)
    #[miniextendr(s7(convert_to = "Celsius"))]
    pub fn to_celsius(&self) -> Celsius {
        Celsius { value: (self.value - 32.0) * 5.0 / 9.0 }
    }
}
```

In R:
```r
c <- Celsius(100)
f <- S7::convert(c, Fahrenheit)  # Uses convert_from
c2 <- S7::convert(f, Celsius)    # Uses convert_to
```

**Note:** Classes must be defined before they can be referenced in convert methods.
Define the "from" class before the "to" class to avoid forward reference issues.

#### Traits (ABI)

Apply `#[miniextendr]` to a trait to generate ABI metadata, then use
`#[miniextendr] impl Trait for Type`. Registration is automatic.

#### ALTREP

Apply `#[miniextendr(class = "...", base = "...")]` to a one-field
wrapper struct. Registration is automatic.

### `miniextendr_init!`

Generate the `R_init_*` entry point for a miniextendr R package.

This macro consolidates all package initialization into a single line.
It generates an `extern "C-unwind"` function that R calls when loading
the shared library.

#### Usage

```ignore
// Auto-detects package name from CARGO_CRATE_NAME (recommended):
miniextendr_api::miniextendr_init!();

// Or specify explicitly (for edge cases):
miniextendr_api::miniextendr_init!(mypkg);
```

The generated function calls `miniextendr_api::init::package_init` which
handles panic hooks, runtime init, locale assertion, ALTREP setup, trait ABI
registration, routine registration, and symbol locking.

### `#[r_ffi_checked]`

Generate thread-safe wrappers for R FFI functions.

Apply this to an `extern "C-unwind"` block to generate, **for each
non-variadic function**, a pair of entry points:

- The original name (e.g. `Rf_allocVector`) — a safe Rust wrapper that
  debug-asserts the caller is on R's main thread, routing through
  `miniextendr_api::worker::with_r_thread` when called from a worker.
- A `*_unchecked` sibling (`Rf_allocVector_unchecked`) — the raw
  `extern "C-unwind"` declaration with no main-thread assertion and no
  worker round-trip.

User code should reach for the checked variant by default; the unchecked
sibling exists for three known-safe contexts:

1. **Inside ALTREP callbacks** — R is already calling us on the main
   thread, so the assertion would always pass and the route would
   deadlock the call back to R.
2. **Inside a `with_r_unwind_protect` body** — the guard has established
   main-thread context, and re-entering `with_r_thread` would nest two
   `R_UnwindProtect` frames (paying the longjmp-leak cost twice).
3. **Inside a `with_r_thread` body** — the assertion is redundant; you
   are already where you needed to be.

The build-time lint **MXL301** enforces this: calling `*_unchecked`
outside one of those three contexts is a compile-time error. Outside
the worker-thread feature gate, the checked variant collapses to a thin
call and the two variants are observationally identical, but the lint
still applies so the same code is correct under `--features worker-thread`.

#### Tradeoffs at a glance

| Variant | Asserts main thread | Routes to main | When to use |
|---|---|---|---|
| `Rf_foo` (checked) | yes (debug) | yes (from worker) | default |
| `Rf_foo_unchecked` | no | no | ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread` |

#### Behavior

All non-variadic functions are routed to the main thread via `with_r_thread`
when called from a worker thread. The return value is wrapped in `Sendable`
and sent back to the caller. This applies to both value-returning functions
(SEXP, i32, etc.) and pointer-returning functions (`*const T`, `*mut T`).

Pointer-returning functions (like `INTEGER`, `REAL`) are safe to route because
the underlying SEXP must be GC-protected by the caller, and R's GC only runs
during R API calls which are serialized through `with_r_thread`.

#### Initialization Requirement

`miniextendr_runtime_init()` must be called before using any wrapped function.
Calling before initialization will panic with a descriptive error message.

#### Limitations

- Variadic functions are passed through unchanged (no wrapper)
- Statics are passed through unchanged
- Functions with `#[link_name]` are passed through unchanged

#### Example

```ignore
#[r_ffi_checked]
unsafe extern "C-unwind" {
    // Routed to main thread via with_r_thread when called from worker
    pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut i32;
}
```

### `typed_dataframe!`

Define a compile-time-validated wrapper for an R `data.frame` input.

`typed_dataframe!` mirrors [`typed_list!`] for the data.frame shape:
declare the columns once, get a struct that implements `TryFromSexp`
(validating both the `data.frame` class and per-column SEXPTYPE) plus
per-column borrowed accessors that return `&[T]`.

#### Syntax

```ignore
typed_dataframe! {
    /// The shape we accept for the Theoph PK dataset.
    pub TheophDf {
        subject: i32,
        weight: f64,
        dose: f64,
        flag: Option<i32>,   // optional column
    }
}
```

For strict mode (reject any column not declared):
```ignore
typed_dataframe! {
    @exact;
    pub Strict { x: i32 }
}
```

#### Supported element types

v1 supports column element types that implement
`miniextendr_api::RNativeType`:

- `i32` — `INTSXP`
- `f64` — `REALSXP`
- `u8` — `RAWSXP`
- `miniextendr_api::RLogical` — `LGLSXP`
- `miniextendr_api::Rcomplex` — `CPLXSXP`

`String`/`&str` column types are not yet supported (character vectors
don't expose a contiguous slice). `bool` is also not yet supported as
a direct field type — use `RLogical` and convert per-element, or
follow the open follow-up issues from PR #698.

#### Generated API

For each `name: T` column the macro emits:
- `pub fn name(&self) -> &[T]` (required)
- `pub fn name(&self) -> Option<&[T]>` (optional, `Option<T>`)

Plus housekeeping:
- `pub fn nrow(&self) -> usize`
- `pub fn ncol(&self) -> usize` (count of *declared* columns)
- `pub fn as_sexp(&self) -> SEXP`

All borrowed accessors are bound to `&self`; the SEXP is protected
by the surrounding `#[miniextendr]` call wrapper while the struct is
alive.

#### Error reporting

`TryFromSexp::try_from_sexp` batches every per-column error into a
single `SexpError::InvalidValue`, so the R user sees one diagnostic
covering all missing or wrong-typed columns rather than a sequence of
stop-on-first-failure messages.

#### Example

```ignore
use miniextendr_api::{miniextendr, typed_dataframe};

typed_dataframe! {
    pub TheophDf {
        subject: i32,
        weight: f64,
        dose: f64,
    }
}

#[miniextendr]
pub fn theoph_nrow(df: TheophDf) -> i32 {
    // df.subject() -> &[i32], df.weight() -> &[f64]
    // Lengths are guaranteed equal across columns (data.frame invariant).
    df.nrow() as i32
}
```

[`typed_list!`]: macro@typed_list

### `typed_list!`

Create a `TypedListSpec` for validating `...` arguments or lists.

This macro provides ergonomic syntax for defining typed list specifications
that can be used with `Dots::typed()` to validate the structure of
`...` arguments passed from R.

#### Syntax

```text
typed_list!(
    name => type_spec,    // required field with type
    name? => type_spec,   // optional field with type
    name,                 // required field, any type
    name?,                // optional field, any type
)
```

For strict mode (no extra fields allowed):
```text
typed_list!(@exact; name => type_spec, ...)
```

#### Type Specifications

##### Base types (with optional length)
- `numeric()` / `numeric(4)` - Real/double vector
- `integer()` / `integer(4)` - Integer vector
- `logical()` / `logical(4)` - Logical vector
- `character()` / `character(4)` - Character vector
- `raw()` / `raw(4)` - Raw vector
- `complex()` / `complex(4)` - Complex vector
- `list()` / `list(4)` - List (VECSXP)

##### Special types
- `data_frame()` - Data frame
- `factor()` - Factor
- `matrix()` - Matrix
- `array()` - Array
- `function()` - Function
- `environment()` - Environment
- `null()` - NULL only
- `any()` - Any type

##### String literals
- `"numeric"`, `"integer"`, etc. - Same as call syntax
- `"data.frame"` - Data frame (alias)
- `"MyClass"` - Any other string is treated as a class name (uses `Rf_inherits`)

#### Examples

##### Basic usage

```ignore
use miniextendr_api::{miniextendr, typed_list, Dots};

#[miniextendr]
pub fn process_args(dots: ...) -> Result<i32, String> {
    let args = dots.typed(typed_list!(
        alpha => numeric(4),
        beta => list(),
        gamma? => "character",
    )).map_err(|e| e.to_string())?;

    let alpha: Vec<f64> = args.get("alpha").map_err(|e| e.to_string())?;
    Ok(alpha.len() as i32)
}
```

##### Strict mode

```ignore
// Reject any extra named fields
let args = dots.typed(typed_list!(@exact;
    x => numeric(),
    y => numeric(),
))?;
```

##### Class checking

```ignore
// Check for specific R class (uses Rf_inherits semantics)
let args = dots.typed(typed_list!(
    data => "data.frame",
    model => "lm",
))?;
```

##### Attribute sugar

Instead of calling `.typed()` manually, you can use `typed_list!` directly in the
`#[miniextendr]` attribute for automatic validation:

```ignore
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn my_func(...) -> String {
    // `dots_typed` is automatically created and validated
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}
```

This injects validation at the start of the function body:
```ignore
let dots_typed = _dots.typed(typed_list!(...)).expect("dots validation failed");
```

See the [`#[miniextendr]`](macro@miniextendr) attribute documentation for more details.

