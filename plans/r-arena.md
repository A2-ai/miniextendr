# Plan: `RArena` - Scoped Transient Allocation on R's Memory Stack

## Goal

Add a scoped arena API backed by `vmaxget`/`vmaxset` + `R_alloc`/`S_alloc` for short-lived scratch allocations within a single `.Call` context.

This complements, not replaces, `RAllocator`.

## Why

`RAllocator` (RAWSXP + preserve list) supports individually managed allocations and cross-call persistence, but has per-allocation overhead.

`R_alloc`-family APIs are optimized for transient bulk-freed memory and are a better fit for temporary buffers.

## Safety and API Constraints

1. `RArena` is **main-thread only** and `!Send + !Sync`.
2. Memory lifetime ends at `Drop`/`reset()` or at `.Call` return (whichever comes first).
3. Do not expose a fully safe generic `alloc_slice<T>()` over arbitrary `T`.
4. Prefer byte-oriented safe APIs; typed APIs must be `unsafe` and alignment-aware.

R docs note `R_alloc` alignment is only guaranteed for `double` unless additional care is taken.

## Proposed Type

```rust
pub struct RArena {
    watermark: *mut c_void,
    active: bool,
    _not_send_sync: PhantomData<Rc<()>>,
}
```

- `active` prevents double-reset in `Drop` after explicit `reset()`.

## Constructor

```rust
impl RArena {
    pub fn new() -> Result<Self, ArenaError> {
        if !crate::worker::is_r_main_thread() {
            return Err(ArenaError::WrongThread);
        }
        let watermark = unsafe { crate::ffi::vmaxget() };
        Ok(Self { watermark, active: true, _not_send_sync: PhantomData })
    }
}
```

Use checked FFI wrappers (`vmaxget`, `vmaxset`, `R_alloc`, `S_alloc`), not imaginary `_unchecked` variants.

## Allocation API

### Safe APIs (preferred)

```rust
impl RArena {
    pub fn alloc_bytes(&self, nbytes: usize) -> *mut u8;
    pub fn alloc_zeroed_bytes(&self, nbytes: usize) -> *mut u8;
}
```

### Unsafe typed API (explicit contracts)

```rust
impl RArena {
    pub unsafe fn alloc_uninit_slice<T>(
        &self,
        n: usize,
    ) -> Result<&mut [std::mem::MaybeUninit<T>], ArenaError>;
}
```

Preconditions for typed allocation:
- caller ensures `align_of::<T>() <= align_of::<f64>()` (or method returns `AlignmentTooStrict`),
- caller initializes before read,
- caller does not outlive arena/reset.

## Reset and Drop

```rust
impl RArena {
    pub fn reset(&mut self) {
        if self.active {
            unsafe { crate::ffi::vmaxset(self.watermark) };
            self.active = false;
        }
    }
}

impl Drop for RArena {
    fn drop(&mut self) {
        if self.active {
            unsafe { crate::ffi::vmaxset(self.watermark) };
        }
    }
}
```

`reset(&mut self)` (not `&self`) helps avoid keeping live typed borrows while freeing arena memory.

## Error Type

```rust
pub enum ArenaError {
    WrongThread,
    AlignmentTooStrict { requested: usize, max_supported: usize },
    SizeOverflow,
}
```

## Nesting

Nested arenas remain valid via watermark stack discipline:
- inner drop/reset restores to inner watermark,
- outer drop/reset restores earlier watermark.

## What `RArena` Does Not Do

- no individual deallocation,
- no `GlobalAlloc` implementation,
- no cross-`.Call` persistence,
- no worker-thread/rayon routing.

## Files to Modify

- `miniextendr-api/src/arena.rs` (new)
- `miniextendr-api/src/lib.rs`

## Verification

1. `cargo check --workspace`
2. `cargo clippy --workspace`
3. `cargo test --workspace`
4. doctest/examples showing:
   - nested arenas,
   - reset invalidation warning,
   - byte allocation usage.

## Optional Benchmark

Compare transient allocation throughput:
- `RArena::alloc_bytes` loop
- `RAllocator` equivalent transient allocations

Measure only within safe `.Call`-style contexts.
