//! Exact comparisons through the RPartialEq adapter (#2149).
use miniextendr_api::{AllEqual, ExternalPtr, RPartialEq as ExactEquality, miniextendr};

macro_rules! exact_class {
    ($name:ident, $system:ident $(, $debug:ident)?) => {
        #[derive(ExternalPtr, PartialEq $(, $debug)?)]
        #[externalptr(restored(class = "partial_eq_saved"))]
        pub struct $name(f64);

        /// A pointer-backed class using exact Rust equality.
        #[miniextendr($system)]
        impl $name {
            pub fn new(value: f64) -> Self { Self(value) }
        }

        #[miniextendr($system)]
        impl ExactEquality for $name {}
    };
}
exact_class!(ExactS3, s3, Debug);
exact_class!(ExactEnv, env);
exact_class!(ExactR6, r6, Debug);
exact_class!(ExactS4, s4, Debug);
exact_class!(ExactS7, s7, Debug);

#[cfg(feature = "vctrs")]
#[derive(ExternalPtr, PartialEq)]
#[externalptr(restored(class = "partial_eq_saved"))]
pub struct ExactVctrs(f64);
/// A vctrs value carrying an exactly compared pointer.
#[cfg(feature = "vctrs")]
#[miniextendr(vctrs, class = "exact_vctrs", internal)]
impl ExactVctrs {}
#[cfg(feature = "vctrs")]
#[miniextendr(vctrs)]
impl ExactEquality for ExactVctrs {}

/// Pointer storage for the exact vctrs comparison fixture.
#[cfg(feature = "vctrs")]
#[miniextendr(noexport)]
pub fn exact_vctrs_pointer(value: f64) -> ExternalPtr<ExactVctrs> {
    ExternalPtr::new(ExactVctrs(value))
}

#[derive(ExternalPtr, Debug)]
pub struct ExactBlanket(i32);
impl PartialEq for ExactBlanket {
    fn eq(&self, other: &Self) -> bool {
        self.0 % 10 == other.0 % 10
    }
}
/// A renamed class whose Rust equality compares the last decimal digit.
#[miniextendr(s3, class = "exact_digit")]
impl ExactBlanket {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
}
type DigitAlias = ExactBlanket;
#[miniextendr(s3, blanket)]
impl ExactEquality for DigitAlias {
    fn r_eq(&self, other: &Self) -> bool {
        unreachable!("metadata only")
    }
}

#[derive(ExternalPtr, AllEqual)]
pub struct ExactNested {
    #[all_equal(with = "Self::exact", label = "Exact model")]
    value: f64,
}
impl ExactNested {
    fn exact(&self) -> ExactS3 {
        ExactS3(self.value)
    }
}
/// A fieldwise comparison containing an exactly compared class.
#[miniextendr(s3)]
impl ExactNested {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}
