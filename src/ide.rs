use crate::{JsonCompletion, JsonHover};
use regex::Regex;

const KEYWORDS: &[(&str, &str)] = &[
    ("let", "Declares a local variable. Example: `let x = 5`"),
    ("const", "Declares a constant. Example: `const x = 5`"),
    ("mut", "Keyword for mutable reference/variable. Example: `let mut x = 5`"),
    ("fn", "Declares a function. Example: `fn do_something() {}`"),
    ("async", "Declares an asynchronous function. Example: `async fn do_something() {}`"),
    ("struct", "Declares a struct. Example: `struct Point { x: Num, y: Num }`"),
    ("enum", "Declares an enum. Example: `enum Color { Red, Green, Blue }`"),
    ("trait", "Declares a trait (interface). Example: `trait Drawable { fn draw(); }`"),
    ("impl", "Implements methods for a struct or enum. Example: `impl Point { fn new() -> Point {} }`"),
    ("if", "Conditional execution. Example: `if condition { ... }`"),
    ("else", "Alternative conditional execution. Example: `else { ... }`"),
    ("while", "Looping construct based on condition. Example: `while condition { ... }`"),
    ("loop", "Infinite looping construct. Example: `loop { ... }`"),
    ("for", "Iterating construct. Example: `for i in 0..10 { ... }`"),
    ("in", "Used in for-loops or containment checks. Example: `for x in list`"),
    ("match", "Pattern matching. Example: `match x { 1 => ..., _ => ... }`"),
    ("return", "Returns from a function. Example: `return 5`"),
    ("break", "Breaks out of a loop. Example: `break`"),
    ("continue", "Skips to the next iteration of a loop. Example: `continue`"),
    ("defer", "Defers execution until the end of the scope. Example: `defer file.close()`"),
    ("import", "Imports a module. Example: `import std.fs`"),
    ("native", "Native dependencies import prefix. Example: `import native.mysql`"),
    ("std", "Standard library import prefix. Example: `import std.math`"),
    ("thread", "Spawns a new background thread block. Example: `thread { thread.sleep(3000) }`"),
    ("await", "Waits for an asynchronous task or thread to complete. Example: `await task`"),
    ("print", "Prints to standard output. Example: `print(\"hello\")`"),
    ("eprint", "Prints to standard error. Example: `eprint(\"error\")`"),
    ("formula", "Declares a static map/dictionary structure. Example: `formula { key: \"value\" }`"),
    ("Formula", "Built-in Type: A map-like literal data structure."),
    ("Int", "Built-in Type: A 64-bit signed integer."),
    ("Float", "Built-in Type: A 64-bit floating point number."),
    ("String", "Built-in Type: A UTF-8 text string."),
    ("Bool", "Built-in Type: A boolean value (true or false)."),
    ("Nil", "Built-in Type: Represents the absence of a value."),
    ("Vec", "Built-in Type: A dynamically-sized array."),
    ("ThreadHandler", "Built-in Type: A handle to a spawned background thread."),
];

pub fn get_keyword_completions(prefix: &str) -> Vec<JsonCompletion> {
    KEYWORDS.iter()
        .filter(|(kw, _)| kw.starts_with(prefix) || prefix.is_empty())
        .map(|(kw, doc)| JsonCompletion {
            label: kw.to_string(),
            kind: "keyword".to_string(),
            detail: "keyword".to_string(),
            documentation: Some(doc.to_string()),
        })
        .collect()
}

pub fn get_keyword_hover(word: &str) -> Option<JsonHover> {
    KEYWORDS.iter()
        .find(|(kw, _)| *kw == word)
        .map(|(kw, doc)| {
            let mut formatted_doc = String::new();
            if let Some((desc, ex)) = doc.split_once("Example: `") {
                let clean_ex = ex.trim_end_matches('`');
                formatted_doc = format!("```flame\nkeyword {}\n```\n{}\n\n**Example:**\n```flame\n{}\n```", kw, desc.trim(), clean_ex);
            } else if doc.starts_with("Built-in Type:") {
                formatted_doc = format!("```flame\ntype {}\n```\n{}", kw, doc.trim());
            } else {
                formatted_doc = format!("```flame\nkeyword {}\n```\n{}", kw, doc.trim());
            }
            
            JsonHover {
                label: kw.to_string(),
                documentation: Some(formatted_doc),
            }
        })
}

#[derive(Debug)]
pub struct ScannedVar {
    pub name: String,
    pub typ: Option<String>,
}

#[derive(Debug)]
pub struct ScannedStruct {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: Vec<String>,
}

