use crate::vm::Value;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "TcpListener".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            // Mark as module so static methods don't receive `self`
            obj.insert("__module__".to_string(), Value::Bool(true));
            
            obj.insert(
                "bind".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(addr)) = args.get(0) {
                        let listener = TcpListener::bind(addr)
                            .map_err(|e| format!("TCP bind error: {}", e))?;
                        
                        let mut l_obj = HashMap::new();
                        let listener = std::sync::Arc::new(std::sync::Mutex::new(listener));
                        let listener_clone = listener.clone();
                        
                        l_obj.insert(
                            "accept".to_string(),
                            Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                                // _args[0] is self
                                let (stream, _) = listener_clone.lock().unwrap().accept()
                                    .map_err(|e| format!("TCP accept error: {}", e))?;
                                
                                let mut s_obj = HashMap::new();
                                let stream = std::sync::Arc::new(std::sync::Mutex::new(stream));
                                
                                let stream_read = stream.clone();
                                s_obj.insert("read".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                                    // _args[0] is self
                                    let mut buf = [0; 4096];
                                    let n = stream_read.lock().unwrap().read(&mut buf).unwrap_or(0);
                                    let s = String::from_utf8_lossy(&buf[..n]).into_owned();
                                    Ok(Value::String(s))
                                }))));

                                let stream_write = stream.clone();
                                s_obj.insert("write".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                                    // args[0] is self, args[1] is data
                                    if args.len() >= 2 {
                                        if let Value::String(data) = &args[1] {
                                            stream_write.lock().unwrap().write_all(data.as_bytes())
                                                .map_err(|e| e.to_string())?;
                                            Ok(Value::Nil)
                                        } else {
                                            Err("write expects string".to_string())
                                        }
                                    } else {
                                        Err("write expects 1 argument".to_string())
                                    }
                                }))));

                                let stream_close = stream.clone();
                                s_obj.insert("close".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                                    // _args[0] is self
                                    stream_close.lock().unwrap().shutdown(std::net::Shutdown::Both)
                                        .unwrap_or(());
                                    Ok(Value::Nil)
                                }))));

                                Ok(Value::Object(s_obj))
                            }))),
                        );
                        
                        Ok(Value::Object(l_obj))
                    } else {
                        Err(format!("TcpListener.bind expects address string, got: {:?}", args.get(0)))
                    }
                }),
            );
            obj
        })
    );

    map.insert(
        "TcpSocket".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            // Mark as module so static methods don't receive `self`
            obj.insert("__module__".to_string(), Value::Bool(true));
            
            obj.insert(
                "connect".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(addr)) = args.get(0) {
                        let stream = TcpStream::connect(addr)
                            .map_err(|e| format!("TCP connect error: {}", e))?;
                        
                        let mut s_obj = HashMap::new();
                        let stream = std::sync::Arc::new(std::sync::Mutex::new(stream));
                        
                        let stream_read = stream.clone();
                        s_obj.insert("read".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                            // _args[0] is self
                            let mut buf = [0; 4096];
                            let n = stream_read.lock().unwrap().read(&mut buf).unwrap_or(0);
                            let s = String::from_utf8_lossy(&buf[..n]).into_owned();
                            Ok(Value::String(s))
                        }))));

                        let stream_write = stream.clone();
                        s_obj.insert("write".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                            // args[0] is self, args[1] is data
                            if args.len() >= 2 {
                                if let Value::String(data) = &args[1] {
                                    stream_write.lock().unwrap().write_all(data.as_bytes())
                                        .map_err(|e| e.to_string())?;
                                    Ok(Value::Nil)
                                } else {
                                    Err("write expects string".to_string())
                                }
                            } else {
                                Err("write expects 1 argument".to_string())
                            }
                        }))));

                        let stream_close = stream.clone();
                        s_obj.insert("close".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |_args| {
                            // _args[0] is self
                            stream_close.lock().unwrap().shutdown(std::net::Shutdown::Both)
                                .unwrap_or(());
                            Ok(Value::Nil)
                        }))));

                        Ok(Value::Object(s_obj))
                    } else {
                        Err(format!("TcpSocket.connect expects address string, got {:?}", args.get(0)))
                    }
                }),
            );
            obj
        })
    );

    map
}
