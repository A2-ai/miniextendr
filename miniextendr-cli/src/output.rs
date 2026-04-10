/// Print a simple status message.
pub fn print_status(msg: &str, json: bool) {
    if json {
        println!("{}", serde_json::json!({ "status": msg }));
    } else {
        println!("{msg}");
    }
}