pub fn scan_document(content: &str) -> (Vec<ScannedVar>, Vec<ScannedStruct>) {
    let mut vars = Vec::new();
    let mut structs = Vec::new();

    // Scan for structs: `struct Name { field: type, ... }`
    let struct_re = Regex::new(r"(?s)struct\s+([a-zA-Z_]\w*)\s*\{([^}]*)\}").unwrap();
    for cap in struct_re.captures_iter(content) {
        let name = cap[1].to_string();
        let body = &cap[2];
        let mut fields = Vec::new();
        // naive split by comma or newline, finding `ident:`
        let field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:").unwrap();
        for field_cap in field_re.captures_iter(body) {
            fields.push(field_cap[1].to_string());
        }
        structs.push(ScannedStruct { name, fields, methods: vec![] });
    }

    // Scan for impls: `impl Name { fn method(...) }`
    let impl_re = Regex::new(r"(?s)impl\s+([a-zA-Z_]\w*)\s*\{([^}]*)\}").unwrap();
    let fn_re = Regex::new(r"fn\s+([a-zA-Z_]\w*)\s*\(").unwrap();
    for cap in impl_re.captures_iter(content) {
        let name = &cap[1];
        let body = &cap[2];
        if let Some(s) = structs.iter_mut().find(|s| s.name == name) {
            for fn_cap in fn_re.captures_iter(body) {
                s.methods.push(fn_cap[1].to_string());
            }
        }
    }

    // Scan for variables: `let x = ...`, `const x = StructName(...)`, `let mut x`
    let var_re = Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*(?:=\s*([a-zA-Z_]\w*))?").unwrap();
    for cap in var_re.captures_iter(content) {
        let name = cap[1].to_string();
        let typ = cap.get(2).map(|m| m.as_str().to_string());
        vars.push(ScannedVar { name, typ });
    }

    // Scan for variables with formula bodies to extract fields
    let formula_re = Regex::new(r"(?s)(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*=\s*formula\s*\{([^}]*)\}");
    if let Ok(formula_re) = formula_re {
        let field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:").unwrap();
        for cap in formula_re.captures_iter(content) {
            let name = cap[1].to_string();
            let body = &cap[2];
            let mut fields = Vec::new();
            for field_cap in field_re.captures_iter(body) {
                fields.push(field_cap[1].to_string());
            }
            let synthetic_type = format!("__formula_{}", name);
            structs.push(ScannedStruct {
                name: synthetic_type.clone(),
                fields,
                methods: vec![],
            });
            // Overwrite or add to vars at the beginning so it is found first
            vars.insert(0, ScannedVar {
                name,
                typ: Some(synthetic_type),
            });
        }
    }
    
    // Scan for function decls: `fn name(a: Type, b: Type)`
    let fn_decl_re = Regex::new(r"fn\s+([a-zA-Z_]\w*)\s*\(([^)]*)\)").unwrap();
    let arg_re = Regex::new(r"([a-zA-Z_]\w*)\s*:\s*([a-zA-Z_]\w*)").unwrap();
    for cap in fn_decl_re.captures_iter(content) {
        vars.push(ScannedVar {
            name: cap[1].to_string(),
            typ: Some("fn".to_string()),
        });
        
        let args_body = &cap[2];
        for arg_cap in arg_re.captures_iter(args_body) {
            vars.push(ScannedVar {
                name: arg_cap[1].to_string(),
                typ: Some(arg_cap[2].to_string()),
            });
        }
    }

    (vars, structs)
}

pub fn get_std_module_methods(module: &str) -> Option<Vec<String>> {
    let mut parts = module.split('.');
    let base = parts.next()?;
    
    let base_module = if base == "std" {
        parts.next()?
    } else {
        base
    };

    let mut map = match base_module {
        "thread" => Some(crate::native_std::thread::init()),
        "process" => Some(crate::native_std::process::init()),
        "fs" => Some(crate::native_std::fs::init()),
        "math" => Some(crate::native_std::math::init()),
        "time" => Some(crate::native_std::time::init()),
        "os" => Some(crate::native_std::os::init()),
        "hardware" => Some(crate::native_std::hardware::init()),
        "desktop" => Some(crate::native_std::desktop::init()),
        "env" => Some(crate::native_std::env::init()),
        "hid" => Some(crate::native_std::hid::init()),
        "camera" => Some(crate::native_std::camera::init()),
        "bluetooth" => Some(crate::native_std::bluetooth::init()),
        "serial" => Some(crate::native_std::serial::init()),
        _ => None,
    }?;

    for part in parts {
        match map.get(part) {
            Some(crate::vm::Value::Object(inner)) | Some(crate::vm::Value::Formula(inner)) => {
                map = inner.clone();
            }
            _ => return None,
        }
    }
    
    Some(map.keys().cloned().collect())
}
