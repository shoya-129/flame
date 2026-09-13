use regex::Regex;
pub use crate::utils::text::{extract_balanced_block, strip_comments_and_strings};

#[derive(Debug)]
pub struct ScannedVar {
    pub name: String,
    pub typ: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug)]
pub struct ScannedStruct {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub methods: Vec<String>,
}


pub fn scan_document(content: &str) -> (Vec<ScannedVar>, Vec<ScannedStruct>) {
    let stripped = strip_comments_and_strings(content);
    let content = &stripped;
    let mut vars = Vec::new();
    let mut structs = Vec::new();

    // Scan for structs: `struct Name { field: type, ... }`
    let struct_header_re = Regex::new(r"struct\s+([a-zA-Z_]\w*)\s*\{").unwrap();
    let field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:\s*([a-zA-Z_]\w*)").unwrap();
    for cap in struct_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        let mut fields = Vec::new();
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            for field_cap in field_re.captures_iter(body) {
                fields.push((field_cap[1].to_string(), field_cap[2].to_string()));
            }
        }
        structs.push(ScannedStruct {
            name,
            fields,
            methods: vec![],
        });
    }

    // Scan for impls: `impl Name { fn method(...) { ... } }`
    let impl_header_re = Regex::new(r"impl\s+([a-zA-Z_]\w*)\s*\{").unwrap();
    let fn_re = Regex::new(r"fn\s+([a-zA-Z_]\w*)\s*\(").unwrap();
    for cap in impl_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            let mut methods = Vec::new();
            for fn_cap in fn_re.captures_iter(body) {
                methods.push(fn_cap[1].to_string());
            }
            if let Some(s) = structs.iter_mut().find(|s| s.name == name) {
                s.methods.extend(methods);
            } else {
                structs.push(ScannedStruct {
                    name,
                    fields: vec![],
                    methods,
                });
            }
        }
    }

    // Scan for variables: `let x = StructName.new(...)`, `let x: StructName = ...`, `let x = StructName { ... }`
    let var_re =
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)(?:\s*:\s*([a-zA-Z_]\w*))?(?:\s*=\s*(?:(await)\s+)?(?:[a-zA-Z_]\w*\.)*([a-zA-Z_]\w*)(?:\.new|\s*\{|\s*\()?)?").unwrap();
    for cap in var_re.captures_iter(content) {
        let name = cap[1].to_string();
        let typ = cap.get(2).map(|m| m.as_str().to_string()).or_else(|| {
            if cap.get(3).is_some() {
                Some("Promise".to_string())
            } else {
                cap.get(4).map(|m| m.as_str().to_string())
            }
        });
        vars.push(ScannedVar {
            name,
            typ,
            doc: None,
        });
    }

    // Scan for tuple destructuring from channel: `let (tx, rx) = ...channel(...)`
    let channel_destructure_re = Regex::new(
        r"(?:let|const)\s*\(\s*([a-zA-Z_]\w*)\s*,\s*([a-zA-Z_]\w*)\s*\)\s*=\s*(?:[a-zA-Z_]\w*\.)*channel\s*\(",
    )
    .unwrap();
    for cap in channel_destructure_re.captures_iter(content) {
        vars.push(ScannedVar {
            name: cap[1].to_string(),
            typ: Some("Sender".to_string()),
            doc: Some("Thread message channel sender".to_string()),
        });
        vars.push(ScannedVar {
            name: cap[2].to_string(),
            typ: Some("Receiver".to_string()),
            doc: Some("Thread message channel receiver".to_string()),
        });
    }

    // Scan for cloned senders: `let tx2 = tx.clone()`
    let clone_sender_re =
        Regex::new(r"(?:let|const)\s+([a-zA-Z_]\w*)\s*=\s*([a-zA-Z_]\w*)\.clone\s*\(").unwrap();
    for cap in clone_sender_re.captures_iter(content) {
        let new_var = cap[1].to_string();
        let orig_var = &cap[2];
        if vars
            .iter()
            .any(|v| v.name == *orig_var && v.typ.as_deref() == Some("Sender"))
        {
            vars.push(ScannedVar {
                name: new_var,
                typ: Some("Sender".to_string()),
                doc: Some("Thread message channel sender (cloned)".to_string()),
            });
        }
    }

    // Scan for parameters in functions or closures (naive): `name: Type` where Type starts with uppercase
    let param_re = Regex::new(r"\b([a-z_]\w*)\s*:\s*([A-Z]\w*)").unwrap();
    for cap in param_re.captures_iter(content) {
        vars.push(ScannedVar {
            name: cap[1].to_string(),
            typ: Some(cap[2].to_string()),
            doc: None,
        });
    }

    // Scan for annotations to inject implicit variables
    let annotation_re = Regex::new(r"@([a-zA-Z_]\w*)\s*(?:\(|$)").unwrap();
    for cap in annotation_re.captures_iter(content) {
        let name = cap[1].to_string();
        let var_name = name.to_lowercase();
        // Set typ to a special marker so main.rs knows it's from an annotation
        vars.push(ScannedVar {
            name: var_name.clone(),
            typ: Some(format!("annotation_plugin:{}", var_name)),
            doc: None,
        });
    }

    // Scan for variables with formula bodies to extract fields
    let formula_header_re =
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*=\s*formula\s*\{").unwrap();
    for cap in formula_header_re.captures_iter(content) {
        let name = cap[1].to_string();
        let match_obj = cap.get(0).unwrap();
        let open_brace_pos = match_obj.end() - 1;
        if let Some(body) = extract_balanced_block(content, open_brace_pos) {
            let mut fields = Vec::new();
            let formula_field_re = Regex::new(r"([a-zA-Z_]\w*)\s*:").unwrap();
            for field_cap in formula_field_re.captures_iter(body) {
                fields.push((field_cap[1].to_string(), "Unknown".to_string()));
            }
            let synthetic_type = format!("__formula_{}", name);
            structs.push(ScannedStruct {
                name: synthetic_type.clone(),
                fields,
                methods: vec!["toString".to_string(), "toString".to_string()],
            });
            // Overwrite or add to vars at the beginning so it is found first
            vars.insert(
                0,
                ScannedVar {
                    name,
                    typ: Some(synthetic_type),
                    doc: None,
                },
            );
        }
    }

    // Scan for function and annotation decls: `fn name(a: Type, b: Type)` or `annotation name(...) -> Ret`
    let fn_decl_re = Regex::new(
        r"(?:export\s+)?(?:async\s+)?(fn|annotation)\s+([a-zA-Z_]\w*)\s*\(([^)]*)\)(?:\s*->\s*([a-zA-Z0-9_<>, \t]+))?",
    )
    .unwrap();

    let mut annotation_returns = std::collections::HashMap::new();
    for cap in fn_decl_re.captures_iter(content) {
        let kind_kw = &cap[1];
        let name_str = &cap[2];
        let params_str = cap[3].trim();
        let ret_str = cap.get(4).map_or("()", |m| m.as_str().trim());

        let sig = if kind_kw == "annotation" {
            annotation_returns.insert(name_str.to_string(), ret_str.to_string());
            if ret_str == "()" {
                format!("annotation @{}({})", name_str, params_str)
            } else {
                format!("annotation @{}({}) -> {}", name_str, params_str, ret_str)
            }
        } else {
            if ret_str == "()" {
                format!("fn {}({})", name_str, params_str)
            } else {
                format!("fn {}({}) -> {}", name_str, params_str, ret_str)
            }
        };

        vars.push(ScannedVar {
            name: name_str.to_string(),
            typ: Some(sig),
            doc: None,
        });
    }

    // Scan for annotation usages: `@Component` -> injects `component: ReturnType`
    let ann_usage_re = Regex::new(r"@([A-Z]\w*)").unwrap();
    for cap in ann_usage_re.captures_iter(content) {
        let ann_name = &cap[1];
        let mut c = ann_name.chars();
        let lower_name = match c.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        };

        let typ = annotation_returns
            .get(ann_name)
            .cloned()
            .unwrap_or_else(|| ann_name.to_string());

        vars.push(ScannedVar {
            name: lower_name,
            typ: Some(typ),
            doc: None,
        });
    }

    // Scan for imports: `import path as alias` or `import path`
    let import_re = Regex::new(
        r"import\s+([a-zA-Z_][\w]*(?:\.[a-zA-Z_][\w]*)*)(?:\s+as\s+([a-zA-Z_]\w*))?",
    )
    .unwrap();
    for cap in import_re.captures_iter(content) {
        let path = cap[1].to_string();
        let alias = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_else(|| {
            path.rsplit('.').next().unwrap_or(&path).to_string()
        });
        let doc = format!("```flame\nimport {}\n```\nImported module `{}`", path, path);
        vars.push(ScannedVar {
            name: alias,
            typ: Some(format!("import:{}", path)),
            doc: Some(doc),
        });
    }

    (vars, structs)
}

