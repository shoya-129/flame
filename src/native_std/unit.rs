use crate::vm::Value;
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    m.insert(
        "Equation".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 3 {
                return Err("unit.Equation expects exactly 3 arguments: (kg, m, s)".to_string());
            }

            let mut unit_map = HashMap::new();

            // Extract kg
            if let Value::Int(kg) = &args[0] {
                if *kg != 0 {
                    unit_map.insert("kg".to_string(), *kg as i32);
                }
            } else {
                return Err("kg must be an integer".to_string());
            }

            // Extract m
            if let Value::Int(m) = &args[1] {
                if *m != 0 {
                    unit_map.insert("m".to_string(), *m as i32);
                }
            } else {
                return Err("m must be an integer".to_string());
            }

            // Extract s
            if let Value::Int(s) = &args[2] {
                if *s != 0 {
                    unit_map.insert("s".to_string(), *s as i32);
                }
            } else {
                return Err("s must be an integer".to_string());
            }

            Ok(Value::Unit(unit_map))
        }),
    );

    m.insert(
        "meter".to_string(),
        Value::Unit(HashMap::from([("m".to_string(), 1)])),
    );
    m.insert(
        "second".to_string(),
        Value::Unit(HashMap::from([("s".to_string(), 1)])),
    );
    m.insert(
        "kilogram".to_string(),
        Value::Unit(HashMap::from([("kg".to_string(), 1)])),
    );

    m
}
