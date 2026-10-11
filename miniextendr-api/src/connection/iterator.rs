//! Byte iterators as ordinary Rust readers and R connection sources.

use std::io::{self, Read};
use std::iter::Fuse;

/// A byte iterator exposed as a sequential `std::io::Read` source.
///
/// The concrete iterator is retained. An empty read never advances it; the first
/// `None` becomes permanent EOF. No length, collection, or seek capability is
/// required. A source used in an R connection must own its data (`'static`).
///
/// Encode structured items before constructing the reader. For example,
/// `samples.flat_map(f64::to_le_bytes)` emits doubles for R's
/// `readBin(con, "double", n, size = 8L, endian = "little")`. The `FlatMap`
/// retains the rest of a sample when a read ends partway through its bytes.
///
/// ```
/// use std::io::Read;
/// use miniextendr_api::connection::IteratorReader;
///
/// let mut reader = IteratorReader::new([1.5_f64, 2.5].into_iter().flat_map(f64::to_le_bytes));
/// let mut prefix = [0; 3];
/// reader.read_exact(&mut prefix).unwrap();
/// assert_eq!(prefix, 1.5_f64.to_le_bytes()[..3]);
/// ```
pub struct IteratorReader<I: Iterator<Item = u8>> {
    iter: Fuse<I>,
}

impl<I: Iterator<Item = u8>> IteratorReader<I> {
    /// Own a byte iterator without advancing it.
    pub fn new(iter: I) -> Self {
        Self { iter: iter.fuse() }
    }
}

impl<I: Iterator<Item = u8>> Read for IteratorReader<I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for (n, slot) in buf.iter_mut().enumerate() {
            let Some(byte) = self.iter.next() else {
                return Ok(n);
            };
            *slot = byte;
        }
        Ok(buf.len())
    }
}
