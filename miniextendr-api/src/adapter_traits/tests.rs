use super::*;

#[test]
fn test_rdebug() {
    let v = vec![1, 2, 3];
    assert_eq!(v.debug_str(), "[1, 2, 3]");
    assert!(v.debug_str_pretty().contains('\n') || v.debug_str_pretty() == "[1, 2, 3]");
}

#[test]
fn test_rdisplay() {
    let s = "hello";
    assert_eq!(s.as_r_string(), "hello");

    let n = 42i32;
    assert_eq!(n.as_r_string(), "42");
}

#[test]
fn test_rhash() {
    let a = "test";
    let b = "test";
    let c = "other";

    assert_eq!(a.r_hash(), b.r_hash());
    assert_ne!(a.r_hash(), c.r_hash());
}

#[test]
fn test_rord() {
    assert_eq!(1i32.r_cmp(&2), -1);
    assert_eq!(2i32.r_cmp(&2), 0);
    assert_eq!(3i32.r_cmp(&2), 1);
}

#[test]
fn test_rpartialord() {
    assert_eq!(1.0f64.r_partial_cmp(&2.0), Some(-1));
    assert_eq!(2.0f64.r_partial_cmp(&2.0), Some(0));
    assert_eq!(3.0f64.r_partial_cmp(&2.0), Some(1));
    assert_eq!(f64::NAN.r_partial_cmp(&1.0), None);
}

#[test]
fn test_rerror_simple() {
    use std::io;
    let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    assert_eq!(err.error_message(), "file not found");
    assert_eq!(err.error_chain().len(), 1);
    assert_eq!(err.error_chain_length(), 1);
}

#[test]
fn test_rerror_chain() {
    use std::fmt;

    #[derive(Debug)]
    struct OuterError {
        msg: &'static str,
        source: InnerError,
    }

    #[derive(Debug)]
    struct InnerError {
        msg: &'static str,
    }

    impl fmt::Display for OuterError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl fmt::Display for InnerError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for InnerError {}

    impl std::error::Error for OuterError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.source)
        }
    }

    let err = OuterError {
        msg: "outer error",
        source: InnerError { msg: "inner error" },
    };

    assert_eq!(err.error_message(), "outer error");
    let chain = err.error_chain();
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0], "outer error");
    assert_eq!(chain[1], "inner error");
    assert_eq!(err.error_chain_length(), 2);
}

#[test]
fn test_rfromstr_success() {
    assert_eq!(i32::r_from_str("42"), Some(42));
    assert_eq!(
        f64::r_from_str("3.141592653589793"),
        Some(std::f64::consts::PI)
    );
    assert_eq!(bool::r_from_str("true"), Some(true));
}

#[test]
fn test_rfromstr_failure() {
    assert_eq!(i32::r_from_str("not a number"), None);
    assert_eq!(f64::r_from_str("abc"), None);
}

#[test]
fn test_rclone() {
    let v = vec![1, 2, 3];
    let cloned = v.r_clone();
    assert_eq!(v, cloned);

    // Verify it's a deep copy
    let s = String::from("hello");
    let cloned_s = s.r_clone();
    assert_eq!(s, cloned_s);
}

#[test]
fn test_rdefault() {
    assert_eq!(i32::r_default(), 0);
    assert!(Vec::<i32>::r_default().is_empty());
    assert_eq!(String::r_default(), "");
    assert!(!bool::r_default());
}

#[test]
fn test_rcopy() {
    // Primitives are Copy
    let x = 42i32;
    let y = RCopy::copy(&x);
    assert_eq!(x, y);
    assert!(x.is_copy());

    // Tuples of Copy types are Copy
    let point = (1.0f64, 2.0f64);
    let point2 = RCopy::copy(&point);
    assert_eq!(point, point2);
    assert!(point.is_copy());

    // Arrays of Copy types are Copy
    let arr = [1, 2, 3];
    let arr2 = RCopy::copy(&arr);
    assert_eq!(arr, arr2);
}

// Tests for RIterator
use std::cell::RefCell;

/// Test iterator wrapper using RefCell for interior mutability.
struct TestIter(RefCell<std::vec::IntoIter<i32>>);

impl TestIter {
    fn new(data: Vec<i32>) -> Self {
        Self(RefCell::new(data.into_iter()))
    }
}

impl RIterator for TestIter {
    type Item = i32;

