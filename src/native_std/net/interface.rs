use crate::vm::Value;
use std::collections::HashMap;

#[cfg(feature = "net")]
pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "interfaces".to_string(),
        Value::NativeCallback(|_| {
            let interfaces = default_net::get_interfaces();
            let mut result_list = Vec::new();
            for iface in interfaces {
                let mut iface_obj = HashMap::new();
                iface_obj.insert("name".to_string(), Value::String(iface.name));
                iface_obj.insert("mac".to_string(), iface.mac_addr.map(|m| Value::String(m.address())).unwrap_or(Value::Nil));
                
                let mut ips = Vec::new();
                for ip in iface.ipv4 {
                    ips.push(Value::String(ip.addr.to_string()));
                }
                for ip in iface.ipv6 {
                    ips.push(Value::String(ip.addr.to_string()));
                }
                iface_obj.insert("ips".to_string(), Value::Tuple(ips));
                
                result_list.push(Value::Object(iface_obj));
            }
            Ok(Value::Tuple(result_list))
        })
    );

    map
}

#[cfg(not(feature = "net"))]
pub fn init() -> HashMap<String, Value> {
    HashMap::new()
}
