use super::*;

#[test]
fn test_parallel_collect() {
    use rayon::prelude::*;

    let result: Vec<i32> = (0..100).into_par_iter().collect();
    assert_eq!(result.len(), 100);
    assert_eq!(result[0], 0);
    assert_eq!(result[99], 99);
}

#[test]
fn test_parallel_map() {
    use rayon::prelude::*;

    let data = vec![1.0, 2.0, 3.0, 4.0];
    let doubled: Vec<f64> = data.par_iter().map(|&x| x * 2.0).collect();

    assert_eq!(doubled.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
}

// Test implementation for RParallelIterator
struct TestParData {
    values: Vec<f64>,
}

impl TestParData {
    fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

impl RParallelIterator for TestParData {
    type Item = f64;

    fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = Self::Item> + '_ {
        self.values.par_iter().copied()
    }

    fn par_len(&self) -> i32 {
        self.values.len() as i32
    }
}

#[test]
fn test_rparalleliterator_sum() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(data.par_sum(), 15.0);
}

#[test]
fn test_rparalleliterator_mean() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(data.par_mean(), 3.0);
}

#[test]
fn test_rparalleliterator_mean_empty() {
    let data = TestParData::new(vec![]);
    assert!(data.par_mean().is_nan());
}

#[test]
fn test_rparalleliterator_min_max() {
    let data = TestParData::new(vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0]);
    assert_eq!(data.par_min_f64(), 1.0);
    assert_eq!(data.par_max_f64(), 9.0);
}

#[test]
fn test_rparalleliterator_count() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(data.par_count(), 5);
    assert_eq!(data.par_len(), 5);
}

#[test]
fn test_rparalleliterator_product() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(data.par_product(), 24.0);
}

#[test]
fn test_rparalleliterator_predicates() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    assert!(data.par_any_gt(4.0)); // 5.0 > 4.0
    assert!(!data.par_all_gt(4.0)); // 1.0, 2.0, 3.0, 4.0 are not > 4.0
    assert!(data.par_any_lt(2.0)); // 1.0 < 2.0
    assert!(!data.par_all_lt(2.0)); // 2.0, 3.0, 4.0, 5.0 are not < 2.0
}

#[test]
fn test_rparalleliterator_count_predicates() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    assert_eq!(data.par_count_gt(3.0), 2); // 4.0, 5.0
    assert_eq!(data.par_count_lt(3.0), 2); // 1.0, 2.0
    assert_eq!(data.par_count_eq(3.0, 0.0), 1); // exactly 3.0
}

#[test]
fn test_rparalleliterator_variance_stddev() {
    let data = TestParData::new(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let var = data.par_variance();
    let std = data.par_std_dev();

    // Variance should be approximately 4.571 (sample variance)
    assert!((var - 4.571428571).abs() < 0.001);
    assert!((std - var.sqrt()).abs() < 0.001);
}

#[test]
fn test_rparalleliterator_filter() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let gt3: Vec<f64> = data.par_filter_gt(3.0);
    assert_eq!(gt3.len(), 2);
    assert!(gt3.contains(&4.0));
    assert!(gt3.contains(&5.0));

    let lt3: Vec<f64> = data.par_filter_lt(3.0);
    assert_eq!(lt3.len(), 2);
    assert!(lt3.contains(&1.0));
    assert!(lt3.contains(&2.0));
}

#[test]
fn test_rparalleliterator_transform() {
    let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0]);

    let scaled = data.par_scale(2.0);
    assert_eq!(scaled, vec![2.0, 4.0, 6.0, 8.0]);

    let offset = data.par_offset(10.0);
    assert_eq!(offset, vec![11.0, 12.0, 13.0, 14.0]);

    let clamped = data.par_clamp(2.0, 3.0);
    assert_eq!(clamped, vec![2.0, 2.0, 3.0, 3.0]);
}

#[test]
fn test_rparalleliterator_math() {
    let data = TestParData::new(vec![1.0, 4.0, 9.0, 16.0]);

    let sqrt = data.par_sqrt();
    assert_eq!(sqrt, vec![1.0, 2.0, 3.0, 4.0]);

    let pow2 = data.par_pow(0.5);
    assert_eq!(pow2, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_rparalleliterator_abs() {
    let data = TestParData::new(vec![-1.0, 2.0, -3.0, 4.0]);
    let abs = data.par_abs();
    assert_eq!(abs, vec![1.0, 2.0, 3.0, 4.0]);
}

// Test implementation for RParallelExtend
use std::sync::Mutex;

struct TestParBuffer {
    data: Mutex<Vec<f64>>,
}

impl TestParBuffer {
    fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
        }
    }

    fn get_data(&self) -> Vec<f64> {
        self.data.lock().unwrap().clone()
    }
}

impl RParallelExtend<f64> for TestParBuffer {
    fn par_extend(&self, items: Vec<f64>) {
        let mut guard = self.data.lock().unwrap();
        guard.extend(items);
    }

