# Linkme Simplification: Reducing User Crate Boilerplate

## Current State (linkme branch)

User crate needs these pieces to make linkme work:

### Files the user must have/maintain

| File | Boilerplate | Why |
|------|------------|-----|
| `Cargo.toml` | `crate-type = ["staticlib", "lib"]` + `[[bin]]` section | Need staticlib for .so, lib/rlib for document binary |
| `lib.rs` | `include!(concat!(env!("OUT_DIR"), "/miniextendr_link_registrations.rs"));` | Defines the anchor function so entrypoint.c can reference it |
| `document.rs` | 8-line binary calling `link_registrations()` + `write_r_wrappers_to_file()` | Generates R wrappers by reading distributed slices |
| `build.rs` | `miniextendr_lint::build_script();` | Runs lint + generates link_registrations.rs |
| `entrypoint.c.in` | `extern void miniextendr_link_registrations(void);` + call | Forces linker to extract user .o files from archive |
| `Makevars.in` | `PKG_LIBS = -L... -l...` | Links the staticlib |

### The core problem

The linker only extracts `.o` members from a `.a` archive that resolve undefined symbols.
`miniextendr_register_routines` (which reads the slices) is defined in miniextendr-api, not the
user crate. So the user crate's `.o` files — which contain the distributed_slice entries — are
never extracted unless something explicitly references a symbol in them.

On `main` this wasn't a problem: `miniextendr_module!` generated `R_init_{pkg}_miniextendr()`,
a big function defined in the user crate that entrypoint.c called directly. That function
explicitly referenced every registered item, pulling all `.o` files into the link.

### Ideal end state

```toml
# Cargo.toml — just deps, nothing special
[dependencies]
miniextendr-api = "..."
[build-dependencies]
miniextendr-lint = "..."
```

```rust
// lib.rs — just code, no boilerplate
use miniextendr_api::miniextendr;

#[miniextendr]
pub fn add(a: f64, b: f64) -> f64 { a + b }
```

---

## Approaches to Try

### A. force_load / whole-archive (linker flags)

**Idea**: Force the linker to extract ALL `.o` members from the archive, regardless of symbol
references. This is what extendr does.

**For the .so build** (Makevars.in):
```makefile
# macOS
PKG_LIBS = -Wl,-force_load,$(CARGO_AR)
# Linux
PKG_LIBS = -Wl,--whole-archive $(CARGO_AR) -Wl,--no-whole-archive
# Cross-platform (let configure choose)
PKG_LIBS = @FORCE_LOAD_PREFIX@ $(CARGO_AR) @FORCE_LOAD_SUFFIX@
```

**For the document binary** (build.rs):
```rust
// Emit linker args for bin targets only
#[cfg(target_os = "macos")]
println!("cargo::rustc-link-arg-bins=-Wl,-all_load");
#[cfg(target_os = "linux")]
println!("cargo::rustc-link-arg-bins=-Wl,--whole-archive");
```

**Eliminates**: `include!` in lib.rs, `miniextendr_link_registrations` entirely, extern decl
in entrypoint.c.

**Risk**: `-all_load` applies to ALL archives linked into the binary (not just the user crate).
Could cause duplicate symbol errors with system libraries. `-force_load` is targeted to a
single archive but requires the full path. Need to test whether `cargo::rustc-link-arg-bins`
with `-all_load` causes issues in practice.

**Size impact**: Might include unused symbols from miniextendr-api. Measure with and without.

**Test plan**:
1. Restore force_load in Makevars.in (macOS/Linux variants via configure)
2. Remove link_registrations from entrypoint.c.in
3. Remove include! from lib.rs
4. Add `cargo::rustc-link-arg-bins` to build.rs
5. Remove link_registrations call from document.rs
6. Build .so → verify routines register (run R tests)
7. Build document binary → verify R wrappers generated
8. Measure .so and binary size vs current approach

### B. Targeted -force_load with archive path from build.rs

**Idea**: Instead of `-all_load` (all archives), use `-force_load <path>` on just the user
crate's archive. build.rs knows the output directory.

**For the document binary** (build.rs):
```rust
let out_dir = std::env::var("OUT_DIR").unwrap();
// The staticlib is at target/{profile}/lib{name}.a
// but OUT_DIR is deeper: target/{profile}/build/{name}-{hash}/out
// We'd need to reconstruct the path or use CARGO_TARGET_DIR
```

