//! Shared truthiness parsing for environment-variable flags.
//!
//! Two flags in this crate read boolean-ish env vars — `MINIEXTENDR_BACKTRACE`
//! (`backtrace.rs`) and `_R_CHECK_LIMIT_CORES_` (`optionals/parallel.rs`) — and
//! they used to disagree on what counts as "on" (`true`/`1` vs. anything but
//! `false`). [`parse_bool`] is the single source of truth, deliberately aligned
//! with R's own `config_val_to_logical` (`tools/R/utils.R`): a case-insensitive,
//! whitespace-trimmed match against explicit truthy/falsy sets.
//!
//! Recognized values (case-insensitive, surrounding whitespace trimmed):
//! - truthy: `yes`, `true`, `1`, `on`
//! - falsy: `no`, `false`, `0`, `off`, `` (empty)
//!
//! Anything else is unrecognized (`None`) — the caller supplies the default,
//! so each flag keeps control of its own "garbage" policy (see the two call
//! sites for their deliberately different defaults).

/// Parse an environment-flag value as a boolean, using R's
/// `config_val_to_logical` rules (see module docs). Returns `None` for values
/// outside the recognized truthy/falsy sets so the caller picks the default.
pub(crate) fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" | "1" | "on" => Some(true),
        "no" | "false" | "0" | "off" | "" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_truthy_values_case_insensitively() {
        for v in [
            "yes", "YES", "true", "TRUE", "True", "1", "on", "ON", " true ",
        ] {
            assert_eq!(parse_bool(v), Some(true), "{v:?}");
        }
    }

    #[test]
    fn recognizes_falsy_values_case_insensitively() {
        for v in ["no", "NO", "false", "FALSE", "0", "off", "OFF", "", "  "] {
            assert_eq!(parse_bool(v), Some(false), "{v:?}");
        }
    }

    #[test]
    fn returns_none_for_unrecognized() {
        for v in ["garbage", "2", "tru", "yesno"] {
            assert_eq!(parse_bool(v), None, "{v:?}");
        }
    }
}
