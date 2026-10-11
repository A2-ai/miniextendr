//! Shared slice owners with coordinated COW and duplicate policies.
//!
//! Element/region reads share Rc/Arc storage. A native contiguous pointer request
//! establishes unique ownership, including detaching any Weak references. Once
//! unique, duplication copies the allocation instead of re-sharing it, preserving
//! exposed pointers. This conservatively copies even an unexposed unique owner.
//!
//! bool and String keep R's values in data2 (a logical vector or the string
//! element cache). Duplication copies data2 when present, through
//! `duplicate_altrep_data2`, and serialization falls back to an ordinary vector
//! once R has written to it.

use std::rc::Rc;
use std::sync::Arc;

use crate::altrep::RegisterAltrep;
use crate::altrep_data::{AltrepExtract, altrep_values_written, materialized_altrep_data2};
use crate::altrep_ext::AltrepSexpExt;
use crate::altrep_traits::Altrep;
use crate::{IntoR, OwnedProtect, SEXP, SexpExt, TryFromSexp};

macro_rules! impl_shared_slice {
    ($owner:ident, $elem:ty, $class:literal, $register:ident, $entry:ident) => {
        impl Altrep for $owner<[$elem]> {
            fn length(x: SEXP) -> crate::R_xlen_t {
                if let Some(cache) = unsafe { materialized_altrep_data2(x) } {
                    return cache
                        .len()
                        .try_into()
                        .expect("ALTREP length exceeds R_xlen_t");
                }
                unsafe { Self::altrep_extract_ref(x) }
                    .len()
                    .try_into()
                    .expect("ALTREP length exceeds R_xlen_t")
            }

            const HAS_DUPLICATE: bool = true;
            fn duplicate(x: SEXP, _deep: bool) -> SEXP {
                let data = unsafe { Self::altrep_extract_ref(x) };
                let owner = if $owner::strong_count(data) == 1 && $owner::weak_count(data) == 0 {
                    // This allocation may have an outstanding R pointer.
                    $owner::<[$elem]>::from(data.as_ref())
                } else {
                    $owner::clone(data)
                };
                let out = unsafe { OwnedProtect::new(crate::Altrep(owner).into_sexp()) };
                if let Some(data2) = unsafe { materialized_altrep_data2(x) } {
                    // R may have written into data2: copy its current values
                    // rather than rebuild them from data1.
                    unsafe {
                        let copy = super::duplicate_altrep_data2(data2);
                        AltrepSexpExt::set_altrep_data2(&out.get(), copy);
                    }
                }
                out.get()
            }

            const HAS_SERIALIZED_STATE: bool = true;
            fn serialized_state(x: SEXP) -> SEXP {
                if unsafe { altrep_values_written(x) } {
                    return SEXP::null();
                }
                unsafe { Self::altrep_extract_ref(x) }.to_vec().into_sexp()
            }

            const HAS_UNSERIALIZE: bool = true;
            fn unserialize(class: SEXP, state: SEXP) -> SEXP {
                let values = Vec::<$elem>::try_from_sexp(state)
                    .expect("invalid shared-slice ALTREP serialized state");
                let owner = $owner::<[$elem]>::from(values);
                unsafe {
                    let ptr = crate::ExternalPtr::new_unchecked(owner);
                    let data1 = OwnedProtect::new(ptr.as_sexp());
                    crate::sys::altrep::R_altrep_class_t::from_sexp(class)
                        .new_altrep_unchecked(data1.get(), SEXP::nil())
                }
            }
        }

        super::builtins::impl_register_altrep_builtin!($owner<[$elem]>, $class);

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C-unwind" fn $register() {
            <$owner<[$elem]> as RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    crate::linkme::distributed_slice(crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = crate::linkme)
                )]
        static $entry: crate::registry::AltrepRegistration = crate::registry::AltrepRegistration {
            register: $register,
            symbol: stringify!($register),
        };
    };
}

macro_rules! impl_shared_native {
    ($owner:ident, $elem:ty, $storage:ty, $methods:ident, $infer:ident) => {
        crate::__impl_altvec_dataptr!($owner<[$elem]>, $storage);
        crate::$methods!($owner<[$elem]>);
        crate::$infer!($owner<[$elem]>);
    };
}

