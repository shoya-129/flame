use crate::vm::Value;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "parse".to_string(),
        Value::NativeCallback(|args| {
            if let Some(Value::String(json_str)) = args.get(0) {
                match serde_json::from_str::<JsonValue>(json_str.as_str()) {
                    Ok(val) => Ok(json_to_value(&val)),
                    Err(e) => Err(format!("JSON parse error: {}", e))
                }
            } else {
                Err("json.parse expects a string".to_string())
            }
        })
    );

    map.insert(
        "stringify".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                let json_val = value_to_json(val);
                Ok(Value::String(json_val.to_string()))
            } else {
                Err("json.stringify expects an argument".to_string())
            }
        })
    );

    map
}

pub fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Nil
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(json_to_value(item));
            }
            Value::Tuple(vec)
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Object(map)
        }
    }
}

pub fn value_to_json(val: &Value) -> JsonValue {
    match val {
        Value::Nil => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Int(i) => JsonValue::Number((*i).into()),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                JsonValue::Number(n)
            } else {
                JsonValue::Null
            }
        }
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Tuple(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(value_to_json(item));
            }
            JsonValue::Array(vec)
        }
        Value::Object(obj) | Value::Formula(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), value_to_json(v));
            }
            JsonValue::Object(map)
        }
        _ => JsonValue::String(format!("{:?}", val)) // Fallback for unsupported types
    }
}