    fn next(&self) -> Nullable<Self::Item> {
        Nullable(self.0.borrow_mut().next())
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

#[test]
fn test_riterator_next() {
    let it = TestIter::new(vec![1, 2, 3]);
    assert_eq!(it.next().0, Some(1));
    assert_eq!(it.next().0, Some(2));
    assert_eq!(it.next().0, Some(3));
    assert_eq!(it.next().0, None);
    assert_eq!(it.next().0, None); // Stays exhausted
}

#[test]
fn test_riterator_size_hint() {
    let it = TestIter::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(it.size_hint(), (5, Some(5)));
    it.next();
    assert_eq!(it.size_hint(), (4, Some(4)));
    it.next();
    it.next();
    assert_eq!(it.size_hint(), (2, Some(2)));
}

#[test]
fn test_riterator_count() {
    let it = TestIter::new(vec![1, 2, 3, 4, 5]);
    assert_eq!(it.count(), 5);
    // Iterator is now exhausted
    assert_eq!(it.next().0, None);
}

#[test]
fn test_riterator_collect_n() {
    let it = TestIter::new(vec![1, 2, 3, 4, 5]);
    let first_three = it.collect_n(3);
    assert_eq!(first_three, vec![1, 2, 3]);
    let remaining = it.collect_n(10); // Ask for more than available
    assert_eq!(remaining, vec![4, 5]);
}

#[test]
fn test_riterator_skip() {
    let it = TestIter::new(vec![1, 2, 3, 4, 5]);
    let skipped = it.skip(2);
    assert_eq!(skipped, 2);
    assert_eq!(it.next().0, Some(3));

    // Skip more than remaining
    let skipped = it.skip(10);
    assert_eq!(skipped, 2); // Only 2 elements were left
}

#[test]
fn test_riterator_nth() {
    let it = TestIter::new(vec![10, 20, 30, 40, 50]);
    // Get element at index 2 (third element)
    assert_eq!(it.nth(2).0, Some(30));
    // Iterator has consumed 0, 1, 2 - next is index 3
    assert_eq!(it.next().0, Some(40));

    // Negative index returns None
    let it2 = TestIter::new(vec![1, 2, 3]);
    assert_eq!(it2.nth(-1).0, None);
}

#[test]
fn test_riterator_empty() {
    let it = TestIter::new(vec![]);
    assert_eq!(it.next().0, None);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(it.count(), 0);
    assert_eq!(it.collect_n(5), Vec::<i32>::new());
    assert_eq!(it.skip(5), 0);
    assert_eq!(it.nth(0).0, None);
}

// Tests for RExtend
struct TestExtendVec(RefCell<Vec<i32>>);

impl TestExtendVec {
    fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    fn get(&self) -> Vec<i32> {
        Clone::clone(&*self.0.borrow())
    }
}

impl RExtend<i32> for TestExtendVec {
    fn extend_from_vec(&self, items: Vec<i32>) {
        self.0.borrow_mut().extend(items);
    }

    fn r_len(&self) -> i64 {
        i64::try_from(self.0.borrow().len()).expect("length fits i64")
    }
}

#[test]
fn test_rextend_basic() {
    let v = TestExtendVec::new();
    assert_eq!(v.get(), Vec::<i32>::new());
    assert_eq!(v.r_len(), 0);
    assert!(v.r_is_empty());

    v.extend_from_vec(vec![1, 2, 3]);
    assert_eq!(v.get(), vec![1, 2, 3]);
    assert_eq!(v.r_len(), 3);
    assert!(!v.r_is_empty());

    v.extend_from_vec(vec![4, 5]);
    assert_eq!(v.get(), vec![1, 2, 3, 4, 5]);
    assert_eq!(v.r_len(), 5);
}

#[test]
fn test_rextend_empty() {
    let v = TestExtendVec::new();
    v.extend_from_vec(vec![]);
    assert_eq!(v.get(), Vec::<i32>::new());
    assert_eq!(v.r_len(), 0);
}

/// The default `r_len` reports an unknown length, which isn't empty.
#[test]
fn test_rextend_unknown_len() {
    struct Sink;
    impl RExtend<i32> for Sink {
        fn extend_from_vec(&self, _items: Vec<i32>) {}
    }
    assert_eq!(Sink.r_len(), -1);
    assert!(!Sink.r_is_empty());
}

#[test]
fn test_rextend_from_slice() {
    let v = TestExtendVec::new();
    let data = [1, 2, 3];
    v.extend_from_slice(&data);
    assert_eq!(v.get(), vec![1, 2, 3]);
}

// Tests for RFromIter
#[test]
fn test_rfromiter_vec() {
    let v: Vec<i32> = RFromIter::from_vec(vec![1, 2, 3]);
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn test_rfromiter_hashset() {
    use std::collections::HashSet;
    let set: HashSet<i32> = RFromIter::from_vec(vec![1, 2, 2, 3, 3, 3]);
    assert_eq!(set.len(), 3);
    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
}

#[test]
fn test_rfromiter_string() {
    let s: String = RFromIter::from_vec(vec!['h', 'e', 'l', 'l', 'o']);
    assert_eq!(s, "hello");
}

#[test]
fn test_rfromiter_empty() {
    let v: Vec<i32> = RFromIter::from_vec(vec![]);
    assert!(v.is_empty());
}

// Tests for RToVec
#[test]
fn test_rtovec_vec() {
    let v = vec![1, 2, 3];
    assert_eq!(v.r_to_vec(), vec![1, 2, 3]);
    assert_eq!(v.r_len(), 3);
    assert!(!v.r_is_empty());
}

#[test]
fn test_rtovec_empty() {
    let v: Vec<i32> = vec![];
    assert!(v.r_to_vec().is_empty());
    assert_eq!(v.r_len(), 0);
    assert!(v.r_is_empty());
}

#[test]
fn test_rtovec_hashset() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);

    let mut collected = set.r_to_vec();
    collected.sort();
    assert_eq!(collected, vec![1, 2, 3]);
    assert_eq!(set.r_len(), 3);
}

#[test]
fn test_rtovec_slice() {
    let arr = [10, 20, 30];
    assert_eq!(arr.r_to_vec(), vec![10, 20, 30]);
    assert_eq!(arr.r_len(), 3);
}

/// With every adapter trait in scope (`use super::*`), the std methods of
/// the types their blanket impls cover still resolve to std, with std's
/// result types.
#[test]
fn test_std_methods_resolve_with_adapters_in_scope() {
    use std::hash::DefaultHasher;

    let v = vec![1, 2, 3];
    let cloned: Vec<i32> = v.clone();
    assert_eq!(cloned, v);
    assert_eq!(String::default(), "");
    assert_eq!(1.cmp(&2), std::cmp::Ordering::Less);
    assert_eq!(1.0f64.partial_cmp(&2.0), Some(std::cmp::Ordering::Less));
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    let _: u64 = hasher.finish();
    assert_eq!(i32::from_str("7"), Ok(7));

    let arr = [10, 20, 30];
    let len: usize = arr.len();
    assert_eq!(len, 3);
    assert!(!arr.is_empty());
    let boxed: Box<[i32]> = vec![1, 2].into_boxed_slice();
    let boxed_len: usize = boxed.len();
    assert_eq!(boxed_len, 2);
    assert_eq!(boxed.to_vec(), vec![1, 2]);
}

// Tests for RMakeIter
struct TestCollection(Vec<i32>);

struct TestCollectionIter(RefCell<std::vec::IntoIter<i32>>);

impl RIterator for TestCollectionIter {
    type Item = i32;

