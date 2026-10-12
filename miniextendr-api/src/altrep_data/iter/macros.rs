//! Shared storage and constructors for the concrete iterator data adapters.

macro_rules! define_iter_data {
    ($(#[$meta:meta])* $name:ident, $item:ty) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name<I: Iterator<Item = $item>> {
            state: super::IterState<I, $item>,
        }

        impl<I: Iterator<Item = $item>> $name<I> {
            /// Create a lazy adapter for exactly the declared prefix.
            ///
            /// The iterator is not advanced during construction.
            pub fn from_iter(iter: I, len: usize) -> Self {
                Self {
                    state: super::IterState::new(iter, len),
                }
            }
        }

        impl<I: Iterator<Item = $item>> crate::altrep_data::AltrepSerialize for $name<I> {
            fn serialized_state(&self) -> crate::SEXP {
                // A source shorter than the declared length leaves elements no
                // saved state can restore. C NULL asks R to save the values the
                // element reads give instead.
                match self.state.try_materialize_all() {
                    Ok(values) => crate::IntoR::into_sexp(values.to_vec()),
                    Err(_) => crate::SEXP::null(),
                }
            }

            fn unserialize(state: crate::SEXP) -> Option<Self> {
                let values = <Vec<$item> as crate::TryFromSexp>::try_from_sexp(state).ok()?;
                Some(Self { state: super::IterState::from_values(values) })
            }
        }

        impl<I: ExactSizeIterator<Item = $item>> $name<I> {
            /// Create a lazy adapter using the iterator's remaining length.
            pub fn from_exact_iter(iter: I) -> Self {
                let len = iter.len();
                Self::from_iter(iter, len)
            }
        }

        impl<I: Iterator<Item = $item>> crate::altrep_data::AltrepLen for $name<I> {
            fn altrep_len(&self) -> usize {
                self.state.len()
            }
        }
    };
}

pub(super) use define_iter_data;
