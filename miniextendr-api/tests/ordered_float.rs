mod r_test_utils;

#[cfg(feature = "ordered-float")]
use miniextendr_api::OrderedFloat;
#[cfg(feature = "ordered-float")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "ordered-float")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "ordered-float")]
#[test]
fn ordered_float_scalar_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let sexp = 1.25f64.into_sexp();
        let value: OrderedFloat<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(value.0, 1.25);

        let back = value.into_sexp();
        let back_value: f64 = TryFromSexp::try_from_sexp(back).unwrap();
        assert_eq!(back_value, 1.25);
    });
}

#[cfg(feature = "ordered-float")]
#[test]
fn ordered_float_vector_and_option() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![1.0f64, 2.5f64, -3.0f64].into_sexp();
        let values: Vec<OrderedFloat<f64>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[1].0, 2.5);

        let opt_sexp = Option::<f64>::None.into_sexp();
        let opt: Option<OrderedFloat<f64>> = TryFromSexp::try_from_sexp(opt_sexp).unwrap();
        assert!(opt.is_none());
    });
}
