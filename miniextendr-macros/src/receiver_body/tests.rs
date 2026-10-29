use super::*;
use quote::quote;

#[test]
fn preserves_module_paths_and_nested_receivers() {
    let mut body: syn::Block = syn::parse_quote!({
        struct Local(i32);
        impl Local {
            fn read(&self) -> i32 {
                self.0
            }
        }
        use self::support::OFFSET;
        let read = || self.value;
        assert_eq!(self.value, self::support::value());
        self.value + read() + Local(OFFSET).read()
    });
    rewrite(&mut body, &syn::parse_quote!(__receiver));
    let expected: syn::Block = syn::parse_quote!({
        struct Local(i32);
        impl Local {
            fn read(&self) -> i32 {
                self.0
            }
        }
        use self::support::OFFSET;
        let read = || __receiver.value;
        assert_eq!(__receiver.value, self::support::value());
        __receiver.value + read() + Local(OFFSET).read()
    });
    assert_eq!(quote!(#body).to_string(), quote!(#expected).to_string());
}

#[test]
fn local_macros_can_capture_the_outer_receiver() {
    let mut body: syn::Block = syn::parse_quote!({
        macro_rules! read {
            () => {
                self.value + self::OFFSET
            };
        }
        read!()
    });
    rewrite(&mut body, &syn::parse_quote!(__receiver));
    let expected: syn::Block = syn::parse_quote!({
        macro_rules! read {
            () => {
                __receiver.value + self::OFFSET
            };
        }
        read!()
    });
    assert_eq!(quote!(#body).to_string(), quote!(#expected).to_string());
}
