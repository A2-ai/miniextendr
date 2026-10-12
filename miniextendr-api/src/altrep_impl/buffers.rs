//! Registered native buffers with value duplication and retained allocations.
use crate::AltrepBuffer;
use crate::altrep::RegisterAltrep;

macro_rules! buffer_class {
    ($elem:ty, $storage:ty, $methods:ident, $infer:ident, $class:literal, $register:ident, $entry:ident) => {
        crate::__impl_altrep_base!(
            AltrepBuffer<$elem>,
            RUnwind,
            with_serialize,
            extra = {
                const HAS_DUPLICATE: bool = true;
                fn duplicate(x: crate::SEXP, _deep: bool) -> crate::SEXP {
                    unsafe { super::duplicate_altrep_owner::<Self>(x) }
                }
            }
        );
        crate::__impl_altvec_dataptr!(AltrepBuffer<$elem>, $storage);
        crate::$methods!(AltrepBuffer<$elem>);
        crate::$infer!(AltrepBuffer<$elem>);
        super::builtins::impl_register_altrep_builtin!(AltrepBuffer<$elem>, $class);

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C-unwind" fn $register() {
            <AltrepBuffer<$elem> as RegisterAltrep>::get_or_init_class();
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

buffer_class!(
    i32,
    i32,
    __impl_altinteger_methods,
    impl_inferbase_integer,
    "Buffer_i32",
    __mx_altrep_reg_builtin_Buffer_i32,
    __MX_ALTREP_REG_BUFFER_I32
);
buffer_class!(
    f64,
    f64,
    __impl_altreal_methods,
    impl_inferbase_real,
    "Buffer_f64",
    __mx_altrep_reg_builtin_Buffer_f64,
    __MX_ALTREP_REG_BUFFER_F64
);
buffer_class!(
    u8,
    u8,
    __impl_altraw_methods,
    impl_inferbase_raw,
    "Buffer_u8",
    __mx_altrep_reg_builtin_Buffer_u8,
    __MX_ALTREP_REG_BUFFER_U8
);
buffer_class!(
    crate::Rcomplex,
    crate::Rcomplex,
    __impl_altcomplex_methods,
    impl_inferbase_complex,
    "Buffer_Rcomplex",
    __mx_altrep_reg_builtin_Buffer_Rcomplex,
    __MX_ALTREP_REG_BUFFER_RCOMPLEX
);
buffer_class!(
    crate::RLogical,
    i32,
    __impl_altlogical_methods,
    impl_inferbase_logical,
    "Buffer_RLogical",
    __mx_altrep_reg_builtin_Buffer_RLogical,
    __MX_ALTREP_REG_BUFFER_RLOGICAL
);
