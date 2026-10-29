//! `match_arg_param` agrees with the generated wrapper's `.miniextendr_match_arg`
//! (#1741): the same choice, or the same argument error word for word, on every
//! kind of input. So does the generated conversion of a `match_arg` type
//! without the attribute, whose message the conversion arm takes from
//! `__mx_conversion_arg_message!` (#1767), and `match_arg_param_with_default`
//! for a `default = "..."` parameter, against the formal the wrappers writer
//! builds for it (#1767). With aliases, both name the listed ones after the
//! choices, from the clause the wrappers writer passes the R helper (#2148).
//!
//! The R side is the preamble source itself (`registry::ARG_CHECK_HELPERS`),
//! evaluated in a fresh environment, so the comparison is against the text the
//! wrappers file ships, not a copy of it.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::match_arg::MatchArgAlias;
use miniextendr_api::registry::{ARG_CHECK_HELPERS, match_arg_aliases_arg, match_arg_formal};
use miniextendr_api::sys::R_GlobalEnv;
use miniextendr_api::{
    MatchArg, MatchArgError, OwnedProtect, SEXP, match_arg_from_sexp, match_arg_param,
    match_arg_param_with_default,
};

/// A choice type over arbitrary strings: `from_choice` is an exact lookup in
/// `CHOICES`, as the derive's is, and the aliases are `new(alias, choice)`
/// (listed) or `unlisted(alias, choice)` entries, as the derive writes them.
macro_rules! choice_type {
    ($name:ident, [$($choice:literal),+ $(,)?] $(, aliases = [$($kind:ident($alias:literal, $target:literal)),+ $(,)?])?) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct $name(usize);

        impl MatchArg for $name {
            const CHOICES: &'static [&'static str] = &[$($choice),+];
            $(const ALIASES: &'static [MatchArgAlias] = &[$(MatchArgAlias::$kind($alias, $target)),+];)?

            fn from_choice(choice: &str) -> Option<Self> {
                Self::CHOICES.iter().position(|c| *c == choice).map($name)
            }

            fn to_choice(self) -> &'static str {
                Self::CHOICES[self.0]
            }
        }
    };
}

choice_type!(Fill, ["drop", "draw", "error"]);
choice_type!(Mode, ["Fast", "Safe", "Debug"]);
choice_type!(NaPrefixed, ["NAME", "other"]);
choice_type!(Quoted, ["say \"hi\"", r"c:\path", "plain"]);
choice_type!(Single, ["only"]);
choice_type!(WithEmpty, ["", "x"]);
// Aliases (#1843): "gr" is a unique prefix of "gray" and must stay one.
// Written out of the order of their choices, which the messages follow
// (#2148).
choice_type!(
    Shade,
    ["red", "gray", "blue"],
    aliases = [new("grey", "gray"), new("rouge", "red")]
);
choice_type!(
    QuotedAlias,
    ["say \"hi\"", "plain"],
    aliases = [
        new("hello \"you\"", "say \"hi\""),
        new(r"c:\plain", "plain")
    ]
);
// Every alias unlisted: the messages of a type without aliases.
choice_type!(
    QuietShade,
    ["red", "gray", "blue"],
    aliases = [unlisted("grey", "gray"), unlisted("rouge", "red")]
);
// Listed and unlisted aliases, several on one choice.
choice_type!(
    MixedShade,
    ["red", "gray", "blue"],
    aliases = [
        unlisted("grey", "gray"),
        new("Grau", "gray"),
        new("bleu", "blue"),
        new("gris", "gray"),
        unlisted("rouge", "red"),
    ]
);

/// `c("a", "b")`: the formal the wrappers writer splices for a `match_arg`
/// parameter of type `T`, with `default = "<preferred>"` (`""` for none).
fn r_formal<T: MatchArg>(preferred: &str) -> String {
    match_arg_formal(T::CHOICES, preferred, "test")
}

/// `, aliases = base::c(...)` (and `, listed = "..."` when an alias is
/// listed), or nothing: the arguments the wrappers writer splices into the
/// check of a `match_arg` parameter of type `T` (#1843, #2148).
fn r_aliases<T: MatchArg>() -> String {
    match_arg_aliases_arg(T::CHOICES, T::ALIASES, "test")
}

/// The outcome as `c("ok", <choice>)` or `c("error", <message>, <param>)`.
type Outcome = Vec<String>;