    fn par_len(&self) -> i32 {
        self.data.lock().unwrap().len() as i32
    }

    fn par_clear(&self) {
        self.data.lock().unwrap().clear();
    }

    fn par_reserve(&self, additional: i32) {
        self.data.lock().unwrap().reserve(additional as usize);
    }
}

#[test]
fn test_rparallelextend_basic() {
    let buffer = TestParBuffer::new();

    buffer.par_extend(vec![1.0, 2.0, 3.0]);
    assert_eq!(buffer.par_len(), 3);
    assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0]);

    buffer.par_extend(vec![4.0, 5.0]);
    assert_eq!(buffer.par_len(), 5);
    assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_rparallelextend_from_slice() {
    let buffer = TestParBuffer::new();
    let slice = [1.0, 2.0, 3.0];

    buffer.par_extend_from_slice(&slice);
    assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_rparallelextend_clear() {
    let buffer = TestParBuffer::new();

    buffer.par_extend(vec![1.0, 2.0, 3.0]);
    assert!(!buffer.par_is_empty());

    buffer.par_clear();
    assert!(buffer.par_is_empty());
    assert_eq!(buffer.par_len(), 0);
}

#[test]
fn test_rparallelextend_reserve() {
    let buffer = TestParBuffer::new();
    buffer.par_reserve(100);
    // Can't easily verify capacity, but at least it shouldn't panic
    buffer.par_extend(vec![1.0; 100]);
    assert_eq!(buffer.par_len(), 100);
}

#[test]
fn test_column_writer_parallel() {
    let n = 10_000;
    let mut col: Vec<i32> = vec![0; n];
    {
        let w = unsafe { ColumnWriter::new(&mut col) };
        (0..n)
            .into_par_iter()
            .for_each(|i| unsafe { w.write(i, i as i32) });
    }
    for (i, &v) in col.iter().enumerate() {
        assert_eq!(v, i as i32);
    }
}

#[test]
fn test_column_writer_string() {
    let n = 100;
    let mut col: Vec<String> = vec![String::new(); n];
    {
        let w = unsafe { ColumnWriter::new(&mut col) };
        (0..n)
            .into_par_iter()
            .for_each(|i| unsafe { w.write(i, format!("s_{i}")) });
    }
    assert_eq!(col[0], "s_0");
    assert_eq!(col[99], "s_99");
}

#[test]
fn test_column_writer_option() {
    let n = 100;
    let mut col: Vec<Option<f64>> = vec![None; n];
    {
        let w = unsafe { ColumnWriter::new(&mut col) };
        (0..n).into_par_iter().for_each(|i| unsafe {
            w.write(i, if i % 2 == 0 { Some(i as f64) } else { None });
        });
    }
    assert_eq!(col[0], Some(0.0));
    assert_eq!(col[1], None);
}

// region: ParCollectR / ParCollectRIndexed tests

#[test]
fn test_par_collect_r_trait_exists() {
    // Verify that ParCollectR is available on indexed parallel iterators
    // (compile-time test: if this compiles, the blanket impl works)
    fn _assert_has_collect_r<T: ParCollectR>() {}
    fn _assert_par_iter_has_it() {
        _assert_has_collect_r::<rayon::iter::Map<rayon::slice::Iter<'_, f64>, fn(&f64) -> f64>>();
    }
}

#[test]
fn test_par_collect_vec_roundtrip() {
    // Verify standard rayon traits work for Vec<T> (already provided by rayon)
    let data = vec![1.0_f64, 2.0, 3.0, 4.0];
    let doubled: Vec<f64> = data.par_iter().map(|&x| x * 2.0).collect();
    assert_eq!(doubled, vec![2.0, 4.0, 6.0, 8.0]);

    // IntoParallelIterator for Vec<T>
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<i32> = data.into_par_iter().map(|x| x * x).collect();
    assert_eq!(result, vec![1, 4, 9, 16, 25]);
}

#[test]
fn test_par_iter_ref_and_mut() {
    // IntoParallelRefIterator for &[T]
    let data = vec![1.0, 4.0, 9.0, 16.0];
    let sqrts: Vec<f64> = data.par_iter().map(|&x: &f64| x.sqrt()).collect();
    assert_eq!(sqrts, vec![1.0, 2.0, 3.0, 4.0]);

    // IntoParallelRefMutIterator for &mut [T]
    let mut data = vec![1.0, 2.0, 3.0, 4.0];
    data.par_iter_mut().for_each(|x| *x *= 2.0);
    assert_eq!(data, vec![2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_from_par_iter_vec() {
    // FromParallelIterator<T> for Vec<T> (rayon-provided)
    let result: Vec<f64> = (0..100)
        .into_par_iter()
        .map(|i| (i as f64).sqrt())
        .collect();
    assert_eq!(result.len(), 100);
    assert!((result[0] - 0.0).abs() < 1e-10);
    assert!((result[1] - 1.0).abs() < 1e-10);
    assert!((result[4] - 2.0).abs() < 1e-10);
}
// endregion
