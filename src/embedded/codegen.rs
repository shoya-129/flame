use crate::parser::{Annotation, BinaryOp, Expr, LiteralValue, Stmt, UnaryOp};
use std::fs;
use std::path::{Path, PathBuf};

pub fn detect_embedded_target(ast: &[Stmt]) -> Option<(String, u32)> {
    for stmt in ast {
        let annotations = match stmt {
            Stmt::FuncDecl { annotations, .. } => annotations,
            Stmt::LetDecl { annotations, .. } => annotations,
            Stmt::ConstDecl { annotations, .. } => annotations,
            Stmt::StructDecl { annotations, .. } => annotations,
            _ => continue,
        };
        for ann in annotations {
            if ann.name.to_lowercase() == "embedded" {
                let mut target = "arduino-uno".to_string();
                let mut baud = 115200;
                for arg in &ann.args {
                    let cleaned = arg.trim();
                    if cleaned.starts_with("target") && cleaned.contains('=') {
                        if let Some(val) = cleaned.split('=').nth(1) {
                            target = val.trim().trim_matches('"').trim_matches('\'').to_string();
                        }
                    } else if cleaned.starts_with("baud") && cleaned.contains('=') {
                        if let Some(val) = cleaned.split('=').nth(1) {
                            if let Ok(b) = val.trim().parse::<u32>() {
                                baud = b;
                            }
                        }
                    } else if !cleaned.contains('=') {
                        target = cleaned.trim().trim_matches('"').trim_matches('\'').to_string();
                    }
                }
                return Some((target, baud));
            }
        }
    }
    None
}

pub fn is_embedded_function(annotations: &[Annotation]) -> bool {
    annotations.iter().any(|a| a.name.to_lowercase() == "embedded")
}

