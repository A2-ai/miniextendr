//! ALTREP for owned const-generic arrays.
//!
//! The shared family generators keep native pointers and materialized data2
//! behaviour consistent with Vec/Box. Each element family has a separate
//! class per N ([`generic_altrep_class`](crate::altrep::generic_altrep_class)):
//! a static inside a generic function alone would be shared by every N and
//! install callbacks for whichever length registered first.
//!
//! Classes register on first conversion. Duplication and serialization keep
//! R's default ordinary-vector fallback, so restoring a value never depends
//! on having instantiated its array length in the receiving R session.

crate::impl_altinteger_from_data_generic!({const N: usize} [i32; N] {}, dataptr);
crate::impl_altreal_from_data_generic!({const N: usize} [f64; N] {}, dataptr);
crate::impl_altraw_from_data_generic!({const N: usize} [u8; N] {}, dataptr);
crate::impl_altcomplex_from_data_generic!({const N: usize} [crate::Rcomplex; N] {}, dataptr);
crate::impl_altlogical_from_data_generic!({const N: usize} [crate::RLogical; N] {}, dataptr);
crate::impl_altlogical_from_data_generic!({const N: usize} [bool; N] {});
crate::impl_altstring_from_data_generic!({const N: usize} [String; N] {});

macro_rules! register_array_family {
    ($elem:ty, $name:literal) => {
        impl<const N: usize> crate::altrep::RegisterAltrep for [$elem; N] {
            fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
                crate::altrep::generic_altrep_class::<Self>(|| format!("{}_{}", $name, N))
            }
        }
    };
}

register_array_family!(i32, "Array_i32");
register_array_family!(f64, "Array_f64");
register_array_family!(u8, "Array_u8");
register_array_family!(crate::Rcomplex, "Array_Rcomplex");
register_array_family!(crate::RLogical, "Array_RLogical");
register_array_family!(bool, "Array_bool");
register_array_family!(String, "Array_String");