**Problem**: build.rs runs before compilation — the archive doesn't exist yet. And for the
binary target, we're linking the rlib, not the staticlib. The rlib path is also not trivially
available.

**Verdict**: Probably not feasible without cargo providing the archive path.

### C. -u linker flag to force symbol resolution

**Idea**: Use `-Wl,-u,_symbol_name` to create an artificial undefined reference, forcing the
linker to extract the `.o` file containing that symbol.

**For the .so build** (Makevars.in):
```makefile
PKG_LIBS = -L$(CARGO_LIBDIR) -l$(CARGO_STATICLIB_NAME) -Wl,-u,_some_known_symbol
```

**Problem**: We need a known symbol name in the user crate. The `#[miniextendr]` macro
generates C wrapper functions with names like `C_add`, `C_multiply`, etc. — but we don't know
them at Makevars time. build.rs could enumerate them and emit `-u` flags, but that's fragile.

**For the document binary**: build.rs could emit:
```rust
println!("cargo::rustc-link-arg-bins=-Wl,-u,_C_add");
```
But again, build.rs doesn't know function names.

**Variant**: build.rs scans the source files (it already does for lint) and emits `-u` flags
for every `#[miniextendr]` function's C wrapper symbol. This is complex but targeted.

**Verdict**: Possible but fragile. The symbol names must exactly match what the macro generates.

### D. Generate link_registrations with actual references

**Idea**: Instead of an empty function, make `miniextendr_link_registrations()` reference one
symbol from each `.o` file. build.rs already scans sources — it could generate:

```rust
extern "C" { fn C_add(); fn C_multiply(); }
pub extern "C" fn miniextendr_link_registrations() {
    // Black-box references to force extraction
    std::hint::black_box(C_add as *const ());
    std::hint::black_box(C_multiply as *const ());
}
```

**Problem**: These symbols don't exist yet when build.rs runs — they're generated by the
proc macro. Circular dependency.

**Verdict**: Not feasible without a two-pass build.

### E. Emit the anchor from the proc macro itself

**Idea**: Have the FIRST `#[miniextendr]` expansion also emit
`miniextendr_link_registrations()`. Use a `#[distributed_slice]` entry or a global flag to
detect "first".

**Problem**: Proc macros don't have global state — each invocation is independent. Two
`#[miniextendr]` fns in different files would both try to emit the function, causing duplicate
symbol errors.

**Variant**: Use `#[linkme::distributed_slice]` to register a dummy entry, and define the
function in miniextendr-api instead. But the function needs to be in the user crate.

**Verdict**: No good way to do "emit exactly once" from a proc macro.

### F. Make document.rs unnecessary — generate wrappers in build.rs

**Idea**: Move R wrapper generation from the document binary to build.rs. build.rs already
runs at compile time and has access to the source files.

**How it works now**: The proc macros generate R code as `&'static str` constants stored in
`#[distributed_slice(MX_R_WRAPPERS)]`. The document binary reads these at link time.

**Alternative**: build.rs reads source files (it already does for lint), parses `#[miniextendr]`
attributes, and generates R wrappers directly — no proc macro output, no binary, no link-time
slices for wrappers.

**Eliminates**: `[[bin]]` section in Cargo.toml, `document.rs` file, `crate-type` needing `lib`

**Problem**: build.rs would need to replicate the proc macro's R wrapper generation logic.
That's ~2000 lines of code in the macros crate. Duplicating it is a maintenance nightmare.

**Variant**: build.rs calls the macros-core parser (already a dependency of miniextendr-lint)
and generates wrappers from the parse tree. miniextendr-lint already scans sources and builds
a crate index. Could extend it with R wrapper generation.

**Verdict**: Architecturally sound — moves wrapper generation to build time instead of
link time. Eliminates the document binary entirely. But requires significant refactoring of the
R wrapper generation code out of the proc macro and into a shared library.

### G. cdylib instead of staticlib

**Idea**: Build the user crate as a cdylib (.so/.dylib) instead of a staticlib (.a). The
cdylib includes ALL symbols — no archive extraction problem. entrypoint.c links against the
cdylib.

**Problem**: R packages must produce exactly one shared library. We can't have entrypoint.o
linking against a Rust .so — R expects everything in one .so. We'd need to merge the Rust .so
and C .o into one, which is not standard.

**Variant**: Make the Rust crate produce the FINAL .so directly (as cdylib), with the C
entrypoint compiled in via build.rs / cc crate. This eliminates entrypoint.c entirely.

