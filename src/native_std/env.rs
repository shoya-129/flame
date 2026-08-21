use crate::vm::Value;
use std::collections::HashMap;
use std::env;

pub fn init() -> HashMap<String, Value> {
    // Attempt to load .env file silently
    let _ = dotenvy::dotenv();
    
    let mut m = HashMap::new();

    m.insert(
        "get".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("env.get expects 1 argument (key)".to_string());
            }
            let key = args[0].to_string().trim_matches('"').to_string();
            match env::var(&key) {
                Ok(val) => Ok(Value::String(val)),
                Err(_) => Ok(Value::Nil),
            }
        }),
    );

    m.insert(
        "set".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("env.set expects 2 arguments (key, value)".to_string());
            }
            let key = args[0].to_string().trim_matches('"').to_string();
            let value = args[1].to_string().trim_matches('"').to_string();
            unsafe {
                env::set_var(key, value);
            }
            Ok(Value::Nil)
        }),
    );

    m.insert(
        "remove".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("env.remove expects 1 argument (key)".to_string());
            }
            let key = args[0].to_string().trim_matches('"').to_string();
            unsafe {
                env::remove_var(key);
            }
            Ok(Value::Nil)
        }),
    );

    m.insert(
        "vars".to_string(),
        Value::NativeCallback(|_args| {
            let mut map = HashMap::new();
            for (key, value) in env::vars() {
                map.insert(key, Value::String(value));
            }
            Ok(Value::Formula(map))
        }),
    );

    m.insert(
        "temp".to_string(),
        Value::NativeCallback(|_args| Ok(Value::String(env::temp_dir().to_string_lossy().to_string()))),
    );

    m
}
