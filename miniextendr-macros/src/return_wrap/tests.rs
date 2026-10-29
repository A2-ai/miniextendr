use super::*;
use quote::ToTokens;

#[test]
fn marker_and_attribute_share_each_system_and_container_plan() {
    for (marker, system) in [
        ("WrapAsR6", ClassSystem::R6),
        ("WrapAsS7", ClassSystem::S7),
        ("WrapAsS4", ClassSystem::S4),
        ("WrapAsS3", ClassSystem::S3),
        ("WrapAsEnv", ClassSystem::Env),
        ("WrapAsVctrs", ClassSystem::Vctrs),
    ] {
        for shape in [
            "{}",
            "Option<{}>",
            "Result<{}, String>",
            "Vec<{}>",
            "Option<Vec<{}>>",
            "Result<Vec<{}>, String>",
        ] {
            let typed: ReturnType = syn::parse_str(&format!(
                "-> {}",
                shape.replace("{}", &format!("miniextendr_api::{marker}<Board>"))
            ))
            .unwrap();
            let plain: ReturnType =
                syn::parse_str(&format!("-> {}", shape.replace("{}", "Board"))).unwrap();
            let (a, output) = resolve(&typed, None).unwrap();
            let (b, _) = resolve(&plain, Some(system)).unwrap();
            assert_eq!(
                output.to_token_stream().to_string(),
                plain.to_token_stream().to_string()
            );
            assert_eq!(
                a.unwrap().r_expression(".val", None),
                b.unwrap().r_expression(".val", None)
            );
        }
    }
}
/// An `Option` return is `NULL` for `None` (#2088): the wrap lets it through.
#[test]
fn optional_wraps_let_null_through() {
    for (shape, guarded) in [
        ("{}", false),
        ("Option<{}>", true),
        ("Result<{}, String>", false),
        ("Vec<{}>", false),
        ("Option<Vec<{}>>", true),
        ("Result<Vec<{}>, String>", false),
    ] {
        let output: ReturnType =
            syn::parse_str(&format!("-> {}", shape.replace("{}", "WrapAsR6<Board>"))).unwrap();
        let (plan, _) = resolve(&output, None).unwrap();
        let plan = plan.unwrap();
        assert_eq!(plan.is_optional_class(), shape == "Option<{}>", "{shape}");
        let expr = plan.r_expression(".val", None);
        assert_eq!(
            expr.starts_with("if (is.null(.val)) NULL else "),
            guarded,
            "{shape}: {expr}"
        );
    }
}

#[test]
fn rejects_misplaced_markers_and_conflicting_systems() {
    for (output, message) in [
        ("-> WrapAsR6<Vec<Board>>", "closest to the named class"),
        (
            "-> Result<Board, WrapAsR6<Problem>>",
            "successful Result payload",
        ),
        (
            "-> Option<Option<WrapAsR6<Board>>>",
            "supports T, Option<T>",
        ),
        ("-> Result<WrapAsR6<Board>, ()>", "non-unit error type"),
        (
            "-> (WrapAsR6<Board>, i32)",
            "directly around the class payload",
        ),
        ("-> WrapAsR6<&Board>", "owned, named class type"),
        ("-> WrapAsR6<WrapAsS7<Board>>", "closest to the named class"),
    ] {
        let output = syn::parse_str(output).unwrap();
        assert!(
            resolve(&output, None)
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
    let output = syn::parse_quote!(-> WrapAsR6<Board>);
    assert!(
        resolve(&output, Some(ClassSystem::S7))
            .unwrap_err()
            .to_string()
            .contains("different class systems")
    );
    assert!(resolve(&output, Some(ClassSystem::R6)).is_ok());
}

/// An `ExternalPtr<Class>` payload wraps as `Class` in both spellings and
/// keeps the handle as the Rust return type, like inferred wrapping (#1375).
#[test]
fn external_ptr_payload_names_its_class() {
    for (output, attribute, peeled) in [
        (
            "-> WrapAsR6<ExternalPtr<Board>>",
            None,
            "-> ExternalPtr<Board>",
        ),
        (
            "-> ExternalPtr<Board>",
            Some(ClassSystem::R6),
            "-> ExternalPtr<Board>",
        ),
        (
            "-> Option<Vec<WrapAsR6<ExternalPtr<Board>>>>",
            None,
            "-> Option<Vec<ExternalPtr<Board>>>",
        ),
    ] {
        let (plan, output) = resolve(&syn::parse_str(output).unwrap(), attribute).unwrap();
        let expected: ReturnType = syn::parse_str(peeled).unwrap();
        assert_eq!(
            output.to_token_stream().to_string(),
            expected.to_token_stream().to_string()
        );
        assert!(
            plan.unwrap()
                .r_expression(".val", None)
                .contains("Board$new(.ptr = ")
        );
    }
}

#[test]
fn explicit_self_and_qualified_targets_do_not_use_registry_placeholders() {
    for (output, target) in [
        ("-> WrapAsR6<Self>", "ActualClass"),
        ("-> WrapAsR6<other::Board>", "Board"),
    ] {
        let (plan, _) = resolve(&syn::parse_str(output).unwrap(), None).unwrap();
        assert_eq!(
            plan.unwrap().r_expression(".val", Some("ActualClass")),
            format!("{target}$new(.ptr = .val)")
        );
    }
}
