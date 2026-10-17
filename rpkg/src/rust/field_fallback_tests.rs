//! Field lookup fallbacks with exact names and custom diagnostics (#2150).
use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::{IntoR, SEXP, miniextendr};

macro_rules! fallback_class {
    ($ty:ident, $system:ident) => {
        #[derive(miniextendr_api::ExternalPtr, Debug)]
        #[externalptr(
            $system,
            field_error(
                class = "fallback_field_error",
                no_field_hint = "No column named '%s'."
            ),
            restored(class = "fallback_saved", refuse = "every_read")
        )]
        pub struct $ty {
            #[r_data]
            _r: RSidecar,
            #[r_data]
            pub id: i32,
        }

        impl $ty {
            fn column(&self, name: &str) -> Option<SEXP> {
                match name {
                    "id" => panic!("the field must win"),
                    "column" => Some(vec![self.id, self.id + 1].into_sexp()),
                    "null" => Some(SEXP::nil()),
                    "missing" => Some(SEXP::missing_arg()),
                    "ping" => Some(99_i32.into_sexp()),
                    "panic" => panic!("fallback panic"),
                    "warned" => {
                        miniextendr_api::defer_warning!(
                            class = "fallback_warning",
                            "fallback warning"
                        );
                        Some(self.id.into_sexp())
                    }
                    "absent_warned" => {
                        miniextendr_api::defer_warning!(
                            class = "fallback_warning",
                            "absent warning"
                        );
                        None
                    }
                    name if name.starts_with("borrowed-") => {
                        unsafe { miniextendr_api::sys::R_gc() };
                        Some(name.to_string().into_sexp())
                    }
                    _ => None,
                }
            }

            fn column_names(&self) -> Vec<String> {
                vec!["column".to_string(), "null".to_string(), "ping".to_string()]
            }
        }

        /// A field-backed object with dynamic column lookup.
        #[miniextendr($system(
            r_data_accessors,
            fallback = "Self::column",
            fallback_names = "Self::column_names"
        ))]
        impl $ty {
            /// Create a fallback object.
            pub fn new() -> Self {
                Self {
                    _r: RSidecar,
                    id: 7,
                }
            }
        }

        // Repeated settings across separate blocks must not duplicate C symbols.
        #[miniextendr($system(
            r_data_accessors,
            fallback = "Self::column",
            fallback_names = "Self::column_names"
        ))]
        impl $ty {
            /// An ordinary method, ahead of the fallback in env name lookup.
            pub fn ping(&self) -> i32 {
                71
            }
        }
    };
}

fallback_class!(FallbackS3, s3);
fallback_class!(FallbackS4, s4);
fallback_class!(FallbackEnv, env);

/// A scalar fallback with no completion function and no every-read policy.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct FallbackScalar {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub id: i32,
}

fn scalar_column(data: &FallbackScalar, name: &str) -> Option<i32> {
    (name == "scalar").then_some(data.id)
}

/// Lookup-list mode still reports an unknown fallback name as a field error.
#[miniextendr(s3(r_data_accessors = "get", lookup = "list", fallback = "scalar_column"))]
impl FallbackScalar {
    /// Create a scalar fallback object.
    pub fn new() -> Self {
        Self {
            _r: RSidecar,
            id: 3,
        }
    }
}
