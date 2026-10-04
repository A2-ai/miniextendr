//! `match_arg_param` agrees with the generated wrapper's `.miniextendr_match_arg`
//! (#1741): the same choice, or the same argument error word for word, on every
//! kind of input.
//!
//! The R side is the preamble source itself (`registry::ARG_CHECK_HELPERS`),
//! evaluated in a fresh environment, so the comparison is against the text the
//! wrappers file ships, not a copy of it.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::match_arg::escape_r_string;
use miniextendr_api::registry::ARG_CHECK_HELPERS;
use miniextendr_api::sys::R_GlobalEnv;
use miniextendr_api::{MatchArg, MatchArgError, OwnedProtect, SEXP, match_arg_param};

/// A choice type over arbitrary strings: `from_choice` is an exact lookup in
/// `CHOICES`, as the derive's is.
macro_rules! choice_type {
    ($name:ident, [$($choice:literal),+ $(,)?]) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct $name(usize);

        impl MatchArg for $name {
            const CHOICES: &'static [&'static str] = &[$($choice),+];

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
choice_type!(NaPrefixed, ["NAME", "other"]);
choice_type!(Quoted, ["say \"hi\"", r"c:\path", "plain"]);
choice_type!(Single, ["only"]);
choice_type!(WithEmpty, ["", "x"]);

/// `c("a", "b")` for `T::CHOICES`, as the wrapper writes the formal default.
fn r_choices<T: MatchArg>() -> String {
    let quoted: Vec<String> = T::CHOICES
        .iter()
        .map(|c| format!("\"{}\"", escape_r_string(c)))
        .collect();
    format!("c({})", quoted.join(", "))
}

/// The outcome as `c("ok", <choice>)` or `c("error", <message>, <param>)`.
type Outcome = Vec<String>;

/// The wrapper's helper on the R value of `input`.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn r_outcome<T: MatchArg>(env: SEXP, input: &str) -> Outcome {
    let code = format!(
        r#"local({{
  r <- tryCatch(.miniextendr_match_arg({input}, {choices}, "mode"), error = function(e) e)
  if (inherits(r, "error")) {{
    stopifnot(
      identical(class(r), c("rust_error", "simpleError", "error", "condition")),
      identical(r$kind, "conversion")
    )
    c("error", conditionMessage(r), r$param)
  }} else c("ok", r)
}})"#,
        choices = r_choices::<T>()
    );
    let value = unsafe { r_eval_str(&code, env) }.unwrap_or_else(|e| panic!("{input}: {e}"));
    Vec::<String>::try_from_sexp(value).expect("a character vector")
}

/// `match_arg_param` on the R value of `input`.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn rust_outcome<T: MatchArg>(env: SEXP, input: &str) -> Outcome {
    let value = unsafe { r_eval_str(input, env) }.unwrap_or_else(|e| panic!("{input}: {e}"));
    let value = unsafe { OwnedProtect::new(value) };
    match match_arg_param::<T>(value.get(), "mode") {
        Ok(choice) => vec!["ok".into(), choice.to_choice().into()],
        Err(e) => vec!["error".into(), e.message().into(), e.param().into()],
    }
}

/// Every input on both sides; returns the shared outcome for the spot checks.
///
/// # Safety
///
/// R main thread; `env` holds the helpers.
unsafe fn agree<T: MatchArg>(env: SEXP, input: &str) -> Outcome {
    let r = unsafe { r_outcome::<T>(env, input) };
    let rust = unsafe { rust_outcome::<T>(env, input) };
    assert_eq!(
        rust,
        r,
        "{} on `{input}`: match_arg_param (left) vs .miniextendr_match_arg (right)",
        std::any::type_name::<T>()
    );
    r
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
        let env = OwnedProtect::new(
            r_eval_str("new.env(parent = baseenv())", R_GlobalEnv).expect("an environment"),
        );
        let env = env.get();
        r_eval_str(ARG_CHECK_HELPERS, env).expect("the preamble helpers parse and evaluate");
        r_eval_str(".miniextendr_conversion_error_class <- NULL", env).unwrap();

        let one_of = r#"'mode' should be one of "drop", "draw", "error""#;
        let not_character = "'mode' must be NULL or a character vector";
        let not_scalar = "'mode' must be of length 1";

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
            ("factor(NA_character_)", err(one_of)),
            // NA, length, and type.
            ("NA_character_", err(one_of)),
            (r#"c("drop", "draw")"#, err(not_scalar)),
            ("character(0)", err(not_scalar)),
            (r#"c("drop", NA)"#, err(not_scalar)),
            ("1L", err(not_character)),
            ("2.5", err(not_character)),
            ("NA", err(not_character)),
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
            agree::<Quoted>(env, &r_choices::<Quoted>()),
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

/// `match_arg_from_sexp` (what the derived `TryFromSexp` and the generated C
/// wrappers call) has the same semantics; only its error is the conversion
/// error rather than the wrapper's wording.
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
