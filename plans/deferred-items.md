# Deferred Items for Concrete Planning

These are the remaining unchecked items from TODO.typ. Each section provides
enough context about the codebase, existing patterns, and known challenges for
Codex to produce a concrete implementation plan.

---

## 1. Windows CI Debugging

**Priority:** Medium (CI exists but is flaky)
**Blocked on:** Access to Windows CI logs / runner

### Current State

Windows CI already exists at `.github/workflows/ci.yml` (lines 462–596):
- Runner: `windows-latest` with Rtools 44
- Rust target: `x86_64-pc-windows-gnu` (GNU, not MSVC)
- Marked `continue-on-error: true` — failures don't block PRs
- Uses PowerShell for Cargo config, Rtools bash for `./configure`

### Known Issues (from TODO)

1. **`-l` (login) flag in bash** — Rtools bash with `--login` resets PATH and
   may change the working directory. The CI currently uses
   `C:\rtools44\usr\bin\bash.exe -e -o pipefail {0}` (no `-l`), but some steps
   may implicitly get login behavior.

2. **Path format** — MSYS2 paths (`/d/a/miniextendr/...`) vs Windows paths
   (`D:\a\miniextendr\...`). `configure.ac` handles this with `cygpath -m`
   (lines 137–147), but Cargo and R may interpret paths differently.

### Key Files

| File | Role |
|------|------|
| `.github/workflows/ci.yml:462-596` | Windows CI job definition |
| `rpkg/configure.ac:122-147` | CYGPATH detection and path conversion |
| `rpkg/src/Makevars.in` | Build config template (uses `@CARGO_BUILD_TARGET@`) |
| `rpkg/src/win.def.in` | Windows export symbols |
| `minirextendr/inst/templates/rpkg/configure.win` | Windows configure for scaffolded projects |
| `minirextendr/inst/templates/rpkg/configure.ucrt` | UCRT Windows variant |

### CI Architecture

```
PowerShell step:
  - Adds Rtools to PATH
  - Creates libgcc mock stubs (empty .a files for Rust linking)
  - Writes .cargo/config.toml with GNU linker + LIBRARY_PATH

Bash step (Rtools bash):
  - Exports PATH with rtools + cargo
  - Runs: cd $(cygpath -u "$GITHUB_WORKSPACE")/rpkg && NOT_CRAN=true ./configure

R CMD check step:
  - Uses r-lib/actions/check-r-package@v2
```

### What a Plan Should Cover

- Reproduce the exact CI failure (or get recent failure logs from GitHub Actions)
- Audit the PATH chain: PowerShell → Rtools bash → configure → cargo → rustc
- Test whether `cygpath -m` conversion is applied consistently to all paths
  passed to Cargo (CARGO_TARGET_DIR, patch paths, etc.)
- Check if libgcc mock stubs are sufficient or if real stubs are needed
- Consider whether MSVC toolchain would be simpler than GNU

---

## 2. rkyv Zero-Copy Serialization

**Priority:** Low (explicitly DEFERRED)
**Depends on:** Understanding R's GC model constraints

### Why It Was Deferred

rkyv requires that archived data lives in a stable memory region for the
lifetime of zero-copy access. This conflicts with R's garbage collector, which
can relocate or free RAWSXP data at any time unless properly protected.

### Borsh Pattern (Reference Implementation)

The `borsh` feature provides the template to follow:

**File:** `miniextendr-api/src/optionals/borsh_impl.rs`

```rust
// Wrapper type
pub struct Borsh<T>(pub T);

// IntoR: serialize T → RAWSXP
impl<T: borsh::BorshSerialize> IntoR for Borsh<T> { ... }

// TryFromSexp: deserialize RAWSXP → T
impl<T: borsh::BorshDeserialize> TryFromSexp for Borsh<T> { ... }

// Standalone helpers
pub unsafe fn borsh_to_raw<T: borsh::BorshSerialize>(value: &T) -> SEXP { ... }
pub unsafe fn borsh_from_raw<T: borsh::BorshDeserialize>(sexp: SEXP) -> Result<T, SexpError> { ... }
```

**Cargo.toml pattern:**
```toml
[features]
borsh = ["dep:borsh"]

[dependencies]
borsh = { version = "1", optional = true, features = ["derive"] }
```

### rkyv-Specific Challenges

1. **Lifetime management** — rkyv's `ArchivedFoo` borrows from the byte buffer.
   If that buffer is a RAWSXP, the archived view is only valid while the SEXP
   is protected. Need an RAII guard that holds protection.

2. **Validation** — rkyv v0.8 requires validation (`rkyv::check_archived_root`)
   before accessing archived data. This adds overhead but is necessary for safety
   with untrusted R data.

3. **Alignment** — R's RAWSXP data may not be aligned to rkyv's requirements.
   May need a copy-to-aligned-buffer step, partially defeating zero-copy.

4. **API design** — Should it be:
   - `Rkyv<T>` wrapper (like Borsh) — simpler, copies on conversion
   - `ArchivedView<T>` with lifetime tied to SEXP protection — true zero-copy
     but more complex API

