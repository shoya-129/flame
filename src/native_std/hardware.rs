use crate::vm::Value;
use std::collections::HashMap;
use sysinfo::System;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "cpu".to_string(),
        Value::NativeCallback(|_args| {
            let mut sys = System::new_all();
            sys.refresh_cpu();
            let mut cpus = Vec::new();
            for cpu in sys.cpus() {
                let mut map = HashMap::new();
                map.insert("name".to_string(), Value::String(cpu.name().to_string()));
                map.insert("usage".to_string(), Value::Float(cpu.cpu_usage() as f64));
                map.insert("brand".to_string(), Value::String(cpu.brand().to_string()));
                map.insert("frequency".to_string(), Value::Int(cpu.frequency() as i64));
                cpus.push(Value::Formula(map));
            }
            Ok(Value::Tuple(cpus))
        }),
    );

    m.insert(
        "memory".to_string(),
        Value::NativeCallback(|_args| {
            let mut sys = System::new_all();
            sys.refresh_memory();
            let mut map = HashMap::new();
            map.insert("total".to_string(), Value::Int(sys.total_memory() as i64));
            map.insert("used".to_string(), Value::Int(sys.used_memory() as i64));
            map.insert("free".to_string(), Value::Int(sys.free_memory() as i64));
            map.insert("available".to_string(), Value::Int(sys.available_memory() as i64));
            Ok(Value::Formula(map))
        }),
    );

    m.insert(
        "discover".to_string(),
        Value::NativeCallback(|_args| {
            let _sys = System::new_all();
            Ok(Value::String(format!("System Information: {} {} ({})", 
                sysinfo::System::name().unwrap_or_default(), 
                sysinfo::System::os_version().unwrap_or_default(),
                sysinfo::System::kernel_version().unwrap_or_default()
            )))
        }),
    );

    m
}
