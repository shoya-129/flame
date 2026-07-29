use crate::vm::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use std::env;

fn resolve_path(path_str: &str) -> PathBuf {
    let base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let resolved = base.join("src").join(path_str);
    resolved
}

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "read".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.read expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            match fs::read_to_string(&path) {
                Ok(content) => Ok(Value::String(content)),
                Err(e) => Err(format!("fs.read error: {}", e)),
            }
        }),
    );

    m.insert(
        "write".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("fs.write expects 2 arguments (path, content)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let content = match &args[1] {
                Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            match fs::write(&path, content) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("fs.write error: {}", e)),
            }
        }),
    );

    m.insert(
        "exists".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.exists expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            Ok(Value::Bool(path.exists()))
        }),
    );

    m.insert(
        "remove".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.remove expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let p = path.as_path();
            if p.is_dir() {
                if let Err(e) = fs::remove_dir_all(p) {
                    return Err(format!("fs.remove error: {}", e));
                }
            } else {
                if let Err(e) = fs::remove_file(p) {
                    return Err(format!("fs.remove error: {}", e));
                }
            }
            Ok(Value::Nil)
        }),
    );
    
    m.insert(
        "mkdir".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.mkdir expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            match fs::create_dir(&path) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("fs.mkdir error: {}", e)),
            }
        }),
    );

    m.insert(
        "mkdir_all".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.mkdir_all expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            match fs::create_dir_all(&path) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("fs.mkdir_all error: {}", e)),
            }
        }),
    );

    m.insert(
        "copy".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("fs.copy expects 2 arguments (src, dest)".to_string());
            }
            let src = resolve_path(&args[0].to_string().trim_matches('"'));
            let dest = resolve_path(&args[1].to_string().trim_matches('"'));
            match fs::copy(&src, &dest) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("fs.copy error: {}", e)),
            }
        }),
    );

    m
}
