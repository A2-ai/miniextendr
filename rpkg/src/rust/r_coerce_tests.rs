//! Tests for `#[miniextendr(as = "...")]` attribute and R's `as.<class>()` coercion methods.
//!
//! These tests verify that the `as = "..."` attribute generates proper S3 methods
//! for R's coercion generics like `as.data.frame()`, `as.list()`, `as.character()`.

use miniextendr_api::r_coerce::RCoerceError;
use miniextendr_api::{
    ExternalPtr, IntoR, List, ListBuilder, OwnedProtect, ProtectScope, SEXP, SEXPTYPE, SexpExt,
    miniextendr, sys,
};

/// Test struct for `as.<class>` coercion methods.
///
/// This struct contains sample data to test conversion to data.frame, list, and character.
#[derive(ExternalPtr)]
pub struct RCoerceTestData {
    names: Vec<String>,
    values: Vec<f64>,
}

/// Another test struct that returns errors for certain conversions.
#[derive(ExternalPtr)]
pub struct RCoerceErrorTest {
    is_empty: bool,
}

// region: RCoerceTestData impl

/// @title as.<class>() Coercion Test Type
/// @name RCoerceTestData
/// @description Test type for as.data.frame, as.list, and as.character methods
#[miniextendr(s3)]
impl RCoerceTestData {
    /// Create a new test data object.
    /// @param names Character vector of names.
    /// @param values Numeric vector of values.
    pub fn new(names: Vec<String>, values: Vec<f64>) -> Self {
        Self { names, values }
    }

    /// Get the number of items.
    pub fn len(&self) -> i32 {
        self.values.len() as i32
    }

    /// Convert to data.frame.
    ///
    /// Creates a data.frame with columns "name" and "value".
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, RCoerceError> {
        if self.names.len() != self.values.len() {
            return Err(RCoerceError::InvalidData {
                message: "names and values must have same length".into(),
            });
        }

        // Build a data.frame with two columns of different types
        let n = self.names.len();
        unsafe {
            let scope = ProtectScope::new();
            let builder = ListBuilder::new(&scope, 2);

            // Column 1: character (names)
            let names_col = scope.protect_raw(self.names.clone().into_sexp());
            builder.set(0, names_col);

            // Column 2: numeric (values)
            let values_col = scope.protect_raw(self.values.clone().into_sexp());
            builder.set(1, values_col);

            let list = builder
                .into_list()
                .set_names_str(&["name", "value"])
                .set_class_str(&["data.frame"])
                .set_row_names_int(n);

            Ok(list)
        }
    }

    /// Convert to list.
    ///
    /// Creates a list with "names" and "values" elements.
    #[miniextendr(as = "list")]
    pub fn as_list(&self) -> Result<List, RCoerceError> {
        unsafe {
            let scope = ProtectScope::new();
            let builder = ListBuilder::new(&scope, 2);

            // Element 1: names
            let names_el = scope.protect_raw(self.names.clone().into_sexp());
            builder.set(0, names_el);

            // Element 2: values
            let values_el = scope.protect_raw(self.values.clone().into_sexp());
            builder.set(1, values_el);

            Ok(builder.into_list().set_names_str(&["names", "values"]))
        }
    }

    /// Convert to character representation.
    ///
    /// Returns a single string describing the data.
    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<SEXP, RCoerceError> {
        let desc = format!(
            "RCoerceTestData({} items: {})",
            self.values.len(),
            self.names.join(", ")
        );

        // Create a character vector of length 1
        unsafe {
            let result = OwnedProtect::new(sys::Rf_allocVector(SEXPTYPE::STRSXP, 1));
            result.get().set_string_elt(0, SEXP::charsxp(&desc));
            Ok(result.into_sexp())
        }
    }

    /// Convert to numeric vector.
    ///
    /// Returns the values as a numeric vector.
    #[miniextendr(as = "numeric")]
    pub fn as_numeric(&self) -> Result<SEXP, RCoerceError> {
        Ok(self.values.clone().into_sexp())
    }

    /// Convert to integer vector.
    ///
    /// Returns the values truncated to integers.
    #[miniextendr(as = "integer")]
    pub fn as_integer(&self) -> Result<SEXP, RCoerceError> {
        let ints: Vec<i32> = self.values.iter().map(|&v| v as i32).collect();
        Ok(ints.into_sexp())
    }
}
// endregion

// region: RCoerceErrorTest impl - tests error handling

/// @title as.<class>() Error Test Type
/// @name RCoerceErrorTest
#[miniextendr(s3)]
impl RCoerceErrorTest {
    /// Create a new error test object.
    /// @param is_empty Logical; if TRUE, triggers error on as.data.frame.
    pub fn new(is_empty: bool) -> Self {
        Self { is_empty }
    }

