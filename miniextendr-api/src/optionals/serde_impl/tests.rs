use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct TestStruct {
    name: String,
    value: i32,
    enabled: bool,
}

#[test]
fn rserialize_to_json() {
    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
        enabled: true,
    };
    let json = data.to_json().unwrap();
    assert_eq!(json, r#"{"name":"test","value":42,"enabled":true}"#);
}

#[test]
fn rserialize_to_json_pretty() {
    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
        enabled: true,
    };
    let json = data.to_json_pretty().unwrap();
    assert!(json.contains('\n'));
    assert!(json.contains("  ")); // Indentation
    assert!(json.contains("\"name\": \"test\""));
}

#[test]
fn rdeserialize_from_json() {
    let json = r#"{"name":"hello","value":123,"enabled":false}"#;
    let data: Option<TestStruct> = RDeserialize::from_json(json);
    assert!(data.is_some());
    let data = data.unwrap();
    assert_eq!(data.name, "hello");
    assert_eq!(data.value, 123);
    assert!(!data.enabled);
}

#[test]
fn rdeserialize_from_json_result() {
    let json = r#"{"name":"world","value":456,"enabled":true}"#;
    let result: Result<TestStruct, String> = RDeserialize::from_json_result(json);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.name, "world");
}

#[test]
fn rdeserialize_invalid_json() {
    let invalid = "not valid json";
    let data: Option<TestStruct> = RDeserialize::from_json(invalid);
    assert!(data.is_none());

    let result: Result<TestStruct, String> = RDeserialize::from_json_result(invalid);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expected"));
}

#[test]
fn rdeserialize_missing_field() {
    let json = r#"{"name":"test"}"#; // Missing value and enabled
    let data: Option<TestStruct> = RDeserialize::from_json(json);
    assert!(data.is_none());
}

