use super::*;

#[test]
fn regions_cross_the_ring_boundary_without_rearranging_storage() {
    let mut values = VecDeque::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
    values.rotate_left(3);
    let split = values.as_slices().0.len();
    assert!(split > 0 && split < values.len());
    let expected = values.iter().copied().collect::<Vec<_>>();
    for start in 0..=values.len() + 1 {
        for requested in [0, 1, 3, 8, usize::MAX] {
            let mut out = [-1; 4];
            let n = copy_region(&values, start, requested, &mut out);
            let want = expected
                .iter()
                .skip(start)
                .take(requested)
                .take(4)
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(&out[..n], want.as_slice());
            assert!(out[n..].iter().all(|&x| x == -1));
            assert_eq!(values.as_slices().0.len(), split);
        }
    }
    assert_eq!(copy_region(&values, usize::MAX, usize::MAX, &mut [0; 4]), 0);
    assert_eq!(copy_region(&values, 0, 4, &mut []), 0);
    assert_eq!(copy_region(&VecDeque::<i32>::new(), 0, 4, &mut [0; 4]), 0);
}

#[test]
fn pointer_access_makes_the_existing_owner_contiguous() {
    let mut values = VecDeque::from(vec![1, 2, 3, 4]);
    values.rotate_left(1);
    assert!(AltrepDataptr::dataptr_or_null(&values).is_none());
    let pointer = AltrepDataptr::dataptr(&mut values, true).unwrap();
    unsafe { *pointer = 90 };
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [90, 3, 4, 1]);
    assert_eq!(values.as_slices().1.len(), 0);
    assert_eq!(
        AltrepDataptr::dataptr_or_null(&values),
        Some(pointer.cast_const())
    );
}