pub fn generate_baremetal_firmware_project(
    ast: &[Stmt],
    target: &str,
    pkg_name: &str,
) -> Result<PathBuf, String> {
    println!("\x1b[1;36m   Transpiling\x1b[0m Flame AST to zero-cost bare-metal Rust (#![no_std])...");

    let cache_dir = Path::new(".flame").join("baremetal").join(target);
    fs::create_dir_all(cache_dir.join("src"))
        .map_err(|e| format!("Failed to build firmware cache directory: {}", e))?;

    let is_avr = target == "arduino-uno"
        || target == "avr-nano"
        || target == "atmega328p"
        || target == "mega";
    let is_esp = target == "esp32" || target == "esp8266";
    let _is_arm = target == "stm32" || target == "rp2040";

    let cargo_toml = if is_avr {
        let board_feature = match target {
            "mega" => "arduino-mega2560",
            "avr-nano" => "arduino-nano",
            _ => "arduino-uno",
        };
        format!(
            r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
panic-halt = "0.2"
ufmt = "0.2"

[dependencies.arduino-hal]
git = "https://github.com/Rahix/avr-hal"
features = ["{board_feature}"]

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
panic = "abort"
"#
        )
    } else if is_esp {
        format!(
            r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-hal = "0.17.0"
panic-halt = "0.2"

[profile.release]
lto = true
opt-level = "s"
"#
        )
    } else {
        format!(
            r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

[profile.release]
lto = true
opt-level = "s"
"#
        )
    };

    fs::write(cache_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Failed to write hardware Cargo.toml: {}", e))?;

    let mut code = String::new();
    code.push_str("#![no_std]\n#![no_main]\n\n");
    code.push_str("use panic_halt as _;\n");
    if is_avr {
        code.push_str("use arduino_hal::prelude::*;\n\n");
        code.push_str("#[arduino_hal::entry]\n");
        code.push_str("fn main() -> ! {\n");
        code.push_str("    let dp = arduino_hal::Peripherals::take().unwrap();\n");
        code.push_str("    let pins = arduino_hal::pins!(dp);\n");
    } else if is_esp {
        code.push_str("use esp_hal::prelude::*;\n\n");
        code.push_str("#[entry]\n");
        code.push_str("fn main() -> ! {\n");
        code.push_str("    let peripherals = esp_hal::init(esp_hal::Config::default());\n");
    } else {
        code.push_str("use cortex_m_rt::entry;\n\n");
        code.push_str("#[entry]\n");
        code.push_str("fn main() -> ! {\n");
    }

    let mut setup_code = String::new();
    let mut loop_code = String::new();
    let mut found_embedded_func = false;

    for stmt in ast {
        match stmt {
            Stmt::FuncDecl {
                name,
                body,
                annotations,
                ..
            } => {
                if is_embedded_function(annotations) {
                    found_embedded_func = true;
                    for s in body {
                        loop_code.push_str(&format!("        {}\n", transpile_stmt(s, target)));
                    }
                } else if name == "setup" || name == "init" {
                    for s in body {
                        setup_code.push_str(&format!("    {}\n", transpile_stmt(s, target)));
                    }
                } else if name == "loop" || name == "main" {
                    if !found_embedded_func {
                        for s in body {
                            loop_code.push_str(&format!("        {}\n", transpile_stmt(s, target)));
                        }
                    }
                }
            }
            Stmt::LetDecl { name, value, .. } | Stmt::ConstDecl { name, value, .. } => {
                let expr_str = transpile_expr(value, target, name);
                if expr_str.contains(".into_output()") || expr_str.contains(".into_input()") || expr_str.contains("pins.") {
                    setup_code.push_str(&format!("    let mut {} = {};\n", name, expr_str));
                } else {
                    setup_code.push_str(&format!("    let {} = {};\n", name, expr_str));
                }
            }
            Stmt::ExprStmt(e) => {
                let e_str = transpile_expr(e, target, "");
                if !e_str.is_empty() {
                    if !found_embedded_func && (e_str.contains("delay") || e_str.contains("high") || e_str.contains("low") || e_str.contains("toggle")) {
                        loop_code.push_str(&format!("        {};\n", e_str));
                    } else {
                        setup_code.push_str(&format!("    {};\n", e_str));
                    }
                }
            }
            Stmt::LoopStmt { body, .. } | Stmt::WhileStmt { body, .. } => {
                for s in body {
                    loop_code.push_str(&format!("        {}\n", transpile_stmt(s, target)));
                }
            }
            _ => {}
        }
    }

    code.push_str(&setup_code);
    code.push_str("\n    // Flame Real-Time Hardware Execution Loop (@Embedded / void loop)\n");
    code.push_str("    loop {\n");
    code.push_str(&loop_code);
    code.push_str("    }\n");
    code.push_str("}\n");

    fs::write(cache_dir.join("src").join("main.rs"), code)
        .map_err(|e| format!("Failed to write hardware main.rs: {}", e))?;

    Ok(cache_dir)
}

fn transpile_stmt(stmt: &Stmt, target: &str) -> String {
    match stmt {
        Stmt::LetDecl { name, value, .. } | Stmt::ConstDecl { name, value, .. } => {
            let expr_str = transpile_expr(value, target, name);
            format!("let mut {} = {};", name, expr_str)
        }
        Stmt::ExprStmt(e) => format!("{};", transpile_expr(e, target, "")),
        Stmt::IfStmt { cond, then_branch, else_branch, .. } => {
            let mut res = format!("if {} {{\n", transpile_expr(cond, target, ""));
            for s in then_branch {
                res.push_str(&format!("            {}\n", transpile_stmt(s, target)));
            }
            res.push('}');
            if let Some(els) = else_branch {
                res.push_str(" else {\n");
                for s in els {
                    res.push_str(&format!("            {}\n", transpile_stmt(s, target)));
                }
                res.push('}');
            }
            res
        }
        Stmt::LoopStmt { body, .. } | Stmt::WhileStmt { body, .. } => {
            let mut res = "loop {\n".to_string();
            for s in body {
                res.push_str(&format!("            {}\n", transpile_stmt(s, target)));
            }
            res.push('}');
            res
        }
        _ => String::new(),
    }
}

fn transpile_expr(expr: &Expr, target: &str, _target_var: &str) -> String {
    let is_avr = target == "arduino-uno" || target == "avr-nano" || target == "atmega328p" || target == "mega";

    match expr {
        Expr::Literal(val, _) => match val {
            LiteralValue::Int(i) => i.to_string(),
            LiteralValue::Float(f) => f.to_string(),
            LiteralValue::Bool(b) => b.to_string(),
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Nil => "0".to_string(),
        },
        Expr::Identifier(name, _) => name.clone(),
        Expr::Call(callee, args, _) => {
            let func_name = transpile_expr(callee, target, "");
            let mut arg_strs = Vec::new();
            for (_, arg) in args {
                arg_strs.push(transpile_expr(arg, target, ""));
            }

            if func_name == "embedded.pin" || func_name == "pin" || func_name == "std.embedded.pin" {
                let pin_num = arg_strs.get(0).cloned().unwrap_or_else(|| "13".to_string());
                if is_avr {
                    return format!("pins.d{}.into_output()", pin_num);
                } else {
                    return format!("gpio.gpio{}.into_push_pull_output()", pin_num);
                }
            } else if func_name == "sleep" || func_name == "delay" || func_name == "embedded.sleep" || func_name == "thread.sleep" {
                let mut delay_val = arg_strs.get(0).cloned().unwrap_or_else(|| "500".to_string());
                if delay_val.ends_with(".ms") || delay_val.contains(".ms") {
                    delay_val = delay_val.replace(".ms", "");
                }
                if is_avr {
                    return format!("arduino_hal::delay_ms({})", delay_val);
                } else {
                    return format!("delay_ms({})", delay_val);
                }
            } else if func_name == "print" || func_name == "println" {
                let msg = arg_strs.join(", ");
                return format!("// Serial Output: {}", msg);
            }

            if func_name.contains(".high") {
                return format!("_ = {}()", func_name);
            } else if func_name.contains(".low") {
                return format!("_ = {}()", func_name);
            } else if func_name.contains(".toggle") {
                return format!("_ = {}()", func_name);
            }

            format!("{}({})", func_name, arg_strs.join(", "))
        }
        Expr::Dot(obj, method, _) | Expr::SafeDot(obj, method, _) => {
            let obj_str = transpile_expr(obj, target, "");
            match method.as_str() {
                "high" => format!("{}.set_high", obj_str),
                "low" => format!("{}.set_low", obj_str),
                "toggle" => format!("{}.toggle", obj_str),
                "mode" => format!("{}.into_output", obj_str),
                "ms" => obj_str,
                _ => format!("{}.{}", obj_str, method),
            }
        }
        Expr::Binary(left, op, right, _) => {
            let l = transpile_expr(left, target, "");
            let r = transpile_expr(right, target, "");
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                _ => "+",
            };
            format!("{} {} {}", l, op_str, r)
        }
        Expr::Unary(op, operand, _) => {
            let op_str = match op {
                UnaryOp::Not => "!",
                UnaryOp::Neg => "-",
                _ => "",
            };
            format!("{}{}", op_str, transpile_expr(operand, target, ""))
        }
        Expr::Block(stmts, _) => {
            let mut blk = "{\n".to_string();
            for s in stmts {
                blk.push_str(&format!("            {}\n", transpile_stmt(s, target)));
            }
            blk.push('}');
            blk
        }
        _ => String::new(),
    }
}
