#![cfg(feature = "connections")]

use miniextendr_api::connection::IteratorReader;
use std::cell::Cell;
use std::io::Read;

#[test]
fn empty_reads_do_not_draw_and_reads_preserve_sample_boundaries() {
    let draws = Cell::new(0);
    let samples = [1.5_f64, -2.5]
        .into_iter()
        .inspect(|_| draws.set(draws.get() + 1));
    let mut reader = IteratorReader::new(samples.flat_map(f64::to_le_bytes));
    assert_eq!(reader.read(&mut []).unwrap(), 0);
    assert_eq!(draws.get(), 0);
    let mut first = [0; 3];
    reader.read_exact(&mut first).unwrap();
    assert_eq!(draws.get(), 1);
    let mut second = [0; 6];
    reader.read_exact(&mut second).unwrap();
    assert_eq!(draws.get(), 2);
    let mut rest = Vec::new();
    reader.read_to_end(&mut rest).unwrap();
    let actual = [first.as_slice(), second.as_slice(), rest.as_slice()].concat();
    let expected: Vec<_> = [1.5_f64, -2.5]
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(reader.read(&mut [0; 1]).unwrap(), 0);
}

#[test]
fn first_none_is_permanent_eof_even_for_a_nonfused_source() {
    let polls = Cell::new(0);
    let mut reader = IteratorReader::new(std::iter::from_fn(|| {
        polls.set(polls.get() + 1);
        match polls.get() {
            1 => Some(42),
            2 => None,
            _ => panic!("polled after EOF"),
        }
    }));
    let mut buf = [0; 4];
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(buf[0], 42);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(polls.get(), 2);
}

#[cfg(feature = "rand_distr")]
#[test]
fn distribution_stream_matches_direct_sampling_across_small_reads() {
    use miniextendr_api::rand::{SeedableRng, rngs::StdRng};
    use miniextendr_api::rand_distr::{Distribution, Normal};

    let distribution = Normal::new(0.0_f64, 1.0).unwrap();
    let expected: Vec<_> = distribution
        .sample_iter(StdRng::seed_from_u64(73))
        .take(5)
        .flat_map(f64::to_le_bytes)
        .collect();
    let mut reader = IteratorReader::new(
        distribution
            .sample_iter(StdRng::seed_from_u64(73))
            .take(5)
            .flat_map(f64::to_le_bytes),
    );
    let mut actual = Vec::new();
    let mut buf = [0; 3];
    loop {
        let n = reader.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        actual.extend_from_slice(&buf[..n]);
    }
    assert_eq!(actual, expected);
}
