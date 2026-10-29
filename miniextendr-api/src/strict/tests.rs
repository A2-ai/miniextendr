use super::*;

#[test]
fn i64_in_range_has_an_r_integer() {
    for value in [0, 42, -1, i64::from(i32::MAX)] {
        assert_eq!(value.to_r_int(), i32::try_from(value).ok());
    }
}

#[test]
fn i64_out_of_range_panics() {
    let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i64::MAX));
    assert!(result.is_err(), "should panic for i64::MAX");

    let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i32::MIN as i64));
    assert!(result.is_err(), "should panic for i32::MIN (NA_integer_)");

    let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i32::MAX as i64 + 1));
    assert!(result.is_err(), "should panic for i32::MAX + 1");
}

#[test]
fn u64_in_range_has_an_r_integer() {
    assert_eq!(0_u64.to_r_int(), Some(0));
    assert_eq!(u64::try_from(i32::MAX).unwrap().to_r_int(), Some(i32::MAX));
}

#[test]
fn borrowed_and_boxed_rows_report_every_invalid_position() {
    let bad = [i64::MAX, 1, i64::MIN];
    let message = panic_message(
        std::panic::catch_unwind(|| checked_slice_into_sexp("&[i64]", &bad)).unwrap_err(),
    );
    assert!(message.contains("&[i64]"), "{message}");
    assert!(message.contains("element 1"), "{message}");
    assert!(message.contains("element 3"), "{message}");
    for message in [
        panic_message(
            std::panic::catch_unwind(|| {
                checked_rows_into_sexp::<i64, _>(
                    "Vec<Option<&[i64]>>",
                    [Some(bad.as_slice()), None, Some(bad.as_slice())].into_iter(),
                )
            })
            .unwrap_err(),
        ),
        panic_message(
            std::panic::catch_unwind(|| {
                checked_rows_into_sexp::<i64, _>(
                    "Vec<Box<[i64]>>",
                    [
                        Some(Box::<[i64]>::from(bad)),
                        None,
                        Some(Box::<[i64]>::from(bad)),
                    ]
                    .into_iter(),
                )
            })
            .unwrap_err(),
        ),
        panic_message(
            std::panic::catch_unwind(|| {
                checked_rows_into_sexp::<i64, _>(
                    "Vec<[i64; 3]>",
                    [Some(bad), None, Some(bad)].into_iter(),
                )
            })
            .unwrap_err(),
        ),
    ] {
        assert!(message.contains("inner position 1"), "{message}");
        assert!(message.contains("inner position 3"), "{message}");
        assert!(message.contains("elements 1, 3"), "{message}");
    }
}

#[test]
fn u64_out_of_range_panics() {
    let result = std::panic::catch_unwind(|| checked_into_sexp_u64(i32::MAX as u64 + 1));
    assert!(result.is_err());
}

/// Downcast a `panic!` payload (produced via format args, so always a
/// `String`) into an owned `String` for message assertions.
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    *payload
        .downcast::<String>()
        .expect("panic payload should be a String")
}

#[test]
fn vec_i64_batches_multiple_out_of_range_indices() {
    let result =
        std::panic::catch_unwind(|| checked_vec_i64_into_sexp(vec![1, i64::MAX, 2, i64::MIN]));
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<i64>: i64 value"),
        "{msg}"
    );
    assert!(msg.contains("(element 2)"), "{msg}");
    assert!(msg.contains("(element 4)"), "{msg}");
    assert!(!msg.contains("index"), "{msg}");
    assert!(
        !msg.contains("and "),
        "should not summarize under the cap: {msg}"
    );
    assert!(
        msg.ends_with("use a non-strict function to allow lossy f64 widening"),
        "{msg}"
    );
}

#[test]
fn vec_i64_batches_caps_and_summarizes_remainder() {
    let vals: Vec<i64> = std::iter::repeat_n(i64::MAX, 15).collect();
    let result = std::panic::catch_unwind(|| checked_vec_i64_into_sexp(vals));
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(msg.contains("and 5 more"), "{msg}");
}

#[test]
fn vec_u64_batches_multiple_out_of_range_indices() {
    let bad = i32::MAX as u64 + 1;
    let result = std::panic::catch_unwind(|| checked_vec_u64_into_sexp(vec![0, bad, 1, bad]));
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<u64>: u64 value"),
        "{msg}"
    );
    assert!(msg.contains("(elements 2, 4)"), "{msg}");
    assert!(!msg.contains("index"), "{msg}");
}

