# Level 4: Fork-COW Snapshots

Formalized fork-based parallelism. What R already has via `mclapply`, analyzed as a
concurrency model with its strengths, fundamental limitations, and potential improvements.

Source: R source at `background/r-svn/`.

---

## How It Works Today

R's `parallel::mclapply` uses `fork()` to create child processes:

```c
// src/library/parallel/src/fork.c:577-590
pid = fork();
if (pid == 0) { /* child */
    R_isForkedChild = 1;
    /* free children entries inherited from parent */
    while(children) {
        close_fds_child_ci(children);
        child_info_t *ci = children->next;
        free(children);
        children = ci;
    }
    restore_sigchld(&ss);
    restore_sig_handler();
```

The child gets a copy-on-write (COW) snapshot of the parent's entire address space:

1. **Parent** reaches a fork point (all R evaluation paused)
2. **`fork()`** creates a child — same virtual memory pages, marked COW by the kernel
3. **Child** evaluates R code against its private copy of the R runtime
4. **Results** serialized back to parent via pipe (`serialize()` → `unserialize()`)
5. **Child exits**, parent collects results

---

## Strengths

| Property | Why it matters |
|---|---|
| **Perfect isolation** | Each child has its own `R_GlobalContext`, `R_PPStack`, `R_GenHeap` — all globals are private copies |
| **Zero shared mutable state** | COW means reads share pages, writes are private. No data races by construction. |
| **Works with arbitrary R code** | No restrictions on what the child can do — `eval`, `allocate`, `GC`, everything works |
| **No R changes needed** | `fork()` is an OS primitive. R doesn't need any concurrency support. |
| **Mature implementation** | `mclapply` has been in R since 2.14.0 (2011). Well-tested. |

This is the only form of R parallelism that can evaluate arbitrary R code safely.

---

## Weaknesses

### Unix-Only

`fork()` doesn't exist on Windows. `parallel::mclapply` falls back to `lapply` (serial)
on Windows, or uses `parLapply` with separate R processes via sockets (no COW benefit).

### Heavy Process Overhead

Each `fork()` child inherits the entire R process:

- Page table duplication: O(virtual memory size) kernel work
- File descriptor inheritance: must close unneeded FDs
- Signal handler state: must be reset
- Child startup: ~1-5ms per fork on modern Linux

For fine-grained parallelism (millions of small tasks), the per-fork overhead dominates.

### Serialization Cost

Results travel parent ← child via pipes using R's `serialize()`/`unserialize()`:

- Every result object is fully serialized to bytes and deserialized
- Large objects (big data frames, matrices) are copied byte-by-byte through the kernel pipe buffer
- No shared memory return path — even if the child computed in-place on a COW page

### No Shared Mutable State

The child can't update the parent's data in place. If you want to fill column 3 of a
matrix in parallel, each child has to return its entire result, and the parent has to
assemble them. This is the fundamental trade-off of isolation.

---

## The GC-COW Problem

The most significant performance issue with fork-based parallelism in R:

```
// Timeline:
Parent: [protected objects across many pages] → fork()
Child:  [COW sharing parent's pages]
Child:  R_gc_internal() → marks every reachable object → writes mark bit to sxpinfo
        → kernel triggers COW fault on EVERY page containing a reachable object
        → doubles memory usage
```

R objects are spread across many memory pages. GC's mark phase (`RunGenCollect` at
`src/main/memory.c:1681+`) writes the `mark` bit in every reachable object's sxpinfo
header. Each write triggers a COW page fault — the kernel copies the entire 4KB page
to the child's private memory.

For a process with 1GB of R objects spread across ~250K pages, the first GC in a child
can dirty most of them, effectively doubling memory usage per child.

### What R_isForkedChild Does

```c
// src/include/Defn.h:1659
extern Rboolean R_isForkedChild INI_as(FALSE); /* was this forked? */
```

When `R_isForkedChild = 1` (set in `fork.c:581`), R disables:

- **JIT compilation** — avoids writing compiled code to memory pages
- Some signal handlers — avoids signal-related memory writes

But it does **NOT** disable GC. The child's first GC still dirties all reachable pages.

### The Scale of the Problem

| Scenario | Parent RSS | Children (4) | Peak total |
|---|---|---|---|
| No GC in children | 1 GB | ~0 extra (COW) | ~1 GB |
| First GC per child | 1 GB | ~1 GB each | ~5 GB |
| Multiple GC cycles | 1 GB | ~1 GB each | ~5 GB |

This is why `mclapply` with large R heaps can exhaust memory even though the children
are "just reading" the parent's data.

