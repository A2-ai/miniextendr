//! S3 generic targets: package qualification controls registration, never naming.

#[derive(Clone, Copy, Debug)]
pub(crate) struct S3Generic<'a> {
    pub(crate) package: Option<&'a str>,
    pub(crate) name: &'a str,
}

impl<'a> S3Generic<'a> {
    /// Split a target already validated by the attribute parser.
    pub(crate) fn new(target: &'a str) -> Self {
        match target.split_once("::") {
            Some((package, name)) => Self {
                package: Some(package),
                name,
            },
            None => Self {
                package: None,
                name: target,
            },
        }
    }

    pub(crate) fn method_name(self, class: &str) -> String {
        format!("{}.{class}", self.name)
    }

    /// A qualified export supplies roxygen's method metadata itself.
    pub(crate) fn method_tag(self, class: &str) -> Option<String> {
        self.package
            .is_none()
            .then(|| format!("@method {} {class}", self.name))
    }

    pub(crate) fn export_tag(self) -> String {
        match self.package {
            Some(package) => format!("@exportS3Method {package}::{}", self.name),
            None => "@export".to_owned(),
        }
    }

    pub(crate) fn roxygen(self, class: &str, register: bool) -> Vec<String> {
        self.method_tag(class)
            .into_iter()
            .chain(register.then(|| self.export_tag()))
            .map(|tag| format!("#' {tag}"))
            .collect()
    }
}

/// Keep operator names valid, but reject malformed package qualifications at
/// the literal rather than producing an unusable R wrapper or namespace.
pub(crate) fn parse(value: &syn::LitStr) -> syn::Result<String> {
    let target = value.value();
    if let Some((package, name)) = target.split_once("::")
        && (!crate::naming::is_r_package_name(package)
            || name.is_empty()
            || name.contains(':')
            || name.chars().any(char::is_whitespace))
    {
        return Err(syn::Error::new_spanned(
            value,
            "expected an S3 generic name or `package::generic` with a valid R package name and exactly one `::`",
        ));
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_names_keep_double_dispatch_and_operator_spelling() {
        for (target, class, expected) in [
            ("vctrs::vec_ptype2", "a.b", "vec_ptype2.a.b"),
            ("base::[<-", "a", "[<-.a"),
            ("utils::.DollarNames", "a", ".DollarNames.a"),
        ] {
            let literal = syn::LitStr::new(target, proc_macro2::Span::call_site());
            let parsed = parse(&literal).unwrap();
            let generic = S3Generic::new(&parsed);
            assert_eq!(generic.method_name(class), expected);
            assert_eq!(
                generic.roxygen(class, true),
                [format!("#' @exportS3Method {target}")]
            );
            assert!(generic.roxygen(class, false).is_empty());
        }
        assert_eq!(
            S3Generic::new("format").roxygen("a", true),
            ["#' @method format a", "#' @export"],
        );
    }

    #[test]
    fn malformed_qualified_names_are_rejected() {
        for target in [
            "::foo",
            "bad_pkg::foo",
            "1pkg::foo",
            "pkg.::foo",
            "pkg::",
            "pkg::foo::bar",
            "pkg:::foo",
            "pkg::foo bar",
        ] {
            assert!(
                parse(&syn::LitStr::new(target, proc_macro2::Span::call_site())).is_err(),
                "{target}"
            );
        }
    }
}
