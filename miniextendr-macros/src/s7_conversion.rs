//! Feed S7 conversion markers and attributes into the same registration plan.

use crate::miniextendr_impl::{ClassSystem, ParsedMethod, ReceiverKind};
use crate::return_wrap::ConversionKind;
use syn::{GenericArgument, PathArguments, Type};

pub(crate) fn configure(
    method: &mut ParsedMethod,
    original: &syn::ImplItemFn,
    system: ClassSystem,
    implementing_type: &syn::Ident,
) -> syn::Result<()> {
    let marker = method.return_wrap.as_ref().and_then(|wrap| wrap.conversion);
    let attrs = &mut method.method_attrs;
    if marker.is_none() && attrs.s7.convert_to.is_none() && attrs.s7.convert_from.is_none() {
        return Ok(());
    }
    let error = |message| syn::Error::new_spanned(&original.sig, message);
    if system != ClassSystem::S7 {
        return Err(error(
            "S7 conversion markers and attributes require an inherent S7 impl",
        ));
    }
    if attrs.serialize || attrs.unwrap_in_r {
        return Err(error(
            "S7 class conversion cannot be combined with serialize or unwrap_in_r",
        ));
    }
    if attrs.s7.convert_to.is_some() && attrs.s7.convert_from.is_some() {
        return Err(error(
            "cannot specify both `convert_from` and `convert_to` on the same method; convert_from is for static methods, convert_to is for instance methods",
        ));
    }
    let kind = marker.unwrap_or_else(|| {
        if attrs.s7.convert_to.is_some() {
            ConversionKind::To
        } else {
            ConversionKind::From
        }
    });
    let params: Vec<_> = original
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(param) = arg {
                Some(param)
            } else {
                None
            }
        })
        .collect();
    match kind {
        ConversionKind::To => {
            if method.env == ReceiverKind::None || !params.is_empty() {
                return Err(error(
                    "convert_to requires an instance method with no additional parameters",
                ));
            }
            if attrs.s7.convert_from.is_some() {
                return Err(error("ConvertTo disagrees with the convert_from attribute"));
            }
        }
        ConversionKind::From => {
            if method.env != ReceiverKind::None || params.len() != 1 {
                return Err(error(
                    "convert_from requires a static method with exactly one source parameter",
                ));
            }
            if attrs.s7.convert_to.is_some() {
                return Err(error("ConvertFrom disagrees with the convert_to attribute"));
            }
        }
    }
    if method.return_wrap.is_none() {
        method.return_wrap =
            crate::return_wrap::resolve(&method.sig.output, Some(ClassSystem::S7))?.0;
    }
    let wrap = method
        .return_wrap
        .as_mut()
        .expect("a conversion has a named return payload");
    if wrap.system() != ClassSystem::S7 {
        return Err(error(
            "S7 conversion cannot select another return class system",
        ));
    }
    if wrap.has_vector() {
        return Err(error(
            "S7 conversion returns one class instance, not a vector of class instances",
        ));
    }
    match kind {
        ConversionKind::To => {
            if marker.is_some() {
                let target = if wrap.target() == "Self" {
                    crate::naming::ident_name(implementing_type)
                } else {
                    wrap.target().to_owned()
                };
                if attrs
                    .s7
                    .convert_to
                    .as_ref()
                    .is_some_and(|name| name != &target)
                {
                    return Err(error("ConvertTo target and convert_to attribute disagree"));
                }
                attrs.s7.convert_to = Some(target);
            }
            let target = attrs
                .s7
                .convert_to
                .as_ref()
                .expect("convert_to target was selected");
            wrap.set_target(crate::r_class_formatter::class_ref_or_verbatim(target));
        }
        ConversionKind::From => {
            if wrap.target() != "Self"
                && wrap.target() != crate::naming::ident_name(implementing_type)
            {
                return Err(error(
                    "convert_from must return Self or the enclosing class type",
                ));
            }
            if marker.is_some() {
                let source = source_class(&params[0].ty).ok_or_else(|| error("ConvertFrom needs a named source class parameter, such as ExternalPtr<Source> or &Source"))?;
                let source = if source == "Self" {
                    crate::naming::ident_name(implementing_type)
                } else {
                    source
                };
                if attrs
                    .s7
                    .convert_from
                    .as_ref()
                    .is_some_and(|name| name != &source)
                {
                    return Err(error(
                        "ConvertFrom source parameter and convert_from attribute disagree",
                    ));
                }
                attrs.s7.convert_from = Some(source);
            }
            wrap.set_target("Self".to_owned());
        }
    }
    wrap.conversion = Some(kind);
    Ok(())
}

fn source_class(ty: &Type) -> Option<String> {
    if let Type::Reference(reference) = ty {
        return source_class(&reference.elem);
    }
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident == "ExternalPtr" {
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        let GenericArgument::Type(inner) = args.args.first()? else {
            return None;
        };
        return source_class(inner);
    }
    if !matches!(segment.arguments, PathArguments::None) {
        return None;
    }
    Some(crate::naming::ident_name(&segment.ident))
}
