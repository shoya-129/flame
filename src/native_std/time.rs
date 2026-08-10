use crate::vm::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "now".to_string(),
        Value::NativeCallback(|_args| {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => Ok(Value::Int(n.as_millis() as i64)),
                Err(_) => Err("SystemTime before UNIX EPOCH!".to_string()),
            }
        }),
    );

    m.insert(
        "timestamp".to_string(),
        Value::NativeCallback(|_args| {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => Ok(Value::Int(n.as_secs() as i64)),
                Err(_) => Err("SystemTime before UNIX EPOCH!".to_string()),
            }
        }),
    );

    m.insert(
        "parse".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("time.parse requires 1 argument (date string)".to_string());
            }
            if let Value::String(s) = &args[0] {
                // Try parsing standard formats
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    return Ok(Value::Int(dt.timestamp_millis()));
                }
                if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(s) {
                    return Ok(Value::Int(dt.timestamp_millis()));
                }
                return Err(format!("Could not parse time string: {}", s));
            }
            Err("time.parse requires a string".to_string())
        }),
    );

    m
}
