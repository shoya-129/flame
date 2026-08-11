pub mod fs;
pub mod byte;
pub mod thread;
pub mod process;
pub mod env;
pub mod math;
pub mod time;
pub mod os;
pub mod hardware;
pub mod desktop;
pub mod hid;
pub mod serial;
pub mod bluetooth;
pub mod camera;
pub mod embedded;
pub mod json;
#[cfg(feature = "net")]
pub mod net;



use crate::vm::{Env, Value};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Helper to define native callbacks in a module
pub fn define_module(env: Arc<Mutex<Env>>, name: &str, init: fn() -> HashMap<String, Value>) {
    let mut e = env.lock().unwrap();
    let mut map = init();
    map.insert("__module__".to_string(), Value::String(name.to_string()));
    
    // Register top level namespace
    e.define(name.strip_prefix("std.").unwrap_or(name).to_string(), Value::Formula(map.clone()), false);
    
    // Legacy global functions registration to not break parsing/typing if they rely on it
    for (k, v) in map {
        e.define(k, v, false);
    }
}