### What a Plan Should Cover

- Decide between wrapper (copy) vs view (zero-copy) API
- Design SEXP protection RAII guard for archived views
- Handle alignment requirements (check if RAWSXP data is sufficiently aligned)
- Validation strategy (always validate? feature-gated unsafe skip?)
- Test plan for GC safety (stress test with `gc()` calls between operations)

---

## 3. Crossbeam Channel Adapters

**Priority:** Low (POSTPONED)
**Depends on:** Understanding the worker thread model

### Current Thread Architecture

**File:** `miniextendr-api/src/worker.rs`

```
Main Thread (R)              Worker Thread (Rust)
─────────────────           ─────────────────────
.Call() entry       ──→     run_on_worker(closure)
  │                            │
  │  ←── work request ────    with_r_thread(|| { R API call })
  │                            │  (blocks waiting for response)
  │  ──── response ────→      │
  │                            │
  │  ←── final result ────    return value
  │
Rf_error() or return SEXP
```

Key characteristics:
- One worker thread per `.Call()` invocation
- Bidirectional communication via `mpsc::channel`
- Worker can call R APIs via `with_r_thread()` (marshals to main thread)
- Panics caught by `catch_unwind`, converted to R errors

### What RSender/RReceiver Would Provide

User-facing channel adapters for long-running Rust computations that need to
communicate intermediate results to R:

```rust
// Hypothetical API
#[miniextendr]
pub fn start_computation() -> RReceiver<ProgressUpdate> {
    let (tx, rx) = crossbeam::channel::unbounded();
    std::thread::spawn(move || {
        for i in 0..100 {
            tx.send(ProgressUpdate { pct: i }).unwrap();
        }
    });
    RReceiver(rx)
}

#[miniextendr]
pub fn poll_progress(rx: &RReceiver<ProgressUpdate>) -> Option<ProgressUpdate> {
    rx.0.try_recv().ok()
}
```

### Design Considerations

1. **ExternalPtr storage** — `RReceiver<T>` would be an ExternalPtr wrapping
   `crossbeam::Receiver<T>`. Needs `T: Send + 'static`.

2. **Blocking vs non-blocking** — R is single-threaded. `recv()` would block R.
   Must use `try_recv()` or `recv_timeout()` from R's side.

3. **GC cleanup** — When RReceiver is garbage collected, the channel closes.
   Sender threads must handle `SendError` gracefully.

4. **Integration with worker thread** — Should channels work alongside or
   replace the existing `run_on_worker` pattern?

### What a Plan Should Cover

- API design for RSender/RReceiver wrapper types
- Feature-gated optional dependency on `crossbeam-channel`
- IntoR/TryFromSexp implementations (ExternalPtr-based)
- R-side polling pattern (non-blocking recv)
- GC finalizer for channel cleanup
- Example: progress reporting from long computation

---

## 4. Future/Async Adapters

**Priority:** Low (POSTPONED)
**Depends on:** Crossbeam channels (item 3), async runtime choice

### Current State

No async/await or Tokio integration exists. The worker thread model is
synchronous with request-response semantics.

### Why It's Hard

R's `.Call()` interface is inherently synchronous — it blocks until the function
returns a SEXP. True async would require one of:

1. **Background task + polling** — Start async work, return a handle, poll from R.
   Similar to crossbeam channels but with Future semantics.

2. **R event loop integration** — Hook into R's event loop (`R_PolledEvents`)
   to drive an async executor. Complex and fragile.

3. **mirai-style delegation** — Offload to a separate process entirely.
   Clean but heavy.

### Possible API

```rust
// Option A: RFuture wrapper (polling-based)
#[miniextendr]
pub fn fetch_async(url: String) -> RFuture<String> {
    RFuture::spawn(async move {
        reqwest::get(&url).await?.text().await?
    })
}

// From R:
// future <- fetch_async("https://example.com")
// while (!future$is_ready()) { Sys.sleep(0.1) }
// result <- future$value()
```

### Design Considerations

- Which async runtime? tokio (heavy, full-featured) vs smol (light) vs
  async-std? Should be user's choice via feature flags.
- How to drive the executor — dedicated thread? R event loop hook?
- Cancellation — what happens when RFuture is GC'd while task is running?
- Error propagation — async errors → R errors

### What a Plan Should Cover

- Choose between polling-based (simpler) vs event-loop-integrated (richer)
- Runtime selection strategy (feature flags per runtime?)
- RFuture<T> type design with ExternalPtr storage
- Executor lifecycle management (start/stop with package load/unload)
- Cancellation and cleanup semantics
- Integration with existing worker thread model (coexistence)

---

## 5. miniextendr.yml Config File

**Priority:** Low (nice-to-have)
**Scope:** `minirextendr` R package only

### Current Configuration

The `minirextendr` scaffolding package uses function arguments and mustache
templates for configuration:

```r
# minirextendr/R/create.R
miniextendr_new(
  path,
  package_name = "mypkg",
  author_given = "First",
  author_family = "Last",
  ...
)
```

