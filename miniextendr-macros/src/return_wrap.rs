//! One explicit class-wrapping decision for marker and attribute spellings.

use crate::miniextendr_impl::ClassSystem;
use crate::type_inspect::first_type_argument;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericArgument, PathArguments, ReturnType, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Container {
    Option,
    Result,
    Vec,
}

#[derive(Debug, Clone)]
pub struct ReturnWrap {
    system: ClassSystem,
    target: String,
    containers: Vec<Container>,
    typed: bool,
}

pub(crate) fn parse_system(value: &syn::LitStr) -> syn::Result<ClassSystem> {
    value
        .value()
        .parse()
        .map_err(|message| syn::Error::new(value.span(), message))
}

fn marker_system(name: &syn::Ident) -> Option<ClassSystem> {
    Some(match name.to_string().as_str() {
        "WrapAsR6" => ClassSystem::R6,
        "WrapAsS7" => ClassSystem::S7,
        "WrapAsS4" => ClassSystem::S4,
        "WrapAsS3" => ClassSystem::S3,
        "WrapAsEnv" => ClassSystem::Env,
        "WrapAsVctrs" => ClassSystem::Vctrs,
        _ => return None,
    })
}

pub(crate) fn contains_marker(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path.path.segments.iter().any(|segment| {
            marker_system(&segment.ident).is_some()
                || match &segment.arguments {
                    PathArguments::AngleBracketed(args) => args
                        .args
                        .iter()
                        .any(|arg| matches!(arg, GenericArgument::Type(ty) if contains_marker(ty))),
                    _ => false,
                }
        }),
        Type::Reference(ty) => contains_marker(&ty.elem),
        Type::Paren(ty) => contains_marker(&ty.elem),
        Type::Group(ty) => contains_marker(&ty.elem),
        Type::Slice(ty) => contains_marker(&ty.elem),
        Type::Array(ty) => contains_marker(&ty.elem),
        Type::Tuple(ty) => ty.elems.iter().any(contains_marker),
        _ => false,
    }
}

