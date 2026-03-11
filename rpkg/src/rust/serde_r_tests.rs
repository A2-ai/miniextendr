//! Tests for serde_r direct R serialization.
//!
//! Demonstrates:
//! - Primitive types (bool, i32, f64, String)
//! - Option<T> for NA handling
//! - Vectors (smart dispatch to atomic vectors)
//! - Nested structs (lists within lists)
//! - HashMap/BTreeMap (named lists)
//! - Enums (unit and data variants)
//! - Complex nested structures
//! - Round-trip conversions

use crate::serde::{Deserialize, Serialize};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::serde::{RSerdeError, from_r, to_r};
use miniextendr_api::{ExternalPtr, miniextendr};
use std::collections::{BTreeMap, HashMap};

// =============================================================================
// Basic struct types
// =============================================================================

/// Simple 2D point for serde_r testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct SerdeRPoint {
    pub x: f64,
    pub y: f64,
}

#[miniextendr]
impl SerdeRPoint {
    pub fn new(x: f64, y: f64) -> Self {
        SerdeRPoint { x, y }
    }

    /// Serialize this Point to an R list.
    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    /// Deserialize a Point from an R list.
    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

/// 3D point with optional label for serde_r testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct SerdeRPoint3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub label: Option<String>,
}

#[miniextendr]
impl SerdeRPoint3D {
    pub fn new(x: f64, y: f64, z: f64, label: Option<String>) -> Self {
        SerdeRPoint3D { x, y, z, label }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Nested struct types
// =============================================================================

/// Rectangle defined by two points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct Rectangle {
    pub top_left: SerdeRPoint,
    pub bottom_right: SerdeRPoint,
    pub fill_color: Option<String>,
}

#[miniextendr]
impl Rectangle {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Rectangle {
            top_left: SerdeRPoint { x: x1, y: y1 },
            bottom_right: SerdeRPoint { x: x2, y: y2 },
            fill_color: None,
        }
    }

