use super::*;

#[test]
fn tinyvec_can_be_created() {
    let _tv: TinyVec<[i32; 4]> = TinyVec::new();
    let _av: ArrayVec<[f64; 8]> = ArrayVec::new();
}

#[test]
fn tinyvec_from_slice() {
    let data = [1i32, 2, 3];
    let mut tv: TinyVec<[i32; 4]> = TinyVec::new();
    tv.extend_from_slice(&data);
    assert_eq!(tv.len(), 3);
    assert!(tv.is_inline()); // Should be stored inline since len <= 4
}

#[test]
fn arrayvec_capacity_check() {
    let mut av: ArrayVec<[i32; 3]> = ArrayVec::new();
    av.push(1);
    av.push(2);
    av.push(3);
    assert_eq!(av.len(), 3);
    // Cannot push more - would panic
}

/// Two failing elements (i32 values that don't fit in i8) are both
/// reported, by 1-based position, in one batched error, not just the first.
#[test]
fn coerce_slice_to_vec_batches_all_failing_indices() {
    let slice = [1i32, 999, 2, 999];
    let err = coerce_slice_to_vec::<i32, i8>(&slice).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.ends_with("value out of range (elements 2, 4)"),
        "got: {msg}"
    );
    // The valid elements (1 and 3) are not listed, nor the container.
    assert!(!msg.contains("elements 1"), "got: {msg}");
    assert!(!msg.contains("TinyVec"), "got: {msg}");
}

/// The all-valid happy path succeeds (no false batching).
#[test]
fn coerce_slice_to_vec_happy_path_ok() {
    let slice = [1i32, 2, 3];
    let out = coerce_slice_to_vec::<i32, i8>(&slice).unwrap();
    assert_eq!(
        out.into_iter().map(|c| *c).collect::<Vec<_>>(),
        vec![1i8, 2, 3]
    );
}

/// More than 10 failures are capped: the first 10 positions are listed and
/// the remainder is summarized as "and N more".
#[test]
fn coerce_slice_to_vec_batch_caps_at_ten_and_summarizes_rest() {
    let slice = [999i32; 15];
    let err = coerce_slice_to_vec::<i32, i8>(&slice).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.ends_with("value out of range (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 5 more"),
        "got: {msg}"
    );
}
