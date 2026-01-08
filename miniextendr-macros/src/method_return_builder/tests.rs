use super::*;

#[test]
fn test_direct_return() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__value, self)".to_string())
        .with_strategy(ReturnStrategy::Direct);

    let lines = builder.build();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "    .Call(C_Counter__value, self)");
}

#[test]
fn test_return_self() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let lines = builder.build();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("result <-"));
    assert!(lines[1].contains("class(result)"));
    assert_eq!(lines[2], "    result");
}

#[test]
fn test_chainable_mutation() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__inc, self)".to_string())
        .with_strategy(ReturnStrategy::ChainableMutation);

    let lines = builder.build();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains(".Call"));
    assert_eq!(lines[1], "    self");
}

#[test]
fn test_s7_style() {
    let builder = MethodReturnBuilder::new(".Call(C_Counter__new)".to_string())
        .with_strategy(ReturnStrategy::ReturnSelf)
        .with_class_name("Counter".to_string());

    let inline = builder.build_s7_inline();
    assert_eq!(inline, "Counter(.ptr = .Call(C_Counter__new))");
}
