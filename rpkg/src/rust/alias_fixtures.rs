//! Regression fixtures for pre-conversion borrowed-slice alias checks.

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
