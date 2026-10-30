use super::*;
use crate::match_arg::MatchArgAlias;

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn optional_preamble_helpers_follow_fragment_dependencies() {
    use std::borrow::Cow;
    let render = |fragment: &'static str| {
        let mut text = String::new();
        super::append_used_wrapper_helpers(&mut text, &[Cow::Borrowed(fragment)]);
        text
    };
    assert!(render("f <- function() .Call(C_f)").is_empty());

    let caller = render("f <- function() .miniextendr_caller_call()");
    assert!(caller.contains(".miniextendr_caller_call <- function"));
    assert!(caller.contains(".miniextendr_arg_error <- function"));
    assert!(!caller.contains(".miniextendr_field_at <- function"));
    assert!(!caller.contains(".miniextendr_env_method <- function"));
    assert!(!caller.contains(".miniextendr_match_arg"));

    let check = render("f <- function(x) .miniextendr_arg_error('x', 'must be positive')");
    assert!(check.contains(".miniextendr_arg_error <- function"));
    assert!(!check.contains(".miniextendr_match_arg"));

    let field = render("f <- function(x) .miniextendr_field_at(fields, x, environment())");
    assert!(field.contains(".miniextendr_field_at <- function"));
    assert!(!field.contains(".miniextendr_arg_error <- function"));
    let index = render("f <- function(x) .miniextendr_field_index_error('cls', environment())");
    assert!(index.contains(".miniextendr_field_index_error <- function"));

    let choices = render("f <- function(x) .miniextendr_match_arg_several(x, choices, 'x')");
    assert!(choices.contains(".miniextendr_arg_error <- function"));
    assert!(choices.contains(".miniextendr_match_arg_several <- function"));
    assert!(!choices.contains(".miniextendr_match_arg <- function"));

    let scalar = render("f <- function(x) .miniextendr_match_arg(x, choices, 'x')");
    assert!(scalar.contains(".miniextendr_arg_error <- function"));
    assert!(scalar.contains(".miniextendr_match_arg <- function"));
    assert!(!scalar.contains(".miniextendr_match_arg_several <- function"));

    // The `...` check builds its own condition (#2157).
    let dots = render(
        "f <- function(x, ...) {\n  if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\")\n}",
    );
    assert!(dots.contains(".miniextendr_check_dots <- function"));
    assert!(!dots.contains(".miniextendr_arg_error <- function"));
    assert!(!scalar.contains(".miniextendr_check_dots <- function"));
}

/// Run the production class-ref placeholder resolver over `content` using
/// `entries` as the registry.
fn resolve_refs(content: &str, entries: &[ClassNameEntry]) -> String {
    let class_index = build_class_name_index(entries.iter());
    resolve_class_refs(content, &class_index)
}

/// Two impl blocks of one type register identical entries (#1242, #1776);
/// the index collapses them instead of treating the duplicate key as an error.
#[test]
fn class_name_index_dedups_identical_entries() {
    let entries = [
        ClassNameEntry {
            rust_type: "Foo",
            r_class_name: "Foo",
            class_system: "env",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "Foo",
            r_class_name: "Foo",
            class_system: "env",
            field_syntax: false,
        },
    ];
    let index = build_class_name_index(entries.iter());
    assert_eq!(index.len(), 1);
    assert_eq!(index["Foo"].r_class_name, "Foo");
}

/// Disagreeing `class = "..."` overrides across blocks must fail loudly —
/// placeholder resolution would otherwise pick whichever entry linkme
/// ordered last.
#[test]
#[should_panic(expected = "conflicting class registrations for Rust type `Foo`")]
fn class_name_index_panics_on_conflicting_override() {
    let entries = [
        ClassNameEntry {
            rust_type: "Foo",
            r_class_name: "Foo",
            class_system: "env",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "Foo",
            r_class_name: "FooOverride",
            class_system: "env",
            field_syntax: false,
        },
    ];
    let _ = build_class_name_index(entries.iter());
}

/// R class names belong to one Rust type, regardless of the selected
/// class systems or the order linkme supplies the registrations.
#[test]
fn class_name_index_rejects_shared_r_class_names() {
    for class_system in ["env", "r6", "s3", "s4", "s7", "vctrs"] {
        let entries = [
            ClassNameEntry {
                rust_type: "First",
                r_class_name: "Shared",
                class_system: "env",
                field_syntax: false,
            },
            ClassNameEntry {
                rust_type: "Second",
                r_class_name: "Shared",
                class_system,
                field_syntax: true,
            },
        ];
        for ordered in [[&entries[0], &entries[1]], [&entries[1], &entries[0]]] {
            let error = std::panic::catch_unwind(|| build_class_name_index(ordered))
                .err()
                .expect("a shared R class would overwrite the other type's methods");
            let message = error.downcast_ref::<String>().unwrap();
            assert!(message.contains("R class `Shared`"), "{message}");
            assert!(message.contains("`First` (env)"), "{message}");
            assert!(
                message.contains(&format!("`Second` ({class_system})")),
                "{message}"
            );
            assert!(message.contains("class = \"...\""), "{message}");
        }
    }
}

#[test]
fn class_name_index_preserves_case_distinct_r_class_names() {
    let entries = [
        ClassNameEntry {
            rust_type: "First",
            r_class_name: "Widget",
            class_system: "env",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "Second",
            r_class_name: "widget",
            class_system: "env",
            field_syntax: false,
        },
    ];
    let index = build_class_name_index(entries.iter());
    assert_eq!(index["First"].r_class_name, "Widget");
    assert_eq!(index["Second"].r_class_name, "widget");
}

/// A type's standalone sidecar accessors are written internal when one of
/// its impl blocks gives the class field syntax, exported otherwise (#1900).
#[test]
fn sidecar_accessors_follow_the_field_syntax() {
    let accessors = [
        SidecarAccessorsEntry {
            rust_type: "Fields",
            exported: "exported Fields",
            internal: "internal Fields",
            source_file: "lib.rs",
            source_line: 1,
        },
        SidecarAccessorsEntry {
            rust_type: "Plain",
            exported: "exported Plain",
            internal: "internal Plain",
            source_file: "lib.rs",
            source_line: 2,
        },
    ];
    let class = |rust_type, field_syntax| ClassNameEntry {
        rust_type,
        r_class_name: rust_type,
        class_system: "s3",
        field_syntax,
    };
    let classes = [
        class("Fields", false),
        class("Fields", true),
        class("Plain", false),
    ];
    let fragments = sidecar_accessor_fragments(&accessors, &classes);
    let contents: Vec<&str> = fragments.iter().map(|f| f.content.as_ref()).collect();
    assert_eq!(contents, ["internal Fields", "exported Plain"]);
    assert!(
        fragments
            .iter()
            .all(|f| f.priority == RWrapperPriority::Sidecar)
    );
    // A type no impl block registers keeps its exported accessors.
    assert_eq!(
        sidecar_accessor_fragments(&accessors, &[])[0].content,
        "exported Fields"
    );
}

/// The `all.equal()` method of `#[derive(AllEqual)]` takes the R class
/// of the type's impl block: an S3 method for every class system, and an
/// S7 class's own registration too (#2149).
#[test]
fn all_equal_methods_follow_the_class() {
    let entry = |rust_type| AllEqualEntry {
        rust_type,
        method: "\nfunction(target, current, ...) TRUE\n",
        docs: "#' Docs.",
        source_file: "lib.rs",
        source_line: 3,
        source_column: 12,
    };
    let class = |rust_type, r_class_name, class_system| ClassNameEntry {
        rust_type,
        r_class_name,
        class_system,
        field_syntax: false,
    };
    let entries = [entry("Fit"), entry("Shape")];
    let classes = [class("Fit", "fit_rec", "r6"), class("Shape", "Shape", "s7")];
    let fragments = all_equal_fragments(&entries, &classes).unwrap();
    assert!(
        fragments
            .iter()
            .all(|f| f.priority == RWrapperPriority::TraitImpl)
    );
    assert_eq!(
        fragments[0].content,
        "# Generated from Rust derive `AllEqual` on `Fit` (lib.rs:3:12)\n\
             #' Compare two `fit_rec` objects\n\
             #'\n\
             #' Docs.\n\
             #' @method all.equal fit_rec\n\
             #' @export\n\
             all.equal.fit_rec <- function(target, current, ...) TRUE"
    );
    assert!(fragments[1].content.ends_with(
        "all.equal.Shape <- function(target, current, ...) TRUE\n\
             S7::method(all.equal, Shape) <- all.equal.Shape\n\
             base::rm(list = \"all.equal\")"
    ));

    // A class name R can't read as a name is quoted.
    let quoted = all_equal_fragments(&[entry("Fit")], &[class("Fit", "fit-rec", "s3")]);
    assert!(
        quoted.unwrap()[0]
            .content
            .contains("`all.equal.fit-rec` <- function")
    );
}

/// A type without a class, and a vctrs class, are refused together.
#[test]
fn all_equal_methods_need_a_pointer_class() {
    let entry = |rust_type, source_line| AllEqualEntry {
        rust_type,
        method: "function(target, current, ...) TRUE",
        docs: "",
        source_file: "lib.rs",
        source_line,
        source_column: 1,
    };
    let classes = [ClassNameEntry {
        rust_type: "Rec",
        r_class_name: "Rec",
        class_system: "vctrs",
        field_syntax: false,
    }];
    let Err(msg) = all_equal_fragments(&[entry("Loose", 1), entry("Rec", 2)], &classes) else {
        panic!("expected the derives to be refused");
    };
    assert_eq!(
        msg,
        "`#[derive(AllEqual)]` on `Loose` (lib.rs:1): no `#[miniextendr(env | r6 | s3 | s4 | s7)]` \
             impl block gives the type an R class, so there is no class to give the `all.equal()` method\n\
             `#[derive(AllEqual)]` on `Rec` (lib.rs:2): the objects of a vctrs class are R vectors, \
             not pointers to the Rust value, and `all.equal()` already compares them"
    );
}

fn reexport(
    package: &'static str,
    name: &'static str,
    source_file: &'static str,
    source_line: u32,
) -> ReexportEntry {
    ReexportEntry {
        package,
        name,
        source_file,
        source_line,
        source_column: 11,
    }
}

/// One block per `pkg::name` whatever the number of sites, sorted by
/// package then name, a non-syntactic name quoted (#1910).
#[test]
fn reexport_blocks_are_one_per_item_and_sorted() {
    assert!(reexport_fragment(&[]).is_none());
    let entries = [
        reexport("generics", "tidy", "models.rs", 8),
        reexport("magrittr", "%>%", "fits.rs", 4),
        reexport("generics", "tidy", "fits.rs", 3),
        reexport("generics", "glance", "fits.rs", 3),
        reexport("data.table", "fread", "io.rs", 1),
        reexport("generics", "tidy", "models.rs", 8),
    ];
    let fragment = reexport_fragment(&entries).expect("a fragment");
    assert!(fragment.priority == RWrapperPriority::Reexport);
    assert_eq!(
        fragment.content,
        "# Generated from Rust `reexport!` of `data.table::fread` (io.rs:1:11)\n\
             #' @importFrom data.table fread\n\
             #' @export\n\
             data.table::fread\n\
             \n\
             # Generated from Rust `reexport!` of `generics::glance` (fits.rs:3:11)\n\
             #' @importFrom generics glance\n\
             #' @export\n\
             generics::glance\n\
             \n\
             # Generated from Rust `reexport!` of `generics::tidy` (fits.rs:3:11) (models.rs:8:11)\n\
             #' @importFrom generics tidy\n\
             #' @export\n\
             generics::tidy\n\
             \n\
             # Generated from Rust `reexport!` of `magrittr::%>%` (fits.rs:4:11)\n\
             #' @importFrom magrittr \"%>%\"\n\
             #' @export\n\
             magrittr::`%>%`"
    );
    // The line numbers of the sites don't change the normalized file.
    let moved = [reexport("generics", "tidy", "models.rs", 80)];
    assert_eq!(
        normalize_source_locs(&reexport_fragment(&moved).unwrap().content),
        normalize_source_locs(
            &reexport_fragment(&[reexport("generics", "tidy", "models.rs", 8)])
                .unwrap()
                .content
        )
    );
}

/// The blocks bind nothing, so they pass both scans of the file.
#[test]
fn reexport_blocks_pass_the_wrapper_scans() {
    let entries = [
        reexport("generics", "tidy", "fits.rs", 3),
        reexport("magrittr", "%>%", "fits.rs", 4),
    ];
    let content = format!(
        "# Generated from Rust fn `fit` (fits.rs:9:8)\n\
             #' @export\n\
             fit <- function(x) .Call(C_fit, x)\n\n{}",
        reexport_fragment(&entries).unwrap().content
    );
    let names: Vec<&str> = wrapper_names(&content).iter().map(|n| n.name).collect();
    assert_eq!(names, ["fit"]);
    assert!(detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new()).is_ok());
    assert_eq!(reexport_conflicts(&content, &[], &entries), Ok(()));
}

/// A name from two packages, and names the wrappers bind or export, are
/// refused in one message naming every site (#1910).
#[test]
fn reexport_conflicts_are_refused_together() {
    let content = concat!(
        "# Generated from Rust impl `Fit` (fits.rs:20:6)\n",
        "#' @export glance\n",
        "if (!base::exists(\"glance\", mode = \"function\")) {\n",
        "  glance <- function(x, ...) UseMethod(\"glance\")\n",
        "}\n",
        "# Generated from Rust fn `augment` (fits.rs:30:8)\n",
        "#' @export\n",
        "augment <- function(x) .Call(C_augment, x)\n",
        "# Generated from Rust fn `fit_print` (fits.rs:40:8)\n",
        "#' @rawNamespace export(tidy, \"%>%\")\n",
        "#' @method print fit\n",
        "print.fit <- function(x, ...) invisible(x)\n",
    );
    let entries = [
        reexport("generics", "tidy", "fits.rs", 3),
        reexport("broom", "tidy", "models.rs", 5),
        reexport("generics", "tidy", "models.rs", 6),
        reexport("generics", "glance", "fits.rs", 3),
        reexport("generics", "augment", "fits.rs", 3),
        reexport("generics", "interpolate", "fits.rs", 3),
    ];
    assert_eq!(
        reexport_conflicts(content, &[], &entries).unwrap_err(),
        "miniextendr: `reexport!` re-exports `tidy` from 2 packages: `broom::tidy` \
             (models.rs:5:11), `generics::tidy` (fits.rs:3:11) (models.rs:6:11). A package \
             exports one object under a name; re-export it from one of them.\n\
             miniextendr: `reexport!` re-exports `generics::augment` (fits.rs:3:11), and the \
             package also binds `augment` (a function from Rust fn `augment` (fits.rs:30:8)). \
             The package's own `augment` would hide the imported one in its namespace, so \
             `export(augment)` would export it instead. Remove the re-export, or rename the \
             package's binding (`r_name = \"...\"`, `class = \"...\"`).\n\
             miniextendr: `reexport!` re-exports `generics::glance` (fits.rs:3:11), and the \
             package also binds `glance` (a guarded generic from Rust impl `Fit` \
             (fits.rs:20:6)). The package's own `glance` would hide the imported one in its \
             namespace, so `export(glance)` would export it instead. Remove the re-export, or \
             rename the package's binding (`r_name = \"...\"`, `class = \"...\"`).\n\
             miniextendr: `reexport!` re-exports `broom::tidy` (models.rs:5:11), \
             `generics::tidy` (fits.rs:3:11) (models.rs:6:11), and the package also exports \
             `tidy` (an export from Rust fn `fit_print` (fits.rs:40:8)). `reexport!` writes the \
             export itself; drop the other one."
    );
}

