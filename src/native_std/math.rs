use crate::vm::Value;
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "pi".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Float(std::f64::consts::PI))),
    );
    m.insert(
        "e".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Float(std::f64::consts::E))),
    );

    m.insert(
        "abs".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.abs expects 1 argument".to_string()); }
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err("math.abs requires a number".to_string()),
            }
        }),
    );

    m.insert(
        "sin".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.sin expects 1 argument".to_string()); }
            if let Value::Float(f) = args[0] {
                Ok(Value::Float(f.sin()))
            } else if let Value::Int(i) = args[0] {
                Ok(Value::Float((i as f64).sin()))
            } else {
                Err("math.sin requires a number".to_string())
            }
        }),
    );

    m.insert(
        "cos".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.cos expects 1 argument".to_string()); }
            if let Value::Float(f) = args[0] {
                Ok(Value::Float(f.cos()))
            } else if let Value::Int(i) = args[0] {
                Ok(Value::Float((i as f64).cos()))
            } else {
                Err("math.cos requires a number".to_string())
            }
        }),
    );

    m.insert(
        "sqrt".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.sqrt expects 1 argument".to_string()); }
            if let Value::Float(f) = args[0] {
                Ok(Value::Float(f.sqrt()))
            } else if let Value::Int(i) = args[0] {
                Ok(Value::Float((i as f64).sqrt()))
            } else {
                Err("math.sqrt requires a number".to_string())
            }
        }),
    );

    m.insert(
        "inf".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Float(f64::INFINITY))),
    );

    m.insert(
        "min".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 2 { return Err("math.min expects 2 arguments".to_string()); }
            match (&args[0], &args[1]) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                _ => Err("math.min requires numbers".to_string())
            }
        }),
    );

    m.insert(
        "max".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 2 { return Err("math.max expects 2 arguments".to_string()); }
            match (&args[0], &args[1]) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                _ => Err("math.max requires numbers".to_string())
            }
        }),
    );

    m
}
