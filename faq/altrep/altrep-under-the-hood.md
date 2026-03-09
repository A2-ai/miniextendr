# Which R Operations Produce ALTREP Objects?

A comprehensive list of every R operation that silently returns an ALTREP object instead
of a regular SEXP. Most users never know these exist — the deferred/compact representation
is invisible unless you call `.Internal(inspect(x))`.

Source: R source at `background/r-svn/`.

---

## Quick Reference

| R Operation | ALTREP Class | Type | What It Does |
|---|---|---|---|
| `1:n` | `compact_intseq` | INTSXP | Stores (first, incr, length) — no array |
| `seq_len(n)` | `compact_intseq` | INTSXP | Same as `1:n` |
| `seq_along(x)` | `compact_intseq` | INTSXP | Same as `1:length(x)` |
| `seq.int(from, to)` | `compact_intseq` / `compact_realseq` | INTSXP/REALSXP | Compact if integer-representable |
| `as.character(int_vec)` | `deferred_string` | STRSXP | Lazily converts each element via `mkChar` |
| `as.character(real_vec)` | `deferred_string` | STRSXP | Same |
| `sort()` result | `wrap_*` | any vector | Wraps with sorted/no-NA metadata |
| `names<-`, `dimnames<-`, `unclass()`, etc. | `wrap_*` | any vector | Copy-on-modify deferred via wrapper |
| `x[i] <- v` (complex assign) | `wrap_*` | any vector | `EnsureLocal` wraps shared vectors |
| `row.names(df)` (compact) | `compact_intseq` | INTSXP | Data frames store `c(NA, -n)` internally |
| `.Internal(mmap_file(...))` | `mmap_integer` / `mmap_real` | INTSXP/REALSXP | Memory-mapped file (Unix only) |

---

## 1. Compact Integer Sequences (`compact_intseq`)

**Class**: `compact_intseq` (altclasses.c:289-333)

Stores three values: `(first, increment, length)`. No integer array is allocated until
something forces materialization.

### The `:` Operator

The most common source of ALTREP in R. Every `a:b` expression produces a compact sequence.

```c
// eval.c:4747-4751 — the bytecode interpreter's : handler
static SEXP seq_int(int n1, int n2)
{
    return R_compact_intrange(n1, n2);
}
```

The `R_compact_intrange` function (altclasses.c:580-591) creates either a `compact_intseq`
(if both endpoints fit in `int`) or a `compact_realseq` (if they overflow to `R_xlen_t`).

```r
# All of these produce compact_intseq ALTREP:
1:100
5:1        # descending — incr = -1
1:1000000  # million elements, zero allocation
```

### `seq_len(n)`

```c
// seq.c:1095-1122 — do_seq_len
if (len == 0)
    return allocVector(INTSXP, 0);
else
    return R_compact_intrange(1, len);
```

```r
seq_len(1000000)  # compact_intseq, no array allocated
```

### `seq_along(x)`

```c
// seq.c:1060-1093 — do_seq_along
len = xlength(CAR(args));
if (len == 0)
    return allocVector(INTSXP, 0);
else
    return R_compact_intrange(1, len);
```

```r
seq_along(letters)  # compact_intseq 1:26
```

### `seq.int(from, to)` and `seq.int(from, to, by)`

The `seq_colon` function in seq.c:98-135 handles the two-argument case:

```c
// seq.c:104-105
if (n1 == (R_xlen_t) n1 && n2 == (R_xlen_t) n2)
    return R_compact_intrange((R_xlen_t) n1, (R_xlen_t) n2);
```

If both endpoints are exact integers, returns `compact_intseq`. Otherwise falls through
to allocate a regular `REALSXP` with a loop. The three-argument `seq.int(from, to, by)`
also uses `R_compact_intrange` when applicable (seq.c:121-125).

```r
seq.int(1, 100)           # compact_intseq
seq.int(1.5, 100.5)       # regular REALSXP (non-integer endpoints)
seq.int(1, 100, by = 2)   # compact_intseq if integer-representable
```

### Data Frame Row Names (Implicit)

When a data frame stores compact row names as `c(NA, -n)` (the common case for data
frames without custom row names), `getAttrib(df, R_RowNamesSymbol)` expands them into a
compact sequence:

```c
// attrib.c:188-190
if(isInteger(s) && LENGTH(s) == 2 && INTEGER(s)[0] == NA_INTEGER) {
    int n = abs(INTEGER(s)[1]);
    if (n > 0)
        s = R_compact_intrange(1, n);
```

```r
df <- data.frame(x = 1:1000000)
row.names(df)  # compact_intseq 1:1000000
# The data frame stores c(NA, -1000000) as two ints;
# row.names() expands it to a compact ALTREP sequence on the fly
```

---

## 2. Compact Real Sequences (`compact_realseq`)

**Class**: `compact_realseq` (altclasses.c:527-569)

