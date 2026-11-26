use super::*;

#[test]
fn builds_parts_in_order() {
    let call = RCallValue::new("f")
        .arg(RCallPart::symbol("x"))
        .named_arg("n", 2)
        .named_arg("", "text")
        .arg(RCallValue::new("g"));
    assert!(matches!(call.function(), RCallPart::Symbol(name) if name == "f"));
    let args = call.args();
    assert_eq!(args.len(), 4);
    assert!(matches!(&args[0], (None, RCallPart::Symbol(name)) if name == "x"));
    assert!(matches!(
        &args[1],
        (Some(name), RCallPart::Value(RValue::Integer(v))) if name == "n" && v == &[Some(2)]
    ));
    // A string is a constant, and an empty name adds an unnamed argument.
    assert!(matches!(
        &args[2],
        (None, RCallPart::Value(RValue::Character(v))) if v == &[Some("text".to_owned())]
    ));
    assert!(matches!(&args[3], (None, RCallPart::Call(_))));
}

#[test]
fn unbuildable_names() {
    assert_eq!(
        RCallValue::new("f")
            .arg(RCallPart::symbol(""))
            .unbuildable(),
        None
    );

    let long = "x".repeat(MAX_NAME_BYTES + 1);
    let problem = RCallValue::new("f")
        .arg(RCallValue::new("g").arg(RCallPart::symbol(long.clone())))
        .unbuildable()
        .expect("a nested symbol over the limit");
    assert!(problem.contains("10001 bytes"), "{problem}");
    assert!(
        RCallValue::new("f")
            .named_arg(long, 1)
            .unbuildable()
            .is_some()
    );
    assert!(RCallValue::new("f\0").unbuildable().is_some());
    assert_eq!(
        RCallValue::new("x".repeat(MAX_NAME_BYTES)).unbuildable(),
        None
    );
}

#[test]
fn is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<RCallValue>();
    assert_send::<crate::condition::RCondition>();
}