    fn next(&self) -> Nullable<i32> {
        Nullable(self.0.borrow_mut().next())
    }

    fn size_hint(&self) -> (i64, Option<i64>) {
        let (lo, hi) = self.0.borrow().size_hint();
        (lo as i64, hi.map(|h| h as i64))
    }
}

impl RMakeIter<i32, TestCollectionIter> for TestCollection {
    fn make_iter(&self) -> TestCollectionIter {
        TestCollectionIter(RefCell::new(Clone::clone(&self.0).into_iter()))
    }
}

#[test]
fn test_rmakeiter_basic() {
    let coll = TestCollection(vec![1, 2, 3]);
    let iter = coll.make_iter();

    assert_eq!(iter.next().0, Some(1));
    assert_eq!(iter.next().0, Some(2));
    assert_eq!(iter.next().0, Some(3));
    assert_eq!(iter.next().0, None);
}

#[test]
fn test_rmakeiter_independent() {
    let coll = TestCollection(vec![1, 2, 3]);

    // Create two independent iterators
    let iter1 = coll.make_iter();
    let iter2 = coll.make_iter();

    // Consuming one doesn't affect the other
    assert_eq!(iter1.next().0, Some(1));
    assert_eq!(iter1.next().0, Some(2));

    assert_eq!(iter2.next().0, Some(1)); // iter2 starts fresh
    assert_eq!(iter2.size_hint(), (2, Some(2))); // 2 remaining in iter2
}