/// Peel a marker under the supported boundary containers, or select the same
/// plan from `wrap = "..."`. Visibility has already been peeled by the caller.
pub(crate) fn resolve(
    output: &ReturnType,
    attribute: Option<ClassSystem>,
) -> syn::Result<(Option<ReturnWrap>, ReturnType)> {
    let ReturnType::Type(arrow, ty) = output else {
        if attribute.is_some() {
            return Err(syn::Error::new_spanned(
                output,
                "`wrap` requires a named class return value",
            ));
        }
        return Ok((None, output.clone()));
    };
    if attribute.is_none() && !contains_marker(ty) {
        return Ok((None, output.clone()));
    }
    let mut leaf = ty.as_ref();
    let mut containers = Vec::new();
    loop {
        let Type::Path(path) = leaf else { break };
        let segment = path.path.segments.last().expect("type path has a segment");
        let container = match segment.ident.to_string().as_str() {
            "Option" => Container::Option,
            "Result" => Container::Result,
            "Vec" => Container::Vec,
            _ => break,
        };
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            break;
        };
        if container == Container::Result {
            if let Some(GenericArgument::Type(error)) = args.args.iter().nth(1) {
                if contains_marker(error) {
                    return Err(syn::Error::new_spanned(
                        error,
                        "WrapAs* belongs on the successful Result payload",
                    ));
                }
                if matches!(error, Type::Tuple(tuple) if tuple.elems.is_empty()) {
                    return Err(syn::Error::new_spanned(
                        error,
                        "explicit class wrapping does not support Result<T, ()>; use a non-unit error type",
                    ));
                }
            }
        }
        let Some(GenericArgument::Type(inner)) = args.args.first() else {
            break;
        };
        containers.push(container);
        leaf = inner;
    }
    if !matches!(
        containers.as_slice(),
        [] | [Container::Option]
            | [Container::Result]
            | [Container::Vec]
            | [Container::Option, Container::Vec]
            | [Container::Result, Container::Vec]
    ) {
        return Err(syn::Error::new_spanned(
            ty,
            "explicit class wrapping supports T, Option<T>, Result<T, E>, Vec<T>, Option<Vec<T>>, or Result<Vec<T>, E>",
        ));
    }
    let marker = if let Type::Path(path) = &*leaf {
        path.path
            .segments
            .last()
            .and_then(|segment| marker_system(&segment.ident).map(|system| (system, segment)))
    } else {
        None
    };
    let typed = marker.is_some();
    let (system, payload) = match marker {
        Some((system, segment)) => {
            if attribute.is_some_and(|attribute| attribute != system) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "the WrapAs* marker and `wrap` attribute select different class systems",
                ));
            }
            let inner = first_type_argument(segment)
                .ok_or_else(|| {
                    syn::Error::new_spanned(segment, "WrapAs* requires one class type argument")
                })?
                .clone();
            if !matches!(&segment.arguments, PathArguments::AngleBracketed(args) if args.args.len() == 1)
            {
                return Err(syn::Error::new_spanned(
                    segment,
                    "WrapAs* requires exactly one class type argument",
                ));
            }
            (system, inner)
        }
        None => (attribute.ok_or_else(|| syn::Error::new_spanned(ty, "place WrapAs* directly around the class payload, inside Option, Result, or Vec"))?, leaf.clone()),
    };
    let leaf = &payload;
    let Type::Path(path) = &*leaf else {
        return Err(syn::Error::new_spanned(
            leaf,
            "explicit wrapping requires an owned, named class type",
        ));
    };
    let segment = path.path.segments.last().expect("type path has a segment");
    if !matches!(segment.arguments, PathArguments::None) || contains_marker(leaf) {
        return Err(syn::Error::new_spanned(
            leaf,
            "WrapAs* must be closest to the named class type; put containers and visibility outside it",
        ));
    }
    let target = crate::naming::ident_name(&segment.ident);
    // Validate through shared borrows above, then replace the known payload slot.
    let mut peeled = (**ty).clone();
    let mut slot = &mut peeled;
    for _ in &containers {
        let Type::Path(path) = slot else {
            unreachable!("validated container path")
        };
        let PathArguments::AngleBracketed(args) = &mut path
            .path
            .segments
            .last_mut()
            .expect("validated path")
            .arguments
        else {
            unreachable!("validated container arguments")
        };
        let GenericArgument::Type(inner) = args.args.first_mut().expect("validated payload") else {
            unreachable!("validated type argument")
        };
        slot = inner;
    }
    *slot = payload;

    Ok((
        Some(ReturnWrap {
            system,
            target,
            containers,
            typed,
        }),
        ReturnType::Type(*arrow, Box::new(peeled)),
    ))
}

impl ReturnWrap {
    /// Unwrap only syntactic marker values; attributes keep the user's value.
    pub(crate) fn prepare_value(&self, call: TokenStream) -> TokenStream {
        fn map(call: TokenStream, containers: &[Container]) -> TokenStream {
            match containers.split_first() {
                None => quote!((#call).0),
                Some((container, rest)) => {
                    let inner = map(quote!(__mx_value), rest);
                    if *container == Container::Vec {
                        quote!((#call).into_iter().map(|__mx_value| #inner).collect::<::std::vec::Vec<_>>())
                    } else {
                        quote!((#call).map(|__mx_value| #inner))
                    }
                }
            }
        }
        if self.typed {
            map(call, &self.containers)
        } else {
            call
        }
    }

    /// Emit the chosen constructor directly; no write-time registry lookup.
    pub(crate) fn r_expression(&self, value: &str, self_type: Option<&str>) -> String {
        let target = if self.target == "Self" {
            self_type.unwrap_or("Self")
        } else {
            &self.target
        };
        let wrap = |value: &str| match self.system {
            ClassSystem::R6 => format!("{target}$new(.ptr = {value})"),
            ClassSystem::S7 => format!("{target}(.ptr = {value})"),
            ClassSystem::S4 => format!("methods::new(\"{target}\", ptr = {value})"),
            ClassSystem::S3 | ClassSystem::Env => {
                format!("structure({value}, class = \"{target}\")")
            }
            ClassSystem::Vctrs => {
                format!("structure({value}, class = unique(c(\"{target}\", class({value}))))")
            }
        };
        let result = if self.containers.contains(&Container::Vec) {
            format!("lapply({value}, function(.item) {})", wrap(".item"))
        } else {
            wrap(value)
        };
        if self.containers.first() == Some(&Container::Option) {
            format!("if (is.null({value})) NULL else {result}")
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
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
}
