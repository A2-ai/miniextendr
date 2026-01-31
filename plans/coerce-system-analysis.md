# Coerce System Analysis

## Current State

The Coerce system has:
- **57 Coerce impls** (infallible conversions)
- **32 TryCoerce impls** (fallible conversions)
- **10 macros** for generating scalar conversions
- **3 blanket impls** for containers

## ✅ What's Already Good

### Blanket Impls for Containers
```rust
// Slice to Vec coercion (element-wise)
impl<T: Copy + Coerce<R>, R> Coerce<Vec<R>> for &[T] { ... }

// Vec to Vec coercion (element-wise)
impl<T: Coerce<R>, R> Coerce<Vec<R>> for Vec<T> { ... }

// VecDeque to VecDeque coercion (NEW!)
impl<T: Coerce<R>, R> Coerce<VecDeque<R>> for VecDeque<T> { ... }

// Tuple coercions (element-wise, tuples 2-8)
impl<A: Coerce<RA>, B: Coerce<RB>> Coerce<(RA, RB)> for (A, B) { ... }
// ... through 8-tuples

// Coerce automatically implies TryCoerce
impl<T: Coerce<R>, R> TryCoerce<R> for T { ... }
```

### Scalar Conversion Macros (Appropriate)
These generate simple `.into()` calls for widening:
- `impl_identity!(T)` - T → T
- `impl_widen_i32!(T)` - T → i32 widening
- `impl_widen_f64!(T)` - T → f64 widening

## 🤔 Potential Improvements

### 1. Missing Container Blanket Impls

**Could add**:
```rust
// TinyVec/ArrayVec coercion
impl<T: Coerce<R>, const N: usize> Coerce<TinyVec<[R; N]>> for TinyVec<[T; N]> { ... }
impl<T: Coerce<R>, const N: usize> Coerce<ArrayVec<[R; N]>> for ArrayVec<[T; N]> { ... }

// HashSet coercion (for types where ordering doesn't matter)
impl<T: Coerce<R> + Eq + Hash, R: Eq + Hash> Coerce<HashSet<R>> for HashSet<T> { ... }
```

### 2. Identity as Blanket Impl?

**Current**: 5 macro invocations
```rust
impl_identity!(i32);
impl_identity!(f64);
// ...
```

**Could be**:
```rust
impl<T> Coerce<T> for T {
    fn coerce(self) -> T { self }
}
```

**Risk**: Would this conflict with anything? Let's check.

### 3. Widening as Blanket Impl?

**Current**: Many individual impls via macros
```rust
impl_widen_i32!(i8);    // impl Coerce<i32> for i8
impl_widen_i32!(i16);   // impl Coerce<i32> for i16
// ...
```

**Could be**:
```rust
impl<T: Into<R>, R> Coerce<R> for T {
    fn coerce(self) -> R {
        self.into()
    }
}
```

**Risk**: Not all Into impls should be Coerce
- Coerce implies "cheap conversion"
- Into can be expensive (allocations, complex logic)
- Some Into impls shouldn't be auto-coerced

**Verdict**: Probably NOT a good idea - Coerce should be explicit

### 4. Missing TryCoerce Container Impls?

**Current approach**: Manual iteration
```rust
// For TryCoerce only (not Coerce):
slice.iter().map(|x| x.try_coerce()).collect::<Result<Vec<_>, _>>()
```

**Could add**:
```rust
// This would conflict! Already auto-provided via Coerce→TryCoerce blanket
impl<T: TryCoerce<R>, R> TryCoerce<Vec<R>> for &[T] { ... }  // ❌ Overlaps
```

**Why it conflicts**:
- `impl<T: Coerce<R>> Coerce<Vec<R>> for &[T]` exists
- `impl<T: Coerce<R>> TryCoerce<R> for T` exists
- Together they give: `impl<T: Coerce<R>> TryCoerce<Vec<R>> for &[T]`
- Adding `impl<T: TryCoerce<R>> TryCoerce<Vec<R>> for &[T]` would overlap!

**The design is intentional** - for fallible coercions, use manual iteration.

## 🎯 Recommended Actions

### Add Missing Container Coerce Impls

These would be consistent with Vec/VecDeque pattern:

```rust
#[cfg(feature = "tinyvec")]
impl<T, R, const N: usize> Coerce<TinyVec<[R; N]>> for TinyVec<[T; N]>
where
    T: Coerce<R>,
    [T; N]: tinyvec::Array<Item = T>,
    [R; N]: tinyvec::Array<Item = R>,
{ ... }

#[cfg(feature = "tinyvec")]
impl<T, R, const N: usize> Coerce<ArrayVec<[R; N]>> for ArrayVec<[T; N]>
where
    T: Coerce<R>,
    [T; N]: tinyvec::Array<Item = T>,
    [R; N]: tinyvec::Array<Item = R>,
{ ... }
```

This would enable:
```rust
let tiny: TinyVec<[i8; 10]> = ...;
let wide: TinyVec<[i32; 10]> = tiny.coerce();  // Element-wise widening!
```

### Maybe: Identity Blanket Impl

**Try**:
```rust
impl<T> Coerce<T> for T {
    fn coerce(self) -> T { self }
}
```

**Check for conflicts** with existing impls. If it works, removes 5 macro invocations.

## Summary

**Coerce system is mostly well-designed with good blanket impls for common containers.**

Key improvements:
1. ✅ Added VecDeque coercion blanket impl
2. 🤔 Could add TinyVec/ArrayVec coercion
3. 🤔 Could try identity blanket impl
4. ❌ Can't add TryCoerce container blanket (conflicts by design)

The main gap is **container coercions for optional crates** (TinyVec, maybe ndarray/nalgebra).
