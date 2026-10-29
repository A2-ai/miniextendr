use super::*;
use std::cell::RefCell;

// Note: These tests require R to be initialized and RNG state loaded.
// They're primarily for documentation/coverage purposes.

/// Compiles only for a type that is not `Send`: for a `Send` type both
/// impls apply and the call below is ambiguous.
trait AmbiguousIfSend<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

/// Compiles only for a type that is not `Sync`.
trait AmbiguousIfSync<A> {
    fn some_item() {}
}
impl<T: ?Sized> AmbiguousIfSync<()> for T {}
impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}

/// An `RRng` draws on the thread whose guard it borrows (#2005): it can't
/// move into a worker closure or a rayon task.
#[test]
fn rrng_is_neither_send_nor_sync() {
    let _ = <RRng<'static> as AmbiguousIfSend<_>>::some_item;
    let _ = <RRng<'static> as AmbiguousIfSync<_>>::some_item;
}

// Test RRngOps with a mock implementation
struct MockRng(RefCell<u64>);

impl MockRng {
    fn new(seed: u64) -> Self {
        Self(RefCell::new(seed))
    }

    // Simple LCG for testing
    fn next(&self) -> u64 {
        let mut state = self.0.borrow_mut();
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *state
    }
}

impl RRngOps for MockRng {
    fn random_f64(&self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    fn random_i32(&self) -> i32 {
        self.next() as i32
    }

    fn random_bool(&self) -> bool {
        self.next().is_multiple_of(2)
    }

    fn gen_range_f64(&self, low: f64, high: f64) -> f64 {
        assert!(low < high, "low must be less than high");
        low + self.random_f64() * (high - low)
    }

    fn gen_range_i32(&self, low: i32, high: i32) -> i32 {
        assert!(low < high, "low must be less than high");
        let range = (high - low) as u64;
        low + (self.next() % range) as i32
    }

    fn gen_bool(&self, p: f64) -> bool {
        assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
        self.random_f64() < p
    }
}

#[test]
fn test_rrngops_random_f64() {
    let rng = MockRng::new(42);
    let val = rng.random_f64();
    assert!((0.0..1.0).contains(&val));
}

#[test]
fn test_rrngops_random_i32() {
    let rng = MockRng::new(42);
    let _val = rng.random_i32(); // Just verify it doesn't panic
}

#[test]
fn test_rrngops_random_bool() {
    let rng = MockRng::new(42);
    // Generate multiple to verify both outcomes are possible
    let bools: Vec<bool> = (0..100).map(|_| rng.random_bool()).collect();
    assert!(bools.iter().any(|&b| b));
    assert!(bools.iter().any(|&b| !b));
}

#[test]
fn test_rrngops_gen_range_f64() {
    let rng = MockRng::new(42);
    for _ in 0..100 {
        let val = rng.gen_range_f64(10.0, 20.0);
        assert!((10.0..20.0).contains(&val));
    }
}

#[test]
fn test_rrngops_gen_range_i32() {
    let rng = MockRng::new(42);
    for _ in 0..100 {
        let val = rng.gen_range_i32(5, 15);
        assert!((5..15).contains(&val));
    }
}

#[test]
fn test_rrngops_gen_bool() {
    let rng = MockRng::new(42);
    // With p=0.5, should get roughly equal distribution
    let count_true = (0..1000).filter(|_| rng.gen_bool(0.5)).count();
    // Should be roughly 500 ± 100
    assert!(count_true > 350 && count_true < 650);
}

#[test]
fn test_rrngops_vec_methods() {
    let rng = MockRng::new(42);

    let f64_vec = rng.random_f64_vec(10);
    assert_eq!(f64_vec.len(), 10);
    assert!(f64_vec.iter().all(|&v| (0.0..1.0).contains(&v)));

    let range_vec = rng.gen_range_f64_vec(10, -5.0, 5.0);
    assert_eq!(range_vec.len(), 10);
    assert!(range_vec.iter().all(|&v| (-5.0..5.0).contains(&v)));

    let int_vec = rng.gen_range_i32_vec(10, 0, 100);
    assert_eq!(int_vec.len(), 10);
    assert!(int_vec.iter().all(|&v| (0..100).contains(&v)));

    let bool_vec = rng.gen_bool_vec(10, 0.5);
    assert_eq!(bool_vec.len(), 10);
}

#[test]
#[should_panic(expected = "low must be less than high")]
fn test_rrngops_gen_range_f64_invalid() {
    let rng = MockRng::new(42);
    rng.gen_range_f64(10.0, 5.0); // low > high
}

#[test]
#[should_panic(expected = "p must be in [0, 1]")]
fn test_rrngops_gen_bool_invalid() {
    let rng = MockRng::new(42);
    rng.gen_bool(1.5); // p > 1
}

// Test RDistributionOps with a mock uniform distribution
struct MockUniformDist {
    low: f64,
    high: f64,
    rng: RefCell<u64>,
}

impl MockUniformDist {
    fn new(low: f64, high: f64, seed: u64) -> Self {
        Self {
            low,
            high,
            rng: RefCell::new(seed),
        }
    }

