use super::*;

#[test]
fn growth_replacement_and_panics_keep_exposed_allocations_alive() {
    let mut values = AltrepBuffer::new(vec![1_i32, 2]);
    let old = values.dataptr(true).unwrap();
    values.edit(|v| {
        v[0] = 3;
        v.resize(4096, 9);
    });
    let current = values.dataptr(true).unwrap();
    assert_ne!(old, current);
    unsafe {
        assert_eq!(*old, 1);
        assert_eq!(*current, 3);
    }
    assert_eq!(values.len(), 4096);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        values.edit(|v| {
            v.clear();
            panic!("interrupted edit");
        });
    }));
    assert!(result.is_err());
    unsafe {
        assert_eq!(*old, 1);
        assert_eq!(*current, 3);
    }
    assert!(values.is_empty());
}

#[test]
fn unexposed_edits_reuse_capacity_and_clones_detach() {
    let mut values = AltrepBuffer::new(Vec::<i32>::with_capacity(8));
    let before = values.values.as_ptr();
    values.edit(|v| v.extend([1, 2, 3]));
    assert_eq!(values.values.as_ptr(), before);
    let exposed = values.dataptr(true).unwrap();
    let mut copy = values.clone();
    copy.edit(|v| v[0] = 7);
    unsafe {
        assert_eq!(*exposed, 1);
    }
    assert_eq!(copy.as_slice(), [7, 2, 3]);
    assert!(copy.retired.is_empty());
}
