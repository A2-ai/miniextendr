//! S7 conversion marker/attribute parity, including registered R names.

use miniextendr_api::{ConvertFrom, ConvertTo, ExternalPtr, Invisible, miniextendr};

/// Shared source class with an R name different from its Rust name.
#[derive(ExternalPtr)]
pub struct ConversionSource {
    value: i32,
}

#[miniextendr(s7, class = "ConvertInput")]
impl ConversionSource {
    /// Create a conversion input.
    /// @param value Nonnegative integer for successful conversion.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    /// Read a converted value.
    pub fn conversion_value(&self) -> i32 {
        self.value
    }
}

/// S7 class whose conversion registrations come from return markers.
#[derive(ExternalPtr)]
pub struct TypedConversion {
    value: i32,
}

/// Convert between S7 classes with return markers.
#[miniextendr(s7)]
impl TypedConversion {
    /// Construct a typed conversion example.
    /// @param value Integer stored in the object.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    /// Read a converted value.
    pub fn conversion_value(&self) -> i32 {
        self.value
    }
    /// Infer the source class from the static method's parameter.
    /// @param source Conversion input.
    pub fn from_source(source: ExternalPtr<ConversionSource>) -> Result<ConvertFrom<Self>, String> {
        if source.value < 0 {
            return Err("negative conversion input".into());
        }
        Ok(ConvertFrom(Self {
            value: source.value,
        }))
    }
    /// Return and register the target class, preserving invisible results.
    /// @examples
    /// input <- ConvertInput(12L)
    /// converted <- S7::convert(input, TypedConversion)
    /// stopifnot(conversion_value(converted) == 12L)
    /// restored <- S7::convert(converted, ConvertInput)
    /// stopifnot(conversion_value(restored) == 12L)
    pub fn to_source(&self) -> Invisible<Result<ConvertTo<ConversionSource>, String>> {
        Invisible(if self.value < 0 {
            Err("negative conversion output".into())
        } else {
            Ok(ConvertTo(ConversionSource { value: self.value }))
        })
    }
}

/// Equivalent S7 conversion registrations expressed as attributes.
#[derive(ExternalPtr)]
pub struct AttributeConversion {
    value: i32,
}

#[miniextendr(s7)]
impl AttributeConversion {
    /// Construct an attribute conversion example.
    /// @param value Integer stored in the object.
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    /// Read a converted value.
    pub fn conversion_value(&self) -> i32 {
        self.value
    }
    /// Register the source class explicitly.
    /// @param source Conversion input.
    #[miniextendr(s7(convert_from = "ConversionSource"))]
    pub fn from_source(source: ExternalPtr<ConversionSource>) -> Result<Self, String> {
        if source.value < 0 {
            return Err("negative conversion input".into());
        }
        Ok(Self {
            value: source.value,
        })
    }
    /// The matching conversion attribute and visibility flag.
    #[miniextendr(s7(convert_to = "ConversionSource"), invisible)]
    pub fn to_source(&self) -> Result<ConversionSource, String> {
        if self.value < 0 {
            return Err("negative conversion output".into());
        }
        Ok(ConversionSource { value: self.value })
    }
}
