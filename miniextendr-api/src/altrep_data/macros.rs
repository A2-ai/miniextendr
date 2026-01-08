// Convenience macros for implementing InferBase.

/// Implement `InferBase` for an integer ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_integer {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Int;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altinteger_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_int::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a real ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_real {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Real;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altreal_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_real::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a logical ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_logical {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Logical;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlogical_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_lgl::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a raw ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_raw {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Raw;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altraw_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_raw::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a string ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_string {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::String;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altstring_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_str::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a complex ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_complex {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Complex;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altcomplex_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_cplx::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a list ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_list {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::List;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlist_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_list::<$ty>(cls) };
            }
        }
    };
}

// -------------------------------------------------------------------------
// Logical enum tests
// -------------------------------------------------------------------------
#[cfg(test)]
mod tests;
