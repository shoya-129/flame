use crate::vm::Value;
use serialport;
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "ports".into(),
        Value::NativeCallback(|_| {
            let ports = serialport::available_ports().map_err(|e| e.to_string())?;

            let mut out = Vec::new();

            for port in ports {
                let mut map = HashMap::new();

                map.insert("name".into(), Value::String(port.port_name));

                out.push(Value::Formula(map));
            }

            Ok(Value::Tuple(out))
        }),
    );

    m
}