#[test]
fn vec_option_i64_batches_multiple_out_of_range_indices() {
    let result = std::panic::catch_unwind(|| {
        checked_vec_option_i64_into_sexp(vec![Some(1), Some(i64::MAX), None, Some(i64::MIN)])
    });
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<Option<i64>>: i64 value"),
        "{msg}"
    );
    assert!(msg.contains("(element 2)"), "{msg}");
    assert!(msg.contains("(element 4)"), "{msg}");
    assert!(!msg.contains("index"), "{msg}");
}

#[test]
fn vec_option_u64_batches_multiple_out_of_range_indices() {
    let bad = i32::MAX as u64 + 1;
    let result = std::panic::catch_unwind(|| {
        checked_vec_option_u64_into_sexp(vec![Some(0), Some(bad), None, Some(bad)])
    });
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<Option<u64>>: u64 value"),
        "{msg}"
    );
    assert!(msg.contains("(elements 2, 4)"), "{msg}");
    assert!(!msg.contains("index"), "{msg}");
}

#[test]
fn pointer_sized_helpers_name_their_own_type() {
    let big = 3_000_000_000usize;
    let msg = panic_message(
        std::panic::catch_unwind(|| checked_vec_usize_into_sexp(vec![1, big]))
            .expect_err("should panic for out-of-range elements"),
    );
    assert!(
        msg.starts_with("strict conversion failed for Vec<usize>: usize value 3000000000"),
        "{msg}"
    );
    let msg = panic_message(
        std::panic::catch_unwind(|| checked_into_sexp_usize(big))
            .expect_err("should panic for an out-of-range value"),
    );
    assert!(
        msg.starts_with("strict conversion failed: usize value 3000000000 exceeds"),
        "{msg}"
    );
    let msg = panic_message(
        std::panic::catch_unwind(|| {
            checked_vec_option_isize_into_sexp(vec![None, Some(isize::MIN)])
        })
        .expect_err("should panic for out-of-range elements"),
    );
    assert!(
        msg.starts_with("strict conversion failed for Vec<Option<isize>>: isize value"),
        "{msg}"
    );
    assert!(msg.contains("(element 2)"), "{msg}");
}

#[test]
fn vec_vec_usize_batches_every_row_before_panicking() {
    let big = 3_000_000_000usize;
    let result = std::panic::catch_unwind(|| {
        checked_vec_vec_usize_into_sexp(vec![vec![1, 2], vec![big, 3], vec![], vec![4, big]])
    });
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert_eq!(
        msg,
        "strict conversion failed for Vec<Vec<usize>>: usize value 3000000000 exceeds R \
             integer max (2147483647) at inner position 1 (element 2); usize value 3000000000 \
             exceeds R integer max (2147483647) at inner position 2 (element 4); use a \
             non-strict function to allow lossy f64 widening"
    );
}

#[test]
fn vec_vec_i64_names_its_type_and_groups_equal_reasons() {
    let result = std::panic::catch_unwind(|| {
        checked_vec_vec_i64_into_sexp(vec![vec![i64::MAX], vec![0], vec![i64::MAX]])
    });
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<Vec<i64>>: i64 value"),
        "{msg}"
    );
    // The same value at the same inner position shares one position list.
    assert!(msg.contains("at inner position 1 (elements 1, 3)"), "{msg}");
}

#[test]
fn vec_vec_isize_rejects_na_integer_sentinel() {
    let result =
        std::panic::catch_unwind(|| checked_vec_vec_isize_into_sexp(vec![vec![i32::MIN as isize]]));
    let msg = panic_message(result.expect_err("i32::MIN is NA_integer_"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<Vec<isize>>: isize value"),
        "{msg}"
    );
}

#[test]
fn vec_option_vec_u64_skips_none_rows() {
    let bad = i32::MAX as u64 + 1;
    let result = std::panic::catch_unwind(|| {
        checked_vec_option_vec_u64_into_sexp(vec![None, Some(vec![0, bad])])
    });
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(
        msg.starts_with("strict conversion failed for Vec<Option<Vec<u64>>>: u64 value"),
        "{msg}"
    );
    assert!(msg.contains("at inner position 2 (element 2)"), "{msg}");
}

#[test]
fn vec_vec_caps_listed_failures() {
    let rows: Vec<Vec<u64>> = std::iter::repeat_n(vec![u64::MAX], 15).collect();
    let result = std::panic::catch_unwind(|| checked_vec_vec_u64_into_sexp(rows));
    let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
    assert!(msg.contains("and 5 more"), "{msg}");
}
