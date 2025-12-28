# FFI and Rayon Integration Review

This document reviews all the improvements made to miniextendr's FFI layer and the new Rayon integration.

## Summary Statistics

| Component | Lines | Description |
|-----------|-------|-------------|
| `ffi.rs` | 1,702 | Main FFI module (was ~726) |
| `ffi/altrep.rs` | 707 | ALTREP bindings (was ~268) |
| `rayon_bridge.rs` | 1,185 | NEW - Rayon integration |
| `RAYON.md` | 778 | NEW - Rayon user guide |
| **Total** | **4,372** | **~3,000 lines added** |

**Tests:** ✅ All passing (3 new Rayon tests)
**Compilation:** ✅ Clean (zero errors, expected warnings only)
**Documentation:** ✅ 1,200+ lines of docs and examples
**API Coverage:** ✅ ~95% of commonly-used R FFI

---

## Part 1: FFI Quality Improvements

### 1.1 Parameter Names Match R Source (150+ functions)

**Before:**

```rust
pub fn Rf_cons(arg1: SEXP, arg2: SEXP) -> SEXP;
pub fn INTEGER_GET_REGION(arg1: SEXP, arg2: R_xlen_t, arg3: R_xlen_t, arg4: *mut i32);
```

**After:**

```rust
pub fn Rf_cons(car: SEXP, cdr: SEXP) -> SEXP;
pub fn INTEGER_GET_REGION(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32);
```

**Impact:** Self-documenting code, matches R source exactly (verified against R 4.5.2)

### 1.2 Doc Aliases for Searchability (70+ added)

**Before:** Users searching for "PROTECT" found nothing
**After:** `#[doc(alias = "PROTECT")]` on `Rf_protect` - fully searchable

**Examples:**

```rust
#[doc(alias = "PROTECT")]
#[doc(alias = "protect")]
pub fn Rf_protect(s: SEXP) -> SEXP;

#[doc(alias = "mkChar")]
pub fn Rf_mkChar(s: *const c_char) -> SEXP;

#[doc(alias = "ScalarInteger")]
pub fn Rf_ScalarInteger(x: i32) -> SEXP;
```

### 1.3 R Naming Conventions Documented

#### "gets" Suffix (Common Lisp Heritage)

Added comprehensive documentation explaining R's setter naming:

```rust
// R uses "gets" suffix for setters, borrowed from Common Lisp
// namesgets:    Sets names     (R: names(x) <- val)
// dimgets:      Sets dimensions (R: dim(x) <- val)
// classgets:    Sets class      (R: class(x) <- val)
```

#### CAR/CDR (Lisp Etymology)

Added 200+ lines explaining pairlists:

- What they are (cons cells)
- When they're used (function arguments, language objects)
- Etymology from Lisp assembly mnemonics
- Complete examples with structure diagrams

### 1.4 Public `SexpExt` Trait

Idiomatic Rust methods for SEXP operations:

**Before:**

```rust
pub(crate) trait SexpExt { ... }  // Private, limited methods
```

**After:**

```rust
pub trait SexpExt {
    // Type checking
    fn is_integer(&self) -> bool;
    fn is_real(&self) -> bool;
    fn is_character(&self) -> bool;
    fn is_altrep(&self) -> bool;
    // ... 10+ methods
}
```

**Usage:**

```rust
use miniextendr_api::ffi::SexpExt;

if x.is_integer() && x.len() > 0 {  // Clean method syntax!
    let data = x.as_slice::<i32>();
}
```

### 1.5 ALTREP Improvements

#### Builder Pattern

**Before:** Manual method registration

```rust
unsafe {
    R_set_altrep_Length_method(cls, my_length);
    R_set_altinteger_Elt_method(cls, my_elt);
    // ... many more
}
```

**After:** Fluent builder API

```rust
let class = AltrepClassBuilder::new_integer(cname, pname, info)
    .length(my_length)
    .elt_integer(my_elt)
    .get_region_integer(my_get_region)
    .is_sorted_integer(my_is_sorted)
    .build();
```

#### Helper Functions

- `r_sexp(class)` - Extract SEXP from `R_altrep_class_t` (equivalent to `R_SEXP` macro)
- `r_subtype_init(ptr)` - Create `R_altrep_class_t` (equivalent to `R_SUBTYPE_INIT` macro)

### 1.6 Comprehensive Function Coverage (90+ functions added)

#### Memory Management

- **Protection:** `Rf_protect`, `Rf_unprotect`, `R_PreserveObject`, `R_ReleaseObject`
- **Allocation:** `Rf_allocVector`, `Rf_allocMatrix`, `Rf_allocArray`, `Rf_alloc3DArray`
- **Specialized:** `Rf_allocList`, `Rf_allocLang`, `Rf_allocS4Object`, `Rf_allocSExp`

#### Vector Construction

- **Scalars:** `Rf_ScalarInteger`, `Rf_ScalarReal`, `Rf_ScalarLogical`, `Rf_ScalarString`, `Rf_ScalarComplex`, `Rf_ScalarRaw`
- **Strings:** `Rf_mkChar`, `Rf_mkCharLen`, `Rf_mkCharLenCE`, `Rf_mkString`

#### Data Access

- **Direct pointers:** `INTEGER`, `REAL`, `LOGICAL`, `COMPLEX`, `RAW`
- **OR_NULL variants:** `INTEGER_OR_NULL`, `REAL_OR_NULL`, etc.
- **Element access:** `INTEGER_ELT`, `REAL_ELT`, `STRING_ELT`, `VECTOR_ELT` (ALTREP-aware)
- **Setters:** `SET_INTEGER_ELT`, `SET_REAL_ELT`, etc.
- **Generic:** `DATAPTR_RO`, `DATAPTR_OR_NULL`, `DATAPTR` (nonapi)