fn hand_written(name: &str, text: &str) -> HandWrittenFile {
    HandWrittenFile {
        name: name.to_string(),
        text: text.to_string(),
    }
}

/// A hand-written R file that binds or exports a re-exported name is
/// refused like the wrappers, in the same message, naming the file
/// (#2205). Every name in a file is the file's, whatever its comments say.
#[test]
fn reexport_conflicts_read_hand_written_files() {
    let content = concat!(
        "# Generated from Rust fn `augment` (fits.rs:30:8)\n",
        "#' @export\n",
        "augment <- function(x) .Call(C_augment, x)\n",
    );
    let files = [
        hand_written(
            "R/augment.R",
            "augment <- function(x, ...) x\n\
                 glance_helper <- function(x) x\n",
        ),
        hand_written(
            "R/glance.R",
            "#' @export glance\n\
                 NULL\n\
                 # Generated from Rust fn `tidy` (copied.rs:1:1)\n\
                 #' Tidy a fit\n\
                 #' @export\n\
                 tidy <- function(x, ...) {\n\
                 \x20 inner <- function() NULL\n\
                 }\n",
        ),
        hand_written("R/utils.R", "interpolate = function(x) x\n"),
    ];
    let entries = [
        reexport("generics", "tidy", "fits.rs", 3),
        reexport("generics", "glance", "fits.rs", 3),
        reexport("generics", "augment", "fits.rs", 3),
        reexport("generics", "interpolate", "fits.rs", 3),
        reexport("generics", "inner", "fits.rs", 3),
    ];
    assert_eq!(
        reexport_conflicts(content, &files, &entries).unwrap_err(),
        "miniextendr: `reexport!` re-exports `generics::augment` (fits.rs:3:11), and the \
             package also binds `augment` (a function from Rust fn `augment` (fits.rs:30:8), a \
             function in R/augment.R). The package's own `augment` would hide the imported one \
             in its namespace, so `export(augment)` would export it instead. Remove the \
             re-export, or rename the package's binding (`r_name = \"...\"`, `class = \"...\"`) \
             and the one in R/augment.R.\n\
             miniextendr: `reexport!` re-exports `generics::glance` (fits.rs:3:11), and the \
             package also exports `glance` (an export in R/glance.R). `reexport!` writes the \
             export itself; drop the other one.\n\
             miniextendr: `reexport!` re-exports `generics::tidy` (fits.rs:3:11), and the \
             package also binds `tidy` (a function in R/glance.R). The package's own `tidy` \
             would hide the imported one in its namespace, so `export(tidy)` would export it \
             instead. Remove the re-export, or rename the package's `tidy` in R/glance.R."
    );
    // A file that binds other names passes.
    assert_eq!(reexport_conflicts("", &files[..1], &entries[..1]), Ok(()));
}

/// The scan reads the code files R installs from `R/`, in name order,
/// and not the wrappers being written, whatever directory they are
/// written in.
#[test]
fn hand_written_r_files_are_the_installed_code_files() {
    let root =
        std::env::temp_dir().join(format!("miniextendr-hand-written-{}", std::process::id()));
    let r_dir = root.join("R");
    std::fs::create_dir_all(r_dir.join("unix")).unwrap();
    let written: [(&str, &[u8]); 9] = [
        ("zz.R", b"zz <- function() NULL\n"),
        ("pkg-wrappers.R", b"generated <- function() NULL\n"),
        ("a.r", b"a <- function() NULL\n"),
        ("latin1.R", b"caf\xe9 <- function() \"\xe9\"\n"),
        (".hidden.R", b"hidden <- function() NULL\n"),
        ("_draft.R", b"draft <- function() NULL\n"),
        ("notes.txt", b"notes <- function() NULL\n"),
        ("pkg-wrappers.tmp", b"tmp <- function() NULL\n"),
        ("unix/os.R", b"os <- function() NULL\n"),
    ];
    for (name, text) in written {
        std::fs::write(r_dir.join(name), text).unwrap();
    }
    let elsewhere = root.join("tmp").join("pkg-wrappers.R");

    let files = hand_written_r_files(&r_dir, &elsewhere);
    std::fs::remove_dir_all(&root).unwrap();

    let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(names, ["R/a.r", "R/latin1.R", "R/zz.R"]);
    assert_eq!(files[1].text, "caf\u{fffd} <- function() \"\u{fffd}\"\n");
}

#[test]
fn raw_namespace_exports_are_read_by_name() {
    let names: Vec<&str> = raw_namespace_exports(
        "export(a, \"[[\") exportPattern(\"^x\") S3method(print, foo) if (TRUE) export(`b`)",
    )
    .collect();
    assert_eq!(names, ["a", "[[", "b"]);
}

/// Run the production scalar cross-class return wrapper resolver over
/// `content` using `entries` as the class registry.
fn resolve_return_wrappers(content: &str, entries: &[ClassNameEntry]) -> String {
    let class_index = build_class_name_index(entries.iter());
    resolve_scalar_return_wrappers(content, &class_index)
}

/// Run the production list cross-class return wrapper resolver (#1284)
/// over `content` using `entries` as the class registry.
fn resolve_list_wrappers(content: &str, entries: &[ClassNameEntry]) -> String {
    let class_index = build_class_name_index(entries.iter());
    resolve_list_return_wrappers(content, &class_index)
}

#[test]
fn test_class_ref_resolver_with_override() {
    // S7Shape is registered with r_class_name = "Shape" (override)
    let entries = [
        ClassNameEntry {
            rust_type: "S7Shape",
            r_class_name: "Shape",
            class_system: "s7",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "S7Circle",
            r_class_name: "S7Circle",
            class_system: "s7",
            field_syntax: false,
        },
    ];

    let input = r#"S7Circle <- S7::new_class("S7Circle", parent = .__MX_CLASS_REF_S7Shape__, properties = list())"#;
    let output = resolve_refs(input, &entries);
    assert_eq!(
        output,
        r#"S7Circle <- S7::new_class("S7Circle", parent = Shape, properties = list())"#
    );
}

#[test]
fn test_class_ref_resolver_multiple_placeholders() {
    let entries = [
        ClassNameEntry {
            rust_type: "Point2D",
            r_class_name: "Point2D",
            class_system: "s7",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "Point3D",
            r_class_name: "Point3D",
            class_system: "s7",
            field_syntax: false,
        },
    ];

    let input =
        "S7::method(convert, list(.__MX_CLASS_REF_Point2D__, Point3D)) <- function(from, to) {}";
    let output = resolve_refs(input, &entries);
    assert_eq!(
        output,
        "S7::method(convert, list(Point2D, Point3D)) <- function(from, to) {}"
    );
}

#[test]
fn test_class_ref_resolver_unresolved_falls_back_to_rust_name() {
    let entries: [ClassNameEntry; 0] = [];
    let input = "parent = .__MX_CLASS_REF_UnknownClass__";
    let output = resolve_refs(input, &entries);
    // Falls back to the bare Rust name
    assert_eq!(output, "parent = UnknownClass");
}

#[test]
fn test_class_ref_resolver_verbatim_passthrough() {
    // Strings that are NOT bare identifiers should not have been wrapped in
    // a placeholder, so they pass through the resolver unchanged.
    let entries: [ClassNameEntry; 0] = [];
    let input = "parent = S7::class_any";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "parent = S7::class_any");
}

