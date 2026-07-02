//! Bytes adapter tests
use miniextendr_api::bytes_impl::{Bytes, BytesMut};
use miniextendr_api::miniextendr;

/// Test Bytes roundtrip through R raw vector.
/// @param data Raw vector to roundtrip.
#[miniextendr]
pub fn bytes_roundtrip(data: Bytes) -> Bytes {
    data
}

/// Test getting the length of a Bytes buffer.
/// @param data Raw vector to measure.
#[miniextendr]
pub fn bytes_len(data: Bytes) -> i32 {
    data.len() as i32
}

/// Test BytesMut roundtrip through R raw vector.
/// @param data Raw vector to roundtrip.
#[miniextendr]
pub fn bytes_mut_roundtrip(data: BytesMut) -> BytesMut {
    data
}

/// Test concatenating two Bytes buffers.
/// @param a First raw vector.
/// @param b Second raw vector.
#[miniextendr]
pub fn bytes_concat(a: Bytes, b: Bytes) -> Bytes {
    let mut out = BytesMut::with_capacity(a.len() + b.len());
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    out.freeze()
}

/// Test slicing a Bytes buffer by start and end indices.
/// @param data Raw vector to slice.
/// @param start Start index (0-based).
/// @param end End index (exclusive).
#[miniextendr]
pub fn bytes_slice(data: Bytes, start: i32, end: i32) -> Bytes {
    data.slice(start as usize..end as usize)
}

/// Test creating and roundtripping an empty Bytes buffer.
#[miniextendr]
pub fn bytes_empty() -> Bytes {
    Bytes::new()
}

/// Test that an empty Bytes buffer has length zero.
#[miniextendr]
pub fn bytes_empty_len() -> i32 {
    Bytes::new().len() as i32
}

/// Test roundtripping a large buffer (1000 bytes).
#[miniextendr]
pub fn bytes_large() -> Bytes {
    Bytes::from(vec![0xABu8; 1000])
}

/// Test roundtripping all 256 byte values (0x00 through 0xFF).
#[miniextendr]
pub fn bytes_all_values() -> Bytes {
    let data: Vec<u8> = (0..=255).collect();
    Bytes::from(data)
}

// region: RBuf / RBufMut adapter traits

use std::cell::{Cell, RefCell};

/// Cursor over a `Bytes` buffer implementing the `RBuf` adapter trait
/// (audit A7 — the trait had no implementor anywhere in the repo). Interior
/// mutability because the trait reads through `&self`.
struct ByteCursor {
    data: Bytes,
    pos: Cell<usize>,
}

impl miniextendr_api::bytes_impl::RBuf for ByteCursor {
    fn remaining(&self) -> i32 {
        (self.data.len() - self.pos.get()) as i32
    }

    fn get_u8(&self) -> Option<i32> {
        let p = self.pos.get();
        self.data.get(p).map(|b| {
            self.pos.set(p + 1);
            i32::from(*b)
        })
    }

    fn chunk(&self) -> Vec<u8> {
        self.data[self.pos.get()..].to_vec()
    }

    fn copy_to_vec(&self, len: i32) -> Vec<u8> {
        let p = self.pos.get();
        let n = usize::try_from(len).unwrap_or(0).min(self.data.len() - p);
        self.pos.set(p + n);
        self.data[p..p + n].to_vec()
    }

    fn advance(&self, cnt: i32) {
        let n = usize::try_from(cnt).unwrap_or(0);
        self.pos.set((self.pos.get() + n).min(self.data.len()));
    }
}

/// Read a raw vector through the `RBuf` trait: returns
/// `c(remaining, has_remaining, first get_u8, first get_i8-reinterpreted, drained count)`.
/// @param data Raw vector to read.
#[miniextendr]
pub fn rbuf_trait_read(data: Bytes) -> Vec<i32> {
    use miniextendr_api::bytes_impl::RBuf;

    let cur = ByteCursor {
        data,
        pos: Cell::new(0),
    };
    let remaining = RBuf::remaining(&cur);
    let has = i32::from(RBuf::has_remaining(&cur));
    let first_u8 = RBuf::get_u8(&cur).unwrap_or(-1);
    let first_i8 = RBuf::get_i8(&cur).unwrap_or(-999);
    let mut drained = 0;
    while RBuf::get_u8(&cur).is_some() {
        drained += 1;
    }
    vec![remaining, has, first_u8, first_i8, drained]
}

/// `chunk` / `advance` / `copy_to_vec` / default `to_vec` through the `RBuf`
/// trait. For input bytes `b1..bN`: peeks the full chunk, skips one byte,
/// copies two, then drains the rest.
/// @param data Raw vector to read.
#[miniextendr]
pub fn rbuf_trait_chunks(data: Bytes) -> Vec<i32> {
    use miniextendr_api::bytes_impl::RBuf;

    let cur = ByteCursor {
        data,
        pos: Cell::new(0),
    };
    let chunk_len = RBuf::chunk(&cur).len() as i32;
    RBuf::advance(&cur, 1);
    let copied = RBuf::copy_to_vec(&cur, 2);
    let rest = RBuf::to_vec(&cur);
    let mut out = vec![chunk_len, copied.len() as i32];
    out.extend(copied.iter().map(|b| i32::from(*b)));
    out.push(rest.len() as i32);
    out
}

/// Growable sink over `BytesMut` implementing the `RBufMut` adapter trait.
struct ByteSink {
    data: RefCell<BytesMut>,
}

impl miniextendr_api::bytes_impl::RBufMut for ByteSink {
    fn remaining_mut(&self) -> i32 {
        let buf = self.data.borrow();
        (buf.capacity() - buf.len()) as i32
    }

    fn put_u8(&self, val: i32) {
        let byte = u8::try_from(val & 0xFF).expect("masked to u8 range");
        self.data.borrow_mut().extend_from_slice(&[byte]);
    }

    fn put_slice(&self, src: Vec<u8>) {
        self.data.borrow_mut().extend_from_slice(&src);
    }

    fn len(&self) -> i32 {
        self.data.borrow().len() as i32
    }

    fn clear(&self) {
        self.data.borrow_mut().clear();
    }
}

/// Build a raw vector through the `RBufMut` trait: writes `vals` byte-by-byte
/// via `put_u8`, appends `extra` via `put_slice`, then two filler bytes via
/// the default-method `put_bytes`.
/// @param vals Integer vector of byte values (0-255) written via put_u8.
/// @param extra Raw vector appended via put_slice.
#[miniextendr]
pub fn rbufmut_trait_build(vals: Vec<i32>, extra: Vec<u8>) -> Bytes {
    use miniextendr_api::bytes_impl::RBufMut;

    let sink = ByteSink {
        data: RefCell::new(BytesMut::new()),
    };
    for v in vals {
        RBufMut::put_u8(&sink, v);
    }
    RBufMut::put_slice(&sink, extra);
    RBufMut::put_bytes(&sink, 0x2A, 2);
    sink.data.into_inner().freeze()
}

/// Exercise `RBufMut::len` / `is_empty` / `clear` through the trait.
#[miniextendr]
pub fn rbufmut_trait_clear() -> Vec<i32> {
    use miniextendr_api::bytes_impl::RBufMut;

    let sink = ByteSink {
        data: RefCell::new(BytesMut::new()),
    };
    let empty_before = i32::from(RBufMut::is_empty(&sink));
    RBufMut::put_slice(&sink, vec![1, 2, 3]);
    let len_filled = RBufMut::len(&sink);
    RBufMut::clear(&sink);
    let len_cleared = RBufMut::len(&sink);
    vec![empty_before, len_filled, len_cleared]
}

// endregion
