#![allow(dead_code)]

use serde::Serialize;

/// Print output in either human-readable or JSON format.
pub fn print_output<T: Serialize + std::fmt::Display>(value: &T, json: bool) {
    if json {
        match serde_json::to_string_pretty(value) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("JSON serialization error: {e}"),
        }
    } else {
        println!("{value}");
    }
}

/// Print a key-value pair.
pub fn print_kv(key: &str, value: &str, json: bool) {
    if json {
        println!("{}", serde_json::json!({ key: value }));
    } else {
        println!("{key}: {value}");
    }
}

/// Print a simple status message.
pub fn print_status(msg: &str, json: bool) {
    if json {
        println!("{}", serde_json::json!({ "status": msg }));
    } else {
        println!("{msg}");
    }
}
