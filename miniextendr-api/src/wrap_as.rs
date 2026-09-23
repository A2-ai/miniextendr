//! Explicit R class-system return markers.
//!
//! `WrapAsR6<T>` and its siblings select the constructor used by the generated
//! R wrapper. The matching attribute is `#[miniextendr(wrap = "r6")]`.
//! Both forms leave Rust-to-R value conversion to `T`; they only select the
//! R-side wrapping expression. See `docs/MINIEXTENDR_ATTRIBUTE.md` for examples.
//!
//! `ConvertTo<T>` and `ConvertFrom<T>` additionally register an inherent S7
//! method with `S7::convert`. `ConvertTo<T>` names the target payload;
//! `ConvertFrom<Self>` infers the source from its static method's sole typed
//! parameter. Both use the existing S7 class-name resolver, including registered
//! R class renames, and have matching `s7(convert_to = "Target")` /
//! `s7(convert_from = "Source")` attributes.
//!
//! Markers are recognized by their last path segment. Type aliases and renamed
//! imports do not select wrapping. Put `Invisible`/`Visible` outside the whole
//! return, and `Option`, `Result`, or `Vec` outside the class marker.
//!
//! The named target must have a compatible R class definition. Explicit wrapping
//! does not consult the class registry or verify the selected system; a mismatch
//! is reported by R when the generated constructor or method is used.

use crate::IntoR;

macro_rules! wrap_marker {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name<T>(pub T);

        impl<T> $name<T> {
            /// Wrap a value for an explicit R return class.
            pub const fn new(value: T) -> Self {
                Self(value)
            }
            /// Recover the Rust value.
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T> From<T> for $name<T> {
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        impl<T> std::ops::Deref for $name<T> {
            type Target = T;
            fn deref(&self) -> &T {
                &self.0
            }
        }

        impl<T> std::ops::DerefMut for $name<T> {
            fn deref_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }

        impl<T: IntoR> IntoR for $name<T> {
            type Error = T::Error;
            fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
                self.0.try_into_sexp()
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
                // SAFETY: the caller provides T's ordinary conversion guarantees.
                unsafe { self.0.try_into_sexp_unchecked() }
            }
        }
    };
}

wrap_marker!(
    WrapAsR6,
    "Return `T` through its R6 constructor (`T$new(.ptr = value)`)."
);
wrap_marker!(
    WrapAsS7,
    "Return `T` through its S7 constructor (`T(.ptr = value)`)."
);
wrap_marker!(
    WrapAsS4,
    "Return `T` through `methods::new(\"T\", ptr = value)`."
);
wrap_marker!(WrapAsS3, "Return `T` with its named S3 class.");
wrap_marker!(WrapAsEnv, "Return `T` with its Env dispatch class.");
wrap_marker!(
    WrapAsVctrs,
    "Return `T` with its named vctrs class, preserving its existing class hierarchy."
);

wrap_marker!(
    ConvertTo,
    "Return an S7 target class and register the inherent method with `S7::convert`. The equivalent attribute is `s7(convert_to = \"Target\")`."
);
wrap_marker!(
    ConvertFrom,
    "Return the enclosing S7 class from a static conversion method. Its sole typed source parameter determines the `s7(convert_from = \"Source\")` registration."
);