Same idea as `compact_intseq` but for doubles. Created when the `:` operator's endpoints
overflow `int` range:

```c
// altclasses.c:587-588
if (n1 <= INT_MIN || n1 > INT_MAX || n2 <= INT_MIN || n2 > INT_MAX)
    return new_compact_realseq(n, n1, n1 <= n2 ? 1 : -1);
```

Also created when coercing a `compact_intseq` to `REALSXP`:

```c
// altclasses.c:109-114 — compact_intseq_Coerce
if (type == REALSXP) {
    return new_compact_realseq(n, n1, inc);
}
```

```r
1e15:1e15+100     # compact_realseq (endpoints exceed INT_MAX)
as.double(1:100)  # compact_realseq (coercion from compact_intseq)
```

---

## 3. Deferred String Coercion (`deferred_string`)

**Class**: `deferred_string` (altclasses.c:845-897)

The **only** place this is created from user-facing R code is `as.character()` on integer
or real vectors **without attributes**:

```c
// coerce.c:1294-1302
case STRSXP:
    if (ATTRIB(v) == R_NilValue)
        switch(TYPEOF(v)) {
        case INTSXP:
        case REALSXP:
            ans = R_deferred_coerceToString(v, NULL);
            return ans;
        }
    ans = coerceToString(v);  // fallback: immediate materialization
```

The condition is strict: `ATTRIB(v) == R_NilValue`. If the integer/real vector has **any**
attributes (names, class, dim, anything), it falls through to immediate `coerceToString`.

```r
# ALTREP deferred_string:
as.character(1:1000000)       # deferred — each element converted lazily via mkChar
as.character(c(1.5, 2.7))    # deferred
as.character(1:10)            # deferred (even for short vectors)

# NOT ALTREP (regular STRSXP):
as.character(c(a=1, b=2))    # has names attribute → immediate
as.character(factor(1:10))   # has class attribute → immediate
paste0("x", 1:10)            # paste always materializes
c("hello", "world")          # literal construction
readLines("file.txt")        # I/O always materializes
format(1:10)                 # format materializes
sprintf("%d", 1:10)          # sprintf materializes
```

### Why Deferred Strings Are Dangerous for Concurrency

Each element is converted on first access via `deferred_string_Elt` (altclasses.c:776-788),
which calls `StringFromInteger` / `StringFromReal` → `mkChar`. The `mkChar` function:

1. Searches `R_StringHash` (global hash table) — concurrent reads/inserts corrupt the chain
2. May allocate a new `CHARSXP` via `allocVector` — touches `R_GenHeap[c].Free`
3. ALTREP dispatch toggles `R_GCEnabled` (single global int)

See [ALTREP string race demo](altrep-string-race-demo.md) for concrete race examples.

### Subsetting Preserves Deferred Status

When you subset a deferred string, R creates a **new** deferred conversion on the subset
of the underlying integer/real vector (altclasses.c:820-834):

```c
// altclasses.c:830-832
PROTECT(result = ExtractSubset(data, indx, call));
result = R_deferred_coerceToString(result, info);
```

```r
x <- as.character(1:1000000)  # deferred_string
y <- x[1:100]                 # ALSO deferred_string (over subset of 1:1000000)
```

---

## 4. Wrapper Classes (`wrap_*`)

**Classes**: `wrap_integer`, `wrap_logical`, `wrap_real`, `wrap_complex`, `wrap_raw`,
`wrap_string`, `wrap_list` (altclasses.c:1744-1891)

Wrappers are ALTREP objects that hold a reference to another vector plus metadata
(sorted status, no-NA flag). They enable **copy-on-modify deferral** — when R needs to
"copy" a vector to attach new attributes, it creates a wrapper instead of duplicating
the data.

### `sort()` Return Value

`sort()` in R wraps the sorted result with metadata:

```r
# base/R/sort.R:50
.Internal(wrap_meta(vec, sorted, noNA))
```

This calls `wrap_meta` (altclasses.c:1969-2008) → `make_wrapper`. The wrapper records
that the vector is sorted (ascending or descending) and whether it contains NAs.

```r
x <- sort(runif(1000000))  # wrap_real with sorted=TRUE, no_na depends on data
```

### Attribute Modification (Copy-on-Modify Deferral)

`R_shallow_duplicate_attr` (duplicate.c:589-608) wraps vectors >= 64 elements instead of
copying them:

```c
// duplicate.c:589-593
#define WRAP_THRESHOLD 64
static SEXP duplicate_attr(SEXP x, Rboolean deep)
{
    if (isVector(x) && XLENGTH(x) >= WRAP_THRESHOLD) {
        SEXP val = R_tryWrap(x);
```

This is called during:

