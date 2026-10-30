//! ALTREP for owned const-generic arrays.
//!
//! The shared family generators keep native pointers and materialized data2
//! behaviour consistent with Vec/Box. Each element family caches a separate
//! class per N: a static inside a generic function alone would be shared by
//! every N and install callbacks for whichever length registered first.
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
                use std::cell::RefCell;
                use std::collections::BTreeMap;
                use std::ffi::CString;

                // Only R's main thread uses the cache: every access is
                // inside `with_r_thread`.
                thread_local! {
                    static CLASSES: RefCell<BTreeMap<usize, crate::sys::altrep::R_altrep_class_t>> =
                        const { RefCell::new(BTreeMap::new()) };
                }

                crate::worker::with_r_thread(|| {
                    let cached = CLASSES.with_borrow(|classes| classes.get(&N).copied());
                    if let Some(class) = cached {
                        return class;
                    }

                    // R copies/interns these bytes. Do not hold the cache borrow
                    // across R allocation or callback installation.
                    let name = CString::new(format!("{}_{}", $name, N))
                        .expect("array class name contains no NUL");
                    let class = unsafe {
                        <Self as crate::altrep_data::InferBase>::make_class(
                            name.as_ptr(),
                            crate::AltrepPkgName::as_ptr(),
                        )
                    };
                    unsafe {
                        <Self as crate::altrep_data::InferBase>::install_methods(class);
                    }
                    CLASSES.with_borrow_mut(|classes| classes.insert(N, class));
                    class
                })
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