impl_shared_native!(
    Rc,
    i32,
    i32,
    __impl_altinteger_methods,
    impl_inferbase_integer
);
impl_shared_native!(Rc, f64, f64, __impl_altreal_methods, impl_inferbase_real);
impl_shared_native!(Rc, u8, u8, __impl_altraw_methods, impl_inferbase_raw);
impl_shared_native!(
    Rc,
    crate::Rcomplex,
    crate::Rcomplex,
    __impl_altcomplex_methods,
    impl_inferbase_complex
);
impl_shared_native!(
    Rc,
    crate::RLogical,
    i32,
    __impl_altlogical_methods,
    impl_inferbase_logical
);
crate::__impl_altvec_logical_dataptr!(Rc<[bool]>);
crate::__impl_altlogical_methods!(Rc<[bool]>);
crate::impl_inferbase_logical!(Rc<[bool]>);
crate::__impl_altvec_string_dataptr!(Rc<[String]>);
crate::__impl_altstring_methods!(Rc<[String]>);
crate::impl_inferbase_string!(Rc<[String]>);
impl_shared_slice!(
    Rc,
    i32,
    "Rc_i32",
    __mx_altrep_reg_builtin_Rc_i32,
    __MX_ALTREP_REG_SHARED_RC_I32
);
impl_shared_slice!(
    Rc,
    f64,
    "Rc_f64",
    __mx_altrep_reg_builtin_Rc_f64,
    __MX_ALTREP_REG_SHARED_RC_F64
);
impl_shared_slice!(
    Rc,
    u8,
    "Rc_u8",
    __mx_altrep_reg_builtin_Rc_u8,
    __MX_ALTREP_REG_SHARED_RC_U8
);
impl_shared_slice!(
    Rc,
    crate::Rcomplex,
    "Rc_Rcomplex",
    __mx_altrep_reg_builtin_Rc_Rcomplex,
    __MX_ALTREP_REG_SHARED_RC_RCOMPLEX
);
impl_shared_slice!(
    Rc,
    crate::RLogical,
    "Rc_RLogical",
    __mx_altrep_reg_builtin_Rc_RLogical,
    __MX_ALTREP_REG_SHARED_RC_RLOGICAL
);
impl_shared_slice!(
    Rc,
    bool,
    "Rc_bool",
    __mx_altrep_reg_builtin_Rc_bool,
    __MX_ALTREP_REG_SHARED_RC_BOOL
);
impl_shared_slice!(
    Rc,
    String,
    "Rc_String",
    __mx_altrep_reg_builtin_Rc_String,
    __MX_ALTREP_REG_SHARED_RC_STRING
);
impl_shared_native!(
    Arc,
    i32,
    i32,
    __impl_altinteger_methods,
    impl_inferbase_integer
);
impl_shared_native!(Arc, f64, f64, __impl_altreal_methods, impl_inferbase_real);
impl_shared_native!(Arc, u8, u8, __impl_altraw_methods, impl_inferbase_raw);
impl_shared_native!(
    Arc,
    crate::Rcomplex,
    crate::Rcomplex,
    __impl_altcomplex_methods,
    impl_inferbase_complex
);
impl_shared_native!(
    Arc,
    crate::RLogical,
    i32,
    __impl_altlogical_methods,
    impl_inferbase_logical
);
crate::__impl_altvec_logical_dataptr!(Arc<[bool]>);
crate::__impl_altlogical_methods!(Arc<[bool]>);
crate::impl_inferbase_logical!(Arc<[bool]>);
crate::__impl_altvec_string_dataptr!(Arc<[String]>);
crate::__impl_altstring_methods!(Arc<[String]>);
crate::impl_inferbase_string!(Arc<[String]>);
impl_shared_slice!(
    Arc,
    i32,
    "Arc_i32",
    __mx_altrep_reg_builtin_Arc_i32,
    __MX_ALTREP_REG_SHARED_ARC_I32
);
impl_shared_slice!(
    Arc,
    f64,
    "Arc_f64",
    __mx_altrep_reg_builtin_Arc_f64,
    __MX_ALTREP_REG_SHARED_ARC_F64
);
impl_shared_slice!(
    Arc,
    u8,
    "Arc_u8",
    __mx_altrep_reg_builtin_Arc_u8,
    __MX_ALTREP_REG_SHARED_ARC_U8
);
impl_shared_slice!(
    Arc,
    crate::Rcomplex,
    "Arc_Rcomplex",
    __mx_altrep_reg_builtin_Arc_Rcomplex,
    __MX_ALTREP_REG_SHARED_ARC_RCOMPLEX
);
impl_shared_slice!(
    Arc,
    crate::RLogical,
    "Arc_RLogical",
    __mx_altrep_reg_builtin_Arc_RLogical,
    __MX_ALTREP_REG_SHARED_ARC_RLOGICAL
);
impl_shared_slice!(
    Arc,
    bool,
    "Arc_bool",
    __mx_altrep_reg_builtin_Arc_bool,
    __MX_ALTREP_REG_SHARED_ARC_BOOL
);
impl_shared_slice!(
    Arc,
    String,
    "Arc_String",
    __mx_altrep_reg_builtin_Arc_String,
    __MX_ALTREP_REG_SHARED_ARC_STRING
);