- **`names(x) <- value`** (attrib.c:940) — wrap instead of copying before setting names
- **`dimnames(x) <- value`** (attrib.c:1075) — wrap before setting dimnames
- **`unclass(x)`** (objects.c:970) — wrap before removing class
- **`structure(x, ...)`** (attrib.c:1400) — wrap before setting attributes
- **Complex assignment `x[i] <- v`** (eval.c:2577-2582) — `EnsureLocal` wraps shared
  vectors in the local environment to defer data duplication

```r
x <- runif(1000)
names(x) <- paste0("v", 1:1000)
# x is now a wrap_real wrapping the original data
# The data was NOT copied — only a wrapper was created

y <- x
names(y) <- NULL
# y is a NEW wrap_real wrapping x's underlying data
# Still no data copy
```

### When Wrappers Are NOT Created

The `wrap_meta` function (altclasses.c:1969-2008) has conditions that prevent wrapping:

1. **Vectors with attributes** (when `WRAPATTRIB` is not defined — current default):
   vectors that already have attributes are returned as-is, NOT wrapped
2. **Wrapper-of-wrapper avoidance**: if `x` is already a wrapper with no useful metadata,
   `shallow_duplicate` is used instead (altclasses.c:1983-1984)
3. **Short vectors**: `R_shallow_duplicate_attr` only wraps vectors >= 64 elements
   (duplicate.c:589)

### Wrapper Unwrapping

After a complex assignment completes, `R_tryUnwrap` (altclasses.c:2043-2071) checks
whether the wrapper can be dissolved — transferring attributes to the underlying data
and discarding the wrapper shell. This happens when the wrapper has no useful metadata
and only one reference.

---

## 5. Memory-Mapped Files (`mmap_integer`, `mmap_real`)

**Classes**: `mmap_integer`, `mmap_real` (altclasses.c:971-1247)

Unix-only. Created by `.Internal(mmap_file(file, type, ptrOK, wrtOK, serOK))`:

```c
// altclasses.c:986-1004 — make_mmap
SEXP ans = R_new_altrep(class, eptr, state);
```

Registered in `names.c:778`:
```c
{"mmap_file", do_mmap_file, 0, 11, -1, {PP_FUNCALL, PREC_FN, 0}},
```

This is **not commonly used** in regular R code. It's primarily for the serialization
system and experimental use. External packages that use mmap (like `bigmemory`, `ff`)
implement their own backends, not this built-in one.

---

## Summary: What You'll Actually Encounter

In practice, the ALTREP objects a typical R user encounters are:

| Frequency | Class | Created By |
|---|---|---|
| **Extremely common** | `compact_intseq` | `1:n`, `seq_len()`, `seq_along()`, `row.names()` |
| **Common** | `wrap_*` | `sort()`, attribute modification, complex assignment |
| **Occasional** | `deferred_string` | `as.character()` on int/real vectors |
| **Occasional** | `compact_realseq` | Large `:` ranges, coercion from `compact_intseq` |
| **Rare** | `mmap_*` | `.Internal(mmap_file(...))` |

### What Forces Materialization

Once materialized, the ALTREP object caches its expanded data and subsequent accesses
bypass dispatch. These operations force materialization:

| Operation | Why |
|---|---|
| `DATAPTR(x)` / `INTEGER(x)` / `REAL(x)` | Requests writable data pointer |
| `DATAPTR_RO(x)` on compact sequences | Must allocate array for pointer |
| `x[i] <- v` (subassignment) | Needs writable data |
| `sort(x)` on compact sequence | Needs to check/rearrange elements |
| String access on `deferred_string` | Each `STRING_ELT` call converts that element |
| Serialization (if no custom method) | `saveRDS`/`serialize` materializes first |

Note: `INTEGER_ELT(x, i)` on a `compact_intseq` does **not** materialize — it computes
`first + i * incr` via pure arithmetic (altclasses.c:168). Similarly, `Get_region`
fills a caller-provided buffer without allocating.

---

## Implications for miniextendr

For parallel code using rayon:

1. **`compact_intseq` / `compact_realseq`**: Call `DATAPTR` on main thread to force
   materialization, then send raw pointer to workers. Or use `INTEGER_ELT` element-wise
   (but that's ALTREP dispatch — unsafe from threads without Level 3 changes).

2. **`deferred_string`**: **Always materialize on main thread** before any concurrent
   access. Each element access calls `mkChar` which touches globals. Use `DATAPTR(x)` to
   force `expand_deferred_string` (altclasses.c:747-761), then extract `CHAR` pointers.

3. **`wrap_*`**: The wrapper's `Dataptr` method delegates to the wrapped vector. If the
   wrapped vector is non-ALTREP, `DATAPTR` returns its data pointer directly. If the
   wrapped vector is itself ALTREP (e.g., wrapping a compact sequence), dispatch recurses.
   Always materialize on main thread.

4. **General rule**: Call `DATAPTR(x)` / `REAL(x)` / `INTEGER(x)` from the R main thread
   (via `with_r_thread`). This handles all ALTREP cases by forcing materialization, returning
   a stable C pointer that rayon workers can safely use.
