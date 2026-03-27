//! Convenience macros for implementing `InferBase`.
//!
//! The parametric `__impl_inferbase!` macro provides the shared implementation.
//! Each `impl_inferbase_*!` macro is a thin wrapper that passes family-specific
//! parameters (`RBase` variant, `R_make_alt*_class` function, `install_*` function).
//!
//! These macros are used by the `#[derive(Altrep)]` proc macro and by the
//! built-in type implementations in [`super::builtins`].

/// Parametric implementation of `InferBase` for any ALTREP family.
///
/// Takes the type, RBase variant, R_make_alt*_class function, and family installer.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_inferbase {
    ($ty:ty, $base:ident, $make_fn:path, $install_fn:ident) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::$base;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                let cls = unsafe { $make_fn(class_name, pkg_name, core::ptr::null_mut()) };
                let name = unsafe { ::core::ffi::CStr::from_ptr(class_name) };
                $crate::altrep::validate_altrep_class(cls, name, $crate::altrep::RBase::$base)
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::$install_fn::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for an integer ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_integer {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            Int,
            $crate::ffi::altrep::R_make_altinteger_class,
            install_int
        );
    };
}

/// Implement `InferBase` for a real ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_real {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            Real,
            $crate::ffi::altrep::R_make_altreal_class,
            install_real
        );
    };
}

/// Implement `InferBase` for a logical ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_logical {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            Logical,
            $crate::ffi::altrep::R_make_altlogical_class,
            install_lgl
        );
    };
}

/// Implement `InferBase` for a raw ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_raw {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            Raw,
            $crate::ffi::altrep::R_make_altraw_class,
            install_raw
        );
    };
}

/// Implement `InferBase` for a string ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_string {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            String,
            $crate::ffi::altrep::R_make_altstring_class,
            install_str
        );
    };
}

/// Implement `InferBase` for a complex ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_complex {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            Complex,
            $crate::ffi::altrep::R_make_altcomplex_class,
            install_cplx
        );
    };
}

/// Implement `InferBase` for a list ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_list {
    ($ty:ty) => {
        $crate::__impl_inferbase!(
            $ty,
            List,
            $crate::ffi::altrep::R_make_altlist_class,
            install_list
        );
    };
}

// region: Logical enum tests
#[cfg(test)]
mod tests;
// endregion
