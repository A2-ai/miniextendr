//! The host writer validates class systems across macro expansion sites.
use miniextendr_api::{ExternalPtr, miniextendr};

mod declarations {
    #[miniextendr_api::miniextendr]
    pub trait WrongSystem {
        fn wrong_value(&self) -> i32;
    }

    #[miniextendr_api::miniextendr]
    pub trait Existing {
        fn existing_value(&self) -> i32;
    }

    #[miniextendr_api::miniextendr]
    pub trait S3Compatible {
        fn compatible_value(&self) -> i32;
    }
}

#[derive(Default, ExternalPtr)]
pub struct Host;

#[miniextendr(r6)]
impl Host {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Default, ExternalPtr)]
pub struct DefaultHost;

#[miniextendr]
impl DefaultHost {
    pub fn new() -> Self {
        Self
    }
}

#[path = "trait_class_systems/impls.rs"]
mod impls;

#[test]
fn writer_rejects_cross_file_mismatches_from_both_registration_paths() {
    let panic = std::panic::catch_unwind(miniextendr_api::registry::collect_r_wrappers)
        .expect_err("incompatible trait wrappers must be rejected before writing R");
    let message = panic.downcast_ref::<String>().expect("writer diagnostic");
    assert!(message.contains("WrongSystem"), "{message}");
    assert!(message.contains("Existing"), "{message}");
    assert!(
        message.contains("trait_class_systems/impls.rs:"),
        "{message}"
    );
    assert!(message.contains("trait_class_systems.rs:"), "{message}");
    assert!(message.contains("#[miniextendr(r6)]"), "{message}");
    assert!(!message.contains("S3Compatible"), "{message}");
    assert_eq!(message.lines().count(), 2, "{message}");
}

#[test]
fn bare_impls_register_the_resolved_feature_default() {
    use miniextendr_api::registry::{MX_CLASS_BLOCKS, MX_R_WRAPPERS};
    let expected = if cfg!(feature = "r6-default") {
        "r6"
    } else if cfg!(feature = "s7-default") {
        "s7"
    } else {
        "env"
    };
    let path = std::any::type_name::<DefaultHost>();
    let inherent = MX_CLASS_BLOCKS
        .iter()
        .find(|block| (block.type_path)() == path)
        .unwrap();
    assert_eq!(inherent.class_system, expected);
    let traits: Vec<_> = MX_R_WRAPPERS
        .iter()
        .filter_map(|entry| entry.trait_class.as_ref())
        .filter(|class| (class.type_path)() == path)
        .collect();
    assert_eq!(traits.len(), 2, "populated and empty impl registrations");
    for class in traits {
        assert_eq!(class.class_system, expected, "{}", class.trait_name);
    }
}
