mod r_test_utils;

#[cfg(feature = "bytes")]
use bytes::{Bytes, BytesMut};
#[cfg(feature = "bytes")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "bytes")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "bytes")]
#[test]
fn bytes_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let data = vec![10u8, 20, 30, 40, 50];
        let bytes = Bytes::from(data.clone());

        let sexp = bytes.clone().into_sexp();
        let back: Bytes = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 5);
        assert_eq!(&back[..], &data[..]);
    });
}

#[cfg(feature = "bytes")]
#[test]
fn bytesmut_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let data = vec![1u8, 2, 3, 4];
        let bytes = BytesMut::from(data.as_slice());

        let sexp = bytes.clone().into_sexp();
        let back: BytesMut = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 4);
        assert_eq!(&back[..], &data[..]);
    });
}

#[cfg(feature = "bytes")]
#[test]
fn bytes_empty() {
    r_test_utils::with_r_thread(|| {
        let bytes = Bytes::new();
        let sexp = bytes.into_sexp();
        let back: Bytes = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back.len(), 0);
    });
}

#[cfg(feature = "bytes")]
#[test]
fn bytes_from_raw_vector() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{RAW, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, 3));
            let ptr = RAW(sexp);
            *ptr.add(0) = 100;
            *ptr.add(1) = 200;
            *ptr.add(2) = 255;

            let bytes: Bytes = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(bytes.len(), 3);
            assert_eq!(bytes[0], 100);
            assert_eq!(bytes[1], 200);
            assert_eq!(bytes[2], 255);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "bytes")]
#[test]
fn bytesmut_from_raw_vector() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{RAW, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, 4));
            let ptr = RAW(sexp);
            *ptr.add(0) = 10;
            *ptr.add(1) = 20;
            *ptr.add(2) = 30;
            *ptr.add(3) = 40;

            let mut bytes: BytesMut = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(bytes.len(), 4);
            assert_eq!(bytes[0], 10);

            // Verify it's mutable
            bytes[0] = 50;
            assert_eq!(bytes[0], 50);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "bytes")]
#[test]
fn option_bytes_some() {
    r_test_utils::with_r_thread(|| {
        let bytes = Bytes::from(vec![1u8, 2, 3]);
        let opt = Some(bytes);

        let sexp = opt.clone().into_sexp();
        let back: Vec<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, vec![1, 2, 3]);
    });
}

#[cfg(feature = "bytes")]
#[test]
fn option_bytes_none() {
    r_test_utils::with_r_thread(|| {
        let opt: Option<Bytes> = None;
        let sexp = opt.into_sexp();

        use miniextendr_api::ffi::{R_NilValue, SEXPTYPE, SexpExt};
        assert_eq!(sexp, unsafe { R_NilValue });
        assert_eq!(sexp.type_of(), SEXPTYPE::NILSXP);
    });
}
