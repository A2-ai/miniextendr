use miniextendr_api::{IsNA, RLogical, RNativeType, Rcomplex};

#[test]
fn native_na_detection_distinguishes_nan_and_computed_na() {
    assert!(i32::R_NA.is_na());
    assert!(!0i32.is_na());
    assert!(IsNA::is_na(&RLogical::NA));
    assert!(!IsNA::is_na(&RLogical::from_i32(42)));
    assert!(f64::R_NA.is_na());
    assert!(f64::from_bits(0x7ff8_0000_0000_07a2).is_na());
    assert!(!f64::NAN.is_na());
    assert!(!f64::INFINITY.is_na());
    assert!(!f64::from_bits(0x3ff0_0000_0000_07a2).is_na());
}

#[test]
fn either_complex_component_can_be_na() {
    assert!(
        Rcomplex {
            r: f64::R_NA,
            i: 0.0
        }
        .is_na()
    );
    assert!(
        Rcomplex {
            r: 0.0,
            i: f64::R_NA
        }
        .is_na()
    );
    assert!(
        !Rcomplex {
            r: f64::NAN,
            i: 0.0
        }
        .is_na()
    );
    assert!(Rcomplex::R_NA.is_na());
}

#[test]
fn optional_iterator_items_use_the_native_na_sentinel() {
    use miniextendr_api::{AltComplexData, AltLogicalData, AltRealData, AltrepIteratorExt};

    let data = [Some(f64::NAN), None, Some(1.5)]
        .into_iter()
        .into_altrep_data_exact();
    assert!(data.elt(0).is_nan());
    assert!(!data.elt(0).is_na());
    assert!(data.elt(1).is_na());
    assert_eq!(data.elt(2), 1.5);

    let logical = [Some(RLogical::from_i32(42)), None]
        .into_iter()
        .into_altrep_data_exact();
    let mut values = [0; 2];
    logical.get_region(0, 2, &mut values);
    assert_eq!(values, [42, i32::MIN]);

    let complex = [None::<Rcomplex>].into_iter().into_altrep_data_exact();
    assert!(complex.elt(0).is_na());
}
