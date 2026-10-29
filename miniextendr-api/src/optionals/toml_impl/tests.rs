use super::*;

#[test]
fn test_toml_from_str_simple() {
    let toml_str = "name = \"test\"\nvalue = 42";
    let value = toml_from_str(toml_str).unwrap();
    assert!(value.is_table());
}

#[test]
fn test_toml_from_str_invalid() {
    let result = toml_from_str("invalid toml [");
    assert!(result.is_err());
}

#[test]
fn test_toml_to_string() {
    // TOML requires a table at the top level, so test with a table
    let toml_str = r#"name = "hello""#;
    let value = toml_from_str(toml_str).unwrap();
    let s = toml_to_string(&value);
    assert!(s.contains("hello"));
    assert!(s.contains("name"));
}

#[test]
fn test_discriminant() {
    assert_eq!(discriminant(&TomlValue::String("a".into())), 0);
    assert_eq!(discriminant(&TomlValue::Integer(1)), 1);
    assert_eq!(discriminant(&TomlValue::Float(1.0)), 2);
    assert_eq!(discriminant(&TomlValue::Boolean(true)), 3);
}

#[test]
fn test_adapter_trait_types() {
    let s = TomlValue::String("test".into());
    assert!(RTomlOps::is_string(&s));
    assert!(!RTomlOps::is_integer(&s));
    assert_eq!(RTomlOps::type_name(&s), "string");

    let i = TomlValue::Integer(42);
    assert!(RTomlOps::is_integer(&i));
    assert_eq!(RTomlOps::as_integer(&i), Some(42));

    let f = TomlValue::Float(std::f64::consts::PI);
    assert!(RTomlOps::is_float(&f));
    assert_eq!(RTomlOps::as_float(&f), Some(std::f64::consts::PI));

    let b = TomlValue::Boolean(true);
    assert!(RTomlOps::is_boolean(&b));
    assert_eq!(RTomlOps::as_bool(&b), Some(true));
}

#[test]
fn test_adapter_trait_array() {
    let arr = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(2),
        TomlValue::Integer(3),
    ]);
    assert!(RTomlOps::is_array(&arr));
    assert_eq!(RTomlOps::array_len(&arr), Some(3));
}

#[test]
fn test_adapter_trait_table() {
    let toml_str = "alpha = 1\nbeta = 2";
    let value = toml_from_str(toml_str).unwrap();
    assert!(RTomlOps::is_table(&value));
    let keys = RTomlOps::table_keys(&value);
    assert!(keys.contains(&"alpha".to_string()));
    assert!(keys.contains(&"beta".to_string()));
}

#[test]
fn test_i64_precise_as_f64_boundary_positive() {
    // 2^53 is exactly representable as f64
    let result = i64_precise_as_f64(1i64 << 53);
    assert_eq!(result, (1i64 << 53) as f64);
}

#[test]
fn test_i64_precise_as_f64_boundary_negative() {
    // -(2^53) is exactly representable as f64
    let result = i64_precise_as_f64(-(1i64 << 53));
    assert_eq!(result, -(1i64 << 53) as f64);
}

#[test]
#[should_panic(expected = "outside R's lossless integer range")]
fn test_i64_precise_as_f64_overflow() {
    // 2^53 + 1 would lose precision — must panic
    let _ = i64_precise_as_f64((1i64 << 53) + 1);
}
