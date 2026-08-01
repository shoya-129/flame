use crate::vm::Value;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "sleep".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("thread.sleep expects 1 argument (ms)".to_string());
            }
            if let Value::Int(ms) = args[0] {
                thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::Nil)
            } else {
                Err("thread.sleep argument must be an Int".to_string())
            }
        }),
    );

    m.insert(
        "spawn".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("thread.spawn expects 1 argument (function or callback)".to_string());
            }
            if let Value::Function { .. } = &args[0] {
                // Note: Real OS thread execution of Flame functions is intercepted directly
                // in runner.rs during member evaluation, where the Runner creates an isolated
                // environment snapshot and spawns a true std::thread with return-value futures.
                // This fallback is only reached if invoked outside an active evaluation loop.
                Ok(Value::ThreadHandler(0))
            } else if let Value::NativeCallback(cb) = args[0] {
                let mut counter = crate::vm::get_thread_counter().lock().unwrap();
                *counter += 1;
                let id = *counter;
                let handle = thread::spawn(move || {
                    cb(vec![]).unwrap_or(Value::Nil)
                });
                crate::vm::get_threads().lock().unwrap().insert(id, handle);
                Ok(Value::ThreadHandler(id))
            } else {
                Err("thread.spawn argument must be a function or callback".to_string())
            }
        }),
    );

    m.insert(
        "id".to_string(),
        Value::NativeCallback(|_args| {
            // Rust thread ids are opaque, but we can format them
            Ok(Value::String(format!("{:?}", thread::current().id())))
        }),
    );

    m.insert(
        "yield".to_string(),
        Value::NativeCallback(|_args| {
            thread::yield_now();
            Ok(Value::Nil)
        }),
    );

    m.insert(
        "channel".to_string(),
        Value::NativeCallback(|_args| {
            let mut counter = crate::vm::get_channel_counter().lock().unwrap();
            *counter += 1;
            let chan_id = *counter;

            let (tx, rx) = std::sync::mpsc::channel();
            crate::vm::get_channels().lock().unwrap().insert(chan_id, tx);
            crate::vm::get_receivers()
                .lock()
                .unwrap()
                .insert(chan_id, std::sync::Arc::new(std::sync::Mutex::new(rx)));

            Ok(Value::Tuple(vec![
                Value::Sender(chan_id),
                Value::Receiver(chan_id),
            ]))
        }),
    );

    m
}
