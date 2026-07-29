use crate::vm::Value;
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "name".to_string(),
        Value::NativeCallback(|_args| Ok(Value::String(std::env::consts::OS.to_string()))),
    );

    m.insert(
        "arch".to_string(),
        Value::NativeCallback(|_args| Ok(Value::String(std::env::consts::ARCH.to_string()))),
    );

    m.insert(
        "family".to_string(),
        Value::NativeCallback(|_args| Ok(Value::String(std::env::consts::FAMILY.to_string()))),
    );

    m
}
