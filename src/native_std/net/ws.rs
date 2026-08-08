use crate::vm::Value;
use std::collections::HashMap;

#[cfg(feature = "ws")]
pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "WebSocket".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            obj.insert(
                "connect".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(url_str)) = args.get(0) {
                        let (socket, _) = tungstenite::connect(url_str)
                            .map_err(|e| format!("WebSocket connect error: {}", e))?;
                        
                        let mut s_obj = HashMap::new();
                        let socket = std::sync::Arc::new(std::sync::Mutex::new(socket));
                        
                        let sock_recv = socket.clone();
                        s_obj.insert("recv".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_| {
                            let msg = sock_recv.lock().unwrap().read()
                                .map_err(|e| format!("WebSocket recv error: {}", e))?;
                            Ok(Value::String(msg.into_text().unwrap_or_default()))
                        }))));

                        let sock_send = socket.clone();
                        s_obj.insert("send".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                            if let Some(Value::String(data)) = args.get(0) {
                                sock_send.lock().unwrap().write(tungstenite::Message::Text(data.clone()))
                                    .map_err(|e| e.to_string())?;
                                Ok(Value::Nil)
                            } else {
                                Err("send expects a string".to_string())
                            }
                        }))));
                        
                        Ok(Value::Object(s_obj))
                    } else {
                        Err("WebSocket.connect expects URL string".to_string())
                    }
                })
            );
            obj
        })
    );

    map
}

#[cfg(not(feature = "ws"))]
pub fn init() -> HashMap<String, Value> {
    HashMap::new()
}