**Eliminates**: entrypoint.c.in, Makevars complexity, the archive extraction problem entirely.

**Problem**: R's build system (R CMD INSTALL) expects to control the final link step. It
compiles .c files and links them with PKG_LIBS. Having Rust produce the .so directly bypasses
this. Feasible with custom Makevars but fragile across platforms.

**Verdict**: Interesting but fights R's build system. Worth a prototype but risky.

### H. #[used] + link_section tricks

**Idea**: Use `#[used]` on distributed_slice entries to tell the compiler they're needed, and
use platform-specific link section attributes to tell the linker to retain the section.

```rust
#[used]
#[link_section = ".init_array"]  // Linux: run at load time
static FORCE_INIT: extern "C" fn() = || { /* touch slices */ };
```

**Problem**: `#[used]` prevents the COMPILER from stripping, but the LINKER still won't
extract unreferenced `.o` files from an archive. The `.o` must be extracted first for `#[used]`
to matter.

**Verdict**: Doesn't solve the archive extraction problem.

### I. Single .o compilation via LTO

**Idea**: With LTO (Link Time Optimization), the compiler merges all `.o` files into one.
A single reference into the archive extracts the merged `.o` with everything.

**How**: `RUSTFLAGS="-C lto=fat"` or `[profile.release] lto = true` in Cargo.toml.

**Status**: This might already work! If LTO merges all compilation units, then ANY reference
into the archive (like `miniextendr_register_routines` being in the same merged unit) would
pull in all distributed_slice entries.

**Test plan**:
1. Enable LTO in the user crate's Cargo.toml
2. Remove link_registrations entirely
3. Test if distributed_slice entries survive

**Risk**: LTO significantly increases compile time. Not suitable for dev builds.

**Verdict**: Worth testing as a data point but not a universal solution (too slow for dev).

### J. Emit build.rs linker args from scanned symbols

**Idea**: build.rs (miniextendr-lint) already scans all source files and knows every
`#[miniextendr]` item. It could emit `cargo::rustc-link-arg` directives with `-u` flags
for the C wrapper symbols it knows will be generated.

```rust
// In build_script():
for item in &crate_index.miniextendr_items {
    let c_symbol = format!("_C_{}", item.name);
    println!("cargo::rustc-link-arg=-Wl,-u,{c_symbol}");
}
```

**For the document binary specifically**:
```rust
for item in &crate_index.miniextendr_items {
    let c_symbol = format!("_C_{}", item.name);
    println!("cargo::rustc-link-arg-bins=-Wl,-u,{c_symbol}");
}
```

**Eliminates**: `include!` in lib.rs, link_registrations function, document.rs call

**Problem**: Symbol name must exactly match what the proc macro generates, including mangling
for methods (`C_TypeName__method_name`). Also, on macOS symbols have a `_` prefix, on Linux
they don't. build.rs must match the platform convention.

**Risk**: Fragile coupling between lint's symbol name prediction and macro's actual symbol
names. One `-u` for a non-existent symbol = link error.

**Mitigation**: Only need ONE valid `-u` reference to trigger the extraction chain (since
lib.rs `mod` declarations create inter-module references). Find the simplest, most predictable
symbol.

**Verdict**: Promising if we can identify ONE reliable symbol to reference. build.rs knows
the crate name, so a fixed symbol like `_C_{crate_name}__miniextendr_anchor` that the macro
always generates could work.

### K. Proc macro emits a well-known anchor symbol

**Idea**: Every `#[miniextendr]` expansion checks a global `#[distributed_slice]` entry. If
it's the first, it also emits a well-known `#[no_mangle]` anchor symbol. If it's not the
first, it doesn't.

**Problem**: Proc macros can't share state. Can't detect "first".

**Variant**: EVERY `#[miniextendr]` emits the anchor (guarded by `#[allow(duplicate)]` or
using a mechanism that tolerates duplicates). E.g.:

```rust
// Each #[miniextendr] expansion adds:
#[linkme::distributed_slice(::miniextendr_api::registry::MX_ANCHOR)]
static _ANCHOR: () = ();
```

Then the `MX_ANCHOR` slice's existence forces the section to be retained. But this doesn't
solve archive extraction — the `.o` files must still be extracted for the entries to appear.

**Verdict**: Doesn't solve the root problem.

---

## Why static/const/#[used] Don't Help

The archive extraction problem is a LINKER issue, not a compiler issue:

