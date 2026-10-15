//! Instantiate omitted default-method signatures for concrete trait wrappers.

use std::collections::HashMap;

use syn::visit_mut::VisitMut;

/// Default bodies stay on the trait. These synthetic impl methods are consumed
/// only by wrapper/shim generation, never emitted as Rust overrides.
pub(super) fn inherited_methods(
    implementation: &syn::ItemImpl,
    declaration: &[syn::TraitItemFn],
    generics: &syn::Generics,
    trait_path: &syn::Path,
) -> syn::Result<Vec<syn::ImplItemFn>> {
    let missing: Vec<_> = declaration
        .iter()
        .filter(|method| {
            method.default.is_some()
                && !implementation.items.iter().any(|item| {
                    matches!(item, syn::ImplItem::Fn(own) if own.sig.ident == method.sig.ident)
                })
        })
        .collect();
    if missing.is_empty() {
        return Ok(Vec::new());
    }

    let mut arguments = Substitution::new(generics, trait_path)?;
    arguments.self_type = Some((*implementation.self_ty).clone());
    arguments.trait_path = Some(trait_path.clone());
    missing
        .into_iter()
        .map(|method| {
            let mut sig = method.sig.clone();
            arguments.visit_signature_mut(&mut sig);
            let attrs = &method.attrs;
            syn::parse2(quote::quote! { #(#attrs)* #sig {} })
        })
        .collect()
}

#[derive(Default)]
struct Substitution {
    types: HashMap<syn::Ident, syn::Type>,
    consts: HashMap<syn::Ident, syn::Expr>,
    lifetimes: HashMap<syn::Lifetime, syn::Lifetime>,
    self_type: Option<syn::Type>,
    trait_path: Option<syn::Path>,
}

impl Substitution {
    fn new(generics: &syn::Generics, trait_path: &syn::Path) -> syn::Result<Self> {
        let supplied: Vec<_> = match &trait_path.segments.last().unwrap().arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.iter().collect(),
            _ => Vec::new(),
        };
        let mut supplied = supplied.into_iter().peekable();
        let mut result = Self::default();
        for param in &generics.params {
            match param {
                syn::GenericParam::Type(param) => {
                    let ty = match supplied.next() {
                        Some(syn::GenericArgument::Type(ty)) => ty.clone(),
                        None if param.default.is_some() => {
                            let mut ty = param.default.clone().unwrap();
                            // Only declaration defaults refer to earlier parameters;
                            // supplied arguments already belong to the impl's scope.
                            result.visit_type_mut(&mut ty);
                            ty
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                trait_path,
                                format!(
                                    "expected a concrete argument for trait type parameter `{}`",
                                    param.ident,
                                ),
                            ));
                        }
                    };
                    result.types.insert(param.ident.clone(), ty);
                }
                syn::GenericParam::Const(param) => {
                    let value = match supplied.next() {
                        Some(syn::GenericArgument::Const(value)) => value.clone(),
                        // syn parses a bare const name as a Type until its
                        // declaration tells us which kind of parameter it is.
                        Some(syn::GenericArgument::Type(ty)) => syn::parse2(quote::quote!(#ty))?,
                        None if param.default.is_some() => {
                            let mut value = param.default.clone().unwrap();
                            result.visit_expr_mut(&mut value);
                            value
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                trait_path,
                                format!(
                                    "expected a concrete argument for trait const parameter `{}`",
                                    param.ident,
                                ),
                            ));
                        }
                    };
                    result.consts.insert(param.ident.clone(), value);
                }
                syn::GenericParam::Lifetime(param) => {
                    let lifetime = match supplied.peek() {
                        Some(syn::GenericArgument::Lifetime(_)) => {
                            let Some(syn::GenericArgument::Lifetime(value)) = supplied.next()
                            else {
                                unreachable!()
                            };
                            value.clone()
                        }
                        _ => syn::parse_quote!('_),
                    };
                    result.lifetimes.insert(param.lifetime.clone(), lifetime);
                }
            }
        }
        Ok(result)
    }
}

