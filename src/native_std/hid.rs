use crate::vm::Value;
use hidapi::{HidApi, HidDevice};
use std::ffi::CString;
use std::{
    collections::HashMap,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

static HID_API: LazyLock<Mutex<HidApi>> = LazyLock::new(|| Mutex::new(HidApi::new().unwrap()));

static DEVICES: LazyLock<Mutex<HashMap<u64, HidDevice>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "devices".into(),
        Value::NativeCallback(|_| {
            let api = HidApi::new().map_err(|e| e.to_string())?;

            let mut list = Vec::new();

            for dev in api.device_list() {
                let mut map = HashMap::new();

                map.insert("vendor".into(), Value::Int(dev.vendor_id() as i64));
                map.insert("product".into(), Value::Int(dev.product_id() as i64));

                map.insert(
                    "manufacturer".into(),
                    Value::String(dev.manufacturer_string().unwrap_or("").to_string()),
                );

                map.insert(
                    "product_name".into(),
                    Value::String(dev.product_string().unwrap_or("").to_string()),
                );

                map.insert(
                    "serial".into(),
                    Value::String(dev.serial_number().unwrap_or("").to_string()),
                );

                map.insert(
                    "path".into(),
                    Value::String(dev.path().to_string_lossy().to_string()),
                );

                map.insert("usage_page".into(), Value::Int(dev.usage_page() as i64));

                map.insert("usage".into(), Value::Int(dev.usage() as i64));

                map.insert(
                    "interface".into(),
                    Value::Int(dev.interface_number() as i64),
                );

                map.insert("release".into(), Value::Int(dev.release_number() as i64));

                list.push(Value::Formula(map));
            }

            Ok(Value::Tuple(list))
        }),
    );

    m.insert(
        "open".into(),
        Value::NativeCallback(|args| {
            if args.len() != 2 {
                return Err("hid.open(vendor, product)".into());
            }

            let vendor = match args[0] {
                Value::Int(v) => v as u16,
                _ => return Err("vendor must be int".into()),
            };

            let product = match args[1] {
                Value::Int(v) => v as u16,
                _ => return Err("product must be int".into()),
            };

            let api = HID_API.lock().unwrap();

            let device = api.open(vendor, product).map_err(|e| e.to_string())?;

            let id = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);

            DEVICES.lock().unwrap().insert(id, device);

            Ok(Value::Int(id as i64))
        }),
    );
    m.insert(
        "openPath".into(),
        Value::NativeCallback(|args| {
            let path = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("path must be a string".into()),
            };

            let c_path = CString::new(path).map_err(|e| e.to_string())?;

            let api = HID_API.lock().unwrap();

            let device = api
                .open_path(c_path.as_c_str())
                .map_err(|e| e.to_string())?;

            let id = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);

            DEVICES.lock().unwrap().insert(id, device);

            Ok(Value::Int(id as i64))
        }),
    );
    m.insert(
        "close".into(),
        Value::NativeCallback(|args| {
            let id = match args[0] {
                Value::Int(v) => v as u64,
                _ => return Err("handle".into()),
            };

            DEVICES.lock().unwrap().remove(&id);

            Ok(Value::Nil)
        }),
    );
    m.insert(
        "read".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;
            let len = args[1].as_int()? as usize;

            let devices = DEVICES.lock().unwrap();

            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let mut buf = vec![0u8; len];

            let size = dev.read(&mut buf).map_err(|e| e.to_string())?;

            buf.truncate(size);

            Ok(Value::Bytes(buf))
        }),
    );
    m.insert(
        "readTimeout".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;
            let len = args[1].as_int()? as usize;
            let timeout = args[2].as_int()? as i32;

            let devices = DEVICES.lock().unwrap();

            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let mut buf = vec![0u8; len];

            let size = dev
                .read_timeout(&mut buf, timeout)
                .map_err(|e| e.to_string())?;

            buf.truncate(size);

            Ok(Value::Bytes(buf))
        }),
    );
    m.insert(
        "write".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;

            let bytes = args[1].as_bytes()?;

            let devices = DEVICES.lock().unwrap();

            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let written = dev.write(&bytes).map_err(|e| e.to_string())?;

            Ok(Value::Int(written as i64))
        }),
    );
    m.insert(
        "getFeatureReport".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;
            let report = args[1].as_int()? as u8;
            let length = args[2].as_int()? as usize;

            let devices = DEVICES.lock().unwrap();

            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let mut buf = vec![0u8; length];

            buf[0] = report;

            let size = dev
                .get_feature_report(&mut buf)
                .map_err(|e| e.to_string())?;

            buf.truncate(size);

            Ok(Value::Bytes(buf))
        }),
    );

    m.insert(
        "sendOutputReport".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;
            let bytes = args[1].as_bytes()?;

            let devices = DEVICES.lock().unwrap();
            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let written = dev.write(&bytes).map_err(|e| e.to_string())?;

            Ok(Value::Int(written as i64))
        }),
    );

    m.insert(
        "poll".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;
            let len = args[1].as_int()? as usize;

            let devices = DEVICES.lock().unwrap();
            let dev = devices.get(&handle).ok_or("invalid handle")?;

            let mut buf = vec![0u8; len];

            let size = dev.read_timeout(&mut buf, 0).map_err(|e| e.to_string())?;

            buf.truncate(size);

            Ok(Value::Bytes(buf))
        }),
    );

    m.insert(
        "isOpen".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;

            Ok(Value::Bool(DEVICES.lock().unwrap().contains_key(&handle)))
        }),
    );

    m.insert(
        "setBlocking".into(),
        Value::NativeCallback(|args| {
            let handle = args[0].as_int()? as u64;

            let blocking = args[1].as_bool()?;

            let devices = DEVICES.lock().unwrap();

            let dev = devices.get(&handle).ok_or("invalid handle")?;

            dev.set_blocking_mode(blocking).map_err(|e| e.to_string())?;

            Ok(Value::Nil)
        }),
    );

    m
}