/// The wrapper's helper on the R value of `input`, with the choices `formal`
/// (`c(...)`) and the aliases argument `aliases` (`, aliases = ...` or
/// empty).
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn r_outcome(env: SEXP, input: &str, formal: &str, aliases: &str) -> Outcome {
    let code = format!(
        r#"local({{
  r <- tryCatch(.miniextendr_match_arg({input}, {formal}, "mode"{aliases}), error = function(e) e)
  if (inherits(r, "error")) {{
    stopifnot(
      identical(class(r), c("rust_error", "simpleError", "error", "condition")),
      identical(r$kind, "conversion")
    )
    c("error", conditionMessage(r), r$param)
  }} else c("ok", r)
}})"#
    );
    let value = unsafe { r_eval_str(&code, env) }.unwrap_or_else(|e| panic!("{input}: {e}"));
    Vec::<String>::try_from_sexp(value).expect("a character vector")
}

/// `match` on the R value of `input`, as an outcome.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn rust_outcome<T: MatchArg>(
    env: SEXP,
    input: &str,
    matcher: impl FnOnce(SEXP) -> Result<T, (String, String)>,
) -> Outcome {
    let value = unsafe { r_eval_str(input, env) }.unwrap_or_else(|e| panic!("{input}: {e}"));
    let value = unsafe { OwnedProtect::new(value) };
    match matcher(value.get()) {
        Ok(choice) => vec!["ok".into(), choice.to_choice().into()],
        Err((message, param)) => vec!["error".into(), message, param],
    }
}

/// What the generated conversion of a `T` parameter named `mode` reports:
/// the choice, or the message the conversion arm puts on the condition. The
/// derived `TryFromSexp` returns the `SexpError` this builds.
fn by_conversion<T: MatchArg>(sexp: SEXP) -> Result<T, (String, String)> {
    match_arg_from_sexp::<T>(sexp).map_err(|e| {
        let e = SexpError::from(e);
        let message = miniextendr_api::__mx_conversion_arg_message!(e, "mode")
            .expect("a match_arg error words the whole argument error");
        (message, "mode".to_string())
    })
}

/// Every input on all three paths; returns the shared outcome for the spot
/// checks.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn agree<T: MatchArg>(env: SEXP, input: &str) -> Outcome {
    let r = unsafe { r_outcome(env, input, &r_formal::<T>(""), &r_aliases::<T>()) };
    let in_body = unsafe {
        rust_outcome::<T>(env, input, |sexp| {
            match_arg_param::<T>(sexp, "mode").map_err(|e| (e.message().into(), e.param().into()))
        })
    };
    assert_eq!(
        in_body,
        r,
        "{} on `{input}`: match_arg_param (left) vs .miniextendr_match_arg (right)",
        std::any::type_name::<T>()
    );
    let converted = unsafe { rust_outcome::<T>(env, input, by_conversion::<T>) };
    assert_eq!(
        converted,
        r,
        "{} on `{input}`: the conversion (left) vs .miniextendr_match_arg (right)",
        std::any::type_name::<T>()
    );
    r
}

/// `input` on `.miniextendr_match_arg` with the formal of a `default =
/// "<preferred>"` parameter, and on `match_arg_param_with_default` with
/// `default`.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn agree_with_default<T: MatchArg>(
    env: SEXP,
    input: &str,
    preferred: &str,
    default: T,
) -> Outcome {
    let r = unsafe { r_outcome(env, input, &r_formal::<T>(preferred), &r_aliases::<T>()) };
    let in_body = unsafe {
        rust_outcome::<T>(env, input, |sexp| {
            match_arg_param_with_default::<T>(sexp, "mode", default)
                .map_err(|e| (e.message().into(), e.param().into()))
        })
    };
    assert_eq!(
        in_body,
        r,
        "{} on `{input}` with default `{preferred}`: match_arg_param_with_default (left) \
         vs .miniextendr_match_arg (right)",
        std::any::type_name::<T>()
    );
    r
}

/// A fresh environment holding the wrappers preamble's argument-check
/// helpers, with no `conversion_error_class`.
///
/// # Safety
///
/// R main thread.
unsafe fn helpers_env() -> OwnedProtect {
    let env =
        unsafe { r_eval_str("new.env(parent = baseenv())", R_GlobalEnv) }.expect("an environment");
    let env = unsafe { OwnedProtect::new(env) };
    for helper in ARG_CHECK_HELPERS {
        unsafe { r_eval_str(helper, env.get()) }.expect("the preamble helpers parse and evaluate");
    }
    unsafe { r_eval_str(".miniextendr_conversion_error_class <- NULL", env.get()) }.unwrap();
    env
}