#[test]
fn roundtrip() {
    let original = TestStruct {
        name: "roundtrip".to_string(),
        value: 999,
        enabled: true,
    };
    let json = original.to_json().unwrap();
    let parsed: TestStruct = RDeserialize::from_json(&json).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn serialize_vec() {
    let data = vec![1, 2, 3, 4, 5];
    let json = data.to_json().unwrap();
    assert_eq!(json, "[1,2,3,4,5]");
}

#[test]
fn deserialize_vec() {
    let json = "[10,20,30]";
    let data: Option<Vec<i32>> = RDeserialize::from_json(json);
    assert_eq!(data, Some(vec![10, 20, 30]));
}

#[test]
fn serialize_option() {
    let some_val: Option<i32> = Some(42);
    let none_val: Option<i32> = None;

    assert_eq!(some_val.to_json().unwrap(), "42");
    assert_eq!(none_val.to_json().unwrap(), "null");
}

#[test]
fn deserialize_option() {
    let data: Option<Option<i32>> = RDeserialize::from_json("42");
    assert_eq!(data, Some(Some(42)));

    let data: Option<Option<i32>> = RDeserialize::from_json("null");
    assert_eq!(data, Some(None));
}

#[test]
fn serialize_nested() {
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
    }
    #[derive(Serialize)]
    struct Inner {
        x: i32,
    }

    let data = Outer {
        inner: Inner { x: 100 },
    };
    let json = data.to_json().unwrap();
    assert_eq!(json, r#"{"inner":{"x":100}}"#);
}

#[test]
fn serialize_with_special_chars() {
    let data = TestStruct {
        name: "hello\nworld\t\"quoted\"".to_string(),
        value: 0,
        enabled: false,
    };
    let json = data.to_json().unwrap();
    // Special chars should be escaped
    assert!(json.contains("\\n"));
    assert!(json.contains("\\t"));
    assert!(json.contains("\\\""));
}

// region: JsonValue tests

#[test]
fn json_value_adapter_null() {
    let v = JsonValue::Null;
    assert!(RJsonValueOps::is_null(&v));
    assert!(!RJsonValueOps::is_boolean(&v));
    assert_eq!(RJsonValueOps::type_name(&v), "null");
}

#[test]
fn json_value_adapter_bool() {
    let v = JsonValue::Bool(true);
    assert!(RJsonValueOps::is_boolean(&v));
    assert_eq!(RJsonValueOps::as_bool(&v), Some(true));
}

#[test]
fn json_value_adapter_number() {
    let v = serde_json::json!(42);
    assert!(RJsonValueOps::is_number(&v));
    assert_eq!(RJsonValueOps::as_i64(&v), Some(42));
    assert_eq!(RJsonValueOps::as_f64(&v), Some(42.0));

    let v = serde_json::json!(std::f64::consts::PI);
    assert!(RJsonValueOps::is_number(&v));
    assert_eq!(RJsonValueOps::as_f64(&v), Some(std::f64::consts::PI));
}

#[test]
fn json_value_adapter_string() {
    let v = serde_json::json!("hello");
    assert!(RJsonValueOps::is_string(&v));
    assert_eq!(RJsonValueOps::as_str(&v), Some("hello".to_string()));
}

#[test]
fn json_value_adapter_array() {
    let v = serde_json::json!([1, 2, 3]);
    assert!(RJsonValueOps::is_array(&v));
    assert_eq!(RJsonValueOps::array_len(&v), Some(3));
}

#[test]
fn json_value_adapter_object() {
    let v = serde_json::json!({"a": 1, "b": 2});
    assert!(RJsonValueOps::is_object(&v));
    let keys = RJsonValueOps::object_keys(&v);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
}

#[test]
fn json_value_to_string() {
    let v = serde_json::json!({"x": 1});
    let s = RJsonValueOps::to_json_string(&v);
    assert!(s.contains("\"x\""));
    assert!(s.contains("1"));
}

#[test]
fn real_to_json_normal() {
    let opts = JsonOptions::default();
    let result = real_to_json(std::f64::consts::PI, &opts);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(RJsonValueOps::is_number(&val));
}

#[test]
fn real_to_json_nan_strict() {
    let opts = JsonOptions {
        nan: SpecialFloatHandling::Error,
        ..Default::default()
    };
    let result = real_to_json(f64::NAN, &opts);
    assert!(result.is_err());
}

/// A computed NA (`NA_real_ * 1`, bits `0x7FF8_0000_0000_07A2`) takes the NA
/// branch, not the NaN one.
#[test]
fn real_to_json_computed_na_is_na() {
    let opts = JsonOptions {
        na: NaHandling::String("NA".into()),
        nan: SpecialFloatHandling::String,
        ..Default::default()
    };
    let computed = f64::from_bits(0x7FF8_0000_0000_07A2);
    let na = real_to_json(computed, &opts).expect("NA is representable");
    assert_eq!(na, JsonValue::String("NA".into()));
    let nan = real_to_json(f64::NAN, &opts).expect("NaN is representable");
    assert_eq!(nan, JsonValue::String("NaN".into()));
}

#[test]
fn real_to_json_nan_permissive() {
    let opts = JsonOptions {
        nan: SpecialFloatHandling::Null,
        ..Default::default()
    };
    let result = real_to_json(f64::NAN, &opts);
    assert!(result.is_ok());
    assert!(RJsonValueOps::is_null(&result.unwrap()));
}

#[test]
fn real_to_json_inf_permissive() {
    let opts = JsonOptions {
        inf: SpecialFloatHandling::Null,
        ..Default::default()
    };
    let result = real_to_json(f64::INFINITY, &opts);
    assert!(result.is_ok());
    assert!(RJsonValueOps::is_null(&result.unwrap()));
}
// endregion

// region: NA_integer_ exclusion

#[test]
fn json_number_fits_i32_excludes_na_integer() {
    // i32::MIN is NA_integer_ in R - must not be treated as integer
    let n = serde_json::Number::from(i32::MIN as i64);
    assert!(!json_number_fits_i32(&n));

    // i32::MIN + 1 should still work
    let n = serde_json::Number::from(i32::MIN as i64 + 1);
    assert!(json_number_fits_i32(&n));

    // i32::MAX should work
    let n = serde_json::Number::from(i32::MAX as i64);
    assert!(json_number_fits_i32(&n));

    // i32::MAX + 1 should not fit
    let n = serde_json::Number::from(i32::MAX as i64 + 1);
    assert!(!json_number_fits_i32(&n));
}
// endregion

// region: json_discriminant

#[test]
fn discriminant_coverage() {
    assert_eq!(json_discriminant(&JsonValue::Null), 0);
    assert_eq!(json_discriminant(&JsonValue::Bool(true)), 1);
    assert_eq!(json_discriminant(&serde_json::json!(42)), 2);
    assert_eq!(json_discriminant(&serde_json::json!("hi")), 3);
    assert_eq!(json_discriminant(&serde_json::json!([1])), 4);
    assert_eq!(json_discriminant(&serde_json::json!({"a": 1})), 5);
}
// endregion

// region: RJsonBridge roundtrip tests (pure Rust, no R runtime)

#[test]
fn bridge_to_value_roundtrip() {
    // Test that struct -> Value -> struct roundtrips correctly
    let original = TestStruct {
        name: "bridge".to_string(),
        value: 42,
        enabled: true,
    };

    // Serialize to serde_json::Value
    let value = serde_json::to_value(&original).unwrap();
    assert!(value.is_object());
    let obj = value.as_object().unwrap();
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "bridge");
    assert_eq!(obj.get("value").unwrap().as_i64().unwrap(), 42);
    assert!(obj.get("enabled").unwrap().as_bool().unwrap());

    // Deserialize back
    let parsed: TestStruct = serde_json::from_value(value).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn bridge_nested_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Outer {
        label: String,
        inner: Inner,
        items: Vec<i32>,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        x: f64,
        y: f64,
    }

    let original = Outer {
        label: "point".into(),
        inner: Inner { x: 1.5, y: 2.5 },
        items: vec![10, 20, 30],
    };

    let value = serde_json::to_value(&original).unwrap();
    let parsed: Outer = serde_json::from_value(value).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn bridge_with_options() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        debug: Option<bool>,
        max_retries: Option<u32>,
    }

    let cfg = Config {
        name: "app".into(),
        debug: Some(true),
        max_retries: None,
    };

    let value = serde_json::to_value(&cfg).unwrap();
    let obj = value.as_object().unwrap();
    assert_eq!(obj.get("debug").unwrap(), &serde_json::json!(true));
    assert_eq!(obj.get("max_retries").unwrap(), &JsonValue::Null);

    let parsed: Config = serde_json::from_value(value).unwrap();
    assert_eq!(cfg, parsed);
}

#[test]
fn bridge_enum_variants() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(tag = "type")]
    enum Shape {
        Circle { radius: f64 },
        Rect { width: f64, height: f64 },
    }

    let circle = Shape::Circle { radius: 5.0 };
    let value = serde_json::to_value(&circle).unwrap();
    let parsed: Shape = serde_json::from_value(value).unwrap();
    assert_eq!(circle, parsed);

    let rect = Shape::Rect {
        width: 3.0,
        height: 4.0,
    };
    let value = serde_json::to_value(&rect).unwrap();
    let parsed: Shape = serde_json::from_value(value).unwrap();
    assert_eq!(rect, parsed);
}

#[test]
fn bridge_vec_of_structs() {
    let items = vec![
        TestStruct {
            name: "a".into(),
            value: 1,
            enabled: true,
        },
        TestStruct {
            name: "b".into(),
            value: 2,
            enabled: false,
        },
    ];

    let value = serde_json::to_value(&items).unwrap();
    assert!(value.is_array());
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 2);

    let parsed: Vec<TestStruct> = serde_json::from_value(value).unwrap();
    assert_eq!(items, parsed);
}
// endregion
