use crate::native_std::fs::{extract_bytes, resolve_path};
use crate::vm::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "readBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("byte.readBytes expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            match fs::read(&path) {
                Ok(bytes) => Ok(Value::Bytes(bytes)),
                Err(e) => Err(format!("byte.readBytes error: {}", e)),
            }
        }),
    );

    m.insert(
        "writeBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("byte.writeBytes expects 2 arguments (path, bytes)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let bytes = extract_bytes(&args[1]).map_err(|e| format!("byte.writeBytes error: {}", e))?;
            match fs::write(&path, bytes) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("byte.writeBytes error: {}", e)),
            }
        }),
    );

    m.insert(
        "appendBytes".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("byte.appendBytes expects 2 arguments (path, bytes)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let bytes = extract_bytes(&args[1]).map_err(|e| format!("byte.appendBytes error: {}", e))?;
            match fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut file) => match file.write_all(&bytes) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(format!("byte.appendBytes error: {}", e)),
                },
                Err(e) => Err(format!("byte.appendBytes error: {}", e)),
            }
        }),
    );

    m.insert(
        "readByte".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("byte.readByte expects 1 argument (path)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let mut file = match fs::File::open(&path) {
                Ok(file) => file,
                Err(e) => return Err(format!("byte.readByte error: {}", e)),
            };
            let mut byte = [0u8; 1];
            match file.read_exact(&mut byte) {
                Ok(_) => Ok(Value::Byte(byte[0])),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    Err("byte.readByte error: end of file".to_string())
                }
                Err(e) => Err(format!("byte.readByte error: {}", e)),
            }
        }),
    );

    m.insert(
        "writeByte".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("byte.writeByte expects 2 arguments (path, byte)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let byte = match &args[1] {
                Value::Byte(b) => *b,
                Value::Int(n) => {
                    if *n < 0 || *n > 255 {
                        return Err("byte.writeByte: byte must be between 0 and 255".to_string());
                    }
                    *n as u8
                }
                _ => {
                    return Err(format!(
                        "byte.writeByte: expected Byte, found {}",
                        args[1].type_name()
                    ));
                }
            };
            match fs::write(&path, [byte]) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("byte.writeByte error: {}", e)),
            }
        }),
    );

    m.insert(
        "appendByte".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("byte.appendByte expects 2 arguments (path, byte)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let byte = match &args[1] {
                Value::Byte(b) => *b,
                Value::Int(n) => {
                    if *n < 0 || *n > 255 {
                        return Err("byte.appendByte: byte must be between 0 and 255".to_string());
                    }
                    *n as u8
                }
                _ => {
                    return Err(format!(
                        "byte.appendByte: expected Byte, found {}",
                        args[1].type_name()
                    ));
                }
            };
            match fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut file) => match file.write_all(&[byte]) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(format!("byte.appendByte error: {}", e)),
                },
                Err(e) => Err(format!("byte.appendByte error: {}", e)),
            }
        }),
    );

    m.insert(
        "readByteAt".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("byte.readByteAt expects 2 arguments (path, offset)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let offset = match &args[1] {
                Value::Int(n) if *n >= 0 => *n as u64,
                _ => return Err("byte.readByteAt: offset must be a non-negative integer".to_string()),
            };
            let mut file = match fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => return Err(format!("byte.readByteAt error: {}", e)),
            };
            if let Err(e) = file.seek(SeekFrom::Start(offset)) {
                return Err(format!("byte.readByteAt error: {}", e));
            }
            let mut byte = [0u8; 1];
            match file.read_exact(&mut byte) {
                Ok(_) => Ok(Value::Byte(byte[0])),
                Err(e) => Err(format!("byte.readByteAt error: {}", e)),
            }
        }),
    );

    m.insert(
        "writeByteAt".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 3 {
                return Err("byte.writeByteAt expects 3 arguments (path, offset, byte)".to_string());
            }
            let path = resolve_path(&args[0].to_string().trim_matches('"'));
            let offset = match &args[1] {
                Value::Int(n) if *n >= 0 => *n as u64,
                _ => {
                    return Err(
                        "byte.writeByteAt: offset must be a non-negative integer".to_string()
                    );
                }
            };
            let byte = match &args[2] {
                Value::Byte(b) => *b,
                Value::Int(n) => {
                    if *n < 0 || *n > 255 {
                        return Err("byte.writeByteAt: byte must be between 0 and 255".to_string());
                    }
                    *n as u8
                }
                _ => {
                    return Err(format!(
                        "byte.writeByteAt: expected Byte, found {}",
                        args[2].type_name()
                    ));
                }
            };
            let mut file = match fs::OpenOptions::new().write(true).create(true).open(&path) {
                Ok(f) => f,
                Err(e) => return Err(format!("byte.writeByteAt error: {}", e)),
            };
            if let Err(e) = file.seek(SeekFrom::Start(offset)) {
                return Err(format!("byte.writeByteAt error: {}", e));
            }
            match file.write_all(&[byte]) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(format!("byte.writeByteAt error: {}", e)),
            }
        }),
    );

    m.insert(
        "fromByte".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                match val {
                    Value::Byte(b) => Ok(Value::Byte(*b)),
                    Value::Int(n) => {
                        if *n < 0 || *n > 255 {
                            return Err(format!("byte value out of range (0..255): {}", n));
                        }
                        Ok(Value::Byte(*n as u8))
                    }
                    Value::String(s) => {
                        if let Some(b) = s.as_bytes().first() {
                            Ok(Value::Byte(*b))
                        } else {
                            Err("byte.fromByte expects non-empty string".to_string())
                        }
                    }
                    Value::Bytes(b) => {
                        if let Some(first) = b.first() {
                            Ok(Value::Byte(*first))
                        } else {
                            Err("byte.fromByte expects non-empty bytes".to_string())
                        }
                    }
                    _ => Err(format!("byte.fromByte expects Byte, Int, or String, found {}", val.type_name())),
                }
            } else {
                Err("byte.fromByte expects 1 argument".to_string())
            }
        }),
    );

    m.insert(
        "fromBytes".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                let bytes = extract_bytes(val).map_err(|e| format!("byte.fromBytes error: {}", e))?;
                Ok(Value::Bytes(bytes))
            } else {
                Err("byte.fromBytes expects 1 argument".to_string())
            }
        }),
    );

    m.insert(
        "toString".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                let bytes = extract_bytes(val).map_err(|e| format!("byte.toString error: {}", e))?;
                Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
            } else {
                Err("byte.toString expects 1 argument".to_string())
            }
        }),
    );

    m.insert(
        "toHex".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                let bytes = extract_bytes(val).map_err(|e| format!("byte.toHex error: {}", e))?;
                let hex_str = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                Ok(Value::String(hex_str))
            } else {
                Err("byte.toHex expects 1 argument".to_string())
            }
        }),
    );

    m.insert(
        "toInt".to_string(),
        Value::NativeCallback(|args| {
            if let Some(val) = args.get(0) {
                match val {
                    Value::Byte(b) => Ok(Value::Int(*b as i64)),
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(format!("byte.toInt expects Byte or Int, found {}", val.type_name())),
                }
            } else {
                Err("byte.toInt expects 1 argument".to_string())
            }
        }),
    );

    m
}
