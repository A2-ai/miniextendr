use super::*;

#[test]
fn test_interaction_levels_generation() {
    let input: DeriveInput = syn::parse_quote! {
        #[r_factor(interaction = ["Small", "Large"])]
        enum ColorSize {
            Red(Size),
            Green(Size),
            Blue(Size),
        }
    };

    let result = derive_r_factor(input).unwrap();
    let code = result.to_string();

    // Check that combined levels are generated
    assert!(code.contains("Red.Small"));
    assert!(code.contains("Red.Large"));
    assert!(code.contains("Green.Small"));
    assert!(code.contains("Green.Large"));
    assert!(code.contains("Blue.Small"));
    assert!(code.contains("Blue.Large"));

    // Check that const assertion is generated for level validation
    assert!(code.contains("const _ : () ="));
    assert!(code.contains("ACTUAL"));
    assert!(code.contains("EXPECTED"));
    assert!(code.contains("inner type level count mismatch"));
}

#[test]
fn test_interaction_custom_separator() {
    let input: DeriveInput = syn::parse_quote! {
        #[r_factor(interaction = ["X", "Y"], sep = "_")]
        enum AB {
            A(Inner),
            B(Inner),
        }
    };

    let result = derive_r_factor(input).unwrap();
    let code = result.to_string();

    // Check custom separator
    assert!(code.contains("A_X"));
    assert!(code.contains("A_Y"));
    assert!(code.contains("B_X"));
    assert!(code.contains("B_Y"));
}

#[test]
fn test_interaction_with_rename() {
    let input: DeriveInput = syn::parse_quote! {
        #[r_factor(interaction = ["S", "L"], rename_all = "lower")]
        enum ColorSize {
            Red(Size),
            Green(Size),
        }
    };

    let result = derive_r_factor(input).unwrap();
    let code = result.to_string();

    // Check renamed outer levels
    assert!(code.contains("red.S"));
    assert!(code.contains("red.L"));
    assert!(code.contains("green.S"));
    assert!(code.contains("green.L"));
}
