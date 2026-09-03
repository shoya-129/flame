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
                Value::Quantity(v, map) => Ok(Value::Quantity(v.abs(), map.clone())),
                Value::Unit(map) => Ok(Value::Unit(map.clone())),
                _ => Err("math.abs requires a number, quantity, or unit".to_string()),
            }
        }),
    );

    m.insert(
        "sin".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.sin expects 1 argument".to_string()); }
            if let Value::Float(f) = &args[0] {
                Ok(Value::Float(f.sin()))
            } else if let Value::Int(i) = &args[0] {
                Ok(Value::Float((*i as f64).sin()))
            } else if let Value::Quantity(v, map) = &args[0] {
                if !map.is_empty() {
                    return Err("math.sin requires a dimensionless number".to_string());
                }
                Ok(Value::Float(v.sin()))
            } else if let Value::Unit(map) = &args[0] {
                if !map.is_empty() {
                    return Err("math.sin requires a dimensionless number".to_string());
                }
                Ok(Value::Float((1.0_f64).sin()))
            } else {
                Err("math.sin requires a number or dimensionless quantity".to_string())
            }
        }),
    );

    m.insert(
        "cos".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.cos expects 1 argument".to_string()); }
            if let Value::Float(f) = &args[0] {
                Ok(Value::Float(f.cos()))
            } else if let Value::Int(i) = &args[0] {
                Ok(Value::Float((*i as f64).cos()))
            } else if let Value::Quantity(v, map) = &args[0] {
                if !map.is_empty() {
                    return Err("math.cos requires a dimensionless number".to_string());
                }
                Ok(Value::Float(v.cos()))
            } else if let Value::Unit(map) = &args[0] {
                if !map.is_empty() {
                    return Err("math.cos requires a dimensionless number".to_string());
                }
                Ok(Value::Float((1.0_f64).cos()))
            } else {
                Err("math.cos requires a number or dimensionless quantity".to_string())
            }
        }),
    );

    m.insert(
        "sqrt".to_string(),
        Value::NativeCallback(|args| {
            if args.is_empty() { return Err("math.sqrt expects 1 argument".to_string()); }
            if let Value::Float(f) = &args[0] {
                Ok(Value::Float(f.sqrt()))
            } else if let Value::Int(i) = &args[0] {
                Ok(Value::Float((*i as f64).sqrt()))
            } else if let Value::Quantity(v, map) = &args[0] {
                let mut new_map = HashMap::new();
                for (k, val) in map {
                    if val % 2 != 0 {
                        return Err(format!("math.sqrt requires all unit dimensions to have even powers, but '{}' has power {}", k, val));
                    }
                    if *val != 0 {
                        new_map.insert(k.clone(), val / 2);
                    }
                }
                if new_map.is_empty() {
                    Ok(Value::Float(v.sqrt()))
                } else {
                    Ok(Value::Quantity(v.sqrt(), new_map))
                }
            } else if let Value::Unit(map) = &args[0] {
                let mut new_map = HashMap::new();
                for (k, val) in map {
                    if val % 2 != 0 {
                        return Err(format!("math.sqrt requires all unit dimensions to have even powers, but '{}' has power {}", k, val));
                    }
                    if *val != 0 {
                        new_map.insert(k.clone(), val / 2);
                    }
                }
                if new_map.is_empty() {
                    Ok(Value::Float(1.0))
                } else {
                    Ok(Value::Unit(new_map))
                }
            } else {
                Err("math.sqrt requires a number or quantity".to_string())
            }
        }),
    );

    m.insert(
        "inf".to_string(),
        Value::NativeCallback(|_args| Ok(Value::Float(f64::INFINITY))),
    );

    m.insert(
        "pow".to_string(),
        Value::NativeCallback(|args| {
            if args.len() != 2 { return Err("math.pow expects 2 arguments (base, exp)".to_string()); }
            match (&args[0], &args[1]) {
                (Value::Float(b), Value::Float(e)) => Ok(Value::Float(b.powf(*e))),
                (Value::Float(b), Value::Int(e)) => Ok(Value::Float(b.powi(*e as i32))),
                (Value::Int(b), Value::Int(e)) => {
                    if *e >= 0 {
                        Ok(Value::Int(b.pow(*e as u32)))
                    } else {
                        Ok(Value::Float((*b as f64).powi(*e as i32)))
                    }
                },
                (Value::Int(b), Value::Float(e)) => Ok(Value::Float((*b as f64).powf(*e))),
                (Value::Quantity(v, u), Value::Int(e)) => {
                    let pow = *e as i32;
                    let mut res = HashMap::new();
                    for (k, p) in u {
                        let new_pow = p * pow;
                        if new_pow != 0 {
                            res.insert(k.clone(), new_pow);
                        }
                    }
                    let new_v = v.powi(pow);
                    if res.is_empty() {
                        Ok(Value::Float(new_v))
                    } else {
                        Ok(Value::Quantity(new_v, res))
                    }
                },
                (Value::Unit(u), Value::Int(e)) => {
                    let pow = *e as i32;
                    if pow == 0 {
                        Ok(Value::Float(1.0))
                    } else {
                        let mut res = HashMap::new();
                        for (k, v) in u {
                            let new_pow = v * pow;
                            if new_pow != 0 {
                                res.insert(k.clone(), new_pow);
                            }
                        }
                        if res.is_empty() {
                            Ok(Value::Float(1.0))
                        } else {
                            Ok(Value::Unit(res))
                        }
                    }
                },
                _ => Err("math.pow requires numbers or quantities".to_string()),
            }
        }),
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
                (Value::Quantity(v1, map1), Value::Quantity(v2, map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity(v1.min(*v2), map1.clone()))
                    } else {
                        Err("math.min requires quantities to have the same units".to_string())
                    }
                },
                (Value::Quantity(v1, map1), Value::Unit(map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity(v1.min(1.0), map1.clone()))
                    } else {
                        Err("math.min requires quantities to have the same units".to_string())
                    }
                },
                (Value::Unit(map1), Value::Quantity(v2, map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity((1.0_f64).min(*v2), map1.clone()))
                    } else {
                        Err("math.min requires quantities to have the same units".to_string())
                    }
                },
                (Value::Unit(map1), Value::Unit(map2)) => {
                    if map1 == map2 {
                        Ok(Value::Unit(map1.clone()))
                    } else {
                        Err("math.min requires units to have the same dimensions".to_string())
                    }
                },
                _ => Err("math.min requires numbers or quantities of the same unit".to_string())
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
                (Value::Quantity(v1, map1), Value::Quantity(v2, map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity(v1.max(*v2), map1.clone()))
                    } else {
                        Err("math.max requires quantities to have the same units".to_string())
                    }
                },
                (Value::Quantity(v1, map1), Value::Unit(map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity(v1.max(1.0), map1.clone()))
                    } else {
                        Err("math.max requires quantities to have the same units".to_string())
                    }
                },
                (Value::Unit(map1), Value::Quantity(v2, map2)) => {
                    if map1 == map2 {
                        Ok(Value::Quantity((1.0_f64).max(*v2), map1.clone()))
                    } else {
                        Err("math.max requires quantities to have the same units".to_string())
                    }
                },
                (Value::Unit(map1), Value::Unit(map2)) => {
                    if map1 == map2 {
                        Ok(Value::Unit(map1.clone()))
                    } else {
                        Err("math.max requires units to have the same dimensions".to_string())
                    }
                },
                _ => Err("math.max requires numbers or quantities of the same unit".to_string())
            }
        }),
    );

    m
}