    fn next_f64(&self) -> f64 {
        let mut state = self.rng.borrow_mut();
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*state as f64) / (u64::MAX as f64)
    }
}

impl RDistributionOps<f64> for MockUniformDist {
    fn sample(&self) -> f64 {
        self.low + self.next_f64() * (self.high - self.low)
    }

    fn mean(&self) -> Option<f64> {
        Some((self.low + self.high) / 2.0)
    }

    fn variance(&self) -> Option<f64> {
        let range = self.high - self.low;
        Some(range * range / 12.0)
    }
}

#[test]
fn test_rdistributionops_sample() {
    let dist = MockUniformDist::new(0.0, 10.0, 42);
    let sample = dist.sample();
    assert!((0.0..10.0).contains(&sample));
}

#[test]
fn test_rdistributionops_sample_n() {
    let dist = MockUniformDist::new(5.0, 15.0, 42);
    let samples = dist.sample_n(100);
    assert_eq!(samples.len(), 100);
    assert!(samples.iter().all(|&s| (5.0..15.0).contains(&s)));
}

#[test]
fn test_rdistributionops_sample_vec() {
    let dist = MockUniformDist::new(0.0, 1.0, 42);
    let samples = dist.sample_vec(50);
    assert_eq!(samples.len(), 50);
}

#[test]
fn test_rdistributionops_mean() {
    let dist = MockUniformDist::new(0.0, 10.0, 42);
    assert_eq!(dist.mean(), Some(5.0));
}

#[test]
fn test_rdistributionops_variance() {
    let dist = MockUniformDist::new(0.0, 12.0, 42);
    // Variance of uniform(0, 12) = (12-0)^2 / 12 = 144/12 = 12
    assert_eq!(dist.variance(), Some(12.0));
}

#[test]
fn test_rdistributionops_std_dev() {
    let dist = MockUniformDist::new(0.0, 12.0, 42);
    let std_dev = dist.std_dev().unwrap();
    // std_dev = sqrt(12) ≈ 3.464
    assert!((std_dev - 12.0_f64.sqrt()).abs() < 1e-10);
}

// Test distribution with no known statistics
struct MockUnknownDist(RefCell<u64>);

impl RDistributionOps<i32> for MockUnknownDist {
    fn sample(&self) -> i32 {
        let mut state = self.0.borrow_mut();
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*state % 100) as i32
    }
}

#[test]
fn test_rdistributionops_unknown_stats() {
    let dist = MockUnknownDist(RefCell::new(42));
    assert_eq!(dist.mean(), None);
    assert_eq!(dist.variance(), None);
    assert_eq!(dist.std_dev(), None);
    // But sampling still works
    let _sample = dist.sample();
}
