+++
title = "Getting Started"
weight = 1
description = "Create your first R package with a Rust backend"
+++

## Prerequisites

- **Rust** (1.85+): Install from [rustup.rs](https://rustup.rs)
- **R** (4.0+): Install from [CRAN](https://cran.r-project.org)
- **R development tools**: `install.packages("devtools")`

Verify your setup:
```bash
rustc --version   # Should be 1.85+
R --version       # Should be 4.0+
```

## Step 1: Create a New Package

Use the `minirextendr` helper package to scaffold a new project:

```r
# Install minirextendr (once, from GitHub)
remotes::install_github("A2-ai/miniextendr", subdir = "minirextendr")

# Create a new package
library(minirextendr)
create_miniextendr_package("mypackage")
```

This creates:
```text
mypackage/
├── DESCRIPTION
├── NAMESPACE
├── R/
│   └── mypackage-wrappers.R    # Auto-generated R wrappers
├── src/
│   └── rust/
│       ├── Cargo.toml
│       └── lib.rs              # Your Rust code goes here
├── vendor/                     # Vendored miniextendr crates
├── configure                   # Build configuration script
└── configure.ac                # Autoconf source
```

## Step 2: Write Your First Function

Edit `src/rust/lib.rs`:

```rust
use miniextendr_api::miniextendr;

// Required: generates the R_init_mypackage entry point.
miniextendr_api::miniextendr_init!();

/// Add two integers.
/// @param a First number
/// @param b Second number
/// @return The sum of a and b
#[miniextendr]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Greet someone by name.
/// @param name The name to greet
/// @return A greeting string
#[miniextendr]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
// Registration is automatic via #[miniextendr].
```

## Step 3: Build and Test

The simplest approach is `minirextendr::miniextendr_build()`, which runs autoconf, configure, install, and roxygen2 in the correct order:

```r
library(minirextendr)
miniextendr_build("mypackage")
```

Or step by step:

```r
devtools::install("mypackage")   # Compile Rust, generate R wrappers, install
devtools::document("mypackage")  # Update NAMESPACE and man pages
```

Or manually from the shell:

```bash
cd mypackage
autoconf -vif              # Generate configure from configure.ac
bash ./configure           # Generate Makevars and build files
R CMD INSTALL .            # Compile Rust and install
```

## Step 4: Use from R

```r
library(mypackage)

add(1L, 2L)
# [1] 3

greet("World")
# [1] "Hello, World!"
```

## The `#[miniextendr]` Attribute

Mark functions for export to R:

```rust
#[miniextendr]
pub fn my_function(x: i32) -> i32 {
    x * 2
}
```

The macro:
- Generates a C wrapper callable from R
- Handles type conversion (R <-> Rust)
- Manages error handling and panics
- Extracts documentation from Rust doc comments

Items annotated with `#[miniextendr]` are automatically registered via linkme distributed slices. No manual module declarations are needed.

## Type Conversions

miniextendr automatically converts between R and Rust types:

| R Type | Rust Type |
|--------|-----------|
| integer | `i32` |
| numeric | `f64` |
| character | `String`, `&str` |
| logical | `bool` |
| integer vector | `Vec<i32>`, `&[i32]` |
| numeric vector | `Vec<f64>`, `&[f64]` |
| list | Various |
| NULL | `()` |
| NA | `Option<T>` (None = NA) |

## Creating Classes

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]  // Default: env style
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }
}
```

```r
c <- Counter$new(0L)
c$value()      # 0
c$increment()
c$value()      # 1
```

Switch class system with a single attribute change:

```rust
#[miniextendr(r6)]   // R6 class
#[miniextendr(s3)]   // S3 generic functions
#[miniextendr(s4)]   // S4 setClass/setMethod
#[miniextendr(s7)]   // S7 new_class
```

## Error Handling

Rust panics are converted to R errors:

```rust
#[miniextendr]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("Division by zero");
    }
    a / b
}
```

```r
divide(1, 0)
# Error: Division by zero
```

Return `Result<T, E>` for structured error handling:

```rust
#[miniextendr]
pub fn parse_int(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("Parse error: {}", e))
}
```

## Development Workflow

1. Edit Rust code in `src/rust/lib.rs`
2. Run `devtools::install("mypackage")` (compiles Rust, generates R wrappers, installs)
3. Run `devtools::document("mypackage")` (updates NAMESPACE and man pages)
4. Test in R

Or use `minirextendr::miniextendr_build("mypackage")` to run all steps in one call.

When working in the miniextendr monorepo itself, the [`justfile`](https://github.com/A2-ai/miniextendr/blob/main/justfile) provides recipes like `just configure`, `just rcmdinstall`, and `just devtools-document` that automate the full build cycle.

## Next Steps

- [Architecture](/architecture/) - How miniextendr works
- [Type Conversions](/type-conversions/) - Full type mapping reference
- [Class Systems](/class-systems/) - Choosing and using R class systems
- [ALTREP](/altrep/) - Lazy and compact vectors

## Full reference

This page is a curated entry point. See the [user manual](/manual/getting-started/) for the exhaustive treatment, edge cases, and every feature switch.