#### Pairlists (Cons Cells)

- **Constructors:** `Rf_cons`, `Rf_lcons`, `Rf_list1-4`, `Rf_lang1-6`
- **Accessors:** `CAR`, `CDR`, `TAG`, `CADR`, `CADDR`, `CADDDR`, `CAD4R`
- **Nested:** `CAAR`, `CDAR`
- **Setters:** `SETCAR`, `SETCDR`, `SET_TAG`, `SETCADR`, etc.
- **Utilities:** `Rf_elt`, `Rf_lastElt`, `Rf_nthcdr`, `Rf_listAppend`

#### Attributes

- **Get:** `Rf_getAttrib`, `Rf_GetRowNames`, `Rf_GetColNames`, `ATTRIB`
- **Set:** `Rf_setAttrib`, `Rf_namesgets`, `Rf_dimgets`, `Rf_classgets`, `Rf_dimnamesgets`, `SET_ATTRIB`

#### Type Checking

- **Basic:** `TYPEOF`, `Rf_isNull`, `Rf_isSymbol`, `Rf_isLogical`, `Rf_isReal`, `Rf_isComplex`, `Rf_isExpression`, `Rf_isEnvironment`, `Rf_isString`
- **Composite:** `Rf_isArray`, `Rf_isMatrix`, `Rf_isList`, `Rf_isDataFrame`, `Rf_isFactor`, `Rf_isFunction`, `Rf_isPrimitive`, `Rf_isPairList`
- **Inline:** `Rf_isNumeric`, `Rf_isNumber`, `Rf_isVector`, `Rf_isVectorAtomic`, `Rf_isVectorList`
- **Trait:** All available as methods via `SexpExt`

#### Metadata

- **Length:** `LENGTH`, `XLENGTH`, `TRUELENGTH`
- **Properties:** `OBJECT`, `SET_OBJECT`, `LEVELS`, `SETLEVELS`

#### Coercion & Duplication

- `Rf_asLogical`, `Rf_asInteger`, `Rf_asReal`, `Rf_asChar`, `Rf_coerceVector`
- `Rf_duplicate`, `Rf_shallow_duplicate`

#### Matrix Utilities

- `Rf_nrows`, `Rf_ncols`

#### Environments

- **Lookup:** `Rf_findVar`, `Rf_findVarInFrame`, `Rf_findVarInFrame3`, `Rf_findFun`
- **Assignment:** `Rf_defineVar`, `Rf_setVar`

#### Evaluation

- `Rf_eval`, `Rf_applyClosure`, `R_tryEval`, `R_tryEvalSilent`, `R_forceAndCall`

#### Symbols

- `Rf_install`, `PRINTNAME`, `R_CHAR`

#### Inheritance

- `Rf_inherits`, `Rf_isObject`

#### Connections (UNSTABLE API)

- `R_new_custom_connection`, `R_ReadConnection`, `R_WriteConnection`, `R_GetConnection`
- All marked with explicit warnings about API instability

#### Global Constants

- `R_NilValue`, `R_NaString`
- `R_GlobalEnv`, `R_BaseEnv`, `R_EmptyEnv`
- `R_NamesSymbol`, `R_DimSymbol`, `R_ClassSymbol`, `R_RowNamesSymbol`, `R_DimNamesSymbol`
- `R_TrueValue`, `R_FalseValue`, `R_LogicalNAValue`

### 1.7 Code Organization

Replaced `// ===` separators with collapsible `// region:` markers:

```rust
// region: Cons Cell (Pairlist) Accessors
// region: Element-wise accessors (ALTREP-aware)
// region: SEXP metadata accessors
// region: Vector data accessors (mutable pointers)
// region: ALTREP support
// region: Inline Helper Functions
```

---

## Part 2: Rayon Integration

### 2.1 Architecture (current)

- Worker-thread entry points (`#[miniextendr]`) run with main-thread routing.
- Parallel computation happens on Rayon threads with normal Rust stacks.
- R API calls happen on the main/worker thread via `with_r_thread`, never inside
  parallel iterators.

### 2.2 Core APIs

- `with_r_vec<T>`: pre-allocate and protect an R vector, expose a mutable slice
  for parallel writes (`T: RNativeType + Send + Sync`).
- `RVec<T>`: `FromParallelIterator` container that converts via `IntoR` on the
  main thread.
- `reduce::*`: `sum`, `sum_int`, `min`, `max`, `mean` helpers that return R
  scalars.
- `perf::*`: thread pool info (`num_threads`, `in_rayon_thread`, `thread_index`).
- `rayon_bridge::rayon`: re-export to avoid version mismatches.

### 2.3 Safety constraints

- Do not call R APIs inside Rayon closures (including `IntoR::into_sexp`).
- Only write to disjoint indices when using `with_r_vec<T>`.
- R API calls from Rayon threads without a worker context will panic.

### 2.4 Tests and docs

- Integration tests: `miniextendr-api/tests/rayon.rs`.
- Module docs: `miniextendr-api/src/rayon_bridge.rs`.
- User guide: `RAYON.md`.

### 2.5 Notes

- Older docs referenced `run_r`, `collect_r`, `RVecBuilder`, and
  `ParallelIteratorExt`. These were removed in favor of the explicit APIs above.
