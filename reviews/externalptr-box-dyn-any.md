# ExternalPtr: Box<Box<dyn Any>> Storage

## What was changed

`ExternalPtr<T>` internal storage changed from `*mut T` (thin pointer directly to
concrete data) to `*mut Box<dyn Any>` (thin pointer → fat pointer → concrete data).

R's `R_ExternalPtrAddr` stores 8 bytes (one `void*`). A `Box<dyn Any>` is 16 bytes
(data ptr + vtable). The outer `Box` provides the thin pointer that fits in R's slot.

## Why

1. **Runtime type checking via `Any::downcast_ref`** — replaces R symbol comparison
   with Rust's authoritative `TypeId`-based checking. More robust, compiler-guaranteed.

2. **Non-generic finalizer** — `release_any()` replaces `release_raw::<T>()`. The `Any`
   vtable carries the concrete type's drop function. One finalizer for all ExternalPtr types.

3. **Opens the door for `dyn Trait` support** — from any `ExternalPtr`, you can
   `downcast_ref::<ConcreteType>()` and then coerce to `&dyn Trait`. The compiler
   generates the vtable at the coercion site.

## What changed in detail

- `new()` / `new_unchecked()`: allocate `Box::new(Box::new(x) as Box<dyn Any>)`
- `create_extptr_sexp()`: takes `*mut Box<dyn Any>` instead of `*mut T`
- `wrap_sexp()` / `wrap_sexp_with_error()`: use `Any::downcast_ref::<T>()` for type check
- `into_raw()`: disassembles `Box<Box<dyn Any>>`, leaks inner, returns `*mut T`
- `from_raw()`: re-wraps `*mut T` in `Box<dyn Any>` → `Box<Box<dyn Any>>`
- `into_inner()`: uses `Box<dyn Any>::downcast::<T>()` to recover `Box<T>`
- `release_any()`: non-generic finalizer, drops `Box<Box<dyn Any>>`
- `ExternalPtr<()>::is/downcast_ref/downcast_mut`: use `Any::is/downcast_ref/downcast_mut`

## What did NOT change

- `ExternalPtr<T>` struct layout: still `{ sexp, cached_ptr: NonNull<T>, PhantomData<T> }`
- `cached_ptr` still caches the concrete `*mut T` from downcast (zero-cost access)
- `TypedExternal` trait: still provides R-visible type name for display/tags
- R symbol tags in prot slot: still stored for display and cross-package error messages
- All public API signatures: `new`, `as_ref`, `wrap_sexp`, etc. unchanged

## Performance notes

- One extra heap allocation per ExternalPtr (the outer Box holding the fat pointer)
- `as_ref()`/`as_mut()` use `cached_ptr` — no downcast on every access
- Downcast happens once at `wrap_sexp` time (when recovering from R)
- Finalizer is now a single function pointer (no monomorphization overhead)