Templates in `minirextendr/inst/templates/` use `{{package_rs}}`,
`{{package_human}}` etc. as placeholders.

### What miniextendr.yml Would Provide

A per-project config file for user defaults:

```yaml
# miniextendr.yml (project root)
features:
  - borsh
  - serde-json
  - vctrs
strict: true
class_system: r6
rust_edition: "2024"
```

This would let `minirextendr::miniextendr_doctor()` and other tooling read
project-level settings without requiring function arguments.

### Design Considerations

- **yaml package** — `yaml::read_yaml()` is the standard R YAML parser.
   Add as Suggests dependency to minirextendr.
- **Schema** — Define allowed keys and validate on read.
- **Discovery** — Walk up from working directory to find `miniextendr.yml`.
- **Integration points:**
  - `miniextendr_doctor()` reads config for health checks
  - `miniextendr_new()` generates initial config
  - `use_feature()` updates config when adding features
  - Build system could read it to set Cargo features

### What a Plan Should Cover

- YAML schema definition (all supported keys with types and defaults)
- Discovery logic (walk up directories, stop at package root)
- Integration with existing `miniextendr_doctor()` checks
- Config generation during `miniextendr_new()`
- Template for initial `miniextendr.yml` content
- Validation and error messages for invalid config

---

## 6. lifecycle Package Integration

**Priority:** Low (nice-to-have)
**Scope:** Generated R wrapper code

### Current Export Control

The `#[miniextendr(internal)]` and `#[miniextendr(noexport)]` attributes
control roxygen tags:

| Attribute | @export | @keywords internal |
|-----------|---------|-------------------|
| (default, pub) | Yes | No |
| `internal` | No | Yes |
| `noexport` | No | No |

Implementation: `ClassDocBuilder::with_export_control(internal, noexport)` in
`miniextendr-engine` handles all 6 class systems consistently.

### What lifecycle Would Add

The `lifecycle` R package provides standardized deprecation/evolution signals:

```r
# lifecycle badges in documentation
#' @description
#' `r lifecycle::badge("deprecated")`
#'
#' Use [new_function()] instead.

# Runtime deprecation warnings
my_old_function <- function(x) {
  lifecycle::deprecate_warn("1.0.0", "my_old_function()", "new_function()")
  new_function(x)
}
```

### Proposed Attribute Design

```rust
#[miniextendr(lifecycle = "deprecated")]
pub fn old_function(x: i32) -> i32 { x }

#[miniextendr(lifecycle = "experimental")]
pub fn new_function(x: i32) -> i32 { x }

// Possible stages: experimental, stable, deprecated, superseded
```

### Implementation Path

1. **Parse attribute** — Add `lifecycle` field to `MiniextendrFnAttrs`
2. **Generate roxygen** — Inject lifecycle badge into `@description`
3. **Generate runtime warning** — For `deprecated`, wrap body with
   `lifecycle::deprecate_warn()` call
4. **R dependency** — `lifecycle` goes in DESCRIPTION Imports (or Suggests
   if feature-gated)

### Key Files to Modify

| File | Change |
|------|--------|
| `miniextendr-macros-core/src/miniextendr_fn_attrs.rs` | Parse `lifecycle` attribute |
| `miniextendr-macros/src/lib.rs` | Destructure new field |
| `miniextendr-engine/src/roxygen.rs` | Inject lifecycle badge |
| `miniextendr-engine/src/r_wrapper_builder.rs` | Wrap body with deprecation warning |
| All 6 class generators | Pass lifecycle through `ClassDocBuilder` |

### What a Plan Should Cover

- Which lifecycle stages to support (experimental, stable, deprecated, superseded)
- Roxygen badge injection approach
- Runtime warning generation (deprecate_warn with version string)
- Whether lifecycle is always required or feature-gated
- How to specify replacement function (`#[miniextendr(lifecycle = "deprecated", replaced_by = "new_fn")]`)

---

## Architecture Reference

### Optional Feature Pattern

All optional integrations follow this pattern:

```
miniextendr-api/
  Cargo.toml:        feature = ["dep:crate-name"]
  src/optionals/
    mod.rs:          #[cfg(feature = "crate-name")] mod crate_impl;
    crate_impl.rs:   IntoR/TryFromSexp + adapter traits
```

### Adapter Trait Pattern

```rust
// Trait with methods exposed to R
pub trait RFoo: Clone {
    fn method(&self) -> ReturnType;
}

// Blanket impl from upstream trait
impl<T: UpstreamTrait + Clone> RFoo for T {
    fn method(&self) -> ReturnType { ... }
}
```

### Proc-Macro Attribute Flow

```
#[miniextendr(attr)] on fn/impl
  → MiniextendrFnAttrs / ImplAttrs parse
  → lib.rs destructures all fields
  → RWrapperBuilder / ClassDocBuilder use fields
  → Generated R code as const string
```

When adding new attributes, **always update the destructuring in lib.rs** and
**all 6 class system generators** if the attribute applies to impl blocks.
