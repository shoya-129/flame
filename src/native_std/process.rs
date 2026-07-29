use crate::vm::Value;
use std::collections::HashMap;
use std::process::Command;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "spawn".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("process.spawn expects at least 1 argument (command)".to_string());
            }
            let cmd = args[0].to_string().trim_matches('"').to_string();
            let mut command = Command::new(&cmd);
            if args.len() > 1 {
                if let Value::Tuple(arg_list) = &args[1] {
                    for arg in arg_list {
                        command.arg(arg.to_string().trim_matches('"').to_string());
                    }
                } else {
                    return Err("process.spawn second argument must be a list of args".to_string());
                }
            }
            match command.spawn() {
                Ok(child) => {
                    // For now, we return the PID
                    Ok(Value::ChildProcess(child.id() as u64))
                }
                Err(e) => Err(format!("process.spawn error: {}", e)),
            }
        }),
    );

    m.insert(
        "exec".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("process.exec expects at least 1 argument (command)".to_string());
            }
            let cmd = args[0].to_string().trim_matches('"').to_string();
            let mut command = Command::new(&cmd);
            if args.len() > 1 {
                if let Value::Tuple(arg_list) = &args[1] {
                    for arg in arg_list {
                        command.arg(arg.to_string().trim_matches('"').to_string());
                    }
                }
            }
            match command.output() {
                Ok(output) => {
                    let mut map = HashMap::new();
                    map.insert(
                        "stdout".to_string(),
                        Value::String(String::from_utf8_lossy(&output.stdout).to_string()),
                    );
                    map.insert(
                        "stderr".to_string(),
                        Value::String(String::from_utf8_lossy(&output.stderr).to_string()),
                    );
                    let mut status = HashMap::new();
                    status.insert(
                        "code".to_string(),
                        Value::Int(output.status.code().unwrap_or(0) as i64),
                    );
                    map.insert("status".to_string(), Value::Formula(status));
                    Ok(Value::Formula(map))
                }
                Err(e) => Err(format!("process.exec error: {}", e)),
            }
        }),
    );

    m.insert(
        "cwd".to_string(),
        Value::NativeCallback(|_args| {
            match std::env::current_dir() {
                Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
                Err(e) => Err(format!("process.cwd error: {}", e)),
            }
        }),
    );

    m.insert(
        "set_cwd".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("process.set_cwd expects 1 argument (path)".to_string());
            }
            let path = args[0].to_string().trim_matches('"').to_string();
            match std::env::set_current_dir(path) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("process.set_cwd error: {}", e)),
            }
        }),
    );

    m.insert(
        "pid".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Int(std::process::id() as i64))),
    );

    m
}
