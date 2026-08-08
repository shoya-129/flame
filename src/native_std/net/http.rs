use crate::vm::Value;
use std::collections::HashMap;

#[cfg(feature = "http")]
pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "get".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 1 {
                return Err("http.get expects 1 argument (url)".to_string());
            }
            if let Value::String(url) = &args[0] {
                let res = reqwest::blocking::get(url)
                    .map_err(|e| format!("HTTP GET error: {}", e))?;
                
                let status = res.status().as_u16();
                let text = res.text().unwrap_or_default();
                
                let mut response_obj = HashMap::new();
                response_obj.insert("status".to_string(), Value::Int(status as i64));
                
                let text_val = Value::String(text.clone());
                response_obj.insert("text".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new({
                    let t = text_val.clone();
                    move |_| Ok(t.clone())
                }))));
                
                response_obj.insert("json".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(crate::native_std::json::json_to_value(&parsed))
                    } else {
                        Err("Failed to parse JSON".to_string())
                    }
                }))));

                Ok(Value::Object(response_obj))
            } else {
                Err("http.get expects a string url".to_string())
            }
        }),
    );

    map.insert(
        "post".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 2 {
                return Err("http.post expects 2 arguments (url, body)".to_string());
            }
            if let Value::String(url) = &args[0] {
                let body_str = if let Value::String(s) = &args[1] {
                    s.clone()
                } else {
                    crate::native_std::json::value_to_json(&args[1]).to_string()
                };
                
                let client = reqwest::blocking::Client::new();
                let res = client.post(url).body(body_str)
                    .header("Content-Type", "application/json")
                    .send()
                    .map_err(|e| format!("HTTP POST error: {}", e))?;
                
                let status = res.status().as_u16();
                let text = res.text().unwrap_or_default();
                
                let mut response_obj = HashMap::new();
                response_obj.insert("status".to_string(), Value::Int(status as i64));
                
                let text_val = Value::String(text.clone());
                response_obj.insert("text".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new({
                    let t = text_val.clone();
                    move |_| Ok(t.clone())
                }))));
                
                response_obj.insert("json".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(crate::native_std::json::json_to_value(&parsed))
                    } else {
                        Err("Failed to parse JSON".to_string())
                    }
                }))));

                Ok(Value::Object(response_obj))
            } else {
                Err("http.post expects (url: String, body: Any)".to_string())
            }
        }),
    );

    map
}

#[cfg(not(feature = "http"))]
pub fn init() -> HashMap<String, Value> {
    HashMap::new()
}
