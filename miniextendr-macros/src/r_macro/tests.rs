use super::*;

fn parse(src: &str) -> (Option<TokenStream>, TokenStream) {
    parse_r_macro_input(src.parse().unwrap()).unwrap()
}

#[test]
fn env_head_is_split_off() {
    let (env, tail) = parse("env: e; x + 1");
    assert_eq!(env.unwrap().to_string(), "e");
    assert_eq!(tail.to_string(), "x + 1");
}

#[test]
fn double_colon_is_r_code_not_env_head() {
    // `env::foo()` is R namespace access on a package named `env`,
    // not the `env: <expr> ;` head.
    let (env, tail) = parse("env::foo()");
    assert!(env.is_none());
    assert_eq!(tail.to_string(), "env :: foo ()");
}

#[test]
fn env_head_without_semicolon_errors() {
    let err = parse_r_macro_input("env: e".parse().unwrap()).unwrap_err();
    assert!(err.to_string().contains("missing `;`"));
}
