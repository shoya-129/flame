#![allow(dead_code, unused_imports, unused_variables)]
use crate::lexer::{Lexer, Span};
use crate::parser::{
    BinaryOp, Expr, InterpolatedSegment, LiteralValue, Param, Parser, Stmt, UnaryOp,
};
use crate::vm::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};
use std::thread;
use super::core::Runner;

impl Runner {
    pub(crate) fn execute_statement(&mut self, stmt: &Stmt, env: Arc<Mutex<Env>>) -> Result<Value, String> {
        if !self.test_mode && crate::parser::is_test_statement(stmt) {
            return Ok(Value::Nil);
        }
        self.current_span = Some(stmt.span());
        match stmt {
            Stmt::LetDecl {
                name,
                is_mut,
                value,
                annotations,
                ..
            }
            | Stmt::ConstDecl {
                name,
                is_mut,
                value,
                annotations,
                ..
            } => {
                let val = self.eval_expr(value, env.clone())?;
                if let Expr::Identifier(src_name, _) = value {
                    env.lock().unwrap().move_var(src_name);
                }

                for anno in annotations {
                    let mut anno_func_opt = env.lock().unwrap().get(&anno.name);
                    if anno_func_opt.is_none() {
                        anno_func_opt = self.env.lock().unwrap().get(&anno.name);
                    }
                    if anno_func_opt.is_none() {
                        for (_, mod_env) in &self.modules {
                            if let Some(f) = mod_env.lock().unwrap().get(&anno.name) {
                                anno_func_opt = Some(f);
                                break;
                            }
                        }
                    }
                    if anno_func_opt.is_none() && anno.name.contains('.') {
                        let parts: Vec<&str> = anno.name.split('.').collect();
                        if let Some(mut current) = env.lock().unwrap().get(parts[0]) {
                            for part in &parts[1..] {
                                if let Value::Object(map) = &current {
                                    if let Some(next) = map.get(*part) {
                                        current = next.clone();
                                    } else {
                                        break;
                                    }
                                } else if let Value::Formula(map) = &current {
                                    if let Some(next) = map.get(*part) {
                                        current = next.clone();
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            anno_func_opt = Some(current);
                        }
                    }
                    if anno_func_opt.is_none() {
                        let env_lock = env.lock().unwrap();
                        for (_, val) in env_lock.variables.iter() {
                            if let Value::Object(map) = &val.value {
                                if let Some(exported_anno) = map.get(&anno.name) {
                                    anno_func_opt = Some(exported_anno.clone());
                                    break;
                                }
                            } else if let Value::Formula(map) = &val.value {
                                if let Some(exported_anno) = map.get(&anno.name) {
                                    anno_func_opt = Some(exported_anno.clone());
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(anno_func) = anno_func_opt {
                        let mut anno_args = Vec::new();
                        for arg_str in &anno.args {
                            let mut lexer = crate::lexer::Lexer::new(arg_str);
                            let mut tokens = Vec::new();
                            loop {
                                let tok = lexer.next_token();
                                let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                                tokens.push(tok);
                                if is_eof {
                                    break;
                                }
                            }
                            if tokens.len() >= 2
                                && tokens[0].kind == crate::lexer::TokenKind::Identifier
                                && (tokens[1].kind == crate::lexer::TokenKind::Colon
                                    || tokens[1].kind == crate::lexer::TokenKind::Equal)
                            {
                                tokens.remove(0);
                                tokens.remove(0);
                            }
                            let mut parser =
                                crate::parser::Parser::new(tokens, "anno_arg".to_string());
                            if let Ok(expr) = parser.parse_expr() {
                                if let Ok(arg_val) = self.eval_expr(&expr, env.clone()) {
                                    anno_args.push(arg_val);
                                }
                            }
                        }

                        let _ = self.invoke_callback_value(&anno_func, anno_args);
                    }
                }

                if name.starts_with('(') && name.ends_with(')') {
                    let trimmed = &name[1..name.len() - 1];
                    let vars: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
                    let items_opt = match val {
                        Value::Tuple(items) => Some(items.clone()),
                        _ => None,
                    };
                    if let Some(items) = items_opt {
                        for (i, var) in vars.iter().enumerate() {
                            let mut var_name = *var;
                            let mut extract_index = i;

                            if var.contains(':') {
                                let parts: Vec<&str> = var.split(':').collect();
                                var_name = parts[0].trim();
                                if let Ok(idx) = parts[1].trim().parse::<usize>() {
                                    extract_index = idx;
                                }
                            }

                            if env.lock().unwrap().variables.contains_key(var_name) {
                                return Err(format!(
                                    "cannot redeclare variable '{}' in the same scope",
                                    var_name
                                ));
                            }

                            if extract_index < items.len() {
                                env.lock().unwrap().define(
                                    var_name.to_string(),
                                    items[extract_index].clone(),
                                    *is_mut,
                                );
                            } else {
                                return Err(format!(
                                    "index {} out of bounds for tuple/vector destructuring",
                                    extract_index
                                ));
                            }
                        }
                    } else {
                        return Err(format!("cannot destructure a non-tuple/vector value"));
                    }
                } else if name.starts_with('{') && name.ends_with('}') {
                    let trimmed = &name[1..name.len() - 1];
                    let vars: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
                    match val {
                        Value::Formula(map)
                        | Value::Object(map)
                        | Value::StructInstance { fields: map, .. } => {
                            for var in vars {
                                if env.lock().unwrap().variables.contains_key(var) {
                                    return Err(format!(
                                        "cannot redeclare variable '{}' in the same scope",
                                        var
                                    ));
                                }
                                if let Some(field_val) = map.get(var) {
                                    env.lock().unwrap().define(
                                        var.to_string(),
                                        field_val.clone(),
                                        *is_mut,
                                    );
                                } else {
                                    return Err(format!(
                                        "field '{}' not found in object destructuring",
                                        var
                                    ));
                                }
                            }
                        }
                        _ => return Err(format!("cannot destructure a non-object value")),
                    }
                } else {
                    if env.lock().unwrap().variables.contains_key(name) {
                        return Err(format!(
                            "cannot redeclare variable '{}' in the same scope",
                            name
                        ));
                    }
                    env.lock().unwrap().define(name.clone(), val, *is_mut);
                }
                Ok(Value::Nil)
            }
            Stmt::FuncDecl {
                name,
                params,
                body,
                annotations,
                ..
            } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone().unwrap_or_default(),
                    env: env.clone(),
                    annotations: annotations.clone(),
                };
                env.lock().unwrap().define(name.clone(), func, false);
                Ok(Value::Nil)
            }
            Stmt::AnnotationDecl {
                name, params, body, ..
            } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                    annotations: vec![],
                };
                env.lock().unwrap().define(name.clone(), func, false);
                Ok(Value::Nil)
            }
            Stmt::StructDecl { name, fields, .. } => {
                let func = Value::StructConstructor {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                env.lock().unwrap().define(name.clone(), func, false);
                Ok(Value::Nil)
            }
            Stmt::EnumDecl { name, variants, .. } => {
                env.lock().unwrap().define(
                    name.clone(),
                    Value::EnumMeta(name.clone(), variants.clone()),
                    false,
                );
                Ok(Value::Nil)
            }
            Stmt::ImplDecl {
                target_type,
                methods,
                ..
            } => {
                let impl_env = self
                    .modules
                    .get(&format!("impl_{}", target_type))
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Mutex::new(Env::new_child(env.clone()))));
                for method in methods {
                    self.execute_statement(method, impl_env.clone())?;
                }
                self.modules
                    .insert(format!("impl_{}", target_type), impl_env);
                Ok(Value::Nil)
            }
            Stmt::ImportDecl { path, alias, .. } => {
                let mod_name = path.join(".");
                let bind_name = alias.clone().unwrap_or_else(|| path.last().unwrap().clone());
                if mod_name.starts_with("std.") {
                    let mut stdlib_dir = None;
                    let mut current =
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    for _ in 0..5 {
                        let check = current.join("flame-stdlib");
                        if check.exists() {
                            stdlib_dir = Some(check);
                            break;
                        }
                        if let Some(parent) = current.parent() {
                            current = parent.to_path_buf();
                        } else {
                            break;
                        }
                    }

                    let std_file = if let Some(ref dir) = stdlib_dir {
                        let mut f = dir.clone();
                        for part in path {
                            f = f.join(part);
                        }
                        f = f.with_extension("flame");
                        if f.exists() { Some(f) } else { None }
                    } else {
                        None
                    };

                    if let Some(file_path) = std_file {
                        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
                        let mut lexer = Lexer::new(&content);
                        let mut tokens = Vec::new();
                        loop {
                            let tok = lexer.next_token();
                            let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                            tokens.push(tok);
                            if is_eof {
                                break;
                            }
                        }
                        let mut parser =
                            Parser::new(tokens, file_path.to_string_lossy().to_string());
                        let parsed_stmts = parser.parse().map_err(|e| e.message)?;

                        let mod_env = Arc::new(Mutex::new(Env::new()));
                        crate::stdlib::register_global_builtins(mod_env.clone());
                        let mut runner = Runner::new(file_path.clone());
                        runner.env = mod_env.clone();
                        runner.native_methods = self.native_methods.clone();
                        for s in &parsed_stmts {
                            runner.execute_statement(s, mod_env.clone())?;
                        }

                        for (k, v) in runner.modules {
                            self.modules.insert(k, v);
                        }

                        let mut map = mod_env.lock().unwrap().to_formula_map();
                        map.insert("__module__".to_string(), Value::Bool(true));
                        env.lock().unwrap().define(
                            bind_name.clone(),
                            Value::Formula(map),
                            false,
                        );
                        if let Some(al) = alias {
                            self.modules.insert(al.clone(), mod_env.clone());
                        }
                        self.modules.insert(mod_name, mod_env);
                    } else {
                        let mod_env = Arc::new(Mutex::new(Env::new()));
                        crate::stdlib::register_std_module(&mod_name, mod_env.clone());
                        let mut map = mod_env.lock().unwrap().to_formula_map();
                        map.insert("__module__".to_string(), Value::Bool(true));
                        env.lock().unwrap().define(
                            bind_name.clone(),
                            Value::Formula(map),
                            false,
                        );
                        if let Some(al) = alias {
                            self.modules.insert(al.clone(), mod_env.clone());
                        }
                        self.modules.insert(mod_name, mod_env);
                    }
                } else if mod_name.starts_with("native.")
                    || (Path::new(&format!(".flame/pkg/{}", path.last().unwrap())).exists()
                        && !Path::new(&format!(".flame/pkg/{}/src/main.fm", path.last().unwrap()))
                            .exists())
                {
                    let mod_env = Arc::new(Mutex::new(Env::new()));
                    let raw_mod_name = path.last().unwrap();
                    let rel_meta = format!(".flame/pkg/{}/{}.fmi", raw_mod_name, raw_mod_name);
                    let meta_candidates = vec![
                        PathBuf::from(&rel_meta),
                        self.resolve_path(&rel_meta),
                        self.filepath
                            .parent()
                            .unwrap_or(Path::new("."))
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(&rel_meta),
                    ];
                    let mut meta_str = None;
                    for c in meta_candidates {
                        if let Ok(content) = self.read_file_or_vfs(&c) {
                            meta_str = Some(content);
                            break;
                        }
                    }

                    if let Some(meta_str) = meta_str {
                        match serde_json::from_str::<crate::package_manager::FlameMeta>(&meta_str) {
                            Ok(meta) => {
                                if meta.kind == "native" {
                                    mod_env.lock().unwrap().define(
                                        "__crate__".to_string(),
                                        Value::String(raw_mod_name.clone()),
                                        false,
                                    );
                                    for fn_meta in &meta.functions {
                                        mod_env.lock().unwrap().define(
                                            format!("__{}_return_type__", fn_meta.flame_name),
                                            Value::String(fn_meta.return_type.clone()),
                                            false,
                                        );
                                        mod_env.lock().unwrap().define(
                                            fn_meta.flame_name.clone(),
                                            Value::Function {
                                                params: fn_meta
                                                    .params
                                                    .iter()
                                                    .map(|p| Param {
                                                        name: p.name.clone(),
                                                        type_name: p.type_name.clone(),
                                                        default_val: None,
                                                        is_ref: false,
                                                        is_mut: false,
                                                    })
                                                    .collect(),
                                                body: vec![],
                                                env: mod_env.clone(),
                                                annotations: vec![],
                                            },
                                            false,
                                        );
                                    }
                                    for struct_meta in &meta.structs {
                                        let mut struct_map = HashMap::new();
                                        struct_map.insert(
                                            "__crate__".to_string(),
                                            Value::String(raw_mod_name.clone()),
                                        );
                                        struct_map.insert(
                                            "__type__".to_string(),
                                            Value::String(struct_meta.name.clone()),
                                        );
                                        for method in &struct_meta.methods {
                                            struct_map.insert(
                                                format!(
                                                    "__{}_{}_return_type__",
                                                    struct_meta.name, method.flame_name
                                                ),
                                                Value::String(method.return_type.clone()),
                                            );
                                            struct_map.insert(
                                                method.flame_name.clone(),
                                                Value::Function {
                                                    params: method
                                                        .params
                                                        .iter()
                                                        .map(|p| Param {
                                                            name: p.name.clone(),
                                                            type_name: p.type_name.clone(),
                                                            default_val: None,
                                                            is_ref: false,
                                                            is_mut: false,
                                                        })
                                                        .collect(),
                                                    body: vec![],
                                                    env: mod_env.clone(),
                                                    annotations: vec![],
                                                },
                                            );
                                        }
                                        mod_env.lock().unwrap().define(
                                            struct_meta.name.clone(),
                                            Value::Formula(struct_map),
                                            false,
                                        );

                                        if struct_meta.name.to_lowercase()
                                            == raw_mod_name.to_lowercase()
                                        {
                                            for method in &struct_meta.methods {
                                                mod_env.lock().unwrap().define(
                                                    method.flame_name.clone(),
                                                    Value::Function {
                                                        params: method
                                                            .params
                                                            .iter()
                                                            .map(|p| Param {
                                                                name: p.name.clone(),
                                                                type_name: p.type_name.clone(),
                                                                default_val: None,
                                                                is_ref: false,
                                                                is_mut: false,
                                                            })
                                                            .collect(),
                                                        body: vec![],
                                                        env: mod_env.clone(),
                                                        annotations: vec![],
                                                    },
                                                    false,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Native package metadata '{}' invalid: {}",
                                    rel_meta, e
                                ));
                            }
                        }
                    } else {
                        // In BlazeVM (or when metadata is missing), we gracefully fallback
                        // to an empty formula module. Native method bindings are statically
                        // resolved through the `native_methods` map on method invocation.
                        mod_env.lock().unwrap().define(
                            "__crate__".to_string(),
                            Value::String(raw_mod_name.to_string()),
                            false,
                        );
                    }

                    // Fallback registrations for known native modules (e.g. native.bridge)
                    crate::stdlib::register_std_module(&mod_name, mod_env.clone());

                    // Direct Rust interop: auto-scan for matching .rs file in workspace
                    let rs_candidates = vec![
                        self.resolve_path(&format!("{}.rs", raw_mod_name)),
                        self.resolve_path(&format!("src/{}.rs", raw_mod_name)),
                        self.resolve_path(&format!("native/{}.rs", raw_mod_name)),
                    ];
                    for candidate in rs_candidates {
                        if candidate.exists() {
                            if let Ok(rs_code) = fs::read_to_string(&candidate) {
                                self.load_rust_file_methods(&rs_code, mod_env.clone());
                            }
                            break;
                        }
                    }

                    let mut map = mod_env.lock().unwrap().to_formula_map();
                    map.insert("__module__".to_string(), Value::Bool(true));
                    env.lock().unwrap().define(
                        bind_name.clone(),
                        Value::Formula(map),
                        false,
                    );
                    if let Some(al) = alias {
                        self.modules.insert(al.clone(), mod_env.clone());
                    }
                    self.modules.insert(mod_name, mod_env);
                } else {
                    let mut files_to_run = Vec::new();
                    let mut error_msg = String::new();
                    let mut target_is_dir = false;

                    let mut target_path = None;
                    let pkg_main = self
                        .resolve_path(&format!(".flame/pkg/{}/src/main.fm", path.last().unwrap()));
                    let f_fm = self.resolve_path(&format!("{}.fm", path.join("/")));
                    let _f_flame = self.resolve_path(&format!("{}.flame", path.join("/")));

                    if self.vfs.is_some() {
                        let vfs = self.vfs.as_ref().unwrap();
                        let pkg_main_str =
                            format!(".flame/pkg/{}/src/main.fm", path.last().unwrap());
                        let f_fm_str = format!("src/{}.fm", path.join("/"));
                        let f_flame_str = format!("src/{}.flame", path.join("/"));
                        let dir_prefix = format!("src/{}/", path.join("/"));

                        if vfs.contains_key(&pkg_main_str) {
                            target_path = Some(PathBuf::from(pkg_main_str));
                        } else if vfs.contains_key(&f_fm_str) {
                            target_path = Some(PathBuf::from(f_fm_str));
                        } else if vfs.contains_key(&f_flame_str) {
                            target_path = Some(PathBuf::from(f_flame_str));
                        } else {
                            // Check if it's a directory in VFS
                            let mut has_dir = false;
                            for k in vfs.keys() {
                                if k.starts_with(&dir_prefix) {
                                    has_dir = true;
                                    break;
                                }
                            }
                            if has_dir {
                                target_path = Some(PathBuf::from(dir_prefix.trim_end_matches('/')));
                                target_is_dir = true;
                            }
                        }

                        if let Some(f) = target_path {
                            if target_is_dir {
                                let prefix = format!("{}/", f.to_string_lossy().replace("\\", "/"));
                                for (k, content) in vfs {
                                    if k.starts_with(&prefix) && k.ends_with(".fm") {
                                        files_to_run.push((PathBuf::from(k), content.clone()));
                                    }
                                }
                            } else {
                                let k = f.to_string_lossy().replace("\\", "/");
                                if let Some(content) = vfs.get(&k) {
                                    files_to_run.push((f.clone(), content.clone()));
                                }
                            }
                            if files_to_run.is_empty() {
                                error_msg = format!(
                                    "Module '{}' found in VFS but contains no readable .fm files",
                                    mod_name
                                );
                            }
                        } else {
                            error_msg = format!("Module '{}' not found in VFS", mod_name);
                        }
                    } else {
                        // Physical filesystem fallback
                        if let Some(f) = crate::stdlib::locate_import_file(&self.filepath, path) {
                            target_path = Some(f);
                        } else {
                            if self.read_file_or_vfs(&pkg_main).is_ok() {
                                target_path = Some(pkg_main);
                            } else if self.read_file_or_vfs(&f_fm).is_ok() {
                                target_path = Some(f_fm);
                            }
                        }

                        if let Some(f) = target_path {
                            target_is_dir = f.is_dir();
                            if target_is_dir {
                                if let Ok(entries) = std::fs::read_dir(&f) {
                                    for entry in entries.flatten() {
                                        let path = entry.path();
                                        if path.is_file()
                                            && path.extension().and_then(|s| s.to_str())
                                                == Some("fm")
                                        {
                                            if let Ok(c) = self.read_file_or_vfs(&path) {
                                                files_to_run.push((path, c));
                                            }
                                        }
                                    }
                                }
                            } else {
                                if let Ok(c) = self.read_file_or_vfs(&f) {
                                    files_to_run.push((f.clone(), c));
                                }
                            }
                            if files_to_run.is_empty() {
                                error_msg = format!(
                                    "Module '{}' found at {:?} but contains no readable .fm files",
                                    mod_name, f
                                );
                            }
                        } else {
                            error_msg = format!("Module '{}' not found", mod_name);
                        }
                    }

                    if !files_to_run.is_empty() {
                        let expected_pkg = path.last().unwrap();

                        let mod_env = Arc::new(Mutex::new(Env::new()));
                        crate::stdlib::register_global_builtins(mod_env.clone());
                        let mut all_modules = std::collections::HashMap::new();

                        for (local_file, content) in files_to_run {
                            let mut lexer = Lexer::new(&content);
                            let mut tokens = Vec::new();
                            loop {
                                let tok = lexer.next_token();
                                let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                                tokens.push(tok);
                                if is_eof {
                                    break;
                                }
                            }
                            let mut parser =
                                Parser::new(tokens, local_file.to_string_lossy().to_string());
                            let parsed_stmts = parser.parse().map_err(|e| e.message)?;

                            if target_is_dir {
                                let mut found_pkg = None;
                                for s in &parsed_stmts {
                                    if let Stmt::PackageDecl { name, .. } = s {
                                        found_pkg = Some(name.clone());
                                        break;
                                    }
                                }
                                match found_pkg {
                                    Some(name) if &name == expected_pkg => {}
                                    Some(name) => {
                                        return Err(format!(
                                            "File {} declared package '{}', but expected '{}'",
                                            local_file.display(),
                                            name,
                                            expected_pkg
                                        ));
                                    }
                                    None => {
                                        return Err(format!(
                                            "File {} in a folder import must declare 'package {}'",
                                            local_file.display(),
                                            expected_pkg
                                        ));
                                    }
                                }
                            }

                            let mut runner = Runner::new(local_file.clone());
                            runner.env = mod_env.clone();
                            runner.native_methods = self.native_methods.clone();
                            for s in &parsed_stmts {
                                runner.execute_statement(s, mod_env.clone())?;
                                if let Stmt::ExportDecl(inner, _) = s {
                                    match inner.as_ref() {
                                        Stmt::FuncDecl { name, .. }
                                        | Stmt::AnnotationDecl { name, .. }
                                        | Stmt::LetDecl { name, .. }
                                        | Stmt::ConstDecl { name, .. }
                                        | Stmt::StructDecl { name, .. }
                                        | Stmt::EnumDecl { name, .. } => {
                                            if let Some(val) = mod_env.lock().unwrap().get(name) {
                                                env.lock().unwrap().define(
                                                    name.clone(),
                                                    val,
                                                    false,
                                                );
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            for (k, v) in runner.modules {
                                all_modules.insert(k, v);
                            }
                        }

                        for (k, v) in all_modules {
                            self.modules.insert(k, v);
                        }

                        let mut map = mod_env.lock().unwrap().to_formula_map();
                        map.insert("__module__".to_string(), Value::Bool(true));
                        env.lock().unwrap().define(
                            bind_name.clone(),
                            Value::Formula(map),
                            false,
                        );
                        if let Some(al) = alias {
                            self.modules.insert(al.clone(), mod_env.clone());
                        }
                        self.modules.insert(mod_name, mod_env);
                    } else {
                        return Err(error_msg);
                    }
                }
                Ok(Value::Nil)
            }
            Stmt::ExportDecl(inner, _) => {
                self.execute_statement(inner, env)?;
                Ok(Value::Nil)
            }
            Stmt::PluginDecl { name, .. } => {
                self.execute_plugin(name, env)?;
                Ok(Value::Nil)
            }
            Stmt::ExprStmt(expr) => {
                let val = self.eval_expr(expr, env)?;
                Ok(val)
            }
            Stmt::IfStmt {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_val = self.eval_expr(cond, env.clone())?;
                if let Value::Bool(true) = cond_val {
                    let child = Arc::new(Mutex::new(Env::new_child(env)));
                    for s in then_branch {
                        let res = self.execute_statement(s, child.clone())?;
                        if matches!(res, Value::Break) {
                            return Ok(Value::Break);
                        }
                        if matches!(res, Value::Return(_)) {
                            return Ok(res);
                        }
                    }
                } else if let Some(el) = else_branch {
                    let child = Arc::new(Mutex::new(Env::new_child(env)));
                    for s in el {
                        let res = self.execute_statement(s, child.clone())?;
                        if matches!(res, Value::Break) {
                            return Ok(Value::Break);
                        }
                        if matches!(res, Value::Return(_)) {
                            return Ok(res);
                        }
                    }
                }
                Ok(Value::Nil)
            }
            Stmt::WhileStmt { cond, body, .. } => {
                loop {
                    let cond_val = self.eval_expr(cond, env.clone())?;
                    if !matches!(cond_val, Value::Bool(true)) {
                        break;
                    }
                    let child = Arc::new(Mutex::new(Env::new_child(env.clone())));
                    let mut hit_break = false;
                    for s in body {
                        let res = self.execute_statement(s, child.clone())?;
                        if matches!(res, Value::Return(_)) {
                            return Ok(res);
                        }
                        if matches!(res, Value::Break) {
                            hit_break = true;
                            break;
                        }
                    }
                    if hit_break {
                        break;
                    }
                }
                Ok(Value::Nil)
            }
            Stmt::LoopStmt { body, .. } => {
                loop {
                    let child = Arc::new(Mutex::new(Env::new_child(env.clone())));
                    let mut hit_break = false;
                    for s in body {
                        let res = self.execute_statement(s, child.clone())?;
                        if matches!(res, Value::Return(_)) {
                            return Ok(res);
                        }
                        if matches!(res, Value::Break) {
                            hit_break = true;
                            break;
                        }
                    }
                    if hit_break {
                        break;
                    }
                }
                Ok(Value::Nil)
            }
            Stmt::MatchStmt { target, arms, .. } => {
                let target_val = self.eval_expr(target, env.clone())?;

                for arm in arms {
                    let mut is_match = false;
                    let mut child_env_opt = None;

                    for pattern in &arm.patterns {
                        if pattern == "_" {
                            is_match = true;
                            break;
                        }

                        match &target_val {
                            Value::EnumValue(enum_name, variant_name, data) => {
                                let dot_pat = format!("{}.{}", enum_name, variant_name);
                                let colon_pat = format!("{}::{}", enum_name, variant_name);

                                if dot_pat == *pattern
                                    || colon_pat == *pattern
                                    || variant_name == pattern
                                    || pattern.ends_with(&format!(".{}", dot_pat))
                                    || pattern.ends_with(&format!("::{}", colon_pat))
                                {
                                    is_match = true;
                                    let child = Arc::new(Mutex::new(Env::new_child(env.clone())));

                                    match data {
                                        EnumData::Tuple(vals) => {
                                            for (i, field) in arm.destructure.iter().enumerate() {
                                                let field_val =
                                                    vals.get(i).cloned().unwrap_or(Value::Nil);
                                                child.lock().unwrap().define(
                                                    field.clone(),
                                                    field_val,
                                                    false,
                                                );
                                            }
                                        }
                                        EnumData::Struct(map) => {
                                            for field in &arm.destructure {
                                                let field_val =
                                                    map.get(field).cloned().unwrap_or(Value::Nil);
                                                child.lock().unwrap().define(
                                                    field.clone(),
                                                    field_val,
                                                    false,
                                                );
                                            }
                                        }
                                        EnumData::Unit => {}
                                    }
                                    child_env_opt = Some(child);
                                }
                            }
                            Value::Object(map) | Value::Formula(map) => {
                                if let Some(Value::String(variant)) = map.get("$variant") {
                                    if variant == pattern {
                                        is_match = true;
                                        let child =
                                            Arc::new(Mutex::new(Env::new_child(env.clone())));
                                        for field in &arm.destructure {
                                            let field_val =
                                                map.get(field).cloned().unwrap_or(Value::Nil);
                                            child.lock().unwrap().define(
                                                field.clone(),
                                                field_val,
                                                false,
                                            );
                                        }
                                        child_env_opt = Some(child);
                                    }
                                }
                            }
                            Value::String(s) => {
                                if s == pattern {
                                    is_match = true;
                                }
                            }
                            v => {
                                if v.to_string() == *pattern {
                                    is_match = true;
                                }
                            }
                        }

                        if is_match {
                            break;
                        }
                    }

                    if is_match {
                        let exec_env = child_env_opt.unwrap_or_else(|| env.clone());
                        let mut guard_passed = true;

                        if let Some(guard_expr) = &arm.guard {
                            let guard_val = self.eval_expr(guard_expr, exec_env.clone())?;
                            if !guard_val.is_truthy() {
                                guard_passed = false;
                            }
                        }

                        if guard_passed {
                            let res = self.eval_expr(&arm.body, exec_env)?;
                            return Ok(res);
                        }
                    }
                }
                Ok(Value::Nil)
            }
            Stmt::ReturnStmt(expr_opt, _) => {
                if let Some(expr) = expr_opt {
                    let val = self.eval_expr(expr, env)?;
                    Ok(Value::Return(Box::new(val)))
                } else {
                    Ok(Value::Return(Box::new(Value::Nil)))
                }
            }
            Stmt::Break(_) => Ok(Value::Break),
            Stmt::ForStmt {
                var_name,
                iterable,
                body,
                ..
            } => {
                let iter_val = self.eval_expr(iterable, env.clone())?;

                match iter_val {
                    Value::Tuple(items) => {
                        for it in items {
                            let child = Arc::new(Mutex::new(Env::new_child(env.clone())));
                            child.lock().unwrap().define(var_name.clone(), it, false);

                            for s in body {
                                let res = self.execute_statement(s, child.clone())?;
                                if matches!(res, Value::Return(_)) {
                                    return Ok(res);
                                }
                                if matches!(res, Value::Break) {
                                    return Ok(Value::Nil);
                                }
                            }
                        }
                    }

                    Value::Int(limit) => {
                        for i in 0..limit {
                            let child = Arc::new(Mutex::new(Env::new_child(env.clone())));
                            child
                                .lock()
                                .unwrap()
                                .define(var_name.clone(), Value::Int(i), false);

                            for s in body {
                                let res = self.execute_statement(s, child.clone())?;
                                if matches!(res, Value::Return(_)) {
                                    return Ok(res);
                                }
                                if matches!(res, Value::Break) {
                                    return Ok(Value::Nil);
                                }
                            }
                        }
                    }

                    Value::Range(start, end) => {
                        for i in start..end {
                            let child = Arc::new(Mutex::new(Env::new_child(env.clone())));
                            child
                                .lock()
                                .unwrap()
                                .define(var_name.clone(), Value::Int(i), false);

                            for s in body {
                                let res = self.execute_statement(s, child.clone())?;
                                if matches!(res, Value::Return(_)) {
                                    return Ok(res);
                                }
                                if matches!(res, Value::Break) {
                                    return Ok(Value::Nil);
                                }
                            }
                        }
                    }

                    Value::Formula(map) | Value::Object(map) => {
                        // Check for 'accept' or 'next' method
                        let method = map.get("accept").or_else(|| map.get("next"));
                        if let Some(m) = method {
                            loop {
                                let item_res = self.invoke_callback_value(m, vec![]);
                                match item_res {
                                    Ok(val) => {
                                        if matches!(val, Value::Nil) {
                                            break; // End of iteration
                                        }
                                        let child =
                                            Arc::new(Mutex::new(Env::new_child(env.clone())));
                                        child.lock().unwrap().define(var_name.clone(), val, false);

                                        for s in body {
                                            let res = self.execute_statement(s, child.clone())?;
                                            if matches!(res, Value::Return(_)) {
                                                return Ok(res);
                                            }
                                            if matches!(res, Value::Break) {
                                                return Ok(Value::Nil); // Only break out of for loop, not function
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        break; // Error iterating, assume EOF/End
                                    }
                                }
                            }
                        }
                    }

                    _ => {}
                }

                Ok(Value::Nil)
            }

            _ => Ok(Value::Nil),
        }
    }


}
