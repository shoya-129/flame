use crate::parser::Param;
use crate::vm::{Env, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn register_global_builtins(env: Arc<Mutex<Env>>) {
    let mut e = env.lock().unwrap();
    e.define("print".to_string(), Value::Nil, false);
    e.define("eprint".to_string(), Value::Nil, false);
    e.define("println".to_string(), Value::Nil, false);
    e.define("assert".to_string(), Value::Nil, false);
    e.define("assert_eq".to_string(), Value::Nil, false);
    e.define("assert_ne".to_string(), Value::Nil, false);
    e.define("assert_true".to_string(), Value::Nil, false);
    e.define("assert_false".to_string(), Value::Nil, false);
    e.define("mock_api".to_string(), Value::Nil, false);
    e.define("mock_data".to_string(), Value::Nil, false);
    e.define("mock_function".to_string(), Value::Nil, false);
    
    use crate::parser::EnumVariant;
    e.define(
        "Result".to_string(),
        Value::EnumMeta(
            "Result".to_string(),
            vec![
                EnumVariant::Tuple("Ok".to_string(), vec!["Any".to_string()]),
                EnumVariant::Tuple("Err".to_string(), vec!["Any".to_string()]),
            ],
        ),
        false,
    );
    
    e.define(
        "Option".to_string(),
        Value::EnumMeta(
            "Option".to_string(),
            vec![
                EnumVariant::Tuple("Some".to_string(), vec!["Any".to_string()]),
                EnumVariant::Unit("None".to_string()),
            ],
        ),
        false,
    );
    
    e.define("Error".to_string(), Value::StructConstructor { name: "Error".to_string(), fields: vec![("message".to_string(), "String".to_string()), ("code".to_string(), "Int".to_string())] }, false);

    use crate::vm::EnumData;

    // Register global constructors for Result
    e.define("Ok".to_string(), Value::NativeCallback(|args| {
        if args.len() != 1 {
            return Err("Ok expects exactly 1 argument".to_string());
        }
        Ok(Value::EnumValue("Result".to_string(), "Ok".to_string(), EnumData::Tuple(args)))
    }), false);
    
    e.define("Err".to_string(), Value::NativeCallback(|args| {
        if args.len() != 1 {
            return Err("Err expects exactly 1 argument".to_string());
        }
        Ok(Value::EnumValue("Result".to_string(), "Err".to_string(), EnumData::Tuple(args)))
    }), false);

    // Register global constructors for Option
    e.define("Some".to_string(), Value::NativeCallback(|args| {
        if args.len() != 1 {
            return Err("Some expects exactly 1 argument".to_string());
        }
        Ok(Value::EnumValue("Option".to_string(), "Some".to_string(), EnumData::Tuple(args)))
    }), false);

    e.define("None".to_string(), Value::EnumValue("Option".to_string(), "None".to_string(), EnumData::Unit), false);
}
pub fn locate_import_file(current_file: &Path, import_path: &[String]) -> Option<PathBuf> {
    if import_path.is_empty() {
        return None;
    }

    let module_path = import_path.join("/");
    let dotted_path = import_path.join(".");
    let raw_name = import_path.last().unwrap();
    let candidates = vec![
        format!("{}.fm", module_path),
        format!("{}.flame", module_path),
        format!("{}.fm", dotted_path),
        format!("{}.flame", dotted_path),
        format!("{}.fm", raw_name),
        format!("{}.flame", raw_name),
    ];

    let search_roots = [
        "",
        "src",
        "test",
        "tests"
    ];

    let mut candidates_to_search = Vec::new();
    let parent_dir = current_file.parent().unwrap_or_else(|| Path::new("."));
    
    // Add current_file's parent hierarchy
    let mut base_dir = parent_dir.to_path_buf();
    for _ in 0..7 {
        for root in &search_roots {
            let root_dir = if root.is_empty() {
                base_dir.to_path_buf()
            } else {
                base_dir.join(root)
            };
            candidates_to_search.push(root_dir);
        }
        if !base_dir.pop() {
            break;
        }
    }

    // Add current_dir's hierarchy (important for IDE temp files)
    if let Ok(cwd) = std::env::current_dir() {
        let mut base_dir = cwd.clone();
        for _ in 0..7 {
            for root in &search_roots {
                let root_dir = if root.is_empty() {
                    base_dir.to_path_buf()
                } else {
                    base_dir.join(root)
                };
                candidates_to_search.push(root_dir);
            }
            if !base_dir.pop() {
                break;
            }
        }
        
        // Check .flame/pkg for package imports
        let pkg_name = &import_path[0];
        let mut check_dir = cwd.clone();
        loop {
            let pkg_dir = check_dir.join(".flame").join("pkg").join(pkg_name).join("src");
            if pkg_dir.exists() {
                if import_path.len() == 1 {
                    let p = pkg_dir.join("main.fm");
                    if p.exists() {
                        return Some(p);
                    }
                    let p = pkg_dir.join("main.flame");
                    if p.exists() {
                        return Some(p);
                    }
                } else {
                    let sub_path = import_path[1..].join("/");
                    let p = pkg_dir.join(format!("{}.fm", sub_path));
                    if p.exists() {
                        return Some(p);
                    }
                    let p = pkg_dir.join(format!("{}.flame", sub_path));
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
            if !check_dir.pop() {
                break;
            }
        }
    }

    for root_dir in candidates_to_search {
        for candidate in &candidates {
            let full_path = root_dir.join(candidate);
            if full_path.exists() {
                return Some(full_path);
            }
        }
    }

    None
}

fn function_value(params: Vec<Param>) -> Value {
    Value::Function {
        params,
        body: vec![],
        env: Arc::new(Mutex::new(Env::new())),
        annotations: vec![],
    }
}

pub fn register_std_module(mod_name: &str, env: Arc<Mutex<Env>>) {
    let module_val = match mod_name {
        "std.thread" => Some(crate::native_std::thread::init()),
        "std.process" => Some(crate::native_std::process::init()),
        "std.fs" => Some(crate::native_std::fs::init()),
        "std.byte" => Some(crate::native_std::byte::init()),
        #[cfg(feature = "net")]
        "std.net" => {
            let mut map = std::collections::HashMap::new();
            map.extend(crate::native_std::net::init("tcp"));
            map.extend(crate::native_std::net::init("udp"));
            #[cfg(feature = "http")]
            map.extend(crate::native_std::net::init("http"));
            #[cfg(feature = "ws")]
            map.extend(crate::native_std::net::init("ws"));
            #[cfg(feature = "mqtt")]
            map.extend(crate::native_std::net::init("mqtt"));
            map.extend(crate::native_std::net::init("dns"));
            map.extend(crate::native_std::net::init("url"));
            map.extend(crate::native_std::net::init("interface"));
            Some(map)
        },
        #[cfg(feature = "net")]
        "std.net.tcp" => Some(crate::native_std::net::init("tcp")),
        #[cfg(feature = "net")]
        "std.net.udp" => Some(crate::native_std::net::init("udp")),
        #[cfg(all(feature = "net", feature = "http"))]
        "std.net.http" => Some(crate::native_std::net::init("http")),
        #[cfg(all(feature = "net", feature = "ws"))]
        "std.net.ws" => Some(crate::native_std::net::init("ws")),
        #[cfg(all(feature = "net", feature = "mqtt"))]
        "std.net.mqtt" => Some(crate::native_std::net::init("mqtt")),
        #[cfg(feature = "net")]
        "std.net.dns" => Some(crate::native_std::net::init("dns")),
        #[cfg(feature = "net")]
        "std.net.url" => Some(crate::native_std::net::init("url")),
        #[cfg(feature = "net")]
        "std.net.interface" => Some(crate::native_std::net::init("interface")),
        "std.time" => Some(crate::native_std::time::init()),
        "std.os" => Some(crate::native_std::os::init()),
        "std.hardware" => Some(crate::native_std::hardware::init()),
        "std.desktop" => Some(crate::native_std::desktop::init()),
        "std.env" => Some(crate::native_std::env::init()),
        "std.hid" => Some(crate::native_std::hid::init()),
        "std.camera" => Some(crate::native_std::camera::init()),
        "std.bluetooth" => Some(crate::native_std::bluetooth::init()),
        "std.serial" => Some(crate::native_std::serial::init()),
        "std.embedded" => Some(crate::native_std::embedded::init()),
        "std.json" => Some(crate::native_std::json::init()),
        _ => None,
    };

    if let Some(val) = module_val {
        let mut env = env.lock().unwrap();

        for (name, value) in val {
            env.define(name, value, false);
        }
    }
}

pub fn register_native_module(mod_name: &str, env: Arc<Mutex<Env>>) {
    if mod_name == "native.bridge" {
        register_native_bridge(env);
    }
}

pub fn register_native_bridge(env: Arc<Mutex<Env>>) {
    let mut e = env.lock().unwrap();
    e.define(
        "__module__".to_string(),
        Value::String("native.bridge".to_string()),
        false,
    );
    e.define(
        "http".to_string(),
        function_value(vec![Param {
            name: "port".to_string(),
            type_name: "Int".to_string(),
            default_val: None,
            is_ref: false,
            is_mut: false,
        }]),
        false,
    );
}