---

## Potential Improvements

### 1. Pre-Fork GC (~100 lines)

Run a full GC immediately before `fork()` to minimize the number of dirty pages:

```c
// Before fork:
R_gc();  // collect garbage — mark bits are now set and stable
fork();  // child inherits marked state — fewer COW faults
```

If GC runs right before fork, the child inherits clean mark bits. The child's first
allocation may trigger GC, but there's less garbage to collect and fewer pages to dirty.

**Limitation**: Only helps if the parent's heap is stable. If the parent has many
recently-allocated objects, they'll be in generation 0 and marked during the pre-fork GC,
dirtying pages anyway.

### 2. Mark-Bit Segregation (~500 lines)

Move GC mark bits out of the sxpinfo bitfield into a separate bitmap:

```c
// Instead of: sxpinfo.mark (bit in 64-bit header word)
// Use: mark_bitmap[node_index / 8] |= (1 << (node_index % 8))
```

The bitmap lives on a few dedicated pages. GC only dirties bitmap pages, not the pages
containing the actual R objects. Children's GC dirties ~1 page per 32K objects instead
of ~1 page per ~50 objects.

**Cost**: Requires changing `MARK_NODE` / `UNMARK_NODE` macros, adding a bitmap allocator,
and ensuring bitmap pages are allocated contiguously. ~500 lines, minor ABI impact.

### 3. Shared-Memory Result Return (~1000 lines)

Replace pipe serialization with `mmap`'d shared memory regions:

```c
// Parent:
void *shm = mmap(NULL, result_size, PROT_READ|PROT_WRITE,
                 MAP_SHARED|MAP_ANONYMOUS, -1, 0);
fork();

// Child:
// Write result directly into shm (no serialization)
serialize_to_shm(result, shm);
exit(0);

// Parent:
// Read result from shm (no pipe copy)
result = unserialize_from_shm(shm);
munmap(shm, result_size);
```

This avoids the double-copy through kernel pipe buffers. For large results (matrices,
data frames), this can be significantly faster.

**Complexity**: Need to handle variable-size results, synchronization (parent must wait
for child to finish writing), and cleanup on child crash.

### 4. Windows Support via CreateProcess (~2000 lines)

Implement `mclapply` on Windows using `CreateProcess` + shared memory:

1. Parent serializes R state to shared memory
2. `CreateProcess` spawns a new R process
3. Child deserializes state, evaluates function, serializes result
4. Parent reads result from shared memory

This is slower than `fork()` (full process creation + state serialization) but provides
the same isolation model on Windows.

**Precedent**: Python's `multiprocessing` module does exactly this — `fork()` on Unix,
`spawn` + pickle on Windows.

---

## Estimated Scope

| Improvement | Lines | Benefit |
|---|---|---|
| Pre-fork GC | ~100 | Moderate memory reduction |
| Mark-bit segregation | ~500 | Major memory reduction (COW-friendly) |
| Shared-memory results | ~1000 | Faster result return |
| Windows support | ~2000 | Cross-platform |
| **Total** | **~3600** | — |

Each improvement is independent and incremental — they don't require the others.

---

## Comparison with Thread-Based Approaches

| Property | Fork (Level 4) | Threads (Levels 0-3) |
|---|---|---|
| Isolation | Full (separate address space) | None (shared heap) |
| R code support | Arbitrary | Very limited |
| Overhead per task | ~1-5ms (fork) | ~1μs (thread spawn) |
| Memory overhead | COW (low if no GC, high if GC) | Zero |
| Result transfer | Serialization (slow for large data) | Shared memory (zero-copy) |
| Platform support | Unix only | All platforms |
| Implementation | Already exists | Requires R changes |

Fork is the right choice when:
- Tasks are coarse-grained (>10ms each)
- Tasks need full R evaluation
- Memory overhead is acceptable
- Unix-only is acceptable

Threads are better when:
- Tasks are fine-grained (microsecond-level)
- Tasks are pure computation on extracted data
- Zero-copy result sharing is needed
- Cross-platform is needed

---

## Who Uses This

| Package | Pattern |
|---|---|
| `parallel::mclapply` | Fork per element, serialize results back |
| `parallel::mcparallel` | Fork + collect — lower-level API |
| `future` (multisession) | Fork-based or socket-based backends |
| `callr` | Separate R process (not fork, but similar isolation) |

Fork-based parallelism is R's most mature concurrency story. Its limitations are well
understood, and the improvements above would address the most painful ones without
requiring any changes to R's single-threaded runtime.