impl VisitMut for Substitution {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        if let syn::Type::Path(path) = ty
            && path.qself.is_none()
            && path.path.segments.len() > 1
            && path.path.segments[0].ident == "Self"
            && self.self_type.is_some()
            && self.trait_path.is_some()
        {
            let mut associated = path.path.clone();
            associated.segments = associated.segments.into_iter().skip(1).collect();
            self.visit_path_mut(&mut associated);
            let concrete = self.self_type.as_ref().unwrap();
            let trait_path = self.trait_path.as_ref().unwrap();
            *ty = syn::parse_quote!(<#concrete as #trait_path>::#associated);
            return;
        }
        if let syn::Type::Path(path) = ty
            && path.qself.is_none()
            && let Some(name) = path.path.get_ident()
            && let Some(concrete) = self.types.get(name)
        {
            *ty = concrete.clone();
            return;
        }
        syn::visit_mut::visit_type_mut(self, ty);
    }

    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if let syn::Expr::Path(path) = expr
            && path.qself.is_none()
            && let Some(name) = path.path.get_ident()
            && let Some(value) = self.consts.get(name)
        {
            *expr = value.clone();
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_generic_argument_mut(&mut self, arg: &mut syn::GenericArgument) {
        // `Wrapper<N>` parses N as a type, but a const parameter's declaration
        // tells us it must become a const argument after substitution.
        if let syn::GenericArgument::Type(syn::Type::Path(path)) = arg
            && let Some(name) = path.path.get_ident()
            && let Some(value) = self.consts.get(name)
        {
            *arg = syn::parse_quote!({ #value });
            return;
        }
        syn::visit_mut::visit_generic_argument_mut(self, arg);
    }

    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if let Some(concrete) = self.lifetimes.get(lifetime) {
            *lifetime = concrete.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn nested_types_consts_lifetimes_and_parameter_defaults() {
        let generics = syn::parse_quote!(<'a, T, const N: usize, U = Vec<T>>);
        let path = syn::parse_quote!(Example<'static, i32, 3>);
        let mut sub = Substitution::new(&generics, &path).unwrap();
        let mut ty: syn::Type = syn::parse_quote!((&'a [T; N], U, SizedBuffer<N>));
        sub.visit_type_mut(&mut ty);
        assert_eq!(
            quote!(#ty).to_string(),
            quote!((&'static [i32; 3], Vec<i32>, SizedBuffer<{ 3 }>)).to_string()
        );
    }

    #[test]
    fn caller_types_are_not_reinterpreted_as_declaration_parameters() {
        let generics = syn::parse_quote!(<T, U>);
        let path = syn::parse_quote!(Example<i32, T>);
        let mut sub = Substitution::new(&generics, &path).unwrap();
        let mut ty: syn::Type = syn::parse_quote!((T, U));
        sub.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), quote!((i32, T)).to_string());
    }

    #[test]
    fn only_omitted_defaults_get_synthetic_signatures() {
        let implementation = syn::parse_quote!(impl Example<i32> for Host {
            fn own(&self) -> i32 { 9 }
        });
        let declaration = vec![
            syn::parse_quote!(
                fn own(&self) -> T {}
            ),
            syn::parse_quote!(
                fn required(&self) -> T;
            ),
            syn::parse_quote!(
                #[miniextendr(r_name = "values")]
                fn inherited(&self, x: Vec<T>) -> T {}
            ),
        ];
        let methods = inherited_methods(
            &implementation,
            &declaration,
            &syn::parse_quote!(<T>),
            &syn::parse_quote!(Example<i32>),
        )
        .unwrap();
        assert_eq!(methods.len(), 1);
        let method = &methods[0];
        assert_eq!(
            quote!(#method).to_string(),
            quote!(
                #[miniextendr(r_name = "values")]
                fn inherited(&self, x: Vec<i32>) -> i32 {}
            )
            .to_string()
        );
    }
}