fn ok(choice: &str) -> Outcome {
    vec!["ok".into(), choice.into()]
}

fn err(message: &str) -> Outcome {
    vec!["error".into(), message.into(), "mode".into()]
}

#[test]
fn match_arg_param_agrees_with_the_wrapper_helper() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        let one_of = r#"'mode' should be one of "drop", "draw", "error""#;
        let not_character = "'mode' must be NULL or a character vector";
        let not_scalar = "'mode' must be of length 1";
        let is_na = "'mode' must not be NA";

        let rows: &[(&str, Outcome)] = &[
            // NULL and the formal default select the first choice.
            ("NULL", ok("drop")),
            (r#"c("drop", "draw", "error")"#, ok("drop")),
            // ... but only the default itself: another order, or names, is a
            // vector of length 3.
            (r#"c("draw", "drop", "error")"#, err(not_scalar)),
            (r#"c(a = "drop", b = "draw", c = "error")"#, err(not_scalar)),
            // Exact, unique prefix, ambiguous prefix, no match, empty string.
            (r#""draw""#, ok("draw")),
            (r#""e""#, ok("error")),
            (r#""dro""#, ok("drop")),
            (r#""dr""#, err(one_of)),
            (r#""zzz""#, err(one_of)),
            (r#""DROP""#, err(one_of)),
            (r#""""#, err(one_of)),
            // A classed string is still a string.
            (r#"structure("draw", class = "pkg_mode")"#, ok("draw")),
            // Factors are read as their labels.
            (r#"factor("draw")"#, ok("draw")),
            (r#"factor("e", levels = c("x", "e"))"#, ok("error")),
            (r#"factor(c("drop", "draw", "error"))"#, ok("drop")),
            (r#"factor(c("drop", "draw"))"#, err(not_scalar)),
            (r#"factor("zzz")"#, err(one_of)),
            ("factor(NA_character_)", err(is_na)),
            // NA, length, and type: an NA that matches no choice, or one in a
            // value that is not character, is refused as NA.
            ("NA_character_", err(is_na)),
            (r#"c("drop", "draw")"#, err(not_scalar)),
            ("character(0)", err(not_scalar)),
            (r#"c("drop", NA)"#, err(not_scalar)),
            ("1L", err(not_character)),
            ("2.5", err(not_character)),
            ("NA", err(is_na)),
            ("NA_real_", err(is_na)),
            ("c(1, NA)", err(is_na)),
            ("TRUE", err(not_character)),
            (r#"list("drop")"#, err(not_character)),
            ("quote(drop)", err(not_character)),
        ];
        for (input, expected) in rows {
            assert_eq!(&agree::<Fill>(env, input), expected, "Fill on `{input}`");
        }

        // `pmatch()` matches NA as the string "NA".
        assert_eq!(agree::<NaPrefixed>(env, "NA_character_"), ok("NAME"));
        assert_eq!(agree::<NaPrefixed>(env, "factor(NA)"), ok("NAME"));
        assert_eq!(agree::<NaPrefixed>(env, r#""N""#), ok("NAME"));

        // The message quotes the choices as `dQuote(x, FALSE)` does: no escapes.
        let quoted = r#"'mode' should be one of "say "hi"", "c:\path", "plain""#;
        assert_eq!(agree::<Quoted>(env, r#""zzz""#), err(quoted));
        assert_eq!(agree::<Quoted>(env, r#""say""#), ok("say \"hi\""));
        assert_eq!(agree::<Quoted>(env, r#""c:""#), ok(r"c:\path"));
        assert_eq!(
            agree::<Quoted>(env, &r_formal::<Quoted>("")),
            ok("say \"hi\"")
        );

        // A single choice: the empty string still matches nothing.
        assert_eq!(
            agree::<Single>(env, r#""""#),
            err(r#"'mode' should be one of "only""#)
        );
        assert_eq!(agree::<Single>(env, r#""on""#), ok("only"));
        assert_eq!(agree::<Single>(env, "NULL"), ok("only"));

        // An empty choice is the first choice for NULL, never matched by "".
        assert_eq!(agree::<WithEmpty>(env, "NULL"), ok(""));
        assert_eq!(
            agree::<WithEmpty>(env, r#""""#),
            err(r#"'mode' should be one of "", "x""#)
        );
        assert_eq!(agree::<WithEmpty>(env, r#""x""#), ok("x"));
    });
}

/// `match_arg_param_with_default` agrees with `.miniextendr_match_arg` on the
/// formal the wrappers writer builds for a `default = "..."` parameter
/// (#1767): the issue's table, then the other kinds of input.
#[test]
fn match_arg_param_with_default_agrees_with_the_rotated_formal() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        // The default moves to the front; the others keep their order (not a
        // rotation, which would give `"Safe", "Debug", "Fast"`).
        assert_eq!(r_formal::<Mode>("Safe"), r#"c("Safe", "Fast", "Debug")"#);
        assert_eq!(r_formal::<Mode>("Debug"), r#"c("Debug", "Fast", "Safe")"#);
        assert_eq!(r_formal::<Mode>(""), r#"c("Fast", "Safe", "Debug")"#);

        let safe = Mode(1);
        let one_of = r#"'mode' should be one of "Safe", "Fast", "Debug""#;
        let not_scalar = "'mode' must be of length 1";
        let rows: &[(&str, Outcome)] = &[
            // The issue's table.
            ("NULL", ok("Safe")),
            (r#"c("Safe", "Fast", "Debug")"#, ok("Safe")),
            (r#"c("Fast", "Safe", "Debug")"#, err(not_scalar)),
            (r#""zzz""#, err(one_of)),
            // Factors are read as their labels.
            (r#"factor(c("Safe", "Fast", "Debug"))"#, ok("Safe")),
            (r#"factor(c("Fast", "Safe", "Debug"))"#, err(not_scalar)),
            // Matching does not depend on the order.
            (r#""Fast""#, ok("Fast")),
            (r#""Fa""#, ok("Fast")),
            (r#""S""#, ok("Safe")),
            ("NA_character_", err("'mode' must not be NA")),
            ("1L", err("'mode' must be NULL or a character vector")),
        ];
        for (input, expected) in rows {
            assert_eq!(
                &agree_with_default::<Mode>(env, input, "Safe", safe),
                expected,
                "Mode with default Safe on `{input}`"
            );
        }

        // A default that is already first changes nothing.
        for input in ["NULL", r#"c("Fast", "Safe", "Debug")"#, r#""zzz""#] {
            assert_eq!(
                agree_with_default::<Mode>(env, input, "Fast", Mode(0)),
                agree::<Mode>(env, input),
                "on `{input}`"
            );
        }

        // The writer finds a choice it escapes by the escaped text of the
        // `default = "..."` literal; the message lists it unescaped.
        let path = Quoted(1);
        assert_eq!(
            r_formal::<Quoted>(r"c:\\path"),
            r#"c("c:\\path", "say \"hi\"", "plain")"#
        );
        assert_eq!(
            agree_with_default::<Quoted>(env, "NULL", r"c:\\path", path),
            ok(r"c:\path")
        );
        assert_eq!(
            agree_with_default::<Quoted>(env, r#""zzz""#, r"c:\\path", path),
            err(r#"'mode' should be one of "c:\path", "say "hi"", "plain""#)
        );
        assert_eq!(
            agree_with_default::<Quoted>(env, r#""say""#, r"c:\\path", path),
            ok("say \"hi\"")
        );
    });
}

/// Aliases (#1843) on all three paths: an alias typed in full selects its
/// choice; a prefix still matches the choices only, so a prefix of a choice
/// keeps selecting it and a prefix of an alias is no match; the messages name
/// the listed aliases after the choices, in the order of the choices
/// (#2148).
#[test]
fn aliases_agree_with_the_wrapper_helper() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        assert_eq!(
            r_aliases::<Shade>(),
            r#", aliases = base::c("grey" = "gray", "rouge" = "red"), listed = " (or \"rouge\" for \"red\", \"grey\" for \"gray\")""#
        );
        let one_of = r#"'mode' should be one of "red", "gray", "blue" (or "rouge" for "red", "grey" for "gray")"#;
        let rows: &[(&str, Outcome)] = &[
            (r#""grey""#, ok("gray")),
            (r#""rouge""#, ok("red")),
            (r#""gray""#, ok("gray")),
            // A prefix of a choice, as before the alias.
            (r#""gr""#, ok("gray")),
            (r#""r""#, ok("red")),
            // A prefix of an alias is not one.
            (r#""gre""#, err(one_of)),
            (r#""rou""#, err(one_of)),
            // Exact means exact.
            (r#""GREY""#, err(one_of)),
            (r#""grey ""#, err(one_of)),
            // A factor is read as its labels, a classed string is a string.
            (r#"factor("grey")"#, ok("gray")),
            (r#"factor("grey", levels = c("x", "grey"))"#, ok("gray")),
            (r#"structure("grey", class = "pkg_shade")"#, ok("gray")),
            // Everything else as without aliases.
            ("NULL", ok("red")),
            (&r_formal::<Shade>(""), ok("red")),
            ("NA_character_", err("'mode' must not be NA")),
            ("factor(NA_character_)", err("'mode' must not be NA")),
            (r#""""#, err(one_of)),
            (r#""zzz""#, err(one_of)),
            (r#"c("grey", "red")"#, err("'mode' must be of length 1")),
            ("1L", err("'mode' must be NULL or a character vector")),
        ];
        for (input, expected) in rows {
            assert_eq!(&agree::<Shade>(env, input), expected, "Shade on `{input}`");
        }

        // A `default = "..."` formal: the aliases are matched the same way,
        // and the message lists the choices in the formal's order, the
        // aliases in the order of `CHOICES`.
        let blue = Shade(2);
        assert_eq!(
            agree_with_default::<Shade>(env, r#""grey""#, "blue", blue),
            ok("gray")
        );
        assert_eq!(
            agree_with_default::<Shade>(env, "NULL", "blue", blue),
            ok("blue")
        );
        assert_eq!(
            agree_with_default::<Shade>(env, r#""gre""#, "blue", blue),
            err(
                r#"'mode' should be one of "blue", "red", "gray" (or "rouge" for "red", "grey" for "gray")"#
            )
        );
        assert_eq!(
            agree_with_default::<Shade>(env, r#""gre""#, "gray", Shade(1)),
            err(
                r#"'mode' should be one of "gray", "red", "blue" (or "rouge" for "red", "grey" for "gray")"#
            )
        );

        // Escaped in the arguments, matched unescaped, and quoted in the
        // message as the choices are, without escapes.
        assert_eq!(
            r_aliases::<QuotedAlias>(),
            r#", aliases = base::c("hello \"you\"" = "say \"hi\"", "c:\\plain" = "plain"), listed = " (or \"hello \"you\"\" for \"say \"hi\"\", \"c:\\plain\" for \"plain\")""#
        );
        assert_eq!(
            agree::<QuotedAlias>(env, r#""hello \"you\"""#),
            ok("say \"hi\"")
        );
        assert_eq!(agree::<QuotedAlias>(env, r#""c:\\plain""#), ok("plain"));
        assert_eq!(
            agree::<QuotedAlias>(env, r#""hello""#),
            err(
                r#"'mode' should be one of "say "hi"", "plain" (or "hello "you"" for "say "hi"", "c:\plain" for "plain")"#
            )
        );
    });
}

/// A type whose aliases are all unlisted matches them, and words its
/// messages exactly as a type without aliases (#2148): the writer passes no
/// `listed` for it.
#[test]
fn unlisted_aliases_agree_with_the_wrapper_helper() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        assert_eq!(
            r_aliases::<QuietShade>(),
            r#", aliases = base::c("grey" = "gray", "rouge" = "red")"#
        );
        let one_of = r#"'mode' should be one of "red", "gray", "blue""#;
        let rows: &[(&str, Outcome)] = &[
            (r#""grey""#, ok("gray")),
            (r#""rouge""#, ok("red")),
            (r#""gr""#, ok("gray")),
            (r#""gre""#, err(one_of)),
            (r#""zzz""#, err(one_of)),
            ("NA_character_", err("'mode' must not be NA")),
            (r#"c("grey", "red")"#, err("'mode' must be of length 1")),
        ];
        for (input, expected) in rows {
            assert_eq!(
                &agree::<QuietShade>(env, input),
                expected,
                "QuietShade on `{input}`"
            );
        }
        assert_eq!(
            agree_with_default::<QuietShade>(env, r#""zzz""#, "gray", QuietShade(1)),
            err(r#"'mode' should be one of "gray", "red", "blue""#)
        );
    });
}

/// Listed and unlisted aliases together: every alias is matched, and only
/// the listed ones are named, in the order of their choices, then as
/// written.
#[test]
fn mixed_aliases_agree_with_the_wrapper_helper() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        let one_of = r#"'mode' should be one of "red", "gray", "blue" (or "Grau" for "gray", "gris" for "gray", "bleu" for "blue")"#;
        let rows: &[(&str, Outcome)] = &[
            (r#""grey""#, ok("gray")),
            (r#""Grau""#, ok("gray")),
            (r#""gris""#, ok("gray")),
            (r#""rouge""#, ok("red")),
            (r#""bleu""#, ok("blue")),
            (r#""gre""#, err(one_of)),
            (r#"factor("zzz")"#, err(one_of)),
            ("NA_character_", err("'mode' must not be NA")),
            ("1L", err("'mode' must be NULL or a character vector")),
        ];
        for (input, expected) in rows {
            assert_eq!(
                &agree::<MixedShade>(env, input),
                expected,
                "MixedShade on `{input}`"
            );
        }
    });
}

/// The outcome of a `several_ok` matcher: `c("ok", <choices>...)` or
/// `c("error", ...)`.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn r_several_outcome<T: MatchArg>(env: SEXP, input: &str) -> Outcome {
    let code = format!(
        r#"local({{
  r <- tryCatch(
    .miniextendr_match_arg_several({input}, {}, "mode"{}),
    error = function(e) e
  )
  if (inherits(r, "error")) c("error", conditionMessage(r)) else c("ok", r)
}})"#,
        r_formal::<T>(""),
        r_aliases::<T>()
    );
    let value = unsafe { r_eval_str(&code, env) }.unwrap_or_else(|e| panic!("{input}: {e}"));
    Vec::<String>::try_from_sexp(value).expect("a character vector")
}

/// `several_ok` maps aliases element by element, in the R helper and in
/// `match_arg_vec_from_sexp` alike (#1843). The two word a refusal
/// differently (#1806), so a refusal is compared as a refusal, and by the
/// clause naming the listed aliases that both end on (#2148).
#[test]
fn several_ok_aliases_agree_with_the_wrapper_helper() {
    r_test_utils::with_r_thread(|| unsafe {
        let env = helpers_env();
        let env = env.get();

        // What the conversion of a `several_ok` parameter reports.
        let rust_message = |input: &str| {
            let value = OwnedProtect::new(r_eval_str(input, env).expect("evaluates"));
            let e = miniextendr_api::match_arg_vec_from_sexp::<MixedShade>(value.get())
                .expect_err("refused");
            miniextendr_api::__mx_conversion_arg_message!(SexpError::from(e), "mode")
                .expect("a match_arg error words the whole argument error")
        };
        let clause = r#" (or "Grau" for "gray", "gris" for "gray", "bleu" for "blue")"#;
        assert_eq!(
            r_several_outcome::<MixedShade>(env, r#"c("grey", "zzz")"#),
            [
                "error".to_string(),
                format!(
                    r#"'mode' element 2 ("zzz") should be one of "red", "gray", "blue"{clause}"#
                )
            ]
        );
        assert_eq!(
            rust_message(r#"c("grey", "zzz")"#),
            format!(r#"'mode' should be one of "red", "gray", "blue"{clause}"#)
        );
        assert_eq!(
            r_several_outcome::<QuietShade>(env, r#"c("grey", "zzz")"#),
            [
                "error",
                r#"'mode' element 2 ("zzz") should be one of "red", "gray", "blue""#
            ]
        );

        let rows: &[(&str, &[&str])] = &[
            (r#"c("grey", "red")"#, &["gray", "red"]),
            (r#"c("rouge", "gr", "grey")"#, &["red", "gray", "gray"]),
            (r#""grey""#, &["gray"]),
            ("NULL", &["red", "gray", "blue"]),
            (r#"c("grey", "zzz")"#, &[]),
            (r#"c("gre", "red")"#, &[]),
            (r#"c("grey", NA)"#, &[]),
            ("NA", &[]),
        ];
        for (input, expected) in rows {
            let r = r_several_outcome::<Shade>(env, input);
            let value = OwnedProtect::new(r_eval_str(input, env).expect("evaluates"));
            let rust = miniextendr_api::match_arg_vec_from_sexp::<Shade>(value.get());
            if expected.is_empty() {
                assert_eq!(r[0], "error", "R helper on `{input}`: {r:?}");
                assert!(
                    rust.is_err(),
                    "match_arg_vec_from_sexp on `{input}`: {rust:?}"
                );
                if *input == r#"c("grey", NA)"# || *input == "NA" {
                    assert_eq!(r[1], "'mode' must not be NA");
                    assert!(matches!(rust, Err(MatchArgError::IsNa { .. })), "{rust:?}");
                }
                if *input == r#"c("grey", "zzz")"# {
                    assert_eq!(
                        r[1],
                        r#"'mode' element 2 ("zzz") should be one of "red", "gray", "blue" (or "rouge" for "red", "grey" for "gray")"#
                    );
                }
            } else {
                let want: Vec<String> = std::iter::once("ok")
                    .chain(expected.iter().copied())
                    .map(String::from)
                    .collect();
                assert_eq!(r, want, "R helper on `{input}`");
                let got: Vec<&str> = rust
                    .unwrap_or_else(|e| panic!("match_arg_vec_from_sexp on `{input}`: {e}"))
                    .into_iter()
                    .map(MatchArg::to_choice)
                    .collect();
                assert_eq!(&got, expected, "match_arg_vec_from_sexp on `{input}`");
            }
        }

        // `pmatch()` matches an NA element as the string "NA" too.
        let input = r#"c("other", NA)"#;
        assert_eq!(
            r_several_outcome::<NaPrefixed>(env, input),
            ["ok", "other", "NAME"]
        );
        let value = OwnedProtect::new(r_eval_str(input, env).expect("evaluates"));
        let got: Vec<&str> = miniextendr_api::match_arg_vec_from_sexp::<NaPrefixed>(value.get())
            .expect("NA selects NAME")
            .into_iter()
            .map(MatchArg::to_choice)
            .collect();
        assert_eq!(got, ["other", "NAME"]);
    });
}

/// A `default = "..."` that names no choice stops the wrappers writer.
#[test]
#[should_panic(expected = "does not match any choice")]
fn match_arg_formal_refuses_a_default_that_is_no_choice() {
    match_arg_formal(Mode::CHOICES, "Slow", ".__MX_MATCH_ARG_CHOICES_run_mode__");
}

/// `match_arg_from_sexp` (what the derived `TryFromSexp` and the generated C
/// wrappers call) has the same semantics. Its error is the `MatchArgError`,
/// which the conversion arm words as the wrapper's check (see `agree`).
#[test]
fn match_arg_from_sexp_takes_the_formal_default() {
    r_test_utils::with_r_thread(|| unsafe {
        let eval =
            |code: &str| OwnedProtect::new(r_eval_str(code, R_GlobalEnv).expect("evaluates"));
        let full = eval(r#"c("drop", "draw", "error")"#);
        assert_eq!(
            miniextendr_api::match_arg_from_sexp::<Fill>(full.get()).unwrap(),
            Fill(0)
        );
        let factor = eval(r#"factor(c("drop", "draw", "error"))"#);
        assert_eq!(
            miniextendr_api::match_arg_from_sexp::<Fill>(factor.get()).unwrap(),
            Fill(0)
        );
        let named = eval(r#"c(a = "drop", b = "draw", c = "error")"#);
        assert!(matches!(
            miniextendr_api::match_arg_from_sexp::<Fill>(named.get()),
            Err(MatchArgError::InvalidLength { actual: 3, .. })
        ));
        let na = eval("NA_character_");
        assert!(matches!(
            miniextendr_api::match_arg_from_sexp::<Fill>(na.get()),
            Err(MatchArgError::IsNa { .. })
        ));
        let empty = eval(r#""""#);
        assert!(matches!(
            miniextendr_api::match_arg_from_sexp::<Single>(empty.get()),
            Err(MatchArgError::NoMatch { .. })
        ));
        // The several-ok decoder shares the empty-string rule.
        let several = eval(r#"c("x", "")"#);
        assert!(matches!(
            miniextendr_api::match_arg_vec_from_sexp::<WithEmpty>(several.get()),
            Err(MatchArgError::NoMatch { .. })
        ));
        assert_eq!(
            miniextendr_api::match_arg_vec_from_sexp::<WithEmpty>(miniextendr_api::SEXP::nil())
                .unwrap(),
            [WithEmpty(0), WithEmpty(1)]
        );
    });
}
