//! Public glob imports must not change ordinary Rust method lookup (#2064).
//! Keep these outside the adapter modules: importing Float/FloatCore there
//! can mask the OrderedFloat regression by selecting their by-value methods.
#![allow(unused_imports)]

macro_rules! ordinary_methods {
    () => {
        #[test]
        fn slice_methods_remain_const() {
            const LEN: usize = [1i32, 2, 3].len();
            const EMPTY: bool = [0i32; 0].is_empty();
            assert_eq!((LEN, EMPTY), (3, true));

            let values: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
            let len: usize = values.len();
            let empty: bool = values.is_empty();
            assert_eq!((len, empty), (3, false));
            assert_eq!(values.altrep_len(), len);
            assert_eq!(values.altrep_is_empty(), empty);
        }

        #[cfg(feature = "ordered-float")]
        #[test]
        fn ordered_float_keeps_the_inner_float_result_type() {
            macro_rules! check {
                ($value:expr, $ty:ty) => {
                    let value = $value;
                    let results: [$ty; 7] = [
                        value.floor(),
                        value.ceil(),
                        value.round(),
                        value.trunc(),
                        value.fract(),
                        value.abs(),
                        value.signum(),
                    ];
                    assert_eq!(results, [-3.0, -2.0, -3.0, -2.0, -0.5, 2.5, -1.0]);
                    assert!(value.is_finite());
                    assert!(!value.is_infinite());
                    assert!(!value.is_nan());
                    assert!(value.is_sign_negative());
                };
            }
            check!(OrderedFloat(-2.5f32), f32);
            check!(&OrderedFloat(-2.5f32), f32);
            check!(OrderedFloat(-2.5f64), f64);
            check!(&OrderedFloat(-2.5f64), f64);
            let for_r: f64 = ROrderedFloatOps::r_round(&OrderedFloat(-2.5f32));
            assert_eq!(for_r, -3.0);
        }

        #[cfg(feature = "num-traits")]
        #[test]
        fn numeric_adapters_do_not_shadow_numeric_traits() {
            use num_traits::{One, Signed, Zero};
            let zero: f64 = f64::zero();
            let one: f64 = f64::one();
            assert!(zero.is_zero());
            assert!(one.is_one());
            let value = &-2.5f64;
            let absolute: f64 = value.abs();
            let sign: f64 = value.signum();
            assert_eq!((absolute, sign), (2.5, -1.0));
            assert_eq!(RFloat::r_round(value), -3.0);
            assert_eq!(RSigned::r_abs(value), 2.5);
            assert!(RNum::r_is_zero(&zero));
        }
    };
}

mod root {
    use miniextendr_api::*;
    ordinary_methods!();
}

mod prelude {
    use miniextendr_api::prelude::*;
    ordinary_methods!();
}

mod named {
    use miniextendr_api::AltrepLen;
    #[cfg(feature = "ordered-float")]
    use miniextendr_api::{OrderedFloat, ROrderedFloatOps};
    #[cfg(feature = "num-traits")]
    use miniextendr_api::{RFloat, RNum, RSigned};
    ordinary_methods!();
}
