use super::*;
use std::cell::RefCell;

// Test implementation for RBuf
struct TestBuf {
    data: RefCell<Bytes>,
}

impl TestBuf {
    fn new(data: &[u8]) -> Self {
        Self {
            data: RefCell::new(Bytes::copy_from_slice(data)),
        }
    }
}

impl RBuf for TestBuf {
    fn remaining(&self) -> i32 {
        self.data.borrow().remaining() as i32
    }

    fn get_u8(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.has_remaining() {
            Some(buf.get_u8() as i32)
        } else {
            None
        }
    }

    fn get_u16(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 2 {
            Some(buf.get_u16() as i32)
        } else {
            None
        }
    }

    fn get_u16_le(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 2 {
            Some(buf.get_u16_le() as i32)
        } else {
            None
        }
    }

    fn get_i32(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 4 {
            Some(buf.get_i32())
        } else {
            None
        }
    }

    fn get_i32_le(&self) -> Option<i32> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 4 {
            Some(buf.get_i32_le())
        } else {
            None
        }
    }

    fn get_f64(&self) -> Option<f64> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 8 {
            Some(buf.get_f64())
        } else {
            None
        }
    }

    fn get_f64_le(&self) -> Option<f64> {
        let mut buf = self.data.borrow_mut();
        if buf.remaining() >= 8 {
            Some(buf.get_f64_le())
        } else {
            None
        }
    }

    fn chunk(&self) -> Vec<u8> {
        self.data.borrow().chunk().to_vec()
    }

    fn copy_to_vec(&self, len: i32) -> Vec<u8> {
        let mut buf = self.data.borrow_mut();
        let len = (len as usize).min(buf.remaining());
        let mut dst = vec![0u8; len];
        buf.copy_to_slice(&mut dst);
        dst
    }

    fn advance(&self, cnt: i32) {
        let mut buf = self.data.borrow_mut();
        let cnt = (cnt as usize).min(buf.remaining());
        buf.advance(cnt);
    }
}

// Test implementation for RBufMut
struct TestBufMut {
    data: RefCell<BytesMut>,
}

impl TestBufMut {
    fn new() -> Self {
        Self {
            data: RefCell::new(BytesMut::new()),
        }
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            data: RefCell::new(BytesMut::with_capacity(cap)),
        }
    }

    fn freeze(&self) -> Bytes {
        self.data.borrow().clone().freeze()
    }
}

impl RBufMut for TestBufMut {
    fn remaining_mut(&self) -> i32 {
        i32::try_from(self.data.borrow().remaining_mut()).unwrap_or(i32::MAX)
    }

    fn put_u8(&self, val: i32) {
        self.data.borrow_mut().put_u8(val as u8);
    }

    fn put_u16(&self, val: i32) {
        self.data.borrow_mut().put_u16(val as u16);
    }

    fn put_u16_le(&self, val: i32) {
        self.data.borrow_mut().put_u16_le(val as u16);
    }

    fn put_i32(&self, val: i32) {
        self.data.borrow_mut().put_i32(val);
    }

    fn put_i32_le(&self, val: i32) {
        self.data.borrow_mut().put_i32_le(val);
    }

    fn put_f64(&self, val: f64) {
        self.data.borrow_mut().put_f64(val);
    }

    fn put_f64_le(&self, val: f64) {
        self.data.borrow_mut().put_f64_le(val);
    }

    fn put_slice(&self, src: Vec<u8>) {
        self.data.borrow_mut().put_slice(&src);
    }

    fn reserve(&self, additional: i32) {
        self.data.borrow_mut().reserve(additional as usize);
    }

    fn len(&self) -> i32 {
        self.data.borrow().len() as i32
    }

    fn clear(&self) {
        self.data.borrow_mut().clear();
    }
}

#[test]
fn test_rbuf_read_bytes() {
    let buf = TestBuf::new(&[1, 2, 3, 4, 5]);

    assert_eq!(buf.remaining(), 5);
    assert!(buf.has_remaining());

    assert_eq!(buf.get_u8(), Some(1));
    assert_eq!(buf.remaining(), 4);

    assert_eq!(buf.get_u8(), Some(2));
    assert_eq!(buf.get_u8(), Some(3));
    assert_eq!(buf.get_u8(), Some(4));
    assert_eq!(buf.get_u8(), Some(5));

    assert_eq!(buf.remaining(), 0);
    assert!(!buf.has_remaining());
    assert_eq!(buf.get_u8(), None);
}

#[test]
fn test_rbuf_read_u16() {
    // Big-endian: 0x0102 = 258
    let buf = TestBuf::new(&[0x01, 0x02]);
    assert_eq!(buf.get_u16(), Some(258));

    // Little-endian: 0x0201 = 513
    let buf = TestBuf::new(&[0x01, 0x02]);
    assert_eq!(buf.get_u16_le(), Some(513));
}

