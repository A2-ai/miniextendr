//! Tests for `#[miniextendr(as = "...")]` attribute and R's `as.<class>()` coercion methods.
//!
//! These tests verify that the `as = "..."` attribute generates proper S3 methods
//! for R's coercion generics like `as.data.frame()`, `as.list()`, `as.character()`.

use miniextendr_api::{
    ExternalPtr, IntoR, List, ListBuilder, OwnedProtect, ProtectScope, ffi, miniextendr, miniextendr_module,
};
use miniextendr_api::as_coerce::AsCoerceError;

/// Test struct for as.<class> coercion methods.
///
/// This struct contains sample data to test conversion to data.frame, list, and character.
#[derive(ExternalPtr)]
pub struct AsCoerceTestData {
    names: Vec<String>,
    values: Vec<f64>,
}

/// Another test struct that returns errors for certain conversions.
#[derive(ExternalPtr)]
pub struct AsCoerceErrorTest {
    is_empty: bool,
}

// =============================================================================
// AsCoerceTestData impl
// =============================================================================

/// @title as.<class>() Coercion Test Type
/// @name AsCoerceTestData
/// @description Test type for as.data.frame, as.list, and as.character methods
#[miniextendr(s3)]
impl AsCoerceTestData {
    /// Create a new test data object.
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
    pub fn as_data_frame(&self) -> Result<List, AsCoerceError> {
        if self.names.len() != self.values.len() {
            return Err(AsCoerceError::InvalidData {
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

            let list = builder.into_list()
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
    pub fn as_list(&self) -> Result<List, AsCoerceError> {
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
    pub fn as_character(&self) -> Result<ffi::SEXP, AsCoerceError> {
        let desc = format!(
            "AsCoerceTestData({} items: {})",
            self.values.len(),
            self.names.join(", ")
        );

        // Create a character vector of length 1
        unsafe {
            let result = OwnedProtect::new(ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1));
            let chars = ffi::Rf_mkCharLenCE(
                desc.as_ptr().cast(),
                desc.len() as i32,
                ffi::CE_UTF8,
            );
            ffi::SET_STRING_ELT(result.get(), 0, chars);
            Ok(result.into_sexp())
        }
    }

    /// Convert to numeric vector.
    ///
    /// Returns the values as a numeric vector.
    #[miniextendr(as = "numeric")]
    pub fn as_numeric(&self) -> Result<ffi::SEXP, AsCoerceError> {
        Ok(self.values.clone().into_sexp())
    }

    /// Convert to integer vector.
    ///
    /// Returns the values truncated to integers.
    #[miniextendr(as = "integer")]
    pub fn as_integer(&self) -> Result<ffi::SEXP, AsCoerceError> {
        let ints: Vec<i32> = self.values.iter().map(|&v| v as i32).collect();
        Ok(ints.into_sexp())
    }
}

// =============================================================================
// AsCoerceErrorTest impl - tests error handling
// =============================================================================

/// @title as.<class>() Error Test Type
/// @name AsCoerceErrorTest
/// @noRd
#[miniextendr(s3)]
impl AsCoerceErrorTest {
    /// Create a new error test object.
    pub fn new(is_empty: bool) -> Self {
        Self { is_empty }
    }

    /// Convert to data.frame - fails when empty.
    #[miniextendr(as = "data.frame")]
    pub fn as_data_frame(&self) -> Result<List, AsCoerceError> {
        if self.is_empty {
            return Err(AsCoerceError::InvalidData {
                message: "cannot create data.frame from empty data".into(),
            });
        }

        unsafe {
            let scope = ProtectScope::new();
            let builder = ListBuilder::new(&scope, 1);

            let values_col = scope.protect_raw(vec![42.0_f64].into_sexp());
            builder.set(0, values_col);

            Ok(builder.into_list()
                .set_names_str(&["value"])
                .set_class_str(&["data.frame"])
                .set_row_names_int(1))
        }
    }

    /// Convert to list - always returns "not supported" error.
    #[miniextendr(as = "list")]
    pub fn as_list(&self) -> Result<List, AsCoerceError> {
        Err(AsCoerceError::NotSupported {
            from: "AsCoerceErrorTest",
            to: "list",
        })
    }

    /// Convert to character - custom error message.
    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<ffi::SEXP, AsCoerceError> {
        Err(AsCoerceError::Custom("intentional error for testing".into()))
    }
}

miniextendr_module! {
    mod as_coerce_tests;

    // AsCoerceTestData - successful coercions
    impl AsCoerceTestData;

    // AsCoerceErrorTest - error handling
    impl AsCoerceErrorTest;
}