#[test]
fn test_is_bare_identifier_via_resolver() {
    // A placeholder for a bare identifier should be resolved.
    let entries = [ClassNameEntry {
        rust_type: "MyClass",
        r_class_name: "MyClass",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "inherit = .__MX_CLASS_REF_MyClass__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "inherit = MyClass");
}

// #203 — OR_ANY variant: S7 property class constraints should fall back
// silently to S7::class_any when the type isn't a registered S7 class.

#[test]
fn or_any_resolves_registered_s7_class() {
    let entries = [ClassNameEntry {
        rust_type: "S7PropInner",
        r_class_name: "S7PropInner",
        class_system: "s7",
        field_syntax: false,
    }];
    let input = "class = .__MX_CLASS_REF_OR_ANY_S7PropInner__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "class = S7PropInner");
}

#[test]
fn or_any_unregistered_type_falls_back_to_class_any_silently() {
    // No entry for SEXP, PathBuf, Json, etc.
    let entries: [ClassNameEntry; 0] = [];
    let input = "class = .__MX_CLASS_REF_OR_ANY_SEXP__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "class = S7::class_any");
}

#[test]
fn or_any_registered_non_s7_type_falls_back_to_class_any() {
    // A type registered as an R6 class can't serve as an S7 class
    // constraint — S7 would error at load time. Quiet fallback avoids
    // that footgun.
    let entries = [ClassNameEntry {
        rust_type: "R6Counter",
        r_class_name: "R6Counter",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "class = .__MX_CLASS_REF_OR_ANY_R6Counter__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "class = S7::class_any");
}

#[test]
fn loud_class_ref_still_emits_bare_name_when_unresolved() {
    // The existing CLASS_REF variant (without OR_ANY) keeps its loud
    // behavior: bare Rust name + compile-time warning. Regression test
    // for that path alongside the OR_ANY introduction.
    let entries: [ClassNameEntry; 0] = [];
    let input = "parent = .__MX_CLASS_REF_UnknownClass__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "parent = UnknownClass");
}

#[test]
fn mixed_placeholders_resolve_independently() {
    // A single wrapper can mix both variants — `inherit = parent` (loud)
    // and `class = prop_type` (quiet). Verify each takes its own path.
    let entries = [
        ClassNameEntry {
            rust_type: "Parent",
            r_class_name: "Parent",
            class_system: "s7",
            field_syntax: false,
        },
        // PropInner deliberately missing → should resolve to class_any.
    ];
    let input = "inherit = .__MX_CLASS_REF_Parent__, class = .__MX_CLASS_REF_OR_ANY_PropInner__";
    let output = resolve_refs(input, &entries);
    assert_eq!(output, "inherit = Parent, class = S7::class_any");
}

#[test]
fn return_wrapper_resolves_r6_constructor() {
    let entries = [ClassNameEntry {
        rust_type: "R6Board",
        r_class_name: "R6Board",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "return(.__MX_WRAP_RETURN_R6Board__(.val))";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(output, "return(R6Board$new(.ptr = .val))");
}

#[test]
fn return_wrapper_resolves_s7_override_constructor() {
    let entries = [ClassNameEntry {
        rust_type: "S7Board",
        r_class_name: "PrettyBoard",
        class_system: "s7",
        field_syntax: false,
    }];
    let input = ".__MX_WRAP_RETURN_S7Board__(.val)";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(output, "PrettyBoard(.ptr = .val)");
}

#[test]
fn return_wrapper_resolves_s4_constructor() {
    let entries = [ClassNameEntry {
        rust_type: "S4Board",
        r_class_name: "S4Board",
        class_system: "s4",
        field_syntax: false,
    }];
    let input = ".__MX_WRAP_RETURN_S4Board__(.val)";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(output, "methods::new(\"S4Board\", ptr = .val)");
}

#[test]
fn return_wrapper_resolves_attribute_classes() {
    let entries = [
        ClassNameEntry {
            rust_type: "S3Board",
            r_class_name: "S3Board",
            class_system: "s3",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "EnvBoard",
            r_class_name: "EnvBoard",
            class_system: "env",
            field_syntax: false,
        },
    ];
    let input = "a <- .__MX_WRAP_RETURN_S3Board__(.val)\nb <- .__MX_WRAP_RETURN_EnvBoard__(.ptr)";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(
        output,
        "a <- structure(.val, class = \"S3Board\")\nb <- structure(.ptr, class = \"EnvBoard\")"
    );
}

#[test]
fn return_wrapper_unregistered_type_falls_back_to_value() {
    let entries: [ClassNameEntry; 0] = [];
    let input = ".__MX_WRAP_RETURN_JsonValue__(.val)";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(output, ".val");
}

// #1284 — list-shaped cross-class return markers (`Vec<Class>` returns).

/// CRITICAL non-overlap pin: the scalar resolver rewrites *unrecognized*
/// names of its own prefix to the bare expression, so if the list marker
/// shared the `.__MX_WRAP_RETURN_` prefix the scalar pass would consume
/// and destroy it before the list pass ran. The `LIST_` infix keeps the
/// families disjoint — the scalar resolver must leave list markers
/// completely untouched (and vice versa).
#[test]
fn scalar_resolver_leaves_list_markers_untouched() {
    let entries = [ClassNameEntry {
        rust_type: "R6Board",
        r_class_name: "R6Board",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "a <- .__MX_WRAP_LIST_RETURN_R6Board__(.val)";
    let output = resolve_return_wrappers(input, &entries);
    assert_eq!(output, input, "scalar pass must not consume list markers");
}

#[test]
fn list_resolver_leaves_scalar_markers_untouched() {
    let entries = [ClassNameEntry {
        rust_type: "R6Board",
        r_class_name: "R6Board",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "a <- .__MX_WRAP_RETURN_R6Board__(.val)";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(output, input, "list pass must not consume scalar markers");
}

/// Both passes over content mixing the two families resolve each marker
/// with its own resolver, in either pass order.
#[test]
fn scalar_and_list_markers_resolve_independently() {
    let entries = [ClassNameEntry {
        rust_type: "R6Board",
        r_class_name: "R6Board",
        class_system: "r6",
        field_syntax: false,
    }];
    let input =
        "a <- .__MX_WRAP_RETURN_R6Board__(.val)\nb <- .__MX_WRAP_LIST_RETURN_R6Board__(.val)";
    let expected =
        "a <- R6Board$new(.ptr = .val)\nb <- lapply(.val, function(.el) R6Board$new(.ptr = .el))";

    let scalar_first = resolve_list_wrappers(&resolve_return_wrappers(input, &entries), &entries);
    let list_first = resolve_return_wrappers(&resolve_list_wrappers(input, &entries), &entries);
    assert_eq!(scalar_first, expected);
    assert_eq!(list_first, expected);
}

#[test]
fn list_return_wrapper_resolves_r6_lapply() {
    let entries = [ClassNameEntry {
        rust_type: "R6Board",
        r_class_name: "R6Board",
        class_system: "r6",
        field_syntax: false,
    }];
    let input = "return(.__MX_WRAP_LIST_RETURN_R6Board__(.val))";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(
        output,
        "return(lapply(.val, function(.el) R6Board$new(.ptr = .el)))"
    );
}

#[test]
fn list_return_wrapper_resolves_s7_override_lapply() {
    let entries = [ClassNameEntry {
        rust_type: "S7Board",
        r_class_name: "PrettyBoard",
        class_system: "s7",
        field_syntax: false,
    }];
    let input = ".__MX_WRAP_LIST_RETURN_S7Board__(.val)";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(
        output,
        "lapply(.val, function(.el) PrettyBoard(.ptr = .el))"
    );
}

#[test]
fn list_return_wrapper_resolves_s4_lapply() {
    let entries = [ClassNameEntry {
        rust_type: "S4Board",
        r_class_name: "S4Board",
        class_system: "s4",
        field_syntax: false,
    }];
    let input = ".__MX_WRAP_LIST_RETURN_S4Board__(.val)";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(
        output,
        "lapply(.val, function(.el) methods::new(\"S4Board\", ptr = .el))"
    );
}

#[test]
fn list_return_wrapper_resolves_attribute_classes_lapply() {
    let entries = [
        ClassNameEntry {
            rust_type: "S3Board",
            r_class_name: "S3Board",
            class_system: "s3",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "EnvBoard",
            r_class_name: "EnvBoard",
            class_system: "env",
            field_syntax: false,
        },
    ];
    let input =
        "a <- .__MX_WRAP_LIST_RETURN_S3Board__(.val)\nb <- .__MX_WRAP_LIST_RETURN_EnvBoard__(.ptr)";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(
        output,
        "a <- lapply(.val, function(.el) structure(.el, class = \"S3Board\"))\nb <- lapply(.ptr, function(.el) structure(.el, class = \"EnvBoard\"))"
    );
}

#[test]
fn list_return_wrapper_unregistered_type_falls_back_to_bare_list() {
    let entries: [ClassNameEntry; 0] = [];
    let input = ".__MX_WRAP_LIST_RETURN_JsonValue__(.val)";
    let output = resolve_list_wrappers(input, &entries);
    assert_eq!(output, ".val");
}

// region: wrappers_file_content header (#1552)

#[test]
fn wrappers_file_header_carries_version_and_body_digest() {
    // FNV-1a 64 reference vectors: "" and "a".
    assert_eq!(
        crate::abi::fnv1a_64(b"", crate::abi::FNV1A_64_OFFSET),
        0xcbf29ce484222325
    );
    assert_eq!(
        crate::abi::fnv1a_64(b"a", crate::abi::FNV1A_64_OFFSET),
        0xaf63dc4c8601ec8c
    );

    let content = wrappers_file_content("a");
    let mut lines = content.lines();
    assert_eq!(lines.next(), Some(WRAPPERS_FILE_MARKER));
    assert_eq!(
        lines.next(),
        Some(concat!(
            "# miniextendr-api ",
            env!("CARGO_PKG_VERSION"),
            " | fnv1a-64 of the normalized lines below: af63dc4c8601ec8c"
        ))
    );
    assert_eq!(lines.next(), Some("a"));
    assert_eq!(lines.next(), None);
}

#[test]
fn wrappers_file_header_ignores_source_position_shifts() {
    let old = "# Generated from Rust fn `foo` (lib.rs:10:5)\nfoo <- function() 1\n";
    let new = "# Generated from Rust fn `foo` (lib.rs:42:8)\nfoo <- function() 1\n";
    let header = |s: &str| wrappers_file_content(s).lines().nth(1).unwrap().to_string();
    assert_eq!(header(old), header(new));
    assert!(wrappers_semantically_equal(
        &wrappers_file_content(old),
        &wrappers_file_content(new)
    ));

    let changed = "# Generated from Rust fn `foo` (lib.rs:10:5)\nfoo <- function() 2\n";
    assert_ne!(header(old), header(changed));
}

// endregion

// region: normalize_source_locs unit tests (#528)

#[test]
fn normalize_source_locs_noop_on_plain_text() {
    // No `.rs:N:M` pattern — should return the input string unchanged (Borrowed).
    let input = "# just a comment\nfoo <- function() {}\n";
    let result = normalize_source_locs(input);
    assert_eq!(result.as_ref(), input);
    // Verify it's the zero-allocation borrowed path.
    assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
}

#[test]
fn normalize_source_locs_single_attribution() {
    let input = "# Generated from Rust fn `foo` (lib.rs:42:8)";
    let result = normalize_source_locs(input);
    assert_eq!(
        result.as_ref(),
        "# Generated from Rust fn `foo` (lib.rs:_:_)"
    );
}

#[test]
fn normalize_source_locs_multiple_attributions() {
    let input = concat!(
        "# (conversions.rs:1:5)\n",
        "foo <- function() {}\n",
        "# (conversions.rs:158:8)\n",
        "bar <- function() {}\n",
    );
    let result = normalize_source_locs(input);
    assert_eq!(
        result.as_ref(),
        concat!(
            "# (conversions.rs:_:_)\n",
            "foo <- function() {}\n",
            "# (conversions.rs:_:_)\n",
            "bar <- function() {}\n",
        )
    );
}

#[test]
fn normalize_source_locs_does_not_match_extra_colon() {
    // `(lib.rs:1:5:6)` — four components — is NOT a valid attribution; leave alone.
    let input = "(lib.rs:1:5:6)";
    let result = normalize_source_locs(input);
    // The pattern `(lib.rs:1:5` matches, but then we expect `)` after the col
    // digits and find `:` instead — so it should NOT be replaced.
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_source_locs_does_not_match_without_parens() {
    // Bare `lib.rs:1:5` without surrounding parens — leave untouched.
    let input = "lib.rs:1:5";
    let result = normalize_source_locs(input);
    assert_eq!(result.as_ref(), input);
    assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
}

#[test]
fn wrappers_semantically_equal_position_only_diff() {
    // Two wrappers identical except for line:col are semantically equal.
    let old = "# Generated from Rust fn `foo` (lib.rs:42:8)\nfoo <- function() {}\n";
    let new = "# Generated from Rust fn `foo` (lib.rs:99:8)\nfoo <- function() {}\n";
    assert!(wrappers_semantically_equal(old, new));
}

#[test]
fn wrappers_semantically_equal_content_diff() {
    // A real semantic change (different function body) is NOT equal.
    let old = "# Generated from Rust fn `foo` (lib.rs:42:8)\nfoo <- function() { 1L }\n";
    let new = "# Generated from Rust fn `foo` (lib.rs:42:8)\nfoo <- function() { 2L }\n";
    assert!(!wrappers_semantically_equal(old, new));
}

/// An impl block one line further down moves its method comments and
/// sidecar headers too; the wrappers still compare equal.
#[test]
fn wrappers_semantically_equal_impl_method_position_only_diff() {
    let old = "# Generated from Rust impl `Counter` (lib.rs:112:6)\n\
                   Counter <- R6::R6Class(\"Counter\",\n  public = list(\n    \
                   # Counter::new (lib.rs:114:12)\n    initialize = function() {}\n  ))\n\
                   # Counter::get (lib.rs:118:12)\n\
                   # Generated from Rust source (lib.rs:20:5)\n";
    let new = "# Generated from Rust impl `Counter` (lib.rs:113:6)\n\
                   Counter <- R6::R6Class(\"Counter\",\n  public = list(\n    \
                   # Counter::new (lib.rs:115:12)\n    initialize = function() {}\n  ))\n\
                   # Counter::get (lib.rs:119:12)\n\
                   # Generated from Rust source (lib.rs:21:5)\n";
    assert!(wrappers_semantically_equal(old, new));
}

/// Every position a generated wrappers file carries in a comment is one
/// the writer normalizes, so moving Rust lines never rewrites the file.
/// Reads the tracked cross-package wrappers, which real builds produced.
#[test]
fn generated_comment_positions_are_normalized() {
    // A position ends the comment: `(48:8)`, `line 4:5`, `(lib.rs:_:_)`
    // once normalized. R code in a comment (`var(1:10)`) is not one.
    fn ends_with_line_col(line: &str) -> bool {
        let last = line.rsplit(' ').next().unwrap_or_default();
        let last = last
            .strip_prefix('(')
            .and_then(|l| l.strip_suffix(')'))
            .unwrap_or(last);
        let mut parts = last.rsplit(':');
        let is_number = |part: Option<&str>| {
            part.is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        };
        is_number(parts.next()) && is_number(parts.next())
    }
    let manifest = env!("CARGO_MANIFEST_DIR");
    for file in [
        "../tests/cross-package/producer.pkg/R/producer.pkg-wrappers.R",
        "../tests/cross-package/consumer.pkg/R/consumer.pkg-wrappers.R",
    ] {
        let path = std::path::Path::new(manifest).join(file);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let normalized = normalize_source_locs(&text);
        let stale: Vec<&str> = normalized
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("# ") && ends_with_line_col(line))
            .collect();
        assert!(
            stale.is_empty(),
            "{}: positions without a file name: {stale:#?}",
            path.display()
        );
    }
}

#[test]
fn wrappers_semantically_equal_both_position_and_content_diff() {
    // Both positions and content differ — still NOT equal.
    let old = "# Generated from Rust fn `foo` (lib.rs:1:1)\nfoo <- function() { 1L }\n";
    let new = "# Generated from Rust fn `bar` (lib.rs:9:1)\nbar <- function() { 2L }\n";
    assert!(!wrappers_semantically_equal(old, new));
}

// endregion

// region: generic doc marker tests

#[test]
fn test_parse_generic_doc_marker_s7() {
    let marker = r#".__MX_GENERIC_DOC__(kind="S7", generic="get_value", class="MyClass", export=true, dispatch="x", no_dots=false)"#;
    let result = parse_generic_doc_marker(marker);
    assert!(result.is_some());
    let (kind, generic, class, export, dispatch, no_dots) = result.unwrap();
    assert_eq!(kind, "S7");
    assert_eq!(generic, "get_value");
    assert_eq!(class, "MyClass");
    assert!(export);
    assert_eq!(dispatch, "x");
    assert!(!no_dots);
}

#[test]
fn test_parse_generic_doc_marker_s4() {
    let marker =
        r#".__MX_GENERIC_DOC__(kind="S4", generic="s4_get", class="Counter", export=true)"#;
    let result = parse_generic_doc_marker(marker);
    assert!(result.is_some());
    let (kind, generic, class, export, dispatch, no_dots) = result.unwrap();
    assert_eq!(kind, "S4");
    assert_eq!(generic, "s4_get");
    assert_eq!(class, "Counter");
    assert!(export);
    assert_eq!(dispatch, "x"); // default
    assert!(!no_dots); // default
}

#[test]
fn test_resolve_generic_doc_markers_single_class() {
    let input = concat!(
        r#".__MX_GENERIC_DOC__(kind="S7", generic="get_value", class="MyClass", export=true, dispatch="x", no_dots=false)"#,
        "\n",
        "if (!exists(\"get_value\", mode = \"function\")) {\n",
        "  get_value <- S7::new_generic(\"get_value\", \"x\", function(x, ...) S7::S7_dispatch())\n",
        "}\n",
    );
    let output = resolve_generic_doc_markers(input.to_string());

    // Marker must be gone
    assert!(!output.contains(".__MX_GENERIC_DOC__"));
    // Standalone page must have bare @name
    assert!(output.contains("#' @name get_value"));
    // Must list the class
    assert!(output.contains("\\link{MyClass}"));
    // Must have NULL anchor
    assert!(output.contains("\nNULL\n"));
    // Generic guard must survive
    assert!(output.contains("if (!exists(\"get_value\""));
}

#[test]
fn test_resolve_generic_doc_markers_two_classes_one_generic() {
    // Two classes sharing one generic: both appear in the listing, only one doc page.
    let input = concat!(
        r#".__MX_GENERIC_DOC__(kind="S7", generic="get_value", class="ClassA", export=true, dispatch="x", no_dots=false)"#,
        "\n",
        "line_a\n",
        r#".__MX_GENERIC_DOC__(kind="S7", generic="get_value", class="ClassB", export=true, dispatch="x", no_dots=false)"#,
        "\n",
        "line_b\n",
    );
    let output = resolve_generic_doc_markers(input.to_string());

    // No markers remain
    assert!(!output.contains(".__MX_GENERIC_DOC__"));
    // Only one @name line
    let name_count = output.matches("#' @name get_value").count();
    assert_eq!(
        name_count, 1,
        "expected exactly one @name get_value, got {name_count}"
    );
    // Both classes listed
    assert!(output.contains("\\link{ClassA}"));
    assert!(output.contains("\\link{ClassB}"));
    // Other lines survive
    assert!(output.contains("line_a"));
    assert!(output.contains("line_b"));
}

#[test]
fn test_resolve_generic_doc_markers_external_generic_excluded() {
    // External generics (has_generic_override) do NOT emit markers.
    // Verify that content without any markers passes through unchanged.
    let input = "if (!exists(\"size\", mode = \"function\")) {\n  size <- S7::new_external_generic(\"vctrs\", \"size\")\n}\n";
    let output = resolve_generic_doc_markers(input.to_string());
    // Content should be identical (the line-by-line reconstruction adds \n)
    let output_trimmed: Vec<&str> = output.lines().collect();
    let input_trimmed: Vec<&str> = input.lines().collect();
    assert_eq!(output_trimmed, input_trimmed);
}

#[test]
fn test_resolve_generic_doc_markers_no_dots() {
    let input = concat!(
        r#".__MX_GENERIC_DOC__(kind="S7", generic="strict_fn", class="A", export=true, dispatch="x", no_dots=true)"#,
        "\n",
    );
    let output = resolve_generic_doc_markers(input.to_string());
    // no_dots=true → @param ... must NOT appear in the standalone block
    assert!(!output.contains("@param ..."));
    // @param x must appear
    assert!(output.contains("@param x"));
}

#[test]
fn test_resolve_generic_doc_markers_s4() {
    let input = concat!(
        r#".__MX_GENERIC_DOC__(kind="S4", generic="s4_compute", class="S4Counter", export=true)"#,
        "\n",
        "if (!methods::isGeneric(\"s4_compute\")) methods::setGeneric(\"s4_compute\", function(x, ...) standardGeneric(\"s4_compute\"))\n",
    );
    let output = resolve_generic_doc_markers(input.to_string());
    assert!(!output.contains(".__MX_GENERIC_DOC__"));
    assert!(output.contains("#' @name s4_compute"));
    assert!(output.contains("an S4 generic"));
    assert!(output.contains("\\link{S4Counter}"));
    // S4 export uses @exportMethod
    assert!(output.contains("#' @exportMethod s4_compute"));
    // setGeneric guard must survive
    assert!(output.contains("if (!methods::isGeneric(\"s4_compute\")"));
}

#[test]
fn test_resolve_generic_doc_markers_quotes_operator_exports() {
    for generic in ["[", "[[", "$", "%custom%"] {
        let input = format!(
            ".__MX_GENERIC_DOC__(kind=\"S7\", generic=\"{generic}\", class=\"Foo\", export=true, dispatch=\"x\", no_dots=false)\n"
        );
        let output = resolve_generic_doc_markers(input);
        // Must match the method block's directive (`naming::r_namespace_name`
        // in miniextendr-macros); `export([[)` would not parse.
        assert!(
            output.contains(&format!("#' @rawNamespace export(\"{generic}\")")),
            "{output}"
        );
        assert!(
            output.contains(&format!("#' @name {generic}\n")),
            "{output}"
        );
        assert!(
            output.contains(&format!("#' The `{generic}` generic\n")),
            "{output}"
        );
    }
    let output = resolve_generic_doc_markers(
        r#".__MX_GENERIC_DOC__(kind="S7", generic="get", class="Foo", export=true, dispatch="x", no_dots=false)"#
            .to_string(),
    );
    assert!(
        output.contains("#' @rawNamespace export(get)\n"),
        "{output}"
    );
    assert!(output.contains("#' The `get()` generic\n"), "{output}");
}

#[test]
fn test_is_syntactic_r_name_matches_r() {
    for name in ["get", "var", ".hidden", "s7_trait_Ops_value", "a.b_c1"] {
        assert!(is_syntactic_r_name(name), "{name}");
    }
    for name in [
        "[", "[[", "$", "%custom%", "Foo_[[", "if", "NA", ".1x", "1a", "_a", "",
    ] {
        assert!(!is_syntactic_r_name(name), "{name}");
    }
    assert_eq!(r_namespace_name(r#"a"b\c"#), r#""a\"b\\c""#);
}

// endregion

// region: duplicate wrapper-definition detection (#991)

/// Build an accessor-producer map the way `write_r_wrappers_to_file` does,
/// from synthetic sidecar + class registries, for unit testing.
fn accessor_map(
    sidecar: &[SidecarPropEntry],
    classes: &[ClassNameEntry],
) -> std::collections::HashMap<String, String> {
    let class_index: std::collections::HashMap<&str, &ClassNameEntry> =
        classes.iter().map(|e| (e.rust_type, e)).collect();
    sidecar_accessor_names(sidecar, &class_index)
}

#[test]
fn parse_top_level_fn_def_name_accepts_bare_def() {
    assert_eq!(
        parse_top_level_fn_def_name("Foo_get_value <- function(x) .Call(C, x)"),
        Some("Foo_get_value")
    );
    // Dotted/internal helper names are valid R identifiers.
    assert_eq!(
        parse_top_level_fn_def_name(".miniextendr_raise_condition <- function(.val) {"),
        Some(".miniextendr_raise_condition")
    );
}

#[test]
fn parse_top_level_fn_def_name_accepts_backticked_def() {
    // Non-syntactic S3 method names are emitted backtick-quoted (#1475);
    // the scan sees the bare name so two `[.foo` definitions still collide.
    assert_eq!(
        parse_top_level_fn_def_name("`[.mx_bag` <- function(x, i, ...) {"),
        Some("[.mx_bag")
    );
    assert_eq!(
        parse_top_level_fn_def_name("`$.mx_bag` <- function(x, name) {"),
        Some("$.mx_bag")
    );
    // Unterminated or empty quoting is not a definition we own.
    assert_eq!(parse_top_level_fn_def_name("`[.foo <- function() {}"), None);
    assert_eq!(parse_top_level_fn_def_name("`` <- function() {}"), None);
}

#[test]
fn detect_duplicate_wrapper_defs_sees_through_backticks() {
    let content = "`[.foo` <- function(x, i) 1\n`[.foo` <- function(x, i) 2\n";
    let err = detect_duplicate_wrapper_defs(content, &std::collections::HashMap::new())
        .expect_err("duplicate backticked definition must be reported");
    assert!(err.contains("`[.foo`"), "{err}");
}

#[test]
fn has_page_assignment_tag_recognises_rdname_describein_and_name() {
    assert!(has_page_assignment_tag(
        "#' @rdname topic\nf <- function() 1"
    ));
    assert!(has_page_assignment_tag(
        "#' @describeIn topic Short text\n#' wrapped\nf <- function() 1"
    ));
    // `@name topic` documents `topic.Rd` in roxygen2.
    assert!(has_page_assignment_tag("#' @name topic\nf <- function() 1"));
    // A `@name` with no topic is malformed and does not count.
    assert!(!has_page_assignment_tag("#' @name\nf <- function() 1"));
    assert!(!has_page_assignment_tag(
        "#' @title T\n#' @export\nf <- function() 1"
    ));
    // Prefix-only matches do not count.
    assert!(!has_page_assignment_tag("#' @rdnamex\nf <- function() 1"));
}

// region: standalone-function pages (#1590)

fn resolve_pages(fragments: &[(&str, Option<&str>)]) -> Vec<String> {
    let fragments: Vec<StandaloneFragment<'_>> = fragments
        .iter()
        .map(|(content, stem)| StandaloneFragment {
            content,
            stem: stem.map(str::to_string),
        })
        .collect();
    resolve_standalone_pages(&fragments)
}

/// A generated `@param` line as the macro emits it.
fn filler(param: &str) -> String {
    format!("#' {PARAM_FILLER_MARKER}@param {param} (no documentation available)")
}

fn filler_resolved(param: &str) -> String {
    format!("#' @param {param} (no documentation available)")
}

/// The `.Rd` topic each fragment lands on, as `resolve_standalone_pages`
/// reads it.
fn page_topics(fragments: &[(&str, Option<&str>)]) -> Vec<Option<String>> {
    fragments
        .iter()
        .map(|(content, stem)| {
            standalone_page(&StandaloneFragment {
                content,
                stem: stem.map(str::to_string),
            })
            .page
        })
        .collect()
}

/// Two free functions of one source file document a page each (#1289):
/// no file-stem `@rdname` is injected, and each keeps the generated lines
/// of the arguments it leaves undocumented, even an argument the other
/// function documents.
#[test]
fn free_functions_of_one_file_get_a_page_each() {
    let scale = format!(
        "#' @title stem_scale\n#' @param values Numbers.\n{}\n#' @export\nstem_scale <- function(values, factor) 1",
        filler("factor")
    );
    let floor = format!(
        "#' @title stem_floor\n{}\n{}\n#' @export\nstem_floor <- function(values, floor) 1",
        filler("values"),
        filler("floor")
    );
    let fragments = [
        (scale.as_str(), Some("stem_docs")),
        (&floor, Some("stem_docs")),
    ];
    assert_eq!(
        page_topics(&fragments),
        [
            Some("stem_scale".to_string()),
            Some("stem_floor".to_string())
        ]
    );
    let out = resolve_pages(&fragments);
    for page in &out {
        assert!(!page.contains("@rdname"), "{page}");
        assert!(!page.contains(PARAM_FILLER_MARKER), "{page}");
        assert!(!page.contains("@order"), "own page, no order: {page}");
    }
    assert!(out[0].contains(&filler_resolved("factor")), "{}", out[0]);
    assert!(out[1].contains(&filler_resolved("values")), "{}", out[1]);
    assert!(out[1].contains(&filler_resolved("floor")), "{}", out[1]);
}

/// Grouping is opt-in through an explicit `@rdname <file stem>`. On that
/// page a filler survives only for an argument no function on the page
/// documents: roxygen2 keeps the later `@param` of a name, so a filler
/// would replace the real description another block wrote. The blocks are
/// the file's own page, so none of them gets `@order NaN`.
#[test]
fn explicit_stem_page_keeps_fillers_only_for_undocumented_arguments() {
    let first = format!(
        "#' @name stem_docs\n#' @rdname stem_docs\n#' @param values Numbers.\n{}\n#' @export\nstem_scale <- function(values, factor) 1",
        filler("factor")
    );
    let second = format!(
        "#' @rdname stem_docs\n#' @param offset Amount.\n{}\n#' @export\nstem_shift <- function(values, offset) 1",
        filler("values")
    );
    let third = format!(
        "#' @title stem_floor\n#' @rdname stem_docs\n{}\n{}\n#' @export\nstem_floor <- function(values, floor) 1",
        filler("values"),
        filler("floor")
    );
    let fragments = [
        (first.as_str(), Some("stem_docs")),
        (&second, Some("stem_docs")),
        (&third, Some("stem_docs")),
    ];
    assert!(
        page_topics(&fragments)
            .iter()
            .all(|page| page.as_deref() == Some("stem_docs"))
    );
    let out = resolve_pages(&fragments);
    for page in &out {
        assert!(!page.contains(PARAM_FILLER_MARKER), "{page}");
        assert!(!page.contains("@order"), "own page, no order: {page}");
        assert_eq!(page.matches("#' @rdname").count(), 1, "{page}");
    }
    assert!(out[0].contains(&filler_resolved("factor")), "{}", out[0]);
    assert!(!out[1].contains("@param values"), "{}", out[1]);
    assert!(!out[2].contains("@param values"), "{}", out[2]);
    assert!(out[2].contains(&filler_resolved("floor")), "{}", out[2]);
}

/// The case that made a merged `.Rd` reach 3.7 MB and stall
/// `R CMD INSTALL` (#1289): functions of one file with their own `@title`,
/// the first defining a section the others `@inheritSection`. Each
/// function keeps its own page, so the inheritance names another page
/// instead of the merged page itself.
#[test]
fn titled_blocks_inheriting_a_section_keep_separate_pages() {
    let first = "#' @title First setter\n#' @section Shared rules:\n#' Values are checked.\n#' @param x A value.\n#' @export\nfirst_setter <- function(x) 1";
    let second = "#' @title Second setter\n#' @inheritSection first_setter Shared rules\n#' @param x A value.\n#' @export\nsecond_setter <- function(x) 1";
    let third = "#' @title Third setter\n#' @inheritSection first_setter Shared rules\n#' @param x A value.\n#' @export\nthird_setter <- function(x) 1";
    let fragments = [
        (first, Some("setters")),
        (second, Some("setters")),
        (third, Some("setters")),
    ];
    assert_eq!(
        page_topics(&fragments),
        ["first_setter", "second_setter", "third_setter"].map(|t| Some(t.to_string()))
    );
    // Nothing to resolve: the blocks are written as the macro emitted them.
    assert_eq!(resolve_pages(&fragments), [first, second, third]);
}

/// A roxygen block that picks no page and has no `@title` gets the
/// function name as its title, so roxygen2 renders its page; `lib.rs`
/// functions (no stem) too. A block with no roxygen line, or `@noRd`,
/// renders no page and gets none.
#[test]
fn untitled_own_page_blocks_get_the_function_name_as_title() {
    let bare = "#' @export\nbare_fn <- function() 1";
    let internal = "#' @keywords internal\ninternal_fn <- function() 1";
    let undocumented = "plain_fn <- function() 1";
    let no_rd = "#' @noRd\nhidden <- function() 1";
    let out = resolve_pages(&[
        (bare, Some("helpers")),
        (internal, None),
        (undocumented, Some("helpers")),
        (no_rd, Some("helpers")),
    ]);
    assert_eq!(
        out[0],
        "#' @title bare_fn\n#' @export\nbare_fn <- function() 1"
    );
    assert_eq!(
        out[1],
        "#' @title internal_fn\n#' @keywords internal\ninternal_fn <- function() 1"
    );
    assert_eq!(out[2], undocumented);
    assert_eq!(out[3], no_rd);
}

/// A grouped `@param lower,upper` documents both names.
#[test]
fn grouped_param_names_count_as_documented() {
    let a = "#' @rdname bounds\n#' @param lower,upper Bounds.\n#' @export\nf <- function(lower, upper) 1";
    let b = format!(
        "#' @rdname bounds\n{}\n{}\n#' @export\ng <- function(lower, upper) 1",
        filler("lower"),
        filler("upper")
    );
    let out = resolve_pages(&[(a, Some("bounds")), (&b, Some("bounds"))]);
    assert!(!out[1].contains("@param"), "{}", out[1]);
}

/// A block that joins a topic defined elsewhere drops its fillers (that
/// topic's block documents the arguments) and sorts after the topic's own
/// block, whatever the R file order. `@rdname` naming the function itself
/// or its own `@name` is its own page.
#[test]
fn author_topic_blocks_drop_fillers_and_sort_last() {
    let joined = format!(
        "#' @title range_width\n#' @rdname range_summaries\n{}\n#' @export\nrange_width <- function(values) 1",
        filler("values")
    );
    let own_fn = format!(
        "#' @title mx_bag_sum\n#' @rdname mx_bag_sum\n{}\n#' @export\nmx_bag_sum <- function(x) 1",
        filler("x")
    );
    let own_name = format!(
        "#' @name bag_topic\n#' @rdname bag_topic\n#' @title Bags\n{}\nbag_len <- function(x) 1",
        filler("x")
    );
    let ordered = "#' @rdname range_summaries\n#' @order 3\nrange_mid <- function(values) 1";
    let out = resolve_pages(&[
        (&joined, Some("shared_param_docs")),
        (&own_fn, Some("s3_nonsyntactic_tests")),
        (&own_name, Some("bags")),
        (ordered, Some("shared_param_docs")),
    ]);
    assert!(!out[0].contains("@param"), "{}", out[0]);
    assert!(
        out[0].contains("#' @order NaN\n#' @export"),
        "order inserted at a tag boundary: {}",
        out[0]
    );
    for own in &out[1..3] {
        assert!(own.contains(&filler_resolved("x")), "{own}");
        assert!(!own.contains("@order"), "{own}");
    }
    // The author's own `@order` is kept, not doubled.
    assert_eq!(out[3].matches("@order").count(), 1, "{}", out[3]);
    assert!(out[3].contains("#' @order 3"), "{}", out[3]);
}

/// `@describeIn` always joins another topic; an `@noRd` block has no page
/// and keeps no fillers; `lib.rs` functions (no stem) keep theirs.
#[test]
fn describe_in_no_rd_and_stemless_blocks() {
    let described = "#' @describeIn range_summaries Midpoint.\n#' wrapped\nrange_midpoint <- function(values) 1";
    let no_rd = format!("{}\n#' @noRd\nhidden <- function(x) 1", filler("x"));
    let stemless = format!(
        "#' @title top\n{}\n#' @export\ntop <- function(x) 1",
        filler("x")
    );
    let out = resolve_pages(&[
        (described, Some("shared_param_docs")),
        (&no_rd, Some("shared_param_docs")),
        (&stemless, None),
    ]);
    assert_eq!(
        out[0],
        "#' @describeIn range_summaries Midpoint.\n#' wrapped\n#' @order NaN\nrange_midpoint <- function(values) 1"
    );
    assert_eq!(out[1], "#' @noRd\nhidden <- function(x) 1");
    assert!(out[2].contains(&filler_resolved("x")), "{}", out[2]);
    assert!(!out[2].contains("@rdname"), "{}", out[2]);
}

// endregion

#[test]
fn sort_wrapper_entries_is_priority_then_file_then_line() {
    fn entry(
        priority: RWrapperPriority,
        source_file: &'static str,
        source_line: u32,
        content: &'static str,
    ) -> WrapperFragment {
        WrapperFragment {
            priority,
            content: std::borrow::Cow::Borrowed(content),
            source_file,
            source_line,
        }
    }
    let mut entries = vec![
        entry(RWrapperPriority::Function, "b.rs", 10, "b10"),
        entry(RWrapperPriority::Function, "a.rs", 30, "a30"),
        entry(RWrapperPriority::Class, "z.rs", 1, "class"),
        entry(RWrapperPriority::Function, "a.rs", 5, "a5"),
        entry(RWrapperPriority::Function, "a.rs", 5, "a5-second"),
    ];
    sort_wrapper_entries(&mut entries);
    let order: Vec<&str> = entries.iter().map(|e| e.content.as_ref()).collect();
    // Classes first; then functions in file order, then source order; the
    // two entries at the same file:line keep their registration order.
    assert_eq!(order, ["class", "a5", "a5-second", "a30", "b10"]);
}

#[test]
fn parse_top_level_fn_def_name_accepts_quoted_operators() {
    for name in [
        "[.Foo", "[[.Foo", "$.Foo", "==.Foo", "+.Foo", "[<-.Foo", r"a\`b\\c",
    ] {
        let line = format!("`{name}` <- function(x, ...) {{}}");
        assert_eq!(parse_top_level_fn_def_name(&line), Some(name));
    }
    for line in [
        "`obj`$method <- function() {}",
        "`obj`[[1]] <- function() {}",
        "`unclosed <- function() {}",
        "`value` <- 1L",
    ] {
        assert_eq!(parse_top_level_fn_def_name(line), None);
    }
}

#[test]
fn parse_top_level_fn_def_name_reads_escaped_backslashes() {
    // `naming::r_def_name` (miniextendr-macros) writes `a\b` as `` `a\\b` ``
    // and `trail\` as `` `trail\\` ``: R reads backslash escapes inside
    // backticks. The scan returns that escaped spelling, and the doubled
    // backslash before the closing backtick must not hide it.
    for (line, name) in [
        (r"`a\\b` <- function(x) {}", r"a\\b"),
        (r"`trail\\` <- function(x) {}", r"trail\\"),
        (r"`x\\\`y` <- function(x) {}", r"x\\\`y"),
    ] {
        assert_eq!(parse_top_level_fn_def_name(line), Some(name), "{line}");
    }
    // The pre-escaping spelling of `trail\` never closed its backtick.
    assert_eq!(
        parse_top_level_fn_def_name(r"`trail\` <- function(x) {}"),
        None
    );
    let map = std::collections::HashMap::new();
    let twice = "`a\\\\b` <- function() 1\n`a\\\\b` <- function() 2\n";
    assert!(detect_duplicate_wrapper_defs(twice, &map).is_err());
}

#[test]
fn detect_duplicate_quoted_operator_defs() {
    let map = std::collections::HashMap::new();
    let content = "`[.Foo` <- function(x, ...) {}\n`[.Foo` <- function(x, ...) {}\n";
    let err = detect_duplicate_wrapper_defs(content, &map).unwrap_err();
    assert!(err.contains("`[.Foo` is defined more than once"), "{err}");
    let mixed = "foo <- function() {}\n`foo` <- function() {}\n";
    assert!(detect_duplicate_wrapper_defs(mixed, &map).is_err());
    let distinct = "`[.Foo` <- function(x, ...) {}\n`[[.Foo` <- function(x, ...) {}\n";
    assert!(detect_duplicate_wrapper_defs(distinct, &map).is_ok());
}

#[test]
fn parse_top_level_fn_def_name_rejects_non_defs() {
    // Indented closures (R6 methods, S7 getters) are not top-level wrappers.
    assert_eq!(
        parse_top_level_fn_def_name("    get_value = function(self) self$x"),
        None
    );
    // S7::method(...) <- function : LHS is a call, not a bare ident.
    assert_eq!(
        parse_top_level_fn_def_name("S7::method(get_value, Shape) <- function(x) .Call(C, x)"),
        None
    );
    // `$`-assignment target is not a bare ident.
    assert_eq!(
        parse_top_level_fn_def_name("obj$method <- function() {}"),
        None
    );
    // Not a function definition at all.
    assert_eq!(parse_top_level_fn_def_name("x <- 1L"), None);
    // Comment / arbitrary line.
    assert_eq!(parse_top_level_fn_def_name("# a comment"), None);
}

/// The preamble ships in every package's `R/` and source tarball, where a
/// bare `#1548` reads as that package's own issue, so its R text cites
/// none (#1832). The citations live in the Rust comments around it.
#[test]
fn preamble_cites_no_issue_numbers() {
    let binding = format_conversion_error_class_binding(&["pkg_error"]);
    for (name, text) in [
        ("WRAPPERS_PREAMBLE_HEADER", WRAPPERS_PREAMBLE_HEADER),
        ("RAISE_CONDITION_HELPER_FN", RAISE_CONDITION_HELPER_FN),
        (
            "WRAPPER_RAISE_CONDITION_HELPER_FN",
            WRAPPER_RAISE_CONDITION_HELPER_FN,
        ),
        ("CALLER_CALL_HELPER", CALLER_CALL_HELPER),
        ("FRAME_CALL_HELPER", FRAME_CALL_HELPER),
        ("FIELD_AT_HELPER", FIELD_AT_HELPER),
        ("ENV_METHOD_HELPER", ENV_METHOD_HELPER),
        ("ARG_ERROR_HELPER", ARG_ERROR_HELPER),
        ("DOTS_CHECK_HELPER", DOTS_CHECK_HELPER),
        ("MATCH_ARG_SEVERAL_HELPER", MATCH_ARG_SEVERAL_HELPER),
        ("MATCH_ARG_HELPER", MATCH_ARG_HELPER),
        ("the conversion_error_class binding", binding.as_str()),
    ] {
        for line in text.lines() {
            let cites = line
                .as_bytes()
                .windows(2)
                .any(|w| w[0] == b'#' && w[1].is_ascii_digit());
            assert!(!cites, "{name} cites an issue number: {line}");
        }
    }
}

/// The preamble's caller-call helper (#1613) takes the wrapper's `.call`
/// as an argument defaulting to `NULL` (an S3 method calls it without
/// one) and `own`, which a `call_arg` wrapper sets so that a frame no
/// closure owns gives its own call, not its caller's (#1834). It is the
/// only top-level definition in its block, so the duplicate-name scan
/// sees it once.
#[test]
fn caller_call_helper_takes_the_call_formal() {
    let defs: Vec<&str> = CALLER_CALL_HELPER
        .lines()
        .filter_map(parse_top_level_fn_def_name)
        .collect();
    assert_eq!(defs, [".miniextendr_caller_call"]);
    assert!(
        CALLER_CALL_HELPER
            .contains(".miniextendr_caller_call <- function(call = NULL, own = FALSE) {")
    );
    // `own` skips the caller lookup.
    assert!(CALLER_CALL_HELPER.contains("  if (!own) {\n    parent <- sys.parent(2L)"));
    // The fallback call leaves the plumbing formal out.
    assert!(CALLER_CALL_HELPER.contains("call$.call <- NULL"));
    // Every call it returns is the call as written (`sys.call()`), never
    // a matched one.
    assert!(!CALLER_CALL_HELPER.contains("match.call"));
    assert!(CALLER_CALL_HELPER.ends_with("}\n"));
}

/// The preamble's frame-call helper (#1851) binds the bare
/// `frame_call_helper_fn!()` expression, the same text the raise helper
/// inlines for its base-namespace copy, and `.miniextendr_arg_error`
/// resolves a frame with it before raising.
#[test]
fn frame_call_helper_is_shared_by_the_raise_helper_and_the_checks() {
    let fn_text = frame_call_helper_fn!();
    let defs: Vec<&str> = FRAME_CALL_HELPER
        .lines()
        .filter_map(parse_top_level_fn_def_name)
        .collect();
    assert_eq!(defs, [".miniextendr_frame_call"]);
    assert!(FRAME_CALL_HELPER.ends_with(&format!(".miniextendr_frame_call <- {fn_text}\n")));
    assert!(RAISE_CONDITION_HELPER_FN.contains(&format!(
        "if (is.environment(.call)) .call <- ({fn_text})(.call)\n"
    )));
    assert!(
        WRAPPER_RAISE_CONDITION_HELPER_FN
            .contains("if (is.environment(.call)) .call <- .miniextendr_frame_call(.call)\n")
    );
    assert!(!WRAPPER_RAISE_CONDITION_HELPER_FN.contains(fn_text));
    // No comment inside the shared text: it is inlined into the raise
    // helper, whose own comments sit outside it.
    assert!(!fn_text.contains('#'));
    // The generic's call replaces the method's; a replacement generic's
    // call is the assignment, with the value as the symbol `value`.
    assert!(fn_text.contains("generic <- env[[\".Generic\"]]"));
    assert!(fn_text.contains("call(\"<-\", call, quote(value))"));
    assert!(
        ARG_ERROR_HELPER
            .contains("  if (is.environment(call)) call <- .miniextendr_frame_call(call)\n")
    );
}

/// The preamble's field-position helper (#1891) and the bad-index error
/// (#1920) are the two definitions in their block. A position is
/// truncated as `[[` truncates it on a list; one that picks no field
/// (past the last, `0`, negative) raises R's own
/// `subscriptOutOfBoundsError` with the type's `field_error` classes in
/// front (#1899), reporting the generic's call resolved from the method's
/// frame. An index that is no name or position raises the `field_error`
/// classes, then `notSubsettableError`, on that call.
#[test]
fn field_at_helper_picks_a_field_as_a_list_does() {
    let defs: Vec<&str> = FIELD_AT_HELPER
        .lines()
        .filter_map(parse_top_level_fn_def_name)
        .collect();
    assert_eq!(
        defs,
        [".miniextendr_field_at", ".miniextendr_field_index_error"]
    );
    assert!(FIELD_AT_HELPER.contains(
        ".miniextendr_field_index_error <- function(cls, env, class = NULL, strict = FALSE) {"
    ));
    assert!(FIELD_AT_HELPER.contains(
        "class = c(class, \"notSubsettableError\"), call = .miniextendr_frame_call(env)))"
    ));
    assert!(
        FIELD_AT_HELPER
            .contains(".miniextendr_field_at <- function(fields, i, env, class = NULL) {")
    );
    assert!(FIELD_AT_HELPER.contains(
        "if (k < 1 || k > length(fields)) stop(errorCondition(\"subscript out of bounds\", \
             class = c(class, \"subscriptOutOfBoundsError\"), call = .miniextendr_frame_call(env)))"
    ));
    assert!(FIELD_AT_HELPER.contains("  fields[[k]]\n"));
    assert!(FIELD_AT_HELPER.starts_with("# Internal helper"));
    assert!(FIELD_AT_HELPER.ends_with("}\n"));
}

/// The preamble's class binding (#1591): the crate classes as an R
/// character vector, escaped, or `NULL` when the crate sets none. It is a
/// plain assignment, not a function definition, so the duplicate-name
/// scan skips it.
#[test]
fn conversion_error_class_binding_quotes_or_nulls() {
    let last_line = |classes: &[&str]| {
        format_conversion_error_class_binding(classes)
            .lines()
            .last()
            .unwrap()
            .to_string()
    };
    assert_eq!(
        last_line(&[]),
        ".miniextendr_conversion_error_class <- NULL"
    );
    assert_eq!(
        last_line(&["pkg_error_argument", "pkg_error"]),
        ".miniextendr_conversion_error_class <- c(\"pkg_error_argument\", \"pkg_error\")"
    );
    assert_eq!(
        last_line(&["a\"b\\c"]),
        ".miniextendr_conversion_error_class <- c(\"a\\\"b\\\\c\")"
    );
    let binding = format_conversion_error_class_binding(&["pkg_error"]);
    assert!(
        binding
            .lines()
            .all(|l| parse_top_level_fn_def_name(l).is_none())
    );
}

/// The raise helper resolves `arg_error!`'s class marker (#1740) against
/// the binding the preamble defines under the same name, and only inside
/// the `if`, so the base-namespace copy that signals deferred conditions
/// never needs the binding.
#[test]
fn raise_helper_resolves_the_conversion_class_marker() {
    let marker = crate::condition::CONVERSION_ERROR_CLASS_MARKER;
    assert_eq!(
        format!("{marker} <- NULL"),
        format_conversion_error_class_binding(&[])
            .lines()
            .last()
            .unwrap()
    );
    let lookup = format!(".at <- match(\"{marker}\", .class)");
    let resolve = format!("get0(\"{marker}\", ifnotfound = NULL)");
    let at = RAISE_CONDITION_HELPER_FN
        .find(&lookup)
        .expect("marker lookup");
    let at_resolve = RAISE_CONDITION_HELPER_FN
        .find(&resolve)
        .expect("resolution");
    let at_switch = RAISE_CONDITION_HELPER_FN.find("switch(.val$kind").unwrap();
    assert!(at < at_resolve && at_resolve < at_switch);
    // The bare binding name appears only as a string (no direct lookup).
    assert_eq!(
        RAISE_CONDITION_HELPER_FN.matches(marker).count(),
        RAISE_CONDITION_HELPER_FN
            .matches(&format!("\"{marker}\""))
            .count()
    );
}

/// A `match_arg` check as the macro writes it, for `splice_match_arg_entry`.
const ALIASES_CHECK: &str = "f <- function(mode = .__MX_MATCH_ARG_CHOICES_f_mode__) {\n  \
        mode <- .miniextendr_match_arg(mode, .__MX_MATCH_ARG_CHOICES_f_mode__, \"mode\", \
        .__MX_MATCH_ARG_ALIASES_f_mode__)\n}";

fn aliases_entry(
    choices: &'static [&'static str],
    aliases: &'static [MatchArgAlias],
) -> MatchArgChoicesEntry {
    MatchArgChoicesEntry {
        placeholder: ".__MX_MATCH_ARG_CHOICES_f_mode__",
        choices,
        preferred_default: "",
        aliases_placeholder: ".__MX_MATCH_ARG_ALIASES_f_mode__",
        aliases,
    }
}

/// A type without aliases gets the check it had before aliases existed;
/// one with aliases gets the `aliases =` argument, after the others
/// (#1843), then the clause naming the listed ones (#2148), which a type
/// whose aliases are all unlisted does without.
#[test]
fn splice_match_arg_entry_writes_the_aliases_argument() {
    assert_eq!(
        splice_match_arg_entry(ALIASES_CHECK, &aliases_entry(&["red", "gray"], &[])),
        "f <- function(mode = c(\"red\", \"gray\")) {\n  \
             mode <- .miniextendr_match_arg(mode, c(\"red\", \"gray\"), \"mode\")\n}"
    );
    assert_eq!(
        splice_match_arg_entry(
            ALIASES_CHECK,
            &aliases_entry(&["red", "gray"], {
                const LISTED: &[MatchArgAlias] = &[
                    MatchArgAlias::new("grey", "gray"),
                    MatchArgAlias::new("rouge", "red"),
                ];
                LISTED
            })
        ),
        "f <- function(mode = c(\"red\", \"gray\")) {\n  \
             mode <- .miniextendr_match_arg(mode, c(\"red\", \"gray\"), \"mode\", \
             aliases = base::c(\"grey\" = \"gray\", \"rouge\" = \"red\"), \
             listed = \" (or \\\"rouge\\\" for \\\"red\\\", \\\"grey\\\" for \\\"gray\\\")\")\n}"
    );
    assert_eq!(
        splice_match_arg_entry(
            ALIASES_CHECK,
            &aliases_entry(&["red", "gray"], {
                const UNLISTED: &[MatchArgAlias] = &[MatchArgAlias::unlisted("grey", "gray")];
                UNLISTED
            })
        ),
        "f <- function(mode = c(\"red\", \"gray\")) {\n  \
             mode <- .miniextendr_match_arg(mode, c(\"red\", \"gray\"), \"mode\", \
             aliases = base::c(\"grey\" = \"gray\"))\n}"
    );
    // With a call argument before it, as under `call = caller`.
    let caller = ALIASES_CHECK.replace("\"mode\", .__MX", "\"mode\", .mx_call, .__MX");
    assert!(
        splice_match_arg_entry(&caller, &aliases_entry(&["red", "gray"], &[]))
            .contains("c(\"red\", \"gray\"), \"mode\", .mx_call)\n")
    );
}

/// Aliases and choices are escaped for an R string literal, and so is
/// the clause, which quotes them as the R check quotes the choices.
#[test]
fn match_arg_aliases_arg_escapes() {
    assert_eq!(match_arg_aliases_arg(&["a"], &[], "p"), "");
    assert_eq!(
        match_arg_aliases_arg(
            &["say \"hi\"", r"c:\path"],
            &[
                MatchArgAlias::unlisted("hello", "say \"hi\""),
                MatchArgAlias::new(r"d:\", r"c:\path")
            ],
            "p"
        ),
        r#", aliases = base::c("hello" = "say \"hi\"", "d:\\" = "c:\\path"), listed = " (or \"d:\\\" for \"c:\\path\")""#
    );
}

/// The wrappers writer refuses the aliases of a hand-written impl that the
/// derive refuses at compile time, naming the placeholder, every problem
/// at once.
#[test]
fn match_arg_aliases_arg_refuses_bad_aliases() {
    let refused = |aliases: &'static [MatchArgAlias]| {
        std::panic::catch_unwind(|| {
            match_arg_aliases_arg(&["red", "gray", "green"], aliases, ".__MX_ALIASES_p__")
        })
        .expect_err("refused")
        .downcast::<String>()
        .map(|m| *m)
        .unwrap()
    };
    const BAD: &[MatchArgAlias] = &[
        MatchArgAlias::new("", "red"),
        MatchArgAlias::new("NA", "red"),
        MatchArgAlias::new("red", "gray"),
        MatchArgAlias::new("grey", "gray"),
        MatchArgAlias::unlisted("grey", "gray"),
        MatchArgAlias::new("blue", "navy"),
        MatchArgAlias::new("gr", "green"),
    ];
    let message = refused(BAD);
    assert!(message.contains(".__MX_ALIASES_p__"), "{message}");
    for problem in [
        "an alias can't be empty",
        "alias \"NA\" would read as a missing value",
        "alias \"red\" is already a choice",
        "alias \"grey\" is given more than once",
        "alias \"blue\" names \"navy\", which is not a choice",
        "alias \"gr\" for \"green\" is a prefix of the choice \"gray\"",
    ] {
        assert!(message.contains(problem), "{problem:?} in {message}");
    }
    // A prefix of its own choice only is fine.
    assert_eq!(
        match_arg_aliases_arg(
            &["red", "gray"],
            &[MatchArgAlias::unlisted("gr", "gray")],
            "p"
        ),
        r#", aliases = base::c("gr" = "gray")"#
    );
}

/// The argument-check helpers define exactly the three preamble functions
/// and end on a closing brace.
#[test]
fn arg_check_helpers_define_the_three_functions() {
    let defs: Vec<&str> = ARG_CHECK_HELPERS
        .iter()
        .flat_map(|helper| helper.lines())
        .filter_map(parse_top_level_fn_def_name)
        .collect();
    assert_eq!(
        defs,
        [
            ".miniextendr_arg_error",
            ".miniextendr_match_arg_several",
            ".miniextendr_match_arg"
        ]
    );
    for helper in ARG_CHECK_HELPERS {
        assert!(helper.starts_with("# Internal helper"));
        assert!(helper.ends_with("}\n"));
    }
}

#[test]
fn detect_duplicate_clean_content_passes() {
    // A sidecar accessor and an unrelated S7 shortcut: distinct names, no clash.
    let content = concat!(
        "Shape_get_area <- function(x) .Call(C_get, x)\n",
        "Shape_set_area <- function(x, value) { .Call(C_set, x, value); invisible(x) }\n",
        "Shape_describe <- function(x, ...) .Call(C_describe, x)\n",
    );
    let map = accessor_map(
        &[SidecarPropEntry {
            rust_type: "Shape",
            field_name: "area",
            prop_doc: "",
        }],
        &[ClassNameEntry {
            rust_type: "Shape",
            r_class_name: "Shape",
            class_system: "s7",
            field_syntax: false,
        }],
    );
    assert!(detect_duplicate_wrapper_defs(content, &map).is_ok());
}

#[test]
fn detect_duplicate_shortcut_vs_sidecar_accessor_fails() {
    // S7 method `get_area` emits `Shape_get_area`, colliding with the sidecar
    // accessor for field `area`. This is the core #991 case.
    let content = concat!(
        "Shape_get_area <- function(x) .Call(C_get, x)\n",
        "Shape_set_area <- function(x, value) { .Call(C_set, x, value); invisible(x) }\n",
        "Shape_get_area <- function(x, ...) .Call(C_method, x)\n",
    );
    let map = accessor_map(
        &[SidecarPropEntry {
            rust_type: "Shape",
            field_name: "area",
            prop_doc: "",
        }],
        &[ClassNameEntry {
            rust_type: "Shape",
            r_class_name: "Shape",
            class_system: "s7",
            field_syntax: false,
        }],
    );
    let err = detect_duplicate_wrapper_defs(content, &map).unwrap_err();
    assert!(err.contains("Shape_get_area"), "msg: {err}");
    // Message must name the sidecar producer and the field.
    assert!(err.contains("sidecar field `area`"), "msg: {err}");
    assert!(err.contains("Shape"), "msg: {err}");
}

#[test]
fn detect_duplicate_honours_class_override_prefix() {
    // The impl block set `class = "Shape"` on Rust type `S7Shape`, so the
    // sidecar accessor is `Shape_get_area`, not `S7Shape_get_area`. The
    // collision must be attributed via the override prefix.
    let content = concat!(
        "Shape_get_area <- function(x) .Call(C_get, x)\n",
        "Shape_get_area <- function(x, ...) .Call(C_method, x)\n",
    );
    let map = accessor_map(
        &[SidecarPropEntry {
            rust_type: "S7Shape",
            field_name: "area",
            prop_doc: "",
        }],
        &[ClassNameEntry {
            rust_type: "S7Shape",
            r_class_name: "Shape",
            class_system: "s7",
            field_syntax: false,
        }],
    );
    let err = detect_duplicate_wrapper_defs(content, &map).unwrap_err();
    assert!(err.contains("Shape_get_area"), "msg: {err}");
    assert!(err.contains("sidecar field `area`"), "msg: {err}");
}

#[test]
fn duplicate_class_and_function_bindings_fail_in_either_order() {
    for class in [
        "Widget <- new.env(parent = emptyenv())",
        "Widget <- R6::R6Class(\"Widget\")",
        "Widget <- S7::new_class(\"Widget\")",
        "`Widget` <- new.env(parent = emptyenv())",
    ] {
        let class = format!("# Generated from Rust impl `Widget` (lib.rs:10:6)\n{class}\n");
        let function = "# Generated from Rust fn `widget` (lib.rs:30:8)\n\
                            Widget <- function() .Call(C_widget)\n";
        for content in [format!("{class}{function}"), format!("{function}{class}")] {
            let error = detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
                .expect_err("a function must not replace a class or namespace object");
            assert!(error.contains("`Widget`"), "{error}");
            assert!(error.contains("class/namespace object"), "{error}");
            assert!(error.contains("r_name"), "{error}");
            assert!(
                error.contains("Rust impl `Widget` (lib.rs:10:6)"),
                "{error}"
            );
            assert!(error.contains("Rust fn `widget` (lib.rs:30:8)"), "{error}");
        }
    }
}

/// Two class objects of one name in the written file: the second would
/// drop the first one's methods (#1677, #1776).
#[test]
fn duplicate_class_objects_name_both_producers() {
    for class in [
        "Widget <- new.env(parent = emptyenv())",
        "Widget <- R6::R6Class(\"Widget\",",
        "Widget <- S7::new_class(\"Widget\",",
    ] {
        let content = format!(
            "# Generated from Rust impl `Widget` (lib.rs:10:6)\n{class}\n\
                 Widget$new <- function() .Call(C_new)\n\
                 # Generated from Rust impl `Widget` (lib.rs:40:6)\n{class}\n"
        );
        let error = detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
            .expect_err("a second class object drops the first one's methods");
        assert!(
            error.contains("class `Widget` is defined more than once"),
            "{error}"
        );
        assert!(error.contains("(lib.rs:10:6)"), "{error}");
        assert!(error.contains("(lib.rs:40:6)"), "{error}");
        assert!(error.contains("distinct `class = \"...\"`"), "{error}");
    }
}

#[test]
fn duplicate_scan_keeps_class_members_and_s4_constructors_separate() {
    let content = concat!(
        "Widget <- new.env(parent = emptyenv())\n",
        "Widget$new <- function() .Call(C_new)\n",
        "Widget$get <- function() .Call(C_get)\n",
        "methods::setClass(\"S4Widget\", slots = c(ptr = \"externalptr\"))\n",
        "S4Widget <- function() .Call(C_new_s4)\n",
        "R6Widget <- R6::R6Class(\"R6Widget\",\n",
        "  public = list(\n",
        "    initialize = function() {}\n",
        "  )\n",
        ")\n",
        "S7Widget <- S7::new_class(\"S7Widget\",\n",
        "  constructor = function() {}\n",
        ")\n",
        "if (!base::exists(\"shared\", mode = \"function\")) {\n",
        "  shared <- function(x, ...) UseMethod(\"shared\")\n",
        "}\n",
        "if (!base::exists(\"shared\", mode = \"function\")) {\n",
        "  shared <- function(x, ...) UseMethod(\"shared\")\n",
        "}\n",
    );
    assert!(detect_duplicate_wrapper_defs(content, &std::collections::HashMap::new()).is_ok());
}

#[test]
fn guarded_generics_cannot_collide_with_plain_bindings() {
    for generic in [
        concat!(
            "if (!base::exists(\"probe\", mode = \"function\")) {\n",
            "  probe <- function(x, ...) UseMethod(\"probe\")\n",
            "}\n",
        ),
        concat!(
            "if (!base::exists(\"probe\", mode = \"function\")) {\n",
            "  probe <- function(x, ...) base::UseMethod(\"probe\")\n",
            "}\n",
        ),
        concat!(
            "if (!exists(\"probe\", where = topenv(environment()), inherits = FALSE)) ",
            "methods::setGeneric(\"probe\", function(x, ...) standardGeneric(\"probe\"))\n",
        ),
        concat!(
            "if (!base::exists(\"probe\", mode = \"function\")) {\n",
            "  # No existing binding; define a plain package-local S7 generic.\n",
            "  probe <- S7::new_generic(\"probe\", \"x\", function(x, ...) S7::S7_dispatch())\n",
            "}\n",
        ),
        concat!(
            "if (!exists(\"probe\", mode = \"function\")) {\n",
            "  probe <- S7::new_generic(\"probe\", \"x\", function(x, ...) S7::S7_dispatch())\n",
            "}\n",
        ),
        concat!(
            "if (!base::inherits(base::get0(\"probe\", inherits = FALSE), \"S7_external_generic\")) {\n",
            "  probe <- S7::new_external_generic(\"pkg\", \"probe\", \"x\")\n",
            "}\n",
        ),
    ] {
        // The same generic may be emitted by many inherent/trait impls.
        let repeated = format!("{generic}{generic}");
        assert!(
            detect_duplicate_wrapper_defs(&repeated, &std::collections::HashMap::new()).is_ok(),
            "{repeated}"
        );
        let generic = format!("# Generated from Rust impl `Shape` (shape.rs:3:6)\n{generic}");
        for binding in [
            "# Generated from Rust fn `probe` (probe.rs:7:8)\n\
                 probe <- function(x) .Call(C_probe, x)\n",
            "# Generated from Rust impl `probe` (probe.rs:7:8)\n\
                 probe <- new.env(parent = emptyenv())\n",
        ] {
            // Classes precede free functions; trait impls follow them.
            for content in [format!("{generic}{binding}"), format!("{binding}{generic}")] {
                let error =
                    detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
                        .expect_err("a generated binding must not mask a generic");
                assert!(error.contains("`probe`"), "{error}");
                assert!(error.contains("guarded generic"), "{error}");
                assert!(error.contains("r_name"), "{error}");
                assert!(error.contains("`Shape` (shape.rs:3:6)"), "{error}");
                assert!(error.contains("`probe` (probe.rs:7:8)"), "{error}");
            }
        }
    }
}

/// A generic shared by several classes is reported with the class that
/// declared it first, and a sidecar accessor with its field and type.
#[test]
fn duplicate_scan_names_first_generic_and_sidecar_producers() {
    let generic = "if (!base::exists(\"Shape_get_area\", mode = \"function\")) {\n  \
                       Shape_get_area <- function(x, ...) UseMethod(\"Shape_get_area\")\n}\n";
    let content = format!(
        "# Generated from Rust source (shape.rs:4:5)\n\
             # Wraps sidecar field `area` on Rust type `Shape` via `C_get` and `C_set`.\n\
             Shape_get_area <- function(x) .Call(C_get, x)\n\
             # Generated from Rust impl `Circle` (circle.rs:9:6)\n{generic}\
             # Generated from Rust impl `Square` (square.rs:9:6)\n{generic}"
    );
    let error = detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
        .expect_err("the generic would replace the sidecar accessor");
    assert!(
        error.contains("function from sidecar field `area` on Rust type `Shape` (shape.rs:4:5)"),
        "{error}"
    );
    assert!(
        error.contains("guarded generic from Rust impl `Circle` (circle.rs:9:6)"),
        "{error}"
    );
    assert!(!error.contains("Square"), "{error}");

    // Reported with the first declaration when the plain binding follows.
    let content = format!(
        "# Generated from Rust impl `Circle` (circle.rs:9:6)\n{generic}\
             # Generated from Rust impl `Square` (square.rs:9:6)\n{generic}\
             # Generated from Rust fn `Shape_get_area` (lib.rs:1:8)\n\
             Shape_get_area <- function(x) .Call(C_area, x)\n"
    );
    let error = detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
        .expect_err("the function would replace the generic");
    assert!(error.contains("`Circle` (circle.rs:9:6)"), "{error}");
    assert!(
        error.contains("Rust fn `Shape_get_area` (lib.rs:1:8)"),
        "{error}"
    );
    assert!(!error.contains("Square"), "{error}");
}

#[test]
fn guarded_generics_recognize_quoted_operator_names() {
    let generic = concat!(
        "if (!base::exists(\"%probe%\", mode = \"function\")) {\n",
        "  `%probe%` <- function(x, ...) UseMethod(\"%probe%\")\n",
        "}\n",
    );
    let content = format!("{generic}`%probe%` <- function(x, y) .Call(C_probe, x, y)\n");
    let error = detect_duplicate_wrapper_defs(&content, &std::collections::HashMap::new())
        .expect_err("quoted assignment names must match the generic name");
    assert!(error.contains("`%probe%`"), "{error}");
}

#[test]
fn duplicate_scan_ignores_local_and_non_generic_guards() {
    let content = concat!(
        "probe <- function(x) {\n",
        "  if (!base::exists(\"probe\", mode = \"function\")) {\n",
        "    probe <- function(x, ...) UseMethod(\"probe\")\n",
        "  }\n",
        "}\n",
        "if (!base::exists(\"cached\", mode = \"function\")) {\n",
        "  cached <- function(x) x\n",
        "}\n",
        "cached <- function(x) .Call(C_cached, x)\n",
    );
    assert!(detect_duplicate_wrapper_defs(content, &std::collections::HashMap::new()).is_ok());
}

#[test]
fn detect_duplicate_cross_impl_block_generic_message() {
    // Two S7 impl blocks both emit `Counter_inc` -- no sidecar involved, so
    // the generic "more than once" message is used.
    let content = concat!(
        "Counter_inc <- function(x, ...) .Call(C_inc_a, x)\n",
        "Counter_inc <- function(x, ...) .Call(C_inc_b, x)\n",
    );
    let map = std::collections::HashMap::new();
    let err = detect_duplicate_wrapper_defs(content, &map).unwrap_err();
    assert!(err.contains("Counter_inc"), "msg: {err}");
    assert!(err.contains("defined more than once"), "msg: {err}");
}

#[test]
fn detect_duplicate_ignores_repeated_s7_method_assignments() {
    // `S7::method(...) <- function` lines are NOT top-level bare defs; a class
    // may legitimately have many method assignments. These must never trip.
    let content = concat!(
        "S7::method(describe, Shape) <- function(x) .Call(C1, x)\n",
        "S7::method(area, Shape) <- function(x) .Call(C2, x)\n",
        "S7::method(describe, Circle) <- function(x) .Call(C3, x)\n",
    );
    let map = std::collections::HashMap::new();
    assert!(detect_duplicate_wrapper_defs(content, &map).is_ok());
}

// endregion

// region: impl blocks merged per type (#1776)

fn widget_path() -> &'static str {
    "pkg::widgets::Widget"
}

fn other_widget_path() -> &'static str {
    "pkg::legacy::Widget"
}

/// An env-class impl block of `Widget` at `lib.rs:<line>`, with the
/// `parts` given as raw-literal text (framed by newlines) and no
/// settings, definitions or member names.
fn widget_block(line: u32, head: &'static str, members: &'static str) -> ClassBlockEntry {
    ClassBlockEntry {
        rust_type: "Widget",
        type_path: widget_path,
        r_class_name: "Widget",
        source_file: "lib.rs",
        source_line: line,
        source_column: 6,
        settings: "env",
        constructor: "",
        constructor_inferred: false,
        constructor_candidates: &[],
        demoted: None,
        defines: &[],
        head,
        class_docs: "\n\n",
        class_doc_lines: "\n\n",
        members,
        member_names: &[],
        env: "\n\n",
        tail: "\n\n",
    }
}

fn merged(blocks: &[ClassBlockEntry]) -> Vec<String> {
    merge_class_blocks(blocks, &[])
        .unwrap_or_else(|msg| panic!("{msg}"))
        .into_iter()
        .map(|f| f.content.into_owned())
        .collect()
}

/// The defining block's topic governs defaults from every block, while
/// an explicit method page (including the original class page) wins.
#[test]
fn class_member_docs_inherit_the_defining_blocks_topic() {
    let mut first = widget_block(10, "\n.__MX_CLASS_DOCS__\nWidget <- new.env()\n", "\n\n");
    first.class_docs = "\n#' @rdname widget_api\n";
    first.defines = &["constructor"];
    let second = widget_block(
        20,
        "\n\n",
        concat!(
            "\n#' @description Author prose.\n",
            "#' @param n Author parameter.\n",
            ".__MX_CLASS_MEMBER_DEFAULT__#' @title Generated title\n",
            ".__MX_CLASS_MEMBER_DEFAULT__#' @param x An object.\n",
            ".__MX_CLASS_MEMBER_DEFAULT__#' @param missing (undocumented)\n",
            ".__MX_CLASS_MEMBER_DEFAULT__#' @rdname Widget\n",
            "#' @name Widget_origin\nWidget_origin <- function(x, n, missing) 1\n\n",
            "#' @rdname Widget\n",
            ".__MX_CLASS_MEMBER_DEFAULT__#' @param n (undocumented)\n",
            "#' @name Widget_explicit\nWidget_explicit <- function(n) 1\n\n",
            "#' @rdname own_page\n#' @order 2\n",
            "Widget_own <- function() 1\n\n",
            "#' @describeIn widget_api A described member.\n",
            "Widget_described <- function() 1\n",
        ),
    );
    let output = merged(&[second, first]).remove(0);
    assert!(
        output.contains(concat!(
            "#' @description Author prose.\n#' @param n Author parameter.\n",
            "#' @rdname widget_api\n#' @name Widget_origin\n#' @order NaN\n",
            "Widget_origin <- function(x, n, missing) 1"
        )),
        "{output}"
    );
    assert!(
        output.contains("#' @rdname Widget\n#' @param n (undocumented)"),
        "{output}"
    );
    assert!(
        output.contains("#' @rdname own_page\n#' @order 2\nWidget_own"),
        "{output}"
    );
    assert!(
        output.contains("#' @describeIn widget_api A described member.\nWidget_described"),
        "{output}"
    );
    assert!(!output.contains(CLASS_MEMBER_DEFAULT_PREFIX), "{output}");
    assert!(!output.contains("Generated title"), "{output}");
    assert!(!output.contains("@param missing"), "{output}");
}

#[test]
fn class_member_docs_keep_default_page_and_author_order() {
    let member = concat!(
        ".__MX_CLASS_MEMBER_DEFAULT__#' @rdname Widget\n",
        ".__MX_CLASS_MEMBER_DEFAULT__#' @param x An object.\n",
        "#' @order 3\nWidget_value <- function(x) x\n",
    );
    assert_eq!(
        resolve_class_member_docs(member, "Widget", "Widget"),
        concat!(
            "#' @rdname Widget\n#' @param x An object.\n",
            "#' @order 3\nWidget_value <- function(x) x\n",
        )
    );
    assert_eq!(
        resolve_class_member_docs(member, "Widget", "widget_api"),
        "#' @rdname widget_api\n#' @order 3\nWidget_value <- function(x) x\n"
    );
}

/// The docs-only block of a class the package names and builds itself
/// is written when an impl block's docs say something, and dropped up
/// to its `NULL` when they name a page at most (#1918).
#[test]
fn docs_only_class_block_needs_something_to_say() {
    const HEAD: &str = "\n.__MX_CLASS_DOCS_ONLY__\n\
                            .__MX_CLASS_DEFAULT__#' @title widget_rec S3 Class\n\
                            .__MX_CLASS_DEFAULT__#' @name widget_rec\n\
                            .__MX_CLASS_DOCS__\n.__MX_CLASS_DOC_LINES__\nNULL\n\n";
    let with_docs = |first_docs: &'static str, second_docs: &'static str| {
        let mut first = widget_block(10, HEAD, "\n# first\n");
        first.class_docs = first_docs;
        let mut second = widget_block(20, "\n\n", "\n# second\n");
        second.class_docs = second_docs;
        merged(&[first, second]).remove(0)
    };

    let silent = with_docs("\n\n", "\n\n");
    assert!(!silent.contains("widget_rec"), "{silent}");
    assert!(!silent.contains("NULL"), "{silent}");
    assert!(
        silent.contains("# first") && silent.contains("# second"),
        "{silent}"
    );

    let page_only = with_docs("\n#' @rdname widget_rec\n", "\n\n");
    assert!(!page_only.contains("NULL"), "{page_only}");

    let documented = with_docs("\n\n", "\n#' @description Records the package builds.\n");
    assert!(
        documented.contains(
            "#' @title widget_rec S3 Class\n#' @name widget_rec\n\
                 #' @description Records the package builds.\nNULL\n"
        ),
        "{documented}"
    );

    let titled = with_docs(
        "\n\n",
        "\n#' @title Package records\n#' @description Records.\n",
    );
    assert!(!titled.contains("S3 Class"), "{titled}");
    assert!(titled.contains("#' @title Package records\n"), "{titled}");
}

#[test]
fn merge_defines_the_class_once_with_every_blocks_members() {
    let mut ctor = widget_block(
        10,
        "\n#' Widget\n.__MX_CLASS_DOC_LINES__\n#' @export\nWidget <- new.env()\n",
        "\nWidget$get <- function() 1\n",
    );
    ctor.defines = &["a constructor"];
    ctor.class_doc_lines = "\n#' @importFrom lifecycle deprecate_warn\n";
    ctor.tail = "\n`$.Widget` <- function(self, name) NULL\n";
    let mut more = widget_block(
        40,
        "\n#' Widget\n.__MX_CLASS_DOC_LINES__\n#' @export\nWidget <- new.env()\n",
        "\nWidget$set <- function() 2\n",
    );
    more.class_doc_lines =
        "\n#' @importFrom lifecycle deprecate_warn\n#' @importFrom lifecycle badge\n";
    // The registration order of the blocks doesn't matter.
    let classes = merged(&[more, ctor]);
    assert_eq!(classes.len(), 1);
    assert_eq!(
        classes[0],
        "# Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             #' Widget\n\
             #' @importFrom lifecycle deprecate_warn\n\
             #' @importFrom lifecycle badge\n\
             #' @export\n\
             Widget <- new.env()\n\
             Widget$get <- function() 1\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             Widget$set <- function() 2\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             `$.Widget` <- function(self, name) NULL"
    );
}

#[test]
fn merge_defines_the_class_from_the_block_with_the_constructor() {
    let first = widget_block(10, "\nWidget <- new.env() # first\n", "\nWidget$a <- 1\n");
    let mut ctor = widget_block(40, "\nWidget <- new.env() # ctor\n", "\nWidget$b <- 2\n");
    ctor.defines = &["a constructor"];
    let classes = merged(&[first, ctor]);
    assert_eq!(
        classes[0],
        "# Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             Widget <- new.env() # ctor\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             Widget$a <- 1\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             Widget$b <- 2"
    );
    // Without one, the first block in source order defines the class.
    let first = widget_block(10, "\nWidget <- new.env() # first\n", "\n\n");
    let second = widget_block(40, "\nWidget <- new.env() # second\n", "\n\n");
    assert_eq!(
        merged(&[second, first])[0],
        "# Generated from Rust impl `Widget` (lib.rs:10:6)\nWidget <- new.env() # first"
    );
}

#[test]
fn merge_keeps_one_block_as_written() {
    // The head's blank last line separates it from the members.
    let mut block = widget_block(
        10,
        "\n#' Widget\n.__MX_CLASS_DOC_LINES__\n#' @export\nWidget <- new.env()\n\n",
        "\nWidget$get <- function() 1\n",
    );
    block.tail = "\nWidget$lock()\n";
    assert_eq!(
        merged(&[block])[0],
        "# Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             #' Widget\n\
             #' @export\n\
             Widget <- new.env()\n\
             \n\
             Widget$get <- function() 1\n\
             Widget$lock()"
    );
}

#[test]
fn merge_refuses_blocks_with_different_settings() {
    // `@noRd` on one block only: the other's methods would be documented
    // on a page that does not exist.
    let mut hidden = widget_block(10, "\n\n", "\n\n");
    hidden.settings = "env, @noRd";
    let plain = widget_block(40, "\n\n", "\n\n");
    let msg = merge_class_blocks(&[hidden, plain], &[])
        .err()
        .expect("settings differ");
    assert!(
        msg.contains("the impl blocks of `Widget` disagree on the class settings"),
        "{msg}"
    );
    assert!(
        msg.contains("`env, @noRd` on `impl Widget` (lib.rs:10)"),
        "{msg}"
    );
    assert!(msg.contains("`env` on `impl Widget` (lib.rs:40)"), "{msg}");
}

#[test]
fn merge_refuses_two_defining_blocks() {
    let mut ctor = widget_block(10, "\n\n", "\n\n");
    ctor.defines = &["a constructor"];
    let mut props = widget_block(40, "\n\n", "\n\n");
    props.defines = &["a constructor", "S7 properties"];
    let msg = merge_class_blocks(&[ctor, props], &[])
        .err()
        .expect("two blocks define the class");
    assert!(
        msg.contains(
            "two impl blocks of `Widget` each define the class: `impl Widget` (lib.rs:10) \
                 has a constructor, `impl Widget` (lib.rs:40) has a constructor and S7 \
                 properties."
        ),
        "{msg}"
    );
    assert!(
        msg.contains("move what `impl Widget` (lib.rs:40) has into `impl Widget` (lib.rs:10)"),
        "{msg}"
    );
}

#[test]
fn merge_refuses_a_member_bound_by_two_blocks() {
    let mut first = widget_block(10, "\n\n", "\n\n");
    first.member_names = &["`Widget$get`", "`Widget$set`"];
    let mut second = widget_block(40, "\n\n", "\n\n");
    second.member_names = &["`Widget$get`"];
    let msg = merge_class_blocks(&[first, second], &[])
        .err()
        .expect("`$get` is bound twice");
    assert!(
        msg.contains(
            "two impl blocks of `Widget` define `Widget$get`: `impl Widget` (lib.rs:10) \
                 and `impl Widget` (lib.rs:40)."
        ),
        "{msg}"
    );
    assert!(!msg.contains("Widget$set"), "{msg}");
}

#[test]
fn merge_refuses_two_types_with_one_class_name() {
    // Same identifier, different modules: two types, not two blocks of one.
    let mine = widget_block(10, "\nWidget <- new.env()\n", "\n\n");
    let mut theirs = widget_block(40, "\nWidget <- new.env()\n", "\n\n");
    theirs.type_path = other_widget_path;
    theirs.source_file = "legacy.rs";
    let msg = merge_class_blocks(&[mine, theirs], &[])
        .err()
        .expect("one class, two types");
    assert!(
        msg.contains(
            "R class `Widget` is claimed by two Rust types, `pkg::legacy::Widget` \
                 (`impl Widget` (legacy.rs:40)) and `pkg::widgets::Widget` \
                 (`impl Widget` (lib.rs:10))"
        ),
        "{msg}"
    );
}

/// A class head as the class generators write it: generated defaults, the
/// docs and doc-lines markers, then the class object.
const DOCUMENTED_HEAD: &str = "\n\
        .__MX_CLASS_DEFAULT__#' @title Widget Class\n\
        .__MX_CLASS_DEFAULT__#' @name Widget\n\
        .__MX_CLASS_DEFAULT__#' @rdname Widget\n\
        .__MX_CLASS_DOCS__\n\
        .__MX_CLASS_DEFAULT__#' @param .ptr Internal pointer.\n\
        #' @source Generated by miniextendr\n\
        .__MX_CLASS_DOC_LINES__\n\
        .__MX_CLASS_DEFAULT__#' @export\n\
        Widget <- new.env()\n";

#[test]
fn merge_writes_every_blocks_docs_after_the_definers() {
    let mut ctor = widget_block(40, DOCUMENTED_HEAD, "\n\n");
    ctor.defines = &["a constructor"];
    ctor.class_docs = "\n#' @description Widgets hold a value.\n#' @importFrom stats median\n";
    let mut first = widget_block(10, DOCUMENTED_HEAD, "\n\n");
    first.class_docs = "\n#' @description More methods.\n#' @details One.\n#'\n#' }\n\
                            #' @importFrom stats median\n#' @export\n";
    let mut last = widget_block(70, DOCUMENTED_HEAD, "\n\n");
    last.class_docs = "\n#' @details Two.\n#'\n#' }\n#' @param .ptr Pointer.\n";
    let classes = merged(&[last, first, ctor]);
    // The defining block's docs, then the others' in source order; an
    // identical tag once, the blank and `}` lines of two tags each kept;
    // a default another block writes itself gives way.
    assert_eq!(
        classes[0],
        "# Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             #' @title Widget Class\n\
             #' @name Widget\n\
             #' @rdname Widget\n\
             #' @description Widgets hold a value.\n\
             #' @importFrom stats median\n\
             #' @description More methods.\n\
             #' @details One.\n\
             #'\n\
             #' }\n\
             #' @export\n\
             #' @details Two.\n\
             #'\n\
             #' }\n\
             #' @param .ptr Pointer.\n\
             #' @source Generated by miniextendr\n\
             Widget <- new.env()"
    );
}

#[test]
fn merge_takes_the_title_from_any_block() {
    let mut ctor = widget_block(10, DOCUMENTED_HEAD, "\n\n");
    ctor.defines = &["a constructor"];
    let mut more = widget_block(40, DOCUMENTED_HEAD, "\n\n");
    more.class_docs = "\n#' @title Widgets\n#' @rdname Widget\n";
    let classes = merged(&[ctor, more]);
    assert!(
        classes[0].starts_with(
            "# Generated from Rust impl `Widget` (lib.rs:10:6)\n\
                 #' @name Widget\n\
                 #' @title Widgets\n\
                 #' @rdname Widget\n\
                 #' @param .ptr Internal pointer.\n"
        ),
        "{}",
        classes[0]
    );
}

#[test]
fn merge_drops_a_member_default_another_block_documents() {
    let mut ctor = widget_block(10, DOCUMENTED_HEAD, "\n\n");
    ctor.defines = &["a constructor"];
    ctor.class_docs = "\n#' @field doubled Twice the value.\n#' @inheritParams widget_ops\n";
    let mut more = widget_block(40, DOCUMENTED_HEAD, "\n\n");
    more.class_doc_lines = "\n.__MX_CLASS_DEFAULT__#' @field doubled Active binding.\n\
                                .__MX_CLASS_DEFAULT__#' @field halved Active binding.\n";
    let class = &merged(&[ctor, more])[0];
    assert!(
        class.contains("#' @field doubled Twice the value.\n"),
        "{class}"
    );
    assert!(!class.contains("@field doubled Active binding."), "{class}");
    assert!(
        class.contains("#' @field halved Active binding.\n"),
        "{class}"
    );
    // The defining block's own docs were seen when its defaults were
    // generated: its `@param .ptr` default stays.
    assert!(
        class.contains("#' @param .ptr Internal pointer.\n"),
        "{class}"
    );
}

#[test]
fn merge_refuses_two_titles() {
    let mut ctor = widget_block(10, DOCUMENTED_HEAD, "\n\n");
    ctor.defines = &["a constructor"];
    ctor.class_docs = "\n#' @title Widgets\n";
    let mut more = widget_block(40, DOCUMENTED_HEAD, "\n\n");
    more.class_docs = "\n#' @title Gadgets\n";
    let same = widget_block(70, DOCUMENTED_HEAD, "\n\n");
    let msg = merge_class_blocks(&[ctor, more, same], &[])
        .err()
        .expect("two titles");
    assert_eq!(
        msg,
        "miniextendr: the impl blocks of `Widget` give the class two titles: \
             `impl Widget` (lib.rs:10) has `@title Widgets`, `impl Widget` (lib.rs:40) has \
             `@title Gadgets`. A type's impl blocks merge into one class with one help \
             page: keep one `@title` (#1776)."
    );
}

#[test]
fn merge_refuses_a_page_other_than_the_definers() {
    let mut ctor = widget_block(10, DOCUMENTED_HEAD, "\n\n");
    ctor.defines = &["a constructor"];
    let mut more = widget_block(40, DOCUMENTED_HEAD, "\n\n");
    more.class_docs = "\n#' @rdname widget_ops\n";
    let msg = merge_class_blocks(&[ctor, more], &[])
        .err()
        .expect("two pages");
    assert_eq!(
        msg,
        "miniextendr: the impl blocks of `Widget` put the class on two help pages: \
             `impl Widget` (lib.rs:10), which defines the class, has `@rdname Widget`, and \
             `impl Widget` (lib.rs:40) has `@rdname widget_ops`. The block that defines \
             the class decides its page: move `@rdname widget_ops` into `impl Widget` \
             (lib.rs:10), or remove it (#1776)."
    );
}

#[test]
fn merge_reports_every_problem_at_once() {
    let mut first = widget_block(10, "\n\n", "\n\n");
    first.defines = &["a constructor"];
    first.member_names = &["`Widget$get`"];
    let mut second = widget_block(40, "\n\n", "\n\n");
    second.defines = &["a constructor"];
    second.member_names = &["`Widget$get`"];
    second.settings = "env, internal";
    let msg = merge_class_blocks(&[first, second], &[])
        .err()
        .expect("three problems");
    assert_eq!(msg.lines().count(), 3, "{msg}");
}

// endregion

// region: the constructor decided per type

/// A block of `Widget` that infers `from_value` as its constructor: its
/// own rendering defines the class with it, its `demoted` one keeps it a
/// static method.
fn inferring_block(line: u32) -> ClassBlockEntry {
    let mut demoted = widget_block(
        line,
        "\nWidget <- new.env() # no constructor\n",
        "\nWidget$from_value <- function(x) 1\n",
    );
    demoted.constructor_candidates = &["from_value"];
    let mut block = widget_block(
        line,
        "\nWidget <- new.env() # from_value\nWidget$new <- function(x) 1\n",
        "\n\n",
    );
    block.constructor = "from_value";
    block.constructor_inferred = true;
    block.constructor_candidates = &["from_value"];
    block.defines = &["the inferred constructor `from_value`"];
    block.demoted = Some(Box::leak(Box::new(demoted)));
    block
}

#[test]
fn a_lone_candidate_is_the_constructor_whichever_block_has_it() {
    let methods = widget_block(10, "\nWidget <- new.env() # first\n", "\nWidget$get <- 2\n");
    let classes = merged(&[methods, inferring_block(40)]);
    assert_eq!(
        classes[0],
        "# Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             Widget <- new.env() # from_value\n\
             Widget$new <- function(x) 1\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             Widget$get <- 2"
    );
}

#[test]
fn a_declared_constructor_keeps_the_candidate_a_static_method() {
    let mut new = widget_block(10, "\nWidget <- new.env() # new\n", "\n\n");
    new.constructor = "new";
    new.constructor_candidates = &["from_file"];
    new.defines = &["a constructor"];
    let classes = merged(&[inferring_block(40), new]);
    assert_eq!(
        classes[0],
        "# Generated from Rust impl `Widget` (lib.rs:10:6)\n\
             Widget <- new.env() # new\n\
             \n\
             # Generated from Rust impl `Widget` (lib.rs:40:6)\n\
             Widget$from_value <- function(x) 1"
    );
}

#[test]
fn candidates_in_two_blocks_without_a_declared_constructor_are_refused() {
    let mut other = widget_block(10, "\n\n", "\n\n");
    other.constructor_candidates = &["empty", "from_file"];
    let msg = merge_class_blocks(&[other, inferring_block(40)], &[])
        .err()
        .expect("three candidates");
    assert!(
        msg.contains(
            "`Widget` has several methods that could be its constructor, and none is \
                 declared: `empty`, `from_file` in `impl Widget` (lib.rs:10); `from_value` in \
                 `impl Widget` (lib.rs:40)."
        ),
        "{msg}"
    );
    assert!(msg.contains("`#[miniextendr(env(constructor))]`"), "{msg}");
    // The candidates are the only problem reported.
    assert_eq!(msg.lines().count(), 1, "{msg}");
}

// endregion

// region: the S3 class environment decided per type (#2264)

/// An S3 impl block of `Widget` without a constructor, as the macro emits
/// it: its page on `NULL`, a method, and the class environment as its `env`
/// part.
fn s3_widget_block() -> ClassBlockEntry {
    let mut block = widget_block(
        10,
        "\n#' @title Widget S3 Class\n#' @name Widget\nNULL\n\n",
        "\nsize.Widget <- function(x, ...) 1\n\n",
    );
    block.settings = "s3";
    block.env = "\n#' @rdname Widget\n#' @export\nWidget <- new.env(parent = emptyenv())\n\n";
    block
}

/// The classes `blocks` merge into, with `trait_namespaces` registered.
fn merged_with(blocks: &[ClassBlockEntry], trait_namespaces: &[TraitNamespaceEntry]) -> String {
    merge_class_blocks(blocks, trait_namespaces)
        .unwrap_or_else(|msg| panic!("{msg}"))
        .remove(0)
        .content
        .into_owned()
}

/// Without a constructor or a trait impl writing into it, nothing lands in
/// the class environment: it is not written, nor exported, and no header is
/// left behind for the parts after the members.
#[test]
fn class_env_without_constructor_or_trait_namespace_is_dropped() {
    assert_eq!(
        merged_with(&[s3_widget_block()], &[]),
        "# Generated from Rust impl `Widget` (lib.rs:10:6)\n\
         #' @title Widget S3 Class\n\
         #' @name Widget\n\
         NULL\n\
         \n\
         size.Widget <- function(x, ...) 1\n"
    );
}

/// A trait impl's `Class$Trait` namespace keeps the environment.
#[test]
fn class_env_with_a_trait_namespace_is_kept() {
    let output = merged_with(
        &[s3_widget_block()],
        &[TraitNamespaceEntry {
            type_path: widget_path,
        }],
    );
    assert!(
        output.contains(
            "size.Widget <- function(x, ...) 1\n\n\
             #' @rdname Widget\n\
             #' @export\n\
             Widget <- new.env(parent = emptyenv())"
        ),
        "{output}"
    );
}

/// A constructor keeps the environment for `Class$new()`, written between
/// the members and the tail that binds it.
#[test]
fn class_env_with_a_constructor_is_kept_before_the_tail() {
    let mut block = s3_widget_block();
    block.constructor = "new";
    block.defines = &["a constructor"];
    block.tail = "\nWidget$new <- new_widget\n\n";
    let output = merged_with(&[block], &[]);
    assert!(
        output.contains(
            "Widget <- new.env(parent = emptyenv())\n\n\
             Widget$new <- new_widget"
        ),
        "{output}"
    );
}

/// An S3 block of `Widget` that infers `from_value` as its constructor, with
/// its `demoted` rendering (no constructor), as [`inferring_block`].
fn s3_inferring_block() -> ClassBlockEntry {
    let mut demoted = s3_widget_block();
    demoted.constructor_candidates = &["from_value"];
    let mut block = s3_widget_block();
    block.constructor = "from_value";
    block.constructor_inferred = true;
    block.constructor_candidates = &["from_value"];
    block.defines = &["the inferred constructor `from_value`"];
    block.demoted = Some(Box::leak(Box::new(demoted)));
    block
}

/// The constructor is decided across the type's blocks (#1776), and the
/// environment with it: a lone inferred constructor keeps it, and so does a
/// constructor declared in another block, whose environment is written once.
#[test]
fn class_env_follows_the_resolved_constructor() {
    let lone = merged_with(&[s3_inferring_block()], &[]);
    assert!(lone.contains("Widget <- new.env("), "{lone}");

    let mut declared = widget_block(30, "\n\n", "\n\n");
    declared.settings = "s3";
    declared.constructor = "new";
    declared.defines = &["a constructor"];
    declared.env = "\n#' @export\nWidget <- new.env(parent = emptyenv()) # new\n\n";
    let output = merged_with(&[s3_inferring_block(), declared], &[]);
    assert!(
        output.contains("Widget <- new.env(parent = emptyenv()) # new"),
        "{output}"
    );
    assert_eq!(output.matches("new.env(").count(), 1, "{output}");
}

/// Trait namespaces are matched by `core::any::type_name`: a trait impl on
/// another type of the same name keeps nothing here.
#[test]
fn class_env_trait_namespaces_match_the_type_path() {
    let output = merged_with(
        &[s3_widget_block()],
        &[TraitNamespaceEntry {
            type_path: other_widget_path,
        }],
    );
    assert!(!output.contains("new.env("), "{output}");
    assert!(!output.contains("@export"), "{output}");
}

// endregion

/// The `WrapAs*` and trait-impl form (#2265, #2266): a registered class
/// gives its R name whatever its system; a type with no impl block keeps its
/// Rust name, quietly.
#[test]
fn or_bare_resolves_renames_and_keeps_unregistered_names() {
    let entries = [
        ClassNameEntry {
            rust_type: "EngineS3",
            r_class_name: "engine",
            class_system: "s3",
            field_syntax: false,
        },
        ClassNameEntry {
            rust_type: "BoardR6",
            r_class_name: "Board",
            class_system: "r6",
            field_syntax: false,
        },
    ];
    let input = "structure(.val, class = \".__MX_CLASS_REF_OR_BARE_EngineS3__\")\n\
                 print..__MX_CLASS_REF_OR_BARE_EngineS3__ <- function(x, ...) x\n\
                 .__MX_CLASS_REF_OR_BARE_EngineS3__$Trait <- new.env(parent = emptyenv())\n\
                 structure(.val, class = \".__MX_CLASS_REF_OR_BARE_BoardR6__\")\n\
                 structure(.val, class = \".__MX_CLASS_REF_OR_BARE_Unregistered__\")";
    assert_eq!(
        resolve_refs(input, &entries),
        "structure(.val, class = \"engine\")\n\
         print.engine <- function(x, ...) x\n\
         engine$Trait <- new.env(parent = emptyenv())\n\
         structure(.val, class = \"Board\")\n\
         structure(.val, class = \"Unregistered\")"
    );
}

/// The three forms side by side: each takes its own fallback on a miss, and
/// a malformed placeholder is written back whole.
#[test]
fn class_ref_forms_resolve_independently() {
    let entries = [ClassNameEntry {
        rust_type: "Point",
        r_class_name: "point",
        class_system: "s3",
        field_syntax: false,
    }];
    assert_eq!(
        resolve_refs(
            ".__MX_CLASS_REF_Point__ .__MX_CLASS_REF_OR_BARE_Point__ .__MX_CLASS_REF_OR_ANY_Point__",
            &entries
        ),
        "point point S7::class_any"
    );
    assert_eq!(
        resolve_refs(
            ".__MX_CLASS_REF_Gone__ .__MX_CLASS_REF_OR_BARE_Gone__ .__MX_CLASS_REF_OR_ANY_Gone__",
            &entries
        ),
        "Gone Gone S7::class_any"
    );
    assert_eq!(
        resolve_refs("x <- .__MX_CLASS_REF_OR_BARE_Point", &entries),
        "x <- .__MX_CLASS_REF_OR_BARE_Point"
    );
}
