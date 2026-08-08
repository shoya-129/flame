use crate::vm::Value;
use std::collections::HashMap;

#[cfg(feature = "net")]
pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "Url".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            obj.insert(
                "parse".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(url_str)) = args.get(0) {
                        match url::Url::parse(&url_str) {
                            Ok(parsed) => {
                                let mut res = HashMap::new();
                                res.insert("host".to_string(), Value::String(parsed.host_str().unwrap_or("").to_string()));
                                res.insert("query".to_string(), Value::String(parsed.query().unwrap_or("").to_string()));
                                res.insert("path".to_string(), Value::String(parsed.path().to_string()));
                                res.insert("scheme".to_string(), Value::String(parsed.scheme().to_string()));
                                res.insert("port".to_string(), parsed.port().map(|p| Value::Int(p as i64)).unwrap_or(Value::Nil));
                                Ok(Value::Object(res))
                            }
                            Err(e) => Err(format!("URL parse error: {}", e))
                        }
                    } else {
                        Err("Url.parse expects a string".to_string())
                    }
                })
            );
            obj
        })
    );

    map
}

#[cfg(not(feature = "net"))]
pub fn init() -> HashMap<String, Value> {
    HashMap::new()
}
