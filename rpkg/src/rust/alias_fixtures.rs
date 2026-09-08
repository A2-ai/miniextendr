//! Regression fixtures for pre-conversion native-borrow alias checks.

use miniextendr_api::miniextendr;

fn bump(slices: impl IntoIterator<Item = &'static mut [i32]>) -> i32 {
    let mut total = 0;
    for slice in slices {
        for value in slice {
            *value += 1;
            total += *value;
        }
    }
    total
}

fn read(slices: impl IntoIterator<Item = &'static [i32]>) -> i32 {
    slices.into_iter().flatten().copied().sum()
}

/// Exercise borrowed-slice alias checks with list mut direct mut inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_mut_direct_mut(a: Vec<&'static mut [i32]>, b: &'static mut [i32]) -> i32 {
    bump(a) + bump([b])
}

/// Exercise borrowed-slice alias checks with list mut direct shared inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_mut_direct_shared(a: Vec<&'static mut [i32]>, b: &'static [i32]) -> i32 {
    bump(a) + read([b])
}

/// Exercise borrowed-slice alias checks with list shared direct mut inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_shared_direct_mut(a: Vec<&'static [i32]>, b: &'static mut [i32]) -> i32 {
    read(a) + bump([b])
}

/// Exercise borrowed-slice alias checks with list mut list mut inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_mut_list_mut(a: Vec<&'static mut [i32]>, b: Vec<&'static mut [i32]>) -> i32 {
    bump(a) + bump(b)
}

/// Exercise borrowed-slice alias checks with list mut list shared inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_mut_list_shared(a: Vec<&'static mut [i32]>, b: Vec<&'static [i32]>) -> i32 {
    bump(a) + read(b)
}

/// Exercise borrowed-slice alias checks with list shared list mut inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_shared_list_mut(a: Vec<&'static [i32]>, b: Vec<&'static mut [i32]>) -> i32 {
    read(a) + bump(b)
}

/// Exercise borrowed-slice alias checks with list shared list shared inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_list_shared_list_shared(a: Vec<&'static [i32]>, b: Vec<&'static [i32]>) -> i32 {
    read(a) + read(b)
}

/// Exercise borrowed-slice alias checks with optional lists inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_optional_lists(
    a: Option<Vec<Option<&'static mut [i32]>>>,
    b: Option<&'static [i32]>,
) -> i32 {
    bump(a.into_iter().flatten().flatten()) + read(b)
}

/// Exercise borrowed-slice alias checks with nested lists inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_nested_lists(a: Vec<Vec<&'static mut [i32]>>, b: Vec<&'static [i32]>) -> i32 {
    bump(a.into_iter().flatten()) + read(b)
}

/// Exercise borrowed-slice alias checks with owned and list inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker)]
pub fn alias_owned_and_list(a: Vec<i32>, b: Vec<&'static mut [i32]>) -> i32 {
    a.into_iter().sum::<i32>() + bump(b)
}

/// Exercise borrowed-slice alias checks with worker lists inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_lists(a: Vec<&'static mut [i32]>, b: &'static [i32]) -> i32 {
    bump(a) + read([b])
}

/// Exercise borrowed-slice alias checks before worker RNG cleanup.
/// @param a First list of vectors.
/// @param b Second vector.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker, rng)]
pub fn alias_worker_rng_lists(a: Vec<&'static mut [i32]>, b: &'static [i32]) -> i32 {
    bump(a) + read([b])
}

/// Exercise borrowed-slice alias checks with rng lists inputs.
/// @param a First vector or list of vectors.
/// @param b Second vector or list of vectors.
#[miniextendr(no_worker, rng)]
pub fn alias_rng_lists(a: Vec<&'static mut [i32]>, b: &'static [i32]) -> i32 {
    bump(a) + read([b])
}

/// Exercise batched diagnostics for three conflicting arguments.
/// @param a First list of integer vectors.
/// @param b Shared integer vector.
/// @param c Second list of integer vectors.
#[miniextendr(no_worker)]
pub fn alias_three_arguments(
    a: Vec<&'static mut [i32]>,
    b: &'static [i32],
    c: Vec<&'static mut [i32]>,
) -> i32 {
    bump(a) + read([b]) + bump(c)
}

/// Environment-class fixture for borrowed-slice alias checks in methods.
#[derive(Default, miniextendr_api::ExternalPtr)]
pub struct AliasGuardProbe;

#[miniextendr(env)]
impl AliasGuardProbe {
    /// Create an alias-check probe.
    pub fn new() -> Self {
        Self
    }

    /// Check list and direct-slice arguments through an instance method.
    /// @param a List of integer vectors.
    /// @param b Shared integer vector.
    pub fn check(&self, a: Vec<&'static mut [i32]>, b: &'static [i32]) -> i32 {
        bump(a) + read([b])
    }
}

// region: Native scalar and boxed borrows (#1502)

