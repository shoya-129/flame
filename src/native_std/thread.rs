use crate::vm::Value;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "sleep".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("thread.sleep expects 1 argument (ms)".to_string());
            }
            if let Value::Int(ms) = args[0] {
                thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::Nil)
            } else {
                Err("thread.sleep argument must be an Int".to_string())
            }
        }),
    );

    m.insert(
        "spawn".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("thread.spawn expects 1 argument (function or callback)".to_string());
            }
            if let Value::Function { body: _, params: _, env: _ } = &args[0] {
                // Thread spawning a Wren function is complex natively without VM cloning,
                // but we can return a stub thread handle for now.
                Ok(Value::ThreadHandler(999))
            } else if let Value::NativeCallback(cb) = args[0] {
                // We can actually spawn native callbacks safely!
                let _handle = thread::spawn(move || {
                    let _ = cb(vec![]);
                });
                Ok(Value::ThreadHandler(998))
            } else {
                Err("thread.spawn argument must be a function".to_string())
            }
        }),
    );

    m.insert(
        "id".to_string(),
        Value::NativeCallback(|_args| {
            // Rust thread ids are opaque, but we can format them
            Ok(Value::String(format!("{:?}", thread::current().id())))
        }),
    );

    m.insert(
        "yield".to_string(),
        Value::NativeCallback(|_args| {
            thread::yield_now();
            Ok(Value::Nil)
        }),
    );

    m
}