#[test]
fn test_rbuf_read_i32() {
    // Big-endian i32
    let buf = TestBuf::new(&[0x00, 0x00, 0x01, 0x00]); // 256
    assert_eq!(buf.get_i32(), Some(256));

    // Little-endian i32
    let buf = TestBuf::new(&[0x00, 0x01, 0x00, 0x00]); // 256
    assert_eq!(buf.get_i32_le(), Some(256));
}

#[test]
fn test_rbuf_read_f64() {
    let val = std::f64::consts::PI;

    // Big-endian f64
    let bytes: [u8; 8] = val.to_be_bytes();
    let buf = TestBuf::new(&bytes);
    assert!((buf.get_f64().unwrap() - val).abs() < 1e-10);

    // Little-endian f64
    let bytes: [u8; 8] = val.to_le_bytes();
    let buf = TestBuf::new(&bytes);
    assert!((buf.get_f64_le().unwrap() - val).abs() < 1e-10);
}

#[test]
fn test_rbuf_chunk_and_copy() {
    let buf = TestBuf::new(&[1, 2, 3, 4, 5]);

    // chunk returns view without advancing
    assert_eq!(buf.chunk(), vec![1, 2, 3, 4, 5]);
    assert_eq!(buf.remaining(), 5);

    // copy_to_vec reads and advances
    assert_eq!(buf.copy_to_vec(3), vec![1, 2, 3]);
    assert_eq!(buf.remaining(), 2);

    // to_vec reads remaining
    assert_eq!(buf.to_vec(), vec![4, 5]);
    assert_eq!(buf.remaining(), 0);
}

#[test]
fn test_rbuf_advance() {
    let buf = TestBuf::new(&[1, 2, 3, 4, 5]);

    buf.advance(2);
    assert_eq!(buf.remaining(), 3);
    assert_eq!(buf.get_u8(), Some(3));

    // Advance beyond remaining should cap
    buf.advance(100);
    assert_eq!(buf.remaining(), 0);
}

#[test]
fn test_rbufmut_write_bytes() {
    let buf = TestBufMut::new();

    buf.put_u8(1);
    buf.put_u8(2);
    buf.put_u8(3);

    assert_eq!(buf.len(), 3);
    assert_eq!(buf.freeze().as_ref(), &[1, 2, 3]);
}

#[test]
fn test_rbufmut_write_u16() {
    let buf = TestBufMut::new();

    // Big-endian
    buf.put_u16(258); // 0x0102
    assert_eq!(buf.freeze().as_ref(), &[0x01, 0x02]);

    let buf = TestBufMut::new();

    // Little-endian
    buf.put_u16_le(258); // 0x0102 -> stored as 0x02, 0x01
    assert_eq!(buf.freeze().as_ref(), &[0x02, 0x01]);
}

#[test]
fn test_rbufmut_write_i32() {
    let buf = TestBufMut::new();
    buf.put_i32(256);
    assert_eq!(buf.freeze().as_ref(), &[0x00, 0x00, 0x01, 0x00]);

    let buf = TestBufMut::new();
    buf.put_i32_le(256);
    assert_eq!(buf.freeze().as_ref(), &[0x00, 0x01, 0x00, 0x00]);
}

#[test]
fn test_rbufmut_write_f64() {
    let buf = TestBufMut::new();
    let val = std::f64::consts::PI;

    buf.put_f64(val);
    assert_eq!(buf.freeze().as_ref(), &val.to_be_bytes());

    let buf = TestBufMut::new();
    buf.put_f64_le(val);
    assert_eq!(buf.freeze().as_ref(), &val.to_le_bytes());
}

#[test]
fn test_rbufmut_put_slice() {
    let buf = TestBufMut::new();
    buf.put_slice(vec![1, 2, 3, 4, 5]);
    assert_eq!(buf.len(), 5);
    assert_eq!(buf.freeze().as_ref(), &[1, 2, 3, 4, 5]);
}

#[test]
fn test_rbufmut_put_bytes() {
    let buf = TestBufMut::new();
    buf.put_bytes(0xFF, 4);
    assert_eq!(buf.len(), 4);
    assert_eq!(buf.freeze().as_ref(), &[0xFF, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn test_rbufmut_reserve_and_clear() {
    let buf = TestBufMut::with_capacity(10);

    buf.put_slice(vec![1, 2, 3]);
    assert_eq!(buf.len(), 3);
    assert!(!buf.is_empty());

    buf.clear();
    assert_eq!(buf.len(), 0);
    assert!(buf.is_empty());

    // Note: BytesMut::remaining_mut() returns usize::MAX - len, so it's always
    // "infinite" for growable buffers. We just verify reserve doesn't panic.
    buf.reserve(100);
    // After reserve, capacity should be at least 100
    assert!(buf.data.borrow().capacity() >= 100);
}

#[test]
fn test_rbufmut_has_remaining() {
    let buf = TestBufMut::with_capacity(10);
    // BytesMut is growable: remaining_mut() returns usize::MAX - len, which
    // saturates to i32::MAX via try_from. has_remaining_mut() returns true.
    assert!(buf.remaining_mut() > 0);
    buf.put_u8(1);
    assert_eq!(buf.len(), 1);
}
