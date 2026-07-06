//! Convenience macros for implementing `InferBase`.
//!
//! The parametric `__impl_inferbase!` macro provides the shared implementation.
//! Each `impl_inferbase_*!` macro is a thin wrapper that passes family-specific
//! parameters (`RBase` variant, `R_make_alt*_class` function, `install_*` function).
//!
//! These macros are used by the `#[derive(Altrep)]` proc macro and by the
//! built-in type implementations in `super::builtins`.

/// Parametric implementation of `InferBase` for any ALTREP family.
///
/// Takes the type, RBase variant, R_make_alt*_class function, and family installer.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_inferbase {
    ($ty:ty, $base:ident, $make_fn:path, $install_fn:ident) => {
        $crate::__impl_inferbase!({} $ty {}, $base, $make_fn, $install_fn);
    };
    // Generic form: `{$gen} $ty {$where}, $base, $make_fn, $install_fn`. Brace
    // delimiters (not `[]`/`()`) are deliberate — see `__impl_altrep_base!` in
    // `altrep_impl/macros.rs` for why: `{` can never start a `$ty:ty` fragment,
    // so the bare-`$ty:ty` arm above fails cleanly against a brace-led
    // invocation instead of hard-erroring on a bogus type parse. The
    // non-generic arm above forwards here with empty `{}` brackets so there
    // is exactly one emission body.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $base:ident, $make_fn:path, $install_fn:ident) => {
        impl<$($gen)*> $crate::altrep_data::InferBase for $ty where $($whr)* {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::$base;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::sys::altrep::R_altrep_class_t {
                // Use stored DllInfo from package_init. R needs this to find
                // the ALTREP class during cross-session deserialization (readRDS).
                let dll = $crate::altrep_dll_info();
                let cls = unsafe { $make_fn(class_name, pkg_name, dll) };
                let name = unsafe { ::core::ffi::CStr::from_ptr(class_name) };
                $crate::altrep::validate_altrep_class(cls, name, $crate::altrep::RBase::$base)
            }

            unsafe fn install_methods(cls: $crate::sys::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::$install_fn::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for an integer ALTREP data type.
///
/// Accepts an optional generic form: `impl_inferbase_integer!({T} Foo<T> {T:
/// Bound})` — see [`crate::impl_altinteger_from_data_generic!`] for the same
/// convention at the family-macro layer.
#[macro_export]
macro_rules! impl_inferbase_integer {
    ($ty:ty) => {
        $crate::impl_inferbase_integer!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            Int,
            $crate::sys::altrep::R_make_altinteger_class,
            install_int
        );
    };
}

/// Implement `InferBase` for a real ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_real {
    ($ty:ty) => {
        $crate::impl_inferbase_real!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            Real,
            $crate::sys::altrep::R_make_altreal_class,
            install_real
        );
    };
}

/// Implement `InferBase` for a logical ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_logical {
    ($ty:ty) => {
        $crate::impl_inferbase_logical!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            Logical,
            $crate::sys::altrep::R_make_altlogical_class,
            install_lgl
        );
    };
}

/// Implement `InferBase` for a raw ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_raw {
    ($ty:ty) => {
        $crate::impl_inferbase_raw!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            Raw,
            $crate::sys::altrep::R_make_altraw_class,
            install_raw
        );
    };
}

/// Implement `InferBase` for a string ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_string {
    ($ty:ty) => {
        $crate::impl_inferbase_string!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            String,
            $crate::sys::altrep::R_make_altstring_class,
            install_str
        );
    };
}

/// Implement `InferBase` for a complex ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_complex {
    ($ty:ty) => {
        $crate::impl_inferbase_complex!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            Complex,
            $crate::sys::altrep::R_make_altcomplex_class,
            install_cplx
        );
    };
}

/// Implement `InferBase` for a list ALTREP data type. See
/// [`impl_inferbase_integer!`] for the generic calling convention.
#[macro_export]
macro_rules! impl_inferbase_list {
    ($ty:ty) => {
        $crate::impl_inferbase_list!({} $ty {});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::__impl_inferbase!(
            {$($gen)*} $ty {$($whr)*},
            List,
            $crate::sys::altrep::R_make_altlist_class,
            install_list
        );
    };
}

// region: Logical enum tests
#[cfg(test)]
mod tests;
// endregion
