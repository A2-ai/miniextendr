use anyhow::Result;
use serde::Serialize;

/// Serialize `value` as pretty JSON and print it to stdout.
pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
