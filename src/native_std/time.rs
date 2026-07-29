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

    m
}
