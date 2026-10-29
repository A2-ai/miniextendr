use super::*;

#[test]
fn key_generational_safety() {
    let mut gens: Vec<u32> = vec![0; 4];
    let mut free: Vec<usize> = Vec::new();

    let k1 = ProtectKey {
        slot: 0,
        generation: gens[0],
    };
    assert_eq!(gens[0], k1.generation);

    gens[0] = gens[0].wrapping_add(1);
    free.push(0);
    assert_ne!(gens[0], k1.generation);

    let slot = free.pop().unwrap();
    let k2 = ProtectKey {
        slot: u32::try_from(slot).unwrap(),
        generation: gens[slot],
    };
    assert_eq!(gens[0], k2.generation);
    assert_ne!(k1.generation, k2.generation);
}

#[test]
fn key_size() {
    assert_eq!(std::mem::size_of::<ProtectKey>(), 8);
}
