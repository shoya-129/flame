use crate::vm::{Value, set_event_loop_active};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "mqtt")]
pub fn init() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map.insert(
        "Mqtt".to_string(),
        Value::Object({
            let mut obj = HashMap::new();
            obj.insert(
                "connect".to_string(),
                Value::NativeCallback(|args| {
                    if let Some(Value::String(broker_url)) = args.get(0) {
                        let parsed_url = url::Url::parse(&broker_url).unwrap_or_else(|_| url::Url::parse("mqtt://localhost:1883").unwrap());
                        let host = parsed_url.host_str().unwrap_or("localhost");
                        let port = parsed_url.port().unwrap_or(1883);

                        let mut mqttoptions = rumqttc::MqttOptions::new(
                            format!("flame_client_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()), 
                            host, port
                        );
                        mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

                        let (client, mut connection) = rumqttc::Client::new(mqttoptions, 10);
                        
                        let mut s_obj = HashMap::new();
                        let client = Arc::new(Mutex::new(client));
                        let callbacks: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
                        
                        let client_pub = client.clone();
                        s_obj.insert("publish".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                            if args.len() >= 2 {
                                if let (Value::String(topic), Value::String(payload)) = (&args[0], &args[1]) {
                                    client_pub.lock().unwrap().publish(topic, rumqttc::QoS::AtLeastOnce, false, payload.as_bytes())
                                        .map_err(|e| e.to_string())?;
                                    Ok(Value::Nil)
                                } else {
                                    Err("publish expects (string, string)".to_string())
                                }
                            } else {
                                Err("publish expects (topic, payload)".to_string())
                            }
                        }))));

                        let client_sub = client.clone();
                        let cbs = callbacks.clone();
                        s_obj.insert("subscribe".to_string(), Value::NativeClosure(crate::vm::NativeClosureType(std::sync::Arc::new(move |args| {
                            if args.len() >= 2 {
                                if let (Value::String(topic), Value::Function { .. } | Value::NativeCallback(_) | Value::NativeClosure(_)) = (&args[0], &args[1]) {
                                    let callback = args[1].clone();
                                    client_sub.lock().unwrap().subscribe(topic, rumqttc::QoS::AtMostOnce)
                                        .map_err(|e| e.to_string())?;
                                    
                                    cbs.lock().unwrap().insert(topic.clone(), callback);
                                    set_event_loop_active(true);
                                    
                                    Ok(Value::Nil)
                                } else {
                                    Err("subscribe expects (string, function)".to_string())
                                }
                            } else {
                                Err("subscribe expects (topic, callback)".to_string())
                            }
                        }))));
                        
                        std::thread::spawn(move || {
                            for notification in connection.iter() {
                                if let Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(p))) = notification {
                                    let topic = p.topic;
                                    let payload = String::from_utf8_lossy(&p.payload).into_owned();
                                    
                                    if let Some(cb) = callbacks.lock().unwrap().get(&topic).cloned() {
                                        let cb_id = crate::vm::register_callback_value(cb);
                                        let flame_cb = crate::vm::FlameCallback { function_id: cb_id, module_id: 0 };
                                        let _ = crate::vm::enqueue_callback(flame_cb, vec![crate::vm::CValue::from_string(&payload)]);
                                    }
                                }
                            }
                        });

                        Ok(Value::Object(s_obj))
                    } else {
                        Err("Mqtt.connect expects broker URL string".to_string())
                    }
                })
            );
            obj
        })
    );

    map
}

#[cfg(not(feature = "mqtt"))]
pub fn init() -> HashMap<String, Value> {
    HashMap::new()
}
