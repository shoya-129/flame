use crate::vm::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use std::env;

pub fn resolve_path(path_str: &str) -> PathBuf {
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
        "readBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("fs.readBytes expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            match std::fs::read(&path) {
                Ok(bytes) => Ok(Value::Bytes(bytes)),
                Err(e) => Err(format!("fs.readBytes error: {}", e)),
            }
        }),
    );

    m.insert(
        "writeBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("fs.writeBytes expects 2 arguments (path, bytes)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let bytes = match &args[1] {
                Value::Bytes(b) => b.clone(),
                _ => return Err(format!("fs.writeBytes: expected Bytes, found {}", args[1].type_name())),
            };
            match std::fs::write(&path, bytes) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("fs.writeBytes error: {}", e)),
            }
        }),
    );

    m.insert(
        "appendBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("fs.appendBytes expects 2 arguments (path, bytes)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let bytes = match &args[1] {
                Value::Bytes(b) => b.clone(),
                _ => return Err(format!("fs.appendBytes: expected Bytes, found {}", args[1].type_name())),
            };
            match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut file) => {
                    use std::io::Write;
                    match file.write_all(&bytes) {
                        Ok(_) => Ok(Value::Nil),
                        Err(e) => Err(format!("fs.appendBytes error: {}", e)),
                    }
                },
                Err(e) => Err(format!("fs.appendBytes error: {}", e)),
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

    m.insert(
        "open".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("File.open expects 1 argument (path)".to_string()); }
            let path_str = args[0].to_string().trim_matches('"').to_string();
            
            let mut file_instance = std::collections::HashMap::new();
            file_instance.insert("path".to_string(), Value::String(path_str.clone()));
            
            let p1 = path_str.clone();
            file_instance.insert("read".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                match fs::read_to_string(resolve_path(&p1)) {
                    Ok(c) => Ok(Value::String(c)),
                    Err(e) => Err(format!("fs.read error: {}", e)),
                }
            }))));
            
            let p2 = path_str.clone();
            file_instance.insert("write".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args2| {
                if args2.len() < 2 { return Err("write expects 1 argument".to_string()); }
                let c = match &args2[1] { Value::String(s) => s.clone(), v => v.to_string() };
                match fs::write(resolve_path(&p2), c) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(format!("fs.write error: {}", e)),
                }
            }))));
            
            let p3 = path_str.clone();
            file_instance.insert("append".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args3| {
                if args3.len() < 2 { return Err("append expects 1 argument".to_string()); }
                let c = match &args3[1] { Value::String(s) => s.clone(), v => v.to_string() };
                use std::io::Write;
                match fs::OpenOptions::new().append(true).create(true).open(resolve_path(&p3)) {
                    Ok(mut f) => match f.write_all(c.as_bytes()) {
                        Ok(_) => Ok(Value::Nil),
                        Err(e) => Err(format!("fs.append error: {}", e)),
                    },
                    Err(e) => Err(format!("fs.append error: {}", e)),
                }
            }))));
            
            let p4 = path_str.clone();
            file_instance.insert("exists".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                Ok(Value::Bool(resolve_path(&p4).exists()))
            }))));
            
            let p5 = path_str.clone();
            file_instance.insert("size".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                match fs::metadata(resolve_path(&p5)) {
                    Ok(m) => Ok(Value::Int(m.len() as i64)),
                    Err(_) => Ok(Value::Int(0)),
                }
            }))));
            
            let p6 = path_str.clone();
            file_instance.insert("delete".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                match fs::remove_file(resolve_path(&p6)) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(format!("fs.delete error: {}", e)),
                }
            }))));
            
            Ok(Value::Object(file_instance))
        })
    );

    m
}
