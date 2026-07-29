use crate::vm::Value;
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::collections::HashMap;

fn parse_key(name: &str) -> Result<Key, String> {
    let name = name.trim_matches('"').to_lowercase();

    Ok(match name.as_str() {
        "ctrl" | "control" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
        "cmd" | "command" => Key::Meta,
        "win" | "super" => Key::Meta,
        "option" => Key::Alt,

        "enter" => Key::Return,
        "tab" => Key::Tab,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "escape" | "esc" => Key::Escape,

        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,

        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,

        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,

        _ => {
            if name.len() == 1 {
                Key::Unicode(name.chars().next().unwrap())
            } else {
                return Err(format!("Unknown key '{}'", name));
            }
        }
    })
}

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    let mut mouse = HashMap::new();
    let mut keyboard = HashMap::new();

    // ---------------- Mouse ----------------

    mouse.insert(
        "move".into(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("mouseMove expects 2 arguments (x, y)".to_string());
            }
            let x = if let Value::Int(i) = args[0] {
                i as i32
            } else {
                0
            };
            let y = if let Value::Int(i) = args[1] {
                i as i32
            } else {
                0
            };
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
            Ok(Value::Nil)
        }),
    );

    mouse.insert(
        "click".to_string(),
        Value::NativeCallback(|args| {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            let button = if args.is_empty() {
                Button::Left
            } else if let Value::String(s) = &args[0] {
                match s.as_str() {
                    "right" => Button::Right,
                    "middle" => Button::Middle,
                    _ => Button::Left,
                }
            } else {
                Button::Left
            };
            let _ = enigo.button(button, Direction::Click);
            Ok(Value::Nil)
        }),
    );

    // ---------------- Keyboard ----------------

    keyboard.insert(
        "write".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err("keyboardType expects 1 argument (text)".to_string());
            }
            let text = args[0].to_string().trim_matches('"').to_string();
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            let _ = enigo.text(&text);
            Ok(Value::Nil)
        }),
    );

    keyboard.insert(
        "hotkey".to_string(),
        Value::NativeCallback(|args| {
            if args.len() < 2 {
                return Err("keyboard.hotkey expects at least 2 keys".to_string());
            }

            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

            let keys: Result<Vec<_>, _> = args.iter().map(|v| parse_key(&v.to_string())).collect();

            let keys = keys?;

            for key in &keys[..keys.len() - 1] {
                enigo
                    .key(*key, Direction::Press)
                    .map_err(|e| e.to_string())?;
            }

            enigo
                .key(keys[keys.len() - 1], Direction::Click)
                .map_err(|e| e.to_string())?;

            for key in keys[..keys.len() - 1].iter().rev() {
                enigo
                    .key(*key, Direction::Release)
                    .map_err(|e| e.to_string())?;
            }
            Ok(Value::Nil)
        }),
    );

    keyboard.insert(
        "key".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() {
                return Err(
                    "automation.key expects at least 1 argument (key, [action])".to_string()
                );
            }

            let key = parse_key(&args[0].to_string())?;

            let direction = if args.len() >= 2 {
                match args[1].to_string().trim_matches('"') {
                    "press" => Direction::Press,
                    "release" => Direction::Release,
                    _ => Direction::Click,
                }
            } else {
                Direction::Click
            };

            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo.key(key, direction).map_err(|e| e.to_string())?;

            Ok(Value::Nil)
        }),
    );

    m.insert("mouse".into(), Value::Object(mouse));

    m.insert("keyboard".into(), Value::Object(keyboard));
    m
}