    /// Convert to data.frame - fails when empty.
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, RCoerceError> {
        if self.is_empty {
            return Err(RCoerceError::InvalidData {
                message: "cannot create data.frame from empty data".into(),
            });
        }

        unsafe {
            let scope = ProtectScope::new();
            let builder = ListBuilder::new(&scope, 1);

            let values_col = scope.protect_raw(vec![42.0_f64].into_sexp());
            builder.set(0, values_col);

            Ok(builder
                .into_list()
                .set_names_str(&["value"])
                .set_class_str(&["data.frame"])
                .set_row_names_int(1))
        }
    }

    /// Convert to list - always returns "not supported" error.
    #[miniextendr(as = "list")]
    pub fn as_list(&self) -> Result<List, RCoerceError> {
        Err(RCoerceError::NotSupported {
            from: "RCoerceErrorTest",
            to: "list",
        })
    }

    /// Convert to character - custom error message.
    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<SEXP, RCoerceError> {
        Err(RCoerceError::Custom("intentional error for testing".into()))
    }
}
// endregion

// region: Coercion argument handling (#1830)

/// Coercions share ordinary method choice defaults, checks and hooks.
#[derive(ExternalPtr)]
pub struct RCoerceArgsTest;

/// @title Coercion argument handling
#[miniextendr(s3)]
impl RCoerceArgsTest {
    /// Construct the argument-handling fixture.
    pub fn new() -> Self {
        Self
    }

    /// Preserve omission and validate choices before converting a list.
    /// @param mode An optional mode; omission and NULL stay distinct.
    /// @param label An optional low or high label.
    /// @param value A number inheriting from coerce_value, excluding coerce_forbidden.
    #[miniextendr(
        as = "list",
        match_arg(mode),
        choices(label = "low, high"),
        defaults(value = "structure(1, class = 'coerce_value')"),
        no_na(value(message = "coercion value must not be missing")),
        inherits(value = "coerce_value"),
        not_inherits(value = "coerce_forbidden")
    )]
    pub fn coerce_selected(
        &self,
        mode: miniextendr_api::Missing<Option<crate::match_arg_impl_tests::ImplMode>>,
        label: Option<String>,
        value: f64,
    ) -> List {
        let mode = match mode {
            miniextendr_api::Missing::Absent => "absent".to_string(),
            miniextendr_api::Missing::Present(None) => "null".to_string(),
            miniextendr_api::Missing::Present(Some(mode)) => format!("{mode:?}"),
        };
        List::from_pairs(vec![
            ("mode", mode),
            ("label", label.unwrap_or_else(|| "null".to_string())),
            ("value", value.to_string()),
        ])
    }

    /// Entry runs before checks, post-check code before Rust, and cleanup on exit.
    /// @param amount An integer value; the entry hook coerces it before validation.
    #[miniextendr(
        as = "character",
        call = none,
        defaults(amount = "1L"),
        no_na(amount),
        r_entry = "amount <- as.integer(amount)",
        r_post_checks = "amount <- amount + 1L",
        r_on_exit = "message('coercion cleanup')"
    )]
    pub fn coerce_hooked(&self, amount: i32) -> String {
        amount.to_string()
    }

    /// Explicit dots keep their position and a required mode has no default.
    /// @param ... Additional values, counted by the method.
    /// @param mode A mode; optional for list conversion and required for integer conversion.
    #[miniextendr(as = "integer", match_arg(mode), no_default(mode))]
    pub fn coerce_dots(
        &self,
        dots: &miniextendr_api::dots::Dots,
        mode: crate::match_arg_impl_tests::ImplMode,
    ) -> Vec<i32> {
        use crate::match_arg_impl_tests::ImplMode;
        vec![
            i32::try_from(dots.len()).expect("fixture dots length"),
            match mode {
                ImplMode::Fast => 1,
                ImplMode::Safe => 2,
                ImplMode::Debug => 3,
            },
        ]
    }
}

// endregion

// region: Coercion visibility (#1821)

macro_rules! coercion_visibility {
    ($(#[$doc:meta])* $type:ident $(, $visibility:ident)?) => {
        #[derive(ExternalPtr)]
        pub struct $type;

        $(#[$doc])*
        /// Coercion documentation and dispatch visibility.
        #[miniextendr(s3 $(, $visibility)?)]
        impl $type {
            /// Construct a visibility fixture.
            pub fn new() -> Self {
                Self
            }

            /// Convert the fixture into a list.
            /// @return A list containing the fixture value.
            #[miniextendr(as = "list")]
            pub fn coerce_visibility_list(&self) -> Result<List, RCoerceError> {
                Ok(List::from_values(vec![42_i32]))
            }
        }
    };
}

coercion_visibility!(CoerceHidden, noexport);
coercion_visibility!(CoerceInternal, internal);
coercion_visibility!(
    /// @noRd
    CoerceNoRd
);

// endregion
