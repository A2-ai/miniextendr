use super::*;

#[test]
fn env_wrappers_preserve_static_params() {
    let attrs = ImplAttrs {
        class_system: ClassSystem::Env,
        class_name: None,
        label: None,
    };

    let item_impl: syn::ItemImpl = syn::parse_quote! {
        impl ReceiverCounter {
            pub fn new(initial: i32) -> Self {
                unimplemented!()
            }

            pub fn add(&self, amount: i32) -> i32 {
                amount
            }

            pub fn default_counter(step: i32) -> Self {
                unimplemented!()
            }
        }
    };

    let parsed = ParsedImpl::parse(attrs, item_impl).expect("failed to parse impl");
    let wrapper = generate_env_r_wrapper(&parsed);

    assert!(wrapper.contains("ReceiverCounter$new <- function(initial)"));
    assert!(wrapper.contains("ReceiverCounter$add <- function(amount)"));
    assert!(wrapper.contains("ReceiverCounter$default_counter <- function(step)"));
}