```
Compiler: .rs → .o files (distributed_slice entries ARE present in each .o)
    ↓
ar: .o files → .a archive (all .o files are bundled)
    ↓
Linker: .a archive → .so (ONLY extracts .o files that resolve undefined symbols)
```

`static`, `const`, `#[used]`, `#[link_section]` — these all affect step 1 (compiler).
The entries are already in the `.o` files. The problem is step 3: the linker looks at the
archive, sees no undefined symbols referencing `rng_tests.o`, and skips it entirely.

The `.o` file is never read, so its linkme section data is never seen. Making the items
"more static" or "more const" doesn't help — the file itself is never opened.

The fix must operate at the linker level:
- force_load: "open every .o in this archive"
- -u symbol: "pretend you need something from this .o"
- Explicit call: "here's actual code that references a symbol in this .o"

---

## Recommended Test Sequence

### Phase 1: Try force_load for .so (Approach A, .so only)

Restore force_load in Makevars.in. This is the simplest change and eliminates
link_registrations from entrypoint.c.

```bash
# 1. Edit Makevars.in: use force_load
# 2. Remove link_registrations from entrypoint.c.in
# 3. Keep include! and document.rs call for now
# 4. Build + test
just configure && just rcmdinstall && just devtools-test
# 5. Measure .so size
ls -la rpkg/src/miniextendr.so
```

### Phase 2: Try force_load for document binary (Approach A, binary)

Add `cargo::rustc-link-arg-bins` in build.rs for the binary case.

```bash
# 1. Add -all_load (macOS) to build.rs
# 2. Remove link_registrations call from document.rs
# 3. Remove include! from lib.rs
# 4. Remove generate_link_registrations() from miniextendr-lint
# 5. Build document binary + test
just devtools-document
# 6. Diff R wrappers — should be identical
```

### Phase 3: Try LTO (Approach I)

Test if fat LTO merges everything and eliminates the need for any tricks.

```bash
# 1. Add lto = true to [profile.release] in Cargo.toml
# 2. Remove ALL link_registrations code
# 3. Remove force_load from Makevars.in
# 4. Build + test
just rcmdinstall && just devtools-test
# 5. Measure compile time and .so size vs baseline
```

### Phase 4: Try build.rs -u flag (Approach J)

Have build.rs emit one `-u` flag for a known symbol.

```bash
# 1. Have #[miniextendr] on the first fn in lib.rs always generate a
#    predictable symbol like __miniextendr_anchor_{crate_name}
# 2. build.rs emits cargo::rustc-link-arg=-Wl,-u,___miniextendr_anchor_{crate}
# 3. Remove force_load, remove include!, remove link_registrations
# 4. Build + test
```

### Phase 5: Try build-time wrapper generation (Approach F)

Move R wrapper generation from document binary to build.rs/lint.

```bash
# 1. Extract R wrapper generation from proc macros into miniextendr-lint or -engine
# 2. build.rs generates R wrappers directly from source parse
# 3. Remove [[bin]], document.rs, crate-type "lib"
# 4. Test wrapper output matches
```

---

## Size Measurement Plan

For each approach, measure:

```bash
# Static archive size (before linking)
ls -la rpkg/rust-target/release/libminiextendr.a

# Shared library size (final .so)
ls -la $(R RHOME)/library/miniextendr/libs/miniextendr.so

# Document binary size
ls -la rpkg/src/rust/target/debug/document

# Symbol count
nm rpkg/rust-target/release/libminiextendr.a | grep " T " | wc -l

# Section sizes (linkme sections specifically)
size -m rpkg/rust-target/release/libminiextendr.a | head -20
```

---

## Decision Matrix

| Approach | Eliminates include! | Eliminates document.rs | Eliminates entrypoint.c call | Size impact | Complexity | Platform-specific |
|----------|--------------------|-----------------------|-----------------------------:|------------|-----------|-------------------|
| A. force_load | YES | needs build.rs trick | YES | +small | Low | YES (macOS vs Linux flags) |
| B. targeted force_load | YES | YES | YES | neutral | Medium | YES |
| C. -u flags | YES | YES | YES | neutral | Medium | YES (symbol prefix) |
| F. build-time wrappers | N/A | YES (gone) | N/A | neutral | HIGH | No |
| I. LTO | YES | needs testing | YES | -smaller | Low | No |
| J. build.rs -u | YES | YES | YES | neutral | Medium | YES |

The most promising path is **A** (force_load) as the immediate fix, with **F** (build-time
wrappers) as the long-term goal to eliminate the document binary entirely.
