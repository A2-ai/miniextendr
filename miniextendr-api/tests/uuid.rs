mod r_test_utils;

#[cfg(feature = "uuid")]
use miniextendr_api::Uuid;
#[cfg(feature = "uuid")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "uuid")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "uuid")]
#[test]
fn uuid_scalar_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let uuid = Uuid::new_v4();
        let sexp = uuid.into_sexp();
        let back: Uuid = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(uuid, back);
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_from_string() {
    r_test_utils::with_r_thread(|| {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let sexp = uuid_str.to_string().into_sexp();
        let uuid: Uuid = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(uuid.to_string(), uuid_str);
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_option_none() {
    r_test_utils::with_r_thread(|| {
        let opt: Option<String> = None;
        let sexp = opt.into_sexp();
        let back: Option<Uuid> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_none());
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_option_some() {
    r_test_utils::with_r_thread(|| {
        let uuid = Uuid::new_v4();
        let opt = Some(uuid);
        let sexp = opt.into_sexp();
        let back: Option<Uuid> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back, Some(uuid));
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let uuids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let sexp = uuids.clone().into_sexp();
        let back: Vec<Uuid> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(uuids, back);
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_option_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let uuids = vec![Some(Uuid::new_v4()), None, Some(Uuid::new_v4())];
        let sexp = uuids.clone().into_sexp();
        let back: Vec<Option<Uuid>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(uuids, back);
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_invalid_parse_error() {
    r_test_utils::with_r_thread(|| {
        let sexp = "not-a-uuid".to_string().into_sexp();
        let result: Result<Uuid, _> = TryFromSexp::try_from_sexp(sexp);
        assert!(result.is_err());
    });
}