    pub fn with_color(x1: f64, y1: f64, x2: f64, y2: f64, color: String) -> Self {
        Rectangle {
            top_left: SerdeRPoint { x: x1, y: y1 },
            bottom_right: SerdeRPoint { x: x2, y: y2 },
            fill_color: Some(color),
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

/// Deeply nested structure for stress testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct DeepNest {
    pub level1: Level1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct Level1 {
    pub level2: Level2,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct Level2 {
    pub level3: Level3,
    pub values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct Level3 {
    pub data: Vec<f64>,
    pub flag: bool,
}

#[miniextendr]
impl DeepNest {
    pub fn new() -> Self {
        DeepNest {
            level1: Level1 {
                level2: Level2 {
                    level3: Level3 {
                        data: vec![1.0, 2.0, 3.0],
                        flag: true,
                    },
                    values: vec![10, 20, 30],
                },
                name: "nested".to_string(),
            },
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Struct with collections
// =============================================================================

/// Struct containing various collection types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct Collections {
    pub integers: Vec<i32>,
    pub floats: Vec<f64>,
    pub strings: Vec<String>,
    pub bools: Vec<bool>,
    pub points: Vec<SerdeRPoint>,
}

#[miniextendr]
impl Collections {
    pub fn new() -> Self {
        Collections {
            integers: vec![1, 2, 3, 4, 5],
            floats: vec![1.1, 2.2, 3.3],
            strings: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            bools: vec![true, false, true],
            points: vec![
                SerdeRPoint { x: 0.0, y: 0.0 },
                SerdeRPoint { x: 1.0, y: 1.0 },
            ],
        }
    }

    pub fn empty() -> Self {
        Collections {
            integers: vec![],
            floats: vec![],
            strings: vec![],
            bools: vec![],
            points: vec![],
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Struct with maps
// =============================================================================

/// Struct containing HashMap and BTreeMap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct Maps {
    pub string_to_int: HashMap<String, i32>,
    pub string_to_float: BTreeMap<String, f64>,
    pub metadata: HashMap<String, String>,
}

#[miniextendr]
impl Maps {
    pub fn new() -> Self {
        let mut string_to_int = HashMap::new();
        string_to_int.insert("one".to_string(), 1);
        string_to_int.insert("two".to_string(), 2);
        string_to_int.insert("three".to_string(), 3);

        let mut string_to_float = BTreeMap::new();
        string_to_float.insert("pi".to_string(), 3.14159);
        string_to_float.insert("e".to_string(), 2.71828);

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());

        Maps {
            string_to_int,
            string_to_float,
            metadata,
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Enum types
// =============================================================================

/// Simple unit enum (like R factor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

/// Enum with data variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

/// Struct containing enums.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct WithEnums {
    pub status: Status,
    pub shape: Shape,
    pub optional_status: Option<Status>,
}

#[miniextendr]
impl WithEnums {
    pub fn new_circle(radius: f64) -> Self {
        WithEnums {
            status: Status::Active,
            shape: Shape::Circle { radius },
            optional_status: Some(Status::Pending),
        }
    }

    pub fn new_rectangle(width: f64, height: f64) -> Self {
        WithEnums {
            status: Status::Inactive,
            shape: Shape::Rectangle { width, height },
            optional_status: None,
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Option/NA handling
// =============================================================================

/// Struct with optional fields for NA testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ExternalPtr)]
#[serde(crate = "crate::serde")]
pub struct WithOptionals {
    pub required_int: i32,
    pub optional_int: Option<i32>,
    pub optional_float: Option<f64>,
    pub optional_string: Option<String>,
    pub optional_bool: Option<bool>,
}

#[miniextendr]
impl WithOptionals {
    pub fn all_present() -> Self {
        WithOptionals {
            required_int: 42,
            optional_int: Some(100),
            optional_float: Some(3.14),
            optional_string: Some("hello".to_string()),
            optional_bool: Some(true),
        }
    }

    pub fn all_none() -> Self {
        WithOptionals {
            required_int: 0,
            optional_int: None,
            optional_float: None,
            optional_string: None,
            optional_bool: None,
        }
    }

    pub fn mixed() -> Self {
        WithOptionals {
            required_int: 42,
            optional_int: None,
            optional_float: Some(2.71828),
            optional_string: None,
            optional_bool: Some(false),
        }
    }

    pub fn to_r(&self) -> Result<SEXP, String> {
        to_r(self).map_err(|e: RSerdeError| e.to_string())
    }

    pub fn from_r(sexp: SEXP) -> Result<Self, String> {
        from_r(sexp).map_err(|e: RSerdeError| e.to_string())
    }
}

// =============================================================================
// Standalone test functions
// =============================================================================

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_i32(x: i32) -> SEXP {
    to_r(&x).expect("serialize i32")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_f64(x: f64) -> SEXP {
    to_r(&x).expect("serialize f64")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_bool(x: bool) -> SEXP {
    to_r(&x).expect("serialize bool")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_string(x: String) -> SEXP {
    to_r(&x).expect("serialize string")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_option_i32(x: Option<i32>) -> SEXP {
    to_r(&x).expect("serialize option i32")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_vec_i32(x: SEXP) -> SEXP {
    let v: Vec<i32> = from_r(x).expect("deserialize vec i32");
    to_r(&v).expect("serialize vec i32")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_vec_f64(x: SEXP) -> SEXP {
    let v: Vec<f64> = from_r(x).expect("deserialize vec f64");
    to_r(&v).expect("serialize vec f64")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_vec_string(x: SEXP) -> SEXP {
    let v: Vec<String> = from_r(x).expect("deserialize vec string");
    to_r(&v).expect("serialize vec string")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_vec_bool(x: SEXP) -> SEXP {
    let v: Vec<bool> = from_r(x).expect("deserialize vec bool");
    to_r(&v).expect("serialize vec bool")
}

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_hashmap() -> SEXP {
    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);
    to_r(&map).expect("serialize hashmap")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_i32(sexp: SEXP) -> i32 {
    from_r(sexp).expect("deserialize i32")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_f64(sexp: SEXP) -> f64 {
    from_r(sexp).expect("deserialize f64")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_string(sexp: SEXP) -> String {
    from_r(sexp).expect("deserialize string")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_vec_i32(sexp: SEXP) -> SEXP {
    let v: Vec<i32> = from_r(sexp).expect("deserialize vec i32");
    to_r(&v).expect("serialize result")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_vec_f64(sexp: SEXP) -> SEXP {
    let v: Vec<f64> = from_r(sexp).expect("deserialize vec f64");
    to_r(&v).expect("serialize result")
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_point(x: f64, y: f64) -> bool {
    let original = SerdeRPoint { x, y };
    let sexp = to_r(&original).expect("serialize");
    let restored: SerdeRPoint = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_rectangle(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    let original = Rectangle::new(x1, y1, x2, y2);
    let sexp = to_r(&original).expect("serialize");
    let restored: Rectangle = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_deep_nest() -> bool {
    let original = DeepNest::new();
    let sexp = to_r(&original).expect("serialize");
    let restored: DeepNest = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_collections() -> bool {
    let original = Collections::new();
    let sexp = to_r(&original).expect("serialize");
    let restored: Collections = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_optionals_present() -> bool {
    let original = WithOptionals::all_present();
    let sexp = to_r(&original).expect("serialize");
    let restored: WithOptionals = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_roundtrip_optionals_none() -> bool {
    let original = WithOptionals::all_none();
    let sexp = to_r(&original).expect("serialize");
    let restored: WithOptionals = from_r(sexp).expect("deserialize");
    original == restored
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_wrong_type(sexp: SEXP) -> Result<i32, String> {
    from_r::<i32>(sexp).map_err(|e: RSerdeError| e.to_string())
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_missing_field(sexp: SEXP) -> Result<SerdeRPoint, String> {
    from_r::<SerdeRPoint>(sexp).map_err(|e: RSerdeError| e.to_string())
}

// =============================================================================
// Tuple and tuple struct tests
// =============================================================================

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_tuple() -> SEXP {
    let tuple = (42i32, 3.14f64, "hello".to_string());
    to_r(&tuple).expect("serialize tuple")
}

/// Tuple struct for testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
pub struct Pair(pub i32, pub String);

/// @noRd
#[miniextendr]
pub fn serde_r_serialize_tuple_struct() -> SEXP {
    let pair = Pair(42, "answer".to_string());
    to_r(&pair).expect("serialize tuple struct")
}

// =============================================================================
// Complex integration tests
// =============================================================================

/// @noRd
#[miniextendr]
pub fn serde_r_complex_nested() -> SEXP {
    #[derive(Serialize)]
    #[serde(crate = "crate::serde")]
    struct Complex {
        points: Vec<SerdeRPoint>,
        rectangles: Vec<Rectangle>,
        metadata: HashMap<String, String>,
        counts: Vec<i32>,
        flags: Vec<bool>,
    }

    let mut metadata = HashMap::new();
    metadata.insert("name".to_string(), "complex".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());

    let complex = Complex {
        points: vec![
            SerdeRPoint { x: 0.0, y: 0.0 },
            SerdeRPoint { x: 1.0, y: 1.0 },
            SerdeRPoint { x: 2.0, y: 2.0 },
        ],
        rectangles: vec![
            Rectangle::new(0.0, 0.0, 1.0, 1.0),
            Rectangle::with_color(1.0, 1.0, 2.0, 2.0, "red".to_string()),
        ],
        metadata,
        counts: vec![1, 2, 3, 4, 5],
        flags: vec![true, false, true, true],
    };

    to_r(&complex).expect("serialize complex")
}

/// @noRd
#[miniextendr]
pub fn serde_r_deserialize_complex(sexp: SEXP) -> String {
    #[derive(Deserialize, Debug)]
    #[serde(crate = "crate::serde")]
    struct Complex {
        points: Vec<SerdeRPoint>,
        rectangles: Vec<Rectangle>,
        metadata: HashMap<String, String>,
        counts: Vec<i32>,
        flags: Vec<bool>,
    }

    match from_r::<Complex>(sexp) {
        Ok(c) => format!(
            "points={}, rects={}, meta={}, counts={:?}, flags={:?}",
            c.points.len(),
            c.rectangles.len(),
            c.metadata.len(),
            c.counts,
            c.flags
        ),
        Err(e) => format!("error: {}", e),
    }
}

// =============================================================================
// Module registration
// =============================================================================
