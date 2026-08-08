use crate::vm::Value;
use std::collections::HashMap;
use std::net::ToSocketAddrs;

pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "lookup".to_string(),
        Value::NativeCallback(|args| {
            if let Some(Value::String(domain)) = args.get(0) {
                // ToSocketAddrs needs a port to work, we'll append :0
                let domain_with_port = if domain.contains(':') {
                    domain.clone()
                } else {
                    format!("{}:0", domain)
                };
                
                match domain_with_port.to_socket_addrs() {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() {
                            Ok(Value::String(addr.ip().to_string()))
                        } else {
                            Err(format!("No addresses found for {}", domain))
                        }
                    }
                    Err(e) => Err(format!("DNS lookup failed: {}", e))
                }
            } else {
                Err("dns.lookup expects a string".to_string())
            }
        })
    );

    map
}
