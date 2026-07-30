use crate::parser::Param;
use crate::vm::{self, Env, Value};
use std::sync::{Arc, Mutex};

pub fn register_global_builtins(env: Arc<Mutex<Env>>) {
    let mut e = env.lock().unwrap();
    e.define("print".to_string(), Value::Nil, false);
    e.define("eprint".to_string(), Value::Nil, false);
}

fn function_value(params: Vec<Param>) -> Value {
    Value::Function {
        params,
        body: vec![],
        env: Arc::new(Mutex::new(Env::new())),
    }
}

pub fn register_std_module(mod_name: &str, env: Arc<Mutex<Env>>) {
    let module_val = match mod_name {
        "std.thread" => Some(crate::native_std::thread::init()),
        "std.process" => Some(crate::native_std::process::init()),
        "std.fs" => Some(crate::native_std::fs::init()),
        // "std.net" => Some(crate::native_std::net::init()),
        "std.math" => Some(crate::native_std::math::init()),
        "std.time" => Some(crate::native_std::time::init()),
        "std.os" => Some(crate::native_std::os::init()),
        "std.hardware" => Some(crate::native_std::hardware::init()),
        "std.desktop" => Some(crate::native_std::desktop::init()),
        "std.env" => Some(crate::native_std::env::init()),
        "std.hid" => Some(crate::native_std::hid::init()),
        "std.camera" => Some(crate::native_std::camera::init()),
        "std.bluetooth" => Some(crate::native_std::bluetooth::init()),
        "std.serial" => Some(crate::native_std::serial::init()),
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
