pub mod fs;
pub mod byte;
pub mod thread;
pub mod process;
pub mod env;
pub mod math;
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "os")]
pub mod os;
#[cfg(feature = "hardware")]
pub mod hardware;
#[cfg(feature = "robot")]
pub mod desktop;
#[cfg(feature = "hardware")]
pub mod hid;
#[cfg(feature = "hardware")]
pub mod serial;
#[cfg(feature = "bluetooth")]
pub mod bluetooth;
#[cfg(feature = "camera")]
pub mod camera;
pub mod embedded;
pub mod json;
pub mod fmt;
#[cfg(feature = "net")]
pub mod net;



use crate::vm::{Env, Value};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::vm::NativeModuleDef;

pub fn get_module_defs() -> Vec<NativeModuleDef> {
    let mut defs = vec![
        fmt::def(),
    ];
    
    #[cfg(feature = "time")]
    defs.push(time::def());
    
    #[cfg(feature = "net")]
    defs.extend(net::get_module_defs());
    
    defs
}

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