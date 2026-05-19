use super::*;

#[test]
fn test_direct_return() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__value, self)".to_string())
        .with_strategy(ReturnStrategy::Direct);

    let lines = builder.build();
    // .val <- <call>; if (inherits(.val, "rust_condition_value") && ...) return(...); .val
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(".val <- .Call(C_Counter__value, self)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert_eq!(lines[2], "  .val");
}

#[test]
fn test_return_self() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let lines = builder.build();
    // .val <- <call>; if (...) return(...); class(.val) <- "Counter"; .val
    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains(".val <- .Call(C_Counter__new)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert!(lines[2].contains("class(.val) <- \"Counter\""));
    assert_eq!(lines[3], "  .val");
}

#[test]
fn test_chainable_mutation() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__inc, self)".to_string())
        .with_strategy(ReturnStrategy::ChainableMutation);

    let lines = builder.build();
    // .val <- <call>; if (...) return(...); self
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains(".val <- .Call(C_Counter__inc, self)"));
    assert!(lines[1].contains("inherits(.val, \"rust_condition_value\")"));
    assert_eq!(lines[2], "  self");
}

#[test]
fn test_s7_style() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let inline = builder.build_s7_inline();
    // Inline block wraps in `{}` and routes the call through `.val` + the
    // condition-check guard before constructing the S7 object.
    assert!(inline.starts_with('{'));
    assert!(inline.contains(".val <- .Call(C_Counter__new)"));
    assert!(inline.contains("inherits(.val, \"rust_condition_value\")"));
    assert!(inline.contains("Counter(.ptr = .val)"));
}
