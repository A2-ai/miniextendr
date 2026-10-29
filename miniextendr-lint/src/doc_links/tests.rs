use super::*;

fn links(doc: &[&str]) -> Vec<(String, String, bool)> {
    let attrs: Vec<Attribute> = doc.iter().map(|l| syn::parse_quote!(#[doc = #l])).collect();
    pkg_links(&attrs)
        .into_iter()
        .map(|l| (l.written, l.pkg, l.leading_prose))
        .collect()
}

#[test]
fn finds_every_link_form_and_skips_the_rest() {
    let found = links(&[
        " See [`Sources::prepare`] and [Type::method()].",
        " [text][dplyr::bind_rows], [t][crate::x], [`a::b::c`], [x](y::z)",
        " `[code::span]` and \\[esc::aped] and [plain]",
    ]);
    let pkgs: Vec<_> = found.iter().map(|f| f.1.as_str()).collect();
    assert_eq!(pkgs, ["Sources", "Type", "dplyr"]);
    assert!(found.iter().all(|f| f.2), "all leading prose");
}

#[test]
fn skips_code_tags_fences_and_lines_after_single_line_tags() {
    let found = links(&[
        " @details Use [aa::b] here",
        " and [cc::d] there.",
        " ```",
        " [ee::f]",
        " ```",
        " @examples",
        " [gg::h]",
        " @export",
        " Rust callers: [ii::j]",
        " @param x see [kk::l]",
    ]);
    let pkgs: Vec<_> = found.iter().map(|f| f.1.as_str()).collect();
    assert_eq!(pkgs, ["aa", "cc", "kk"]);
    assert!(found.iter().all(|f| !f.2), "all in tags");
}
