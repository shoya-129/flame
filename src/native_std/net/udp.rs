use crate::vm::Value;
use std::collections::HashMap;
use std::net::UdpSocket;

pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "UdpSocket".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            obj.insert("__module__".to_string(), Value::Bool(true));
            
            obj.insert(
                "bind".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(addr)) = args.get(0) {
                        let socket = UdpSocket::bind(addr)
                            .map_err(|e| format!("UDP bind error: {}", e))?;
                        
                        let mut s_obj = HashMap::new();
                        let socket = std::sync::Arc::new(std::sync::Mutex::new(socket));
                        
                        let sock_recv = socket.clone();
                        s_obj.insert("recv".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                            let mut buf = [0; 65535];
                            let (n, src) = sock_recv.lock().unwrap().recv_from(&mut buf)
                                .map_err(|e| format!("UDP recv error: {}", e))?;
                            let s = String::from_utf8_lossy(&buf[..n]).into_owned();
                            
                            Ok(Value::Tuple(vec![Value::String(s), Value::String(src.to_string())]))
                        }))));

                        let sock_send = socket.clone();
                        s_obj.insert("send".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                            if args.len() >= 3 {
                                if let (Value::String(data), Value::String(dest)) = (&args[1], &args[2]) {
                                    sock_send.lock().unwrap().send_to(data.as_bytes(), dest)
                                        .map_err(|e| e.to_string())?;
                                    Ok(Value::Nil)
                                } else {
                                    Err("send expects (string, string)".to_string())
                                }
                            } else {
                                Err("send expects (data, dest)".to_string())
                            }
                        }))));
                        
                        Ok(Value::Object(s_obj))
                    } else {
                        Err(format!("UdpSocket.bind expects address string, got {:?}", args.get(0)))
                    }
                }),
            );
            obj
        })
    );

    map
}
