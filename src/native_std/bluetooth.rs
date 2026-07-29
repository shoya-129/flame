use crate::vm::Value;
use btleplug::{
    api::{Central, Manager as _, Peripheral as _, ScanFilter},
    platform::Manager,
};
use futures::executor::block_on;
use std::collections::HashMap;
use std::time::Duration;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    // bluetooth.supported()
    m.insert(
        "supported".into(),
        Value::NativeCallback(|_| {
            let ok = block_on(async { Manager::new().await.is_ok() });

            Ok(Value::Bool(ok))
        }),
    );

    // bluetooth.adapters()
    m.insert(
        "adapters".into(),
        Value::NativeCallback(|_| {
            let adapters = block_on(async {
                let manager = Manager::new().await.map_err(|e| e.to_string())?;
                manager.adapters().await.map_err(|e| e.to_string())
            })?;

            let mut out = Vec::new();

            for (i, _) in adapters.iter().enumerate() {
                let mut map = HashMap::new();

                map.insert("index".into(), Value::Int(i as i64));

                map.insert(
                    "name".into(),
                    Value::String(format!("Bluetooth Adapter {}", i)),
                );

                out.push(Value::Formula(map));
            }

            Ok(Value::Tuple(out))
        }),
    );

    // bluetooth.scan()
    m.insert(
        "scan".into(),
        Value::NativeCallback(|_| {
            let devices = block_on(async {
                let manager = Manager::new().await.map_err(|e| e.to_string())?;

                let adapters = manager.adapters().await.map_err(|e| e.to_string())?;

                let adapter = adapters
                    .into_iter()
                    .next()
                    .ok_or("No Bluetooth adapter found")?;

                adapter
                    .start_scan(ScanFilter::default())
                    .await
                    .map_err(|e| e.to_string())?;

                // Wait 3 seconds for BLE advertisements
                std::thread::sleep(Duration::from_secs(3));

                let peripherals = adapter.peripherals().await.map_err(|e| e.to_string())?;
                let mut out = Vec::new();

                for p in peripherals {
                    let properties = p.properties().await.unwrap_or(None);

                    let mut map = HashMap::new();

                    if let Some(props) = properties {
                        map.insert(
                            "name".into(),
                            Value::String(props.local_name.unwrap_or("Unknown".into())),
                        );

                        map.insert("address".into(), Value::String(props.address.to_string()));

                        map.insert(
                            "connected".into(),
                            Value::Bool(p.is_connected().await.unwrap_or(false)),
                        );
                    }

                    out.push(Value::Formula(map));
                }

                Ok::<_, String>(out)
            })?;

            Ok(Value::Tuple(devices))
        }),
    );

    m
}
