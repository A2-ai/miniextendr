use super::*;

// Note: These are compile-time tests to ensure the trait is implemented correctly.
// Runtime tests require R to be initialized and should go in the integration tests.

fn _assert_into_r_as<T, Target>()
where
    T: IntoRAs<Target>,
{
}

#[test]
fn test_trait_bounds() {
    // Scalars -> i32
    _assert_into_r_as::<i32, i32>();
    _assert_into_r_as::<i64, i32>();
    _assert_into_r_as::<f64, i32>();
    _assert_into_r_as::<bool, i32>();

    // Scalars -> f64
    _assert_into_r_as::<f64, f64>();
    _assert_into_r_as::<i32, f64>();
    _assert_into_r_as::<i64, f64>();
    _assert_into_r_as::<bool, f64>();

    // Scalars -> u8
    _assert_into_r_as::<u8, u8>();
    _assert_into_r_as::<i32, u8>();

    // Scalars -> RLogical
    _assert_into_r_as::<bool, RLogical>();
    _assert_into_r_as::<i32, RLogical>();

    // Scalars -> String
    _assert_into_r_as::<String, String>();
    _assert_into_r_as::<&str, String>();
    _assert_into_r_as::<f64, String>();
    _assert_into_r_as::<i32, String>();
    _assert_into_r_as::<bool, String>();

    // Vecs -> i32
    _assert_into_r_as::<Vec<i32>, i32>();
    _assert_into_r_as::<Vec<i64>, i32>();
    _assert_into_r_as::<Vec<f64>, i32>();

    // Vecs -> f64
    _assert_into_r_as::<Vec<f64>, f64>();
    _assert_into_r_as::<Vec<i32>, f64>();
    _assert_into_r_as::<Vec<i64>, f64>();

    // Vecs -> u8
    _assert_into_r_as::<Vec<u8>, u8>();
    _assert_into_r_as::<Vec<i32>, u8>();

    // Vecs -> String
    _assert_into_r_as::<Vec<String>, String>();
    _assert_into_r_as::<Vec<f64>, String>();
}

// Batched-error tests (#1097). These only exercise the failure path
// (`into_sexp()` is never reached), so they don't need `with_r_thread` —
// unlike the success-path integration tests in `conversion_coverage.rs`.

/// Failures at indices 3, 17, 42 must all be listed in one `Batched`
/// error's `Display` string, not just the first one.
#[test]
fn batched_vec_i64_to_i32_lists_all_failing_indices() {
    let mut v = vec![0i64; 43];
    for &i in &[3usize, 17, 42] {
        v[i] = i64::MAX;
    }
    let result = IntoRAs::<i32>::into_r_as(v);
    match result {
        Err(StorageCoerceError::Batched {
            container,
            listed,
            total,
        }) => {
            assert_eq!(container, "Vec<i64>");
            assert_eq!(total, 3);
            assert_eq!(listed.len(), 3);
            let display = StorageCoerceError::Batched {
                container,
                listed,
                total,
            }
            .to_string();
            // The 0-based `index` fields read 1-based, as R numbers the result.
            assert!(display.contains("(element 4)"), "{display}");
            assert!(display.contains("(element 18)"), "{display}");
            assert!(display.contains("(element 43)"), "{display}");
            assert!(!display.contains("index"), "{display}");
        }
        other => panic!("expected Batched, got {other:?}"),
    }
}

/// A 15-failure input lists only the first `BATCHED_ERROR_CAP` (10) and
/// summarizes the rest as `"and 5 more"`.
#[test]
fn batched_vec_i64_to_i32_caps_listed_and_summarizes_rest() {
    let v = vec![i64::MAX; 15];
    let result = IntoRAs::<i32>::into_r_as(v);
    match result {
        Err(e @ StorageCoerceError::Batched { .. }) => {
            if let StorageCoerceError::Batched { listed, total, .. } = &e {
                assert_eq!(*total, 15);
                assert_eq!(listed.len(), BATCHED_ERROR_CAP);
            }
            let display = e.to_string();
            assert!(display.contains("and 5 more"), "{display}");
        }
        other => panic!("expected Batched, got {other:?}"),
    }
}

/// The `&[T]` slice arms carry their own container label (`"&[i64]"`,
/// not `"Vec<i64>"`) and batch failures the same way as the `Vec` arms.
#[test]
fn batched_slice_i64_to_i32_lists_all_failing_indices() {
    let mut v = vec![0i64; 6];
    v[1] = i64::MAX;
    v[4] = i64::MIN;
    let result = IntoRAs::<i32>::into_r_as(v.as_slice());
    match result {
        Err(e @ StorageCoerceError::Batched { .. }) => {
            if let StorageCoerceError::Batched {
                container,
                listed,
                total,
            } = &e
            {
                assert_eq!(*container, "&[i64]");
                assert_eq!(*total, 2);
                assert_eq!(listed.len(), 2);
            }
            let display = e.to_string();
            assert!(display.contains("&[i64] conversion failed"), "{display}");
            assert!(display.contains("(element 2)"), "{display}");
            assert!(display.contains("(element 5)"), "{display}");
        }
        other => panic!("expected Batched, got {other:?}"),
    }
}

/// Scalar conversion failures must keep the plain (non-`Batched`) shape
/// and Display grammar — only vector conversions batch.
#[test]
fn scalar_failure_display_unchanged() {
    let result = IntoRAs::<i32>::into_r_as(i64::MAX);
    match result {
        Err(e @ StorageCoerceError::OutOfRange { .. }) => {
            assert_eq!(e.to_string(), "value out of range for i64 → i32");
        }
        other => panic!("expected scalar OutOfRange, got {other:?}"),
    }
}