/// Reject overlapping integer scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[miniextendr(no_worker)]
pub fn alias_scalar_integer(a: &mut i32, b: &mut i32, c: &i32) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping integer scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_scalar_integer(a: &mut i32, b: &mut i32, c: &i32) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping real scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[miniextendr(no_worker)]
pub fn alias_scalar_real(a: &mut f64, b: &mut f64, c: &f64) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping real scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_scalar_real(a: &mut f64, b: &mut f64, c: &f64) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping raw scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[miniextendr(no_worker)]
pub fn alias_scalar_raw(a: &mut u8, b: &mut u8, c: &u8) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping raw scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_scalar_raw(a: &mut u8, b: &mut u8, c: &u8) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping logical scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[miniextendr(no_worker)]
pub fn alias_scalar_logical(
    a: &mut miniextendr_api::RLogical,
    b: &mut miniextendr_api::RLogical,
    c: &miniextendr_api::RLogical,
) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping logical scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_scalar_logical(
    a: &mut miniextendr_api::RLogical,
    b: &mut miniextendr_api::RLogical,
    c: &miniextendr_api::RLogical,
) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping complex scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[miniextendr(no_worker)]
pub fn alias_scalar_complex(
    a: &mut miniextendr_api::Rcomplex,
    b: &mut miniextendr_api::Rcomplex,
    c: &miniextendr_api::Rcomplex,
) -> i32 {
    *a = *c;
    *b = *c;
    1
}

/// Reject overlapping complex scalar borrows before conversion.
/// @param a Mutable scalar.
/// @param b Second mutable scalar.
/// @param c Shared scalar.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
pub fn alias_worker_scalar_complex(
    a: &mut miniextendr_api::Rcomplex,
    b: &mut miniextendr_api::Rcomplex,
    c: &miniextendr_api::Rcomplex,
) -> i32 {
    *a = *c;
    *b = *c;
    1
}

type NestedScalarBorrow = Option<Vec<Vec<&'static mut i32>>>;

/// Native scalar wrapped by a forwarding conversion.
#[derive(miniextendr_api::TryFromSexp)]
pub struct AliasBorrowedScalar(&'static mut i32);

/// Exercise native borrow metadata through scalar shared conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_scalar_shared(a: &i32, b: &i32) -> i32 {
    *a + *b
}

/// Exercise native borrow metadata through scalar slice conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_scalar_slice(a: &mut i32, b: &[i32]) -> i32 {
    *a = b.iter().sum();
    *a
}

/// Exercise native borrow metadata through slice scalar conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_slice_scalar(a: &mut [i32], b: &i32) -> i32 {
    a.fill(*b);
    i32::try_from(a.len()).unwrap()
}

/// Exercise native borrow metadata through scalar lists conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_scalar_lists(a: Vec<&'static mut i32>, b: Vec<&'static i32>) -> i32 {
    let value = b.into_iter().copied().sum();
    for a in a {
        *a = value;
    }
    value
}

/// Exercise native borrow metadata through boxed scalars conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
#[allow(clippy::boxed_local)] // Exercise the actual boxed conversion.
pub fn alias_boxed_scalars(
    a: Box<[Option<&'static mut i32>]>,
    b: Box<[Option<&'static [i32]>]>,
) -> i32 {
    let value = read(b.into_vec().into_iter().flatten());
    for a in a.into_vec().into_iter().flatten() {
        *a = value;
    }
    value
}

/// Exercise native borrow metadata through boxed slices conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
#[allow(clippy::boxed_local)] // Exercise the actual boxed conversion.
pub fn alias_boxed_slices(a: Box<[&'static mut [i32]]>, b: &i32) -> i32 {
    bump(a.into_vec()) + *b
}

/// Exercise native borrow metadata through nested scalars conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_nested_scalars(a: NestedScalarBorrow, b: &i32) -> i32 {
    for a in a.into_iter().flatten().flatten() {
        *a = *b;
    }
    *b
}

/// Exercise native borrow metadata through newtype scalars conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
#[allow(clippy::boxed_local)] // Exercise the actual boxed conversion.
pub fn alias_newtype_scalars(a: Box<[Option<AliasBorrowedScalar>]>, b: &i32) -> i32 {
    for a in a.into_vec().into_iter().flatten() {
        *a.0 = *b;
    }
    *b
}

/// Exercise native borrow metadata through wrapped scalars conversions.
/// @param a First scalar, vector, or list.
/// @param b Second scalar, vector, or list.
#[miniextendr(no_worker)]
pub fn alias_wrapped_scalars(a: Result<&mut i32, ()>, b: miniextendr_api::Missing<&i32>) -> i32 {
    if let (Ok(a), miniextendr_api::Missing::Present(b)) = (a, b) {
        *a = *b;
    }
    1
}

/// Check boxed native borrows before worker dispatch.
/// @param a Optional mutable scalars in a list.
/// @param b Optional shared slices in a list.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker)]
#[allow(clippy::boxed_local)] // Exercise the actual boxed conversion.
pub fn alias_worker_boxed_scalars(
    a: Box<[Option<&'static mut i32>]>,
    b: Box<[Option<&'static [i32]>]>,
) -> i32 {
    alias_boxed_scalars(a, b)
}
// endregion
