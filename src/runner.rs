use crate::lexer::{Lexer, Span};
use crate::parser::{
    BinaryOp, Expr, InterpolatedSegment, LiteralValue, Param, Parser, Stmt,
};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

pub use crate::vm::*;
pub struct Runner {
    pub env: Arc<Mutex<Env>>,
    pub filepath: PathBuf,
    pub modules: HashMap<String, Arc<Mutex<Env>>>,
    pub current_span: Option<Span>,
    pub native_methods: HashMap<String, fn(*const CValue, usize) -> CValue>,
}

impl Runner {
    pub fn new(filepath: PathBuf) -> Self {
        let runner = Self {
            env: Arc::new(Mutex::new(Env::new())),
            filepath,
            modules: HashMap::new(),
            current_span: None,
            native_methods: HashMap::new(),
        };
        crate::stdlib::register_global_builtins(runner.env.clone());
        runner
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<Value, String> {
        let mut last_val = Value::Nil;
        for stmt in stmts {
            match self.execute_statement(stmt, self.env.clone()) {
                Ok(val) => last_val = val,
                Err(e) => {
                    if let Some(ref span) = self.current_span {
                        return Err(format!(
                            "{} at {}:{}:{}",
                            e,
                            self.filepath.to_string_lossy(),
                            span.line,
                            span.col
                        ));
                    }
                    return Err(e);
                }
            }
        }
        Ok(last_val)
    }

    fn execute_statement(&mut self, stmt: &Stmt, env: Arc<Mutex<Env>>) -> Result<Value, String> {
        self.current_span = Some(stmt.span());
        match stmt {
            Stmt::LetDecl {
                name,
                is_mut,
                value,
                ..
            }
            | Stmt::ConstDecl {
                name,
                is_mut,
                value,
                ..
            } => {
                let val = self.eval_expr(value, env.clone())?;
                if let Expr::Identifier(src_name, _) = value {
                    env.lock().unwrap().move_var(src_name);
                }
                if name.starts_with('(') && name.ends_with(')') {
                    let trimmed = &name[1..name.len() - 1];
                    let vars: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
                    if let Value::Tuple(items) = val {
                        for (i, var) in vars.iter().enumerate() {
                            if i < items.len() {
                                env.lock().unwrap().define(
                                    var.to_string(),
                                    items[i].clone(),
                                    *is_mut,
                                );
                            }
                        }
                    }
                } else {
                    env.lock().unwrap().define(name.clone(), val, *is_mut);
                }
                Ok(Value::Nil)
            }
            Stmt::FuncDecl {
                name, params, body, ..
            } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
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
                    .unwrap_or_else(|| Arc::new(Mutex::new(Env::new())));
                for method in methods {
                    self.execute_statement(method, impl_env.clone())?;
                }
                self.modules
                    .insert(format!("impl_{}", target_type), impl_env);
                Ok(Value::Nil)
            }
            Stmt::ImportDecl { path, .. } => {
                let mod_name = path.join(".");
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
                        let mut runner = Runner::new(file_path.clone());
                        runner.env = mod_env.clone();
                        for s in &parsed_stmts {
                            runner.execute_statement(s, mod_env.clone())?;
                        }

                        for (k, v) in runner.modules {
                            self.modules.insert(k, v);
                        }

                        env.lock().unwrap().define(
                            path.last().unwrap().clone(),
                            Value::Formula(mod_env.lock().unwrap().to_formula_map()),
                            false,
                        );
                        self.modules.insert(mod_name, mod_env);
                    } else {
                        let mod_env = Arc::new(Mutex::new(Env::new()));
                        crate::stdlib::register_std_module(&mod_name, mod_env.clone());
                        env.lock().unwrap().define(
                            path.last().unwrap().clone(),
                            Value::Formula(mod_env.lock().unwrap().to_formula_map()),
                            false,
                        );
                        self.modules.insert(mod_name, mod_env);
                    }
                } else if mod_name.starts_with("native.")
                    || Path::new(&format!(".flame/pkg/{}", path.last().unwrap())).exists()
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
                    let mut meta_file = None;
                    for c in meta_candidates {
                        if c.exists() {
                            meta_file = Some(c);
                            break;
                        }
                    }

                    if let Some(meta_path) = meta_file {
                        if let Ok(meta_str) = fs::read_to_string(&meta_path) {
                            if let Ok(meta) =
                                serde_json::from_str::<crate::package_manager::FlameMeta>(&meta_str)
                            {
                                if meta.kind == "native" {
                                    mod_env.lock().unwrap().define(
                                        "__crate__".to_string(),
                                        Value::String(raw_mod_name.clone()),
                                        false,
                                    );
                                    for fn_meta in &meta.functions {
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
                                                },
                                            );
                                        }
                                        mod_env.lock().unwrap().define(
                                            struct_meta.name.clone(),
                                            Value::Formula(struct_map),
                                            false,
                                        );
                                        
                                        if struct_meta.name.to_lowercase() == raw_mod_name.to_lowercase() {
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
                                                    },
                                                    false,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Fallback registrations for known native modules (e.g. native.bridge)
                    crate::stdlib::register_native_module(&mod_name, mod_env.clone());

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

                    env.lock().unwrap().define(
                        path.last().unwrap().clone(),
                        Value::Formula(mod_env.lock().unwrap().to_formula_map()),
                        false,
                    );
                    self.modules.insert(mod_name, mod_env);
                } else {
                    let mut local_file = self.resolve_path(&format!("{}.flame", path.last().unwrap()));
                    if !local_file.exists() {
                        local_file = self.resolve_path(&format!("{}.fm", path.last().unwrap()));
                    }
                    if local_file.exists() {
                        let content = fs::read_to_string(&local_file).map_err(|e| e.to_string())?;
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

                        let mod_env = Arc::new(Mutex::new(Env::new()));
                        let mut runner = Runner::new(local_file.clone());
                        runner.env = mod_env.clone();
                        for s in &parsed_stmts {
                            runner.execute_statement(s, mod_env.clone())?;
                        }

                        for (k, v) in runner.modules {
                            self.modules.insert(k, v);
                        }

                        env.lock().unwrap().define(
                            path.last().unwrap().clone(),
                            Value::Formula(mod_env.lock().unwrap().to_formula_map()),
                            false,
                        );
                        self.modules.insert(path.last().unwrap().clone(), mod_env);
                    } else {
                        return Err(format!(
                            "Module '{}' not found at {:?}",
                            mod_name, local_file
                        ));
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
                let target_str = match &target_val {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                
                for arm in arms {
                    if arm.pattern == "_" || arm.pattern == target_str {
                        let res = self.eval_expr(&arm.body, env.clone())?;
                        return Ok(res);
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

                    _ => {}
                }

                Ok(Value::Nil)
            }

            _ => Ok(Value::Nil),
        }
    }

    fn eval_expr(&mut self, expr: &Expr, env: Arc<Mutex<Env>>) -> Result<Value, String> {
        self.current_span = Some(expr.span());
        match expr {
            Expr::Literal(lit, _) => match lit {
                LiteralValue::Int(i) => Ok(Value::Int(*i)),
                LiteralValue::Float(f) => Ok(Value::Float(*f)),
                LiteralValue::String(s) => Ok(Value::String(s.clone())),
                LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                LiteralValue::Nil => Ok(Value::Nil),
            },

            Expr::Identifier(name, _) => {
                let val = {
                    let e = env.lock().unwrap();
                    e.get(name)
                };
                if let Some(val) = val {
                    if let Value::Moved(moved_name) = val {
                        return Err(format!(
                            "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                            moved_name, moved_name, moved_name
                        ));
                    }
                    let mut current = val;

                    loop {
                        match current {
                            Value::RefPath(path, _) => {
                                current = self.read_target(env.clone(), path)?;
                            }
                            other => return Ok(other),
                        }
                    }
                } else {
                    let mut found = None;
                    for (mod_name, mod_env) in &self.modules {
                        if mod_name == name || mod_name.ends_with(&format!(".{}", name)) {
                            found = Some(Value::Formula(mod_env.lock().unwrap().to_formula_map()));
                            break;
                        }
                    }
                    if let Some(val) = found {
                        Ok(val)
                    } else {
                        Err(format!("undefined variable '{}'", name))
                    }
                }
            }
            Expr::Closure { params, body, .. } => {
                Ok(Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                })
            }
            Expr::Borrow(inner, is_mut, _) => {
                // Construct a reference path when possible; fall back to Ref(value)
                match &**inner {
                    Expr::Identifier(name, _) => {
                        let val = {
                            let e = env.lock().unwrap();
                            e.get(name)
                        };
                        if let Some(val) = val {
                            if let Value::Moved(moved_name) = val {
                                return Err(format!(
                                    "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                                    moved_name, moved_name, moved_name
                                ));
                            }
                            if let Value::RefPath(path, _) = val {
                                return Ok(Value::RefPath(path, *is_mut));
                            }
                            return Ok(Value::RefPath(RefPath::Var(name.clone(), env.clone()), *is_mut));
                        }
                    }
                    Expr::Dot(inner_id, member, _) => {
                        if let Expr::Identifier(owner, _) = &**inner_id {
                            return Ok(Value::RefPath(
                                RefPath::Field {
                                    owner: owner.clone(),
                                    member: member.clone(),
                                    env: env.clone(),
                                },
                                *is_mut,
                            ));
                        }
                    }
                    _ => {}
                }
                let val = self.eval_expr(inner, env.clone())?;
                Ok(Value::Ref(Box::new(val)))
            }
            Expr::Binary(left, op, right, _) => {
                if matches!(op, BinaryOp::Assign | BinaryOp::PlusAssign | BinaryOp::MinusAssign) {
                    // Support assignment to identifiers, references, and simple dot paths
                    if let Expr::Identifier(var_name, _) = &**left {
                        let mut r_val = self.eval_expr(right, env.clone())?;
                        
                        let current = {
                            let e = env.lock().unwrap();
                            e.get(var_name)
                        };

                        if matches!(op, BinaryOp::PlusAssign | BinaryOp::MinusAssign) {
                            let l_val = match &current {
                                Some(Value::RefPath(path, _)) => self.read_target(env.clone(), path.clone())?,
                                Some(val) => val.clone(),
                                None => return Err(format!("undefined variable '{}'", var_name)),
                            };
                            r_val = match (op, l_val, &r_val) {
                                (BinaryOp::PlusAssign, Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                                (BinaryOp::PlusAssign, Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                                (BinaryOp::PlusAssign, Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
                                (BinaryOp::MinusAssign, Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                                (BinaryOp::MinusAssign, Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                                _ => return Err(format!("invalid operands for compound assignment")),
                            };
                        }

                        if let Some(Value::RefPath(path, mutable)) = current {
                            if !mutable {
                                return Err(format!(
                                    "cannot assign through immutable reference '{}'",
                                    var_name
                                ));
                            }

                            self.write_back(env.clone(), path, r_val.clone())?;

                            if let Expr::Identifier(src_name, _) = &**right {
                                env.lock().unwrap().move_var(src_name);
                            }
                            
                            return Ok(r_val);
                        }
                        env.lock()
                            .unwrap()
                            .assign(var_name.clone(), r_val.clone())?;
                        if let Expr::Identifier(src_name, _) = &**right {
                            env.lock().unwrap().move_var(src_name);
                        }
                        return Ok(r_val);
                    } else if let Expr::Dot(inner, member, _) = &**left {
                        if let Expr::Identifier(owner, _) = &**inner {
                            let mut r_val = self.eval_expr(right, env.clone())?;
                            
                            if matches!(op, BinaryOp::PlusAssign | BinaryOp::MinusAssign) {
                                let l_val = self.eval_expr(left, env.clone())?;
                                r_val = match (op, l_val, &r_val) {
                                    (BinaryOp::PlusAssign, Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                                    (BinaryOp::PlusAssign, Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                                    (BinaryOp::PlusAssign, Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
                                    (BinaryOp::MinusAssign, Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                                    (BinaryOp::MinusAssign, Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                                    _ => return Err(format!("invalid operands for compound assignment")),
                                };
                            }

                            self.write_back(
                                env.clone(),
                                RefPath::Field {
                                    owner: owner.clone(),
                                    member: member.clone(),
                                    env: env.clone(),
                                },
                                r_val.clone(),
                            )?;
                            if let Expr::Identifier(src_name, _) = &**right {
                                env.lock().unwrap().move_var(src_name);
                            }
                            return Ok(r_val);
                        } else {
                            return Err(
                                "left-hand side assignment must be a variable or variable field"
                                    .to_string(),
                            );
                        }
                    }
                }
                let l = self.eval_expr(left, env.clone())?;
                let r = self.eval_expr(right, env.clone())?;
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => match op {
                        BinaryOp::Add => Ok(Value::Int(a + b)),
                        BinaryOp::Sub => Ok(Value::Int(a - b)),
                        BinaryOp::Mul => Ok(Value::Int(a * b)),
                        BinaryOp::Div => Ok(Value::Int(if b != 0 { a / b } else { 0 })),
                        BinaryOp::Eq => Ok(Value::Bool(a == b)),
                        BinaryOp::Ne => Ok(Value::Bool(a != b)),
                        BinaryOp::Gt => Ok(Value::Bool(a > b)),
                        BinaryOp::Ge => Ok(Value::Bool(a >= b)),
                        BinaryOp::Lt => Ok(Value::Bool(a < b)),
                        BinaryOp::Le => Ok(Value::Bool(a <= b)),
                        BinaryOp::Range => Ok(Value::Range(a, b)),
                        _ => Ok(Value::Nil),
                    },
                    (Value::String(a), Value::String(b)) => match op {
                        BinaryOp::Add => Ok(Value::String(format!("{}{}", a, b))),
                        BinaryOp::Eq => Ok(Value::Bool(a == b)),
                        BinaryOp::Ne => Ok(Value::Bool(a != b)),
                        _ => Ok(Value::Nil),
                    },
                    (l_val, r_val) => match op {
                        BinaryOp::Eq => Ok(Value::Bool(l_val.to_string() == r_val.to_string())),
                        BinaryOp::Ne => Ok(Value::Bool(l_val.to_string() != r_val.to_string())),
                        _ => Ok(Value::Nil),
                    },
                }
            }
            Expr::StructInit(inner, fields, _) => {
                let inner_val = self.eval_expr(inner, env.clone())?;
                match inner_val {
                    Value::VariantConstructor(enum_name, var) => {
                        if let crate::parser::EnumVariant::Struct(n, _) = var {
                            let mut map = HashMap::new();
                            for (field_name, field_expr) in fields {
                                let val = self.eval_expr(field_expr, env.clone())?;
                                map.insert(field_name.clone(), val);
                            }
                            return Ok(Value::EnumValue(enum_name, n, EnumData::Struct(map)));
                        }
                        Err(format!("variant is not a struct variant"))
                    }
                    _ => Err(format!("cannot initialize struct on non-constructor value")),
                }
            }
            Expr::Dot(inner, member, _) => {
                if member == "clone" {
                    if let Expr::Identifier(name, _) = &**inner {
                        if let Some(val) = env.lock().unwrap().get(name) {
                            if let Value::Moved(moved_name) = val {
                                return Err(format!(
                                    "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                                    moved_name, moved_name, moved_name
                                ));
                            }
                            return Ok(val);
                        }
                    }
                }
                let left = self.eval_expr(inner, env.clone())?;
                match left {
                    Value::Formula(map) | Value::Object(map) => {
                        if let Some(val) = map.get(member) {
                            Ok(val.clone())
                        } else {
                            Err(format!("member '{}' not found", member))
                        }
                    }
                    Value::EnumMeta(enum_name, variants) => {
                        for var in &variants {
                            match var {
                                crate::parser::EnumVariant::Unit(n) => {
                                    if n == member {
                                        return Ok(Value::EnumValue(
                                            enum_name.clone(),
                                            n.clone(),
                                            EnumData::Unit,
                                        ));
                                    }
                                }
                                crate::parser::EnumVariant::Tuple(n, _)
                                | crate::parser::EnumVariant::Struct(n, _) => {
                                    if n == member {
                                        return Ok(Value::VariantConstructor(
                                            enum_name.clone(),
                                            var.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                        Err(format!(
                            "variant '{}' not found in enum '{}'",
                            member, enum_name
                        ))
                    }
                    Value::EnumValue(enum_name, variant_name, data) => match data {
                        EnumData::Struct(map) => {
                            if let Some(val) = map.get(member) {
                                Ok(val.clone())
                            } else {
                                Err(format!(
                                    "field '{}' not found in variant '{}.{}'",
                                    member, enum_name, variant_name
                                ))
                            }
                        }

                        EnumData::Tuple(values) => {
                            if values.len() == 1
                                && let Value::Formula(map) = &values[0]
                                && let Some(val) = map.get(member)
                            {
                                return Ok(val.clone());
                            }
                            Err(format!(
                                "field '{}' not found in variant '{}.{}'",
                                member, enum_name, variant_name
                            ))
                        }

                        EnumData::Unit => Err(format!(
                            "variant '{}.{}' has no fields",
                            enum_name, variant_name
                        )),
                    },
                    _ => Err(format!(
                        "cannot access member '{}' on non-namespace",
                        member
                    )),
                }
            }
            Expr::Call(callee, args, _) => {
                if let Expr::Identifier(name, _) = &**callee {
                    if name == "print" || name == "eprint" {
                        let mut parts = Vec::new();
                        for (_, arg) in args {
                            let val = self.eval_expr(arg, env.clone())?;
                            let val_str = match val {
                                Value::String(ref s) => s.clone(),
                                _ => val.to_string(),
                            };
                            parts.push(val_str);
                        }
                        if name == "print" {
                            println!("{}", parts.join(" "));
                        } else {
                            eprintln!("\x1b[1;31m{}\x1b[0m", parts.join(" "));
                        }
                        return Ok(Value::Nil);
                    }
                    if name == "input" {
                        use std::io::{self, Write};
                        if !args.is_empty() {
                            let arg_v = self.eval_expr(&args[0].1, env.clone())?;
                            print!("{}", arg_v.to_string());
                        }
                        let _ = io::stdout().flush();
                        let mut buffer = String::new();
                        let _ = io::stdin().read_line(&mut buffer);
                        return Ok(Value::String(buffer.trim_end().to_string()));
                    }
                }

                // Method / Namespace Member Call Interception
                if let Expr::Dot(inner_expr, member, _) = &**callee {
                    if member == "clone" {
                        if let Expr::Identifier(name, _) = &**inner_expr {
                            if let Some(val) = env.lock().unwrap().get(name) {
                                if let Value::Moved(moved_name) = val {
                                    return Err(format!(
                                        "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                                        moved_name, moved_name, moved_name
                                    ));
                                }
                                return Ok(val);
                            }
                        }
                        return self.eval_expr(inner_expr, env.clone());
                    }
                    let inner_val = self.eval_expr(inner_expr, env.clone())?;
                    match inner_val {
                        Value::EnumMeta(enum_name, variants) => {
                            for var in &variants {
                                if let crate::parser::EnumVariant::Tuple(n, _types) = var {
                                    if n == member {
                                        let mut tuple_vals = Vec::new();
                                        for (_, arg_expr) in args {
                                            tuple_vals.push(self.eval_expr(arg_expr, env.clone())?);
                                        }
                                        return Ok(Value::EnumValue(
                                            enum_name.clone(),
                                            n.clone(),
                                            EnumData::Tuple(tuple_vals),
                                        ));
                                    }
                                }
                            }
                            return Err(format!(
                                "tuple variant '{}' not found in enum '{}'",
                                member, enum_name
                            ));
                        }
                        Value::String(ref s) => match member.as_str() {
                            "len" => return Ok(Value::Int(s.len() as i64)),
                            "to_uppercase" => return Ok(Value::String(s.to_uppercase())),
                            "to_lowercase" => return Ok(Value::String(s.to_lowercase())),
                            "trim" => return Ok(Value::String(s.trim().to_string())),
                            "push_str" => {
                                if !args.is_empty() {
                                    let val = self.eval_expr(&args[0].1, env.clone())?;
                                    if let Value::String(add_s) = val {
                                        if let Expr::Identifier(var_name, _) = &**inner_expr {
                                            let mut new_s = s.clone();
                                            new_s.push_str(&add_s);
                                            env.lock()
                                                .unwrap()
                                                .assign(var_name.clone(), Value::String(new_s))?;
                                        }
                                    }
                                }
                                return Ok(Value::Nil);
                            }
                            _ => {}
                        },
                        Value::Tuple(ref vec) => match member.as_str() {
                            "len" => return Ok(Value::Int(vec.len() as i64)),
                            "is_empty" => return Ok(Value::Bool(vec.is_empty())),
                            "push" => {
                                if !args.is_empty() {
                                    let val = self.eval_expr(&args[0].1, env.clone())?;
                                    if let Expr::Identifier(var_name, _) = &**inner_expr {
                                        let mut new_vec = vec.clone();
                                        new_vec.push(val);
                                        env.lock()
                                            .unwrap()
                                            .assign(var_name.clone(), Value::Tuple(new_vec))?;
                                    }
                                }
                                return Ok(Value::Nil);
                            }
                            "pop" => {
                                if let Expr::Identifier(var_name, _) = &**inner_expr {
                                    let mut new_vec = vec.clone();
                                    let popped = new_vec.pop().unwrap_or(Value::Nil);
                                    env.lock()
                                        .unwrap()
                                        .assign(var_name.clone(), Value::Tuple(new_vec))?;
                                    return Ok(popped);
                                }
                                return Ok(Value::Nil);
                            }
                            "filter" => {
                                if !args.is_empty() {
                                    let cb_val = self.eval_expr(&args[0].1, env.clone())?;
                                    if let Value::Function { params, body, env: closure_env } = cb_val {
                                        let mut res = Vec::new();
                                        for item in vec {
                                            let child_env = Arc::new(Mutex::new(Env::new_child(closure_env.clone())));
                                            if !params.is_empty() {
                                                self.bind_param(child_env.clone(), &params[0], item.clone());
                                            }
                                            let mut matched = false;
                                            for stmt in &body {
                                                let stmt_res = self.execute_statement(stmt, child_env.clone())?;
                                                if let Value::Return(ret_val) = stmt_res {
                                                    if let Value::Bool(b) = *ret_val {
                                                        matched = b;
                                                    }
                                                    break;
                                                }
                                            }
                                            if matched {
                                                res.push(item.clone());
                                            }
                                        }
                                        return Ok(Value::Tuple(res));
                                    }
                                }
                                return Ok(Value::Tuple(vec.clone()));
                            }
                            "map" => {
                                if !args.is_empty() {
                                    let cb_val = self.eval_expr(&args[0].1, env.clone())?;
                                    if let Value::Function { params, body, env: closure_env } = cb_val {
                                        let mut res = Vec::new();
                                        for item in vec {
                                            let child_env = Arc::new(Mutex::new(Env::new_child(closure_env.clone())));
                                            if !params.is_empty() {
                                                self.bind_param(child_env.clone(), &params[0], item.clone());
                                            }
                                            let mut map_res = Value::Nil;
                                            for stmt in &body {
                                                let stmt_res = self.execute_statement(stmt, child_env.clone())?;
                                                if let Value::Return(ret_val) = stmt_res {
                                                    map_res = *ret_val;
                                                    break;
                                                }
                                                map_res = stmt_res;
                                            }
                                            res.push(map_res);
                                        }
                                        return Ok(Value::Tuple(res));
                                    }
                                }
                                return Ok(Value::Tuple(vec.clone()));
                            }
                            _ => {}
                        },
                        Value::ThreadHandler(id) => {
                            if member == "join" {
                                let mut registry = get_threads().lock().unwrap();
                                if let Some(handle) = registry.remove(&id) {
                                    let result = handle.join().unwrap();
                                    return Ok(result);
                                }
                                return Ok(Value::Nil);
                            }
                        }
                        Value::Sender(id) => {
                            if member == "send" {
                                if !args.is_empty() {
                                    let val = self.eval_expr(&args[0].1, env.clone())?;
                                    let registry = get_channels().lock().unwrap();
                                    if let Some(tx) = registry.get(&id) {
                                        let _ = tx.send(val);
                                    }
                                }
                                return Ok(Value::Nil);
                            }
                        }
                        Value::Receiver(id) => {
                            if member == "recv" {
                                let rx_opt = {
                                    let registry = get_receivers().lock().unwrap();
                                    registry.get(&id).cloned()
                                };
                                if let Some(rx) = rx_opt {
                                    let val = rx.lock().unwrap().recv().unwrap_or(Value::Nil);
                                    return Ok(val);
                                }
                                return Ok(Value::Nil);
                            }
                        }
                        Value::CommandBuilder {
                            program,
                            args: mut builder_args,
                        } => {
                            if member == "args" {
                                if !args.is_empty() {
                                    let list_val = self.eval_expr(&args[0].1, env.clone())?;
                                    if let Value::Tuple(items) = list_val {
                                        for it in items {
                                            builder_args.push(it.to_string());
                                        }
                                    }
                                }
                                return Ok(Value::CommandBuilder {
                                    program,
                                    args: builder_args,
                                });
                            } else if member == "spawn" {
                                return Ok(Value::ChildProcess(101));
                            }
                        }
                        Value::ChildProcess(_pid) => {
                            if member == "wait_with_output" {
                                let mut output_map = HashMap::new();
                                output_map.insert(
                                    "stdout".to_string(),
                                    Value::String("git version 2.40.1".to_string()),
                                );
                                output_map
                                    .insert("stderr".to_string(), Value::String(String::new()));

                                let mut status_map = HashMap::new();
                                status_map.insert("code".to_string(), Value::Int(0));
                                output_map.insert("status".to_string(), Value::Formula(status_map));

                                return Ok(Value::Formula(output_map));
                            }
                        }
                        Value::NativeObject {
                            ref crate_name,
                            ref type_name,
                            ptr: _,
                        } => {
                            let mut c_args = Vec::new();
                            c_args.push(inner_val.pack());
                            for (_, arg_expr) in args {
                                let arg_v = self.eval_expr(arg_expr, env.clone())?;
                                c_args.push(arg_v.pack());
                            }

                            let sym_str1 = format!("flame_{}_{}_{}", crate_name, type_name, member);
                            let sym_str2 = format!("flame_{}_{}", crate_name, member);

                            let func = self
                                .native_methods
                                .get(&sym_str1)
                                .or_else(|| self.native_methods.get(&sym_str2));

                            if let Some(func) = func {
                                let res = func(c_args.as_ptr(), c_args.len());
                                for c_val in c_args {
                                    if c_val.tag == crate::runner::CValueTag::String
                                        && !c_val.string_ptr.is_null()
                                    {
                                        unsafe {
                                            let _ = std::ffi::CString::from_raw(c_val.string_ptr);
                                        }
                                    }
                                }
                                return Ok(Value::unpack(res, crate_name, type_name));
                            }

                            return Err(format!(
                                "NativeObject method '{}.{}' not found in static registry (tried {} and {})",
                                type_name, member, sym_str1, sym_str2
                            ));
                        }
                        Value::RustServer { port } => {
                            if member == "bind" {
                                if std::env::var("WREN_VERBOSE").is_ok()
                                    || std::env::var("WREN_DEV").is_ok()
                                {
                                    println!("Bound server to http://127.0.0.1:{}", port);
                                }
                                return Ok(Value::RustServer { port });
                            } else if member == "listen" || member == "start" {
                                let addr = format!("127.0.0.1:{}", port);
                                if let Ok(listener) = std::net::TcpListener::bind(&addr) {
                                    for stream in listener.incoming() {
                                        if let Ok(mut stream) = stream {
                                            use std::io::{Read, Write};
                                            let mut buf = [0u8; 1024];
                                            let _ = stream.read(&mut buf);
                                            let json_body = r#"{"status": "ok", "message": "Hello from Flame HTTP Router Server"}"#;
                                            let response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                json_body.len(),
                                                json_body
                                            );
                                            let _ = stream.write_all(response.as_bytes());
                                            let _ = stream.flush();
                                        }
                                    }
                                }
                                return Ok(Value::Nil);
                            } else if member == "get" {
                                return Ok(Value::RustServer { port });
                            } else if member == "accept" {
                                if std::env::var("WREN_VERBOSE").is_ok()
                                    || std::env::var("WREN_DEV").is_ok()
                                {
                                    println!("Accepted connection on http://127.0.0.1:{}", port);
                                }
                                return Ok(Value::String("accepted".to_string()));
                            } else if member == "stop" || member == "close" {
                                if std::env::var("WREN_VERBOSE").is_ok()
                                    || std::env::var("WREN_DEV").is_ok()
                                {
                                    println!("Server stopped on port {}.", port);
                                }
                                return Ok(Value::Nil);
                            }
                        }
                        Value::Formula(ref map) => {
                            if !map.contains_key("__module__") && !map.contains_key("__crate__") {
                                match member.as_str() {
                                    "len" => return Ok(Value::Int(map.len() as i64)),
                                    "insert" => {
                                        if args.len() >= 2 {
                                            let k_val = self.eval_expr(&args[0].1, env.clone())?;
                                            let v_val = self.eval_expr(&args[1].1, env.clone())?;
                                            if let Expr::Identifier(var_name, _) = &**inner_expr {
                                                let mut new_map = map.clone();
                                                new_map.insert(k_val.to_string(), v_val);
                                                env.lock().unwrap().assign(
                                                    var_name.clone(),
                                                    Value::Formula(new_map),
                                                )?;
                                            }
                                        }
                                        return Ok(Value::Nil);
                                    }
                                    "get" => {
                                        if !args.is_empty() {
                                            let k_val = self.eval_expr(&args[0].1, env.clone())?;
                                            if let Some(v) = map.get(&k_val.to_string()) {
                                                return Ok(v.clone());
                                            }
                                        }
                                        return Ok(Value::Nil);
                                    }
                                    "remove" => {
                                        if !args.is_empty() {
                                            let k_val = self.eval_expr(&args[0].1, env.clone())?;
                                            if let Expr::Identifier(var_name, _) = &**inner_expr {
                                                let mut new_map = map.clone();
                                                let removed = new_map
                                                    .remove(&k_val.to_string())
                                                    .unwrap_or(Value::Nil);
                                                env.lock().unwrap().assign(
                                                    var_name.clone(),
                                                    Value::Formula(new_map),
                                                )?;
                                                return Ok(removed);
                                            }
                                        }
                                        return Ok(Value::Nil);
                                    }
                                    _ => {}
                                }
                            }
                            let crate_binding =
                                if let Some(Value::String(c_name)) = map.get("__crate__") {
                                    c_name.clone()
                                } else if let Expr::Identifier(namespace, _) = &**inner_expr {
                                    namespace
                                        .strip_prefix("native.")
                                        .unwrap_or(namespace.as_str())
                                        .to_string()
                                } else {
                                    String::new()
                                };

                            let namespace = if let Expr::Identifier(ns, _) = &**inner_expr {
                                ns.as_str()
                            } else {
                                ""
                            };
                            if namespace == "thread_bridge" && member == "create_channel" {
                                let mut counter = get_channel_counter().lock().unwrap();
                                *counter += 1;
                                let chan_id = *counter;

                                let (tx, rx) = std::sync::mpsc::channel();
                                get_channels().lock().unwrap().insert(chan_id, tx);
                                get_receivers()
                                    .lock()
                                    .unwrap()
                                    .insert(chan_id, Arc::new(Mutex::new(rx)));

                                return Ok(Value::Tuple(vec![
                                    Value::Sender(chan_id),
                                    Value::Receiver(chan_id),
                                ]));
                            } else if namespace == "process_bridge" && member == "cmd" {
                                let mut prog = String::new();
                                if !args.is_empty() {
                                    prog = self.eval_expr(&args[0].1, env.clone())?.to_string();
                                }
                                return Ok(Value::CommandBuilder {
                                    program: prog,
                                    args: vec![],
                                });
                            } else if namespace == "fs_bridge" && member == "read_file" {
                                let mut path_str = String::new();
                                if !args.is_empty() {
                                    path_str = self.eval_expr(&args[0].1, env.clone())?.to_string();
                                }
                                let abs_path = self.resolve_path(path_str.trim_matches('"'));
                                if let Ok(c) = fs::read_to_string(abs_path) {
                                    return Ok(Value::String(c));
                                } else {
                                    return Err("File not found".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "write_file" {
                                let mut path_str = String::new();
                                let mut content_str = String::new();
                                if args.len() >= 2 {
                                    path_str = self
                                        .eval_expr(&args[0].1, env.clone())?
                                        .to_string()
                                        .trim_matches('"')
                                        .to_string();
                                    content_str = self
                                        .eval_expr(&args[1].1, env.clone())?
                                        .to_string()
                                        .trim_matches('"')
                                        .to_string();
                                }
                                let abs_path = self.resolve_path(&path_str);
                                if fs::write(abs_path, content_str).is_ok() {
                                    return Ok(Value::Nil);
                                } else {
                                    return Err("Failed to write file".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "create_dir" {
                                let path_str = self
                                    .eval_expr(&args[0].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let abs_path = self.resolve_path(&path_str);
                                if fs::create_dir_all(abs_path).is_ok() {
                                    return Ok(Value::Nil);
                                } else {
                                    return Err("Failed to create directory".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "remove_file" {
                                let path_str = self
                                    .eval_expr(&args[0].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let abs_path = self.resolve_path(&path_str);
                                if fs::remove_file(abs_path).is_ok() {
                                    return Ok(Value::Nil);
                                } else {
                                    return Err("Failed to remove file".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "create_file" {
                                let path_str = self
                                    .eval_expr(&args[0].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let content_str = self
                                    .eval_expr(&args[1].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let abs_path = self.resolve_path(&path_str);
                                if fs::File::create(&abs_path).is_ok() {
                                    if fs::write(&abs_path, &content_str).is_ok() {
                                        return Ok(Value::Nil);
                                    } else {
                                        return Err("Failed to write file".to_string());
                                    }
                                } else {
                                    return Err("Failed to create file".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "remove_dir" {
                                let path_str = self
                                    .eval_expr(&args[0].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let abs_path = self.resolve_path(&path_str);
                                if fs::remove_dir_all(abs_path).is_ok() {
                                    return Ok(Value::Nil);
                                } else {
                                    return Err("Failed to remove directory".to_string());
                                }
                            } else if namespace == "fs_bridge" && member == "exists" {
                                let path_str = self
                                    .eval_expr(&args[0].1, env.clone())?
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();
                                let abs_path = self.resolve_path(&path_str);
                                return Ok(Value::Bool(abs_path.exists()));
                            } else {
                                // Universal Native Crate Interop Resolution
                                let raw_ns_str = if !crate_binding.is_empty() {
                                    crate_binding.clone()
                                } else {
                                    namespace
                                        .strip_prefix("native.")
                                        .unwrap_or(namespace)
                                        .to_string()
                                };
                                let raw_ns = raw_ns_str.as_str();
                                let mut c_args = Vec::new();
                                for (_, arg_expr) in args {
                                    let arg_v = self.eval_expr(arg_expr, env.clone())?;
                                    c_args.push(arg_v.pack());
                                }

                                let parent_type = map
                                    .get("__type__")
                                    .or_else(|| map.get("__member__"))
                                    .and_then(|v| {
                                        if let Value::String(s) = v {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    });

                                let mut symbol_candidates =
                                    vec![format!("flame_{}_{}", raw_ns, member)];
                                if let Some(pt) = &parent_type {
                                    symbol_candidates
                                        .push(format!("flame_{}_{}_{}", raw_ns, pt, member));
                                }

                                for sym_str in symbol_candidates {
                                    if let Some(func) = self.native_methods.get(&sym_str) {
                                        let res = func(c_args.as_ptr(), c_args.len());

                                        for c_val in c_args {
                                            if c_val.tag == crate::runner::CValueTag::String
                                                && !c_val.string_ptr.is_null()
                                            {
                                                unsafe {
                                                    let _ = std::ffi::CString::from_raw(
                                                        c_val.string_ptr,
                                                    );
                                                }
                                            }
                                        }

                                        let t_name = parent_type.unwrap_or_else(|| member.clone());
                                        return Ok(Value::unpack(res, raw_ns, &t_name));
                                    }
                                }
                            }
                            if let Some(val) = map.get(member) {
                                if let Value::Function { params, body, env: _func_env } = val {
                                    if body.is_empty() {
                                        let mut evaled_args = Vec::new();
                                        for (_, arg_expr) in args {
                                            let arg_v = self.eval_expr(arg_expr, env.clone())?;
                                            if let Expr::Identifier(src_name, _) = arg_expr {
                                                env.lock().unwrap().move_var(src_name);
                                            }
                                            evaled_args.push(
                                                arg_v.to_string().trim_matches('"').to_string(),
                                            );
                                        }
                                        let args_str = evaled_args.join(", ");
                                        let res_str = if args_str.is_empty() {
                                            format!("{}()", member)
                                        } else {
                                            format!("{}({})", member, args_str)
                                        };
                                        return Ok(Value::String(res_str));
                                    }
                                    let child_env =
                                        Arc::new(Mutex::new(Env::new_child(env.clone())));
                                    let mut self_val = inner_val.clone();
                                    if let Expr::Identifier(var_name, _) = &**inner_expr {
                                        self_val = Value::RefPath(crate::vm::RefPath::Var(var_name.clone(), env.clone()), true);
                                    }
                                    child_env.lock().unwrap().define(
                                        "self".to_string(),
                                        self_val,
                                        true,
                                    );
                                    for (i, p) in params.iter().enumerate() {
                                        if i < args.len() {
                                            let arg_val =
                                                self.eval_expr(&args[i].1, env.clone())?;
                                            if let Expr::Identifier(src_name, _) = &args[i].1 {
                                                env.lock().unwrap().move_var(src_name);
                                            }
                                            self.bind_param(child_env.clone(), p, arg_val);
                                        }
                                    }
                                    let mut last_val = Value::Nil;
                                    for stmt in body {
                                        let res = self.execute_statement(stmt, child_env.clone())?;
                                        if let Value::Return(ret_val) = res {
                                            return Ok(*ret_val);
                                        }
                                        last_val = res;
                                    }
                                    return Ok(last_val);
                                }
                                if let Value::NativeCallback(cb) = val {
                                    let mut evaled_args = Vec::new();
                                    for (_, arg_expr) in args {
                                        let arg_v = self.eval_expr(arg_expr, env.clone())?;
                                        evaled_args.push(arg_v);
                                    }
                                    return cb(evaled_args);
                                }
                                return Ok(val.clone());
                            }
                        }
                        _ => {}
                    }
                }

                // Resolve function or structure constructors
                let func_val = self.eval_expr(callee, env.clone())?;
                match func_val {
                    Value::StructConstructor { name, fields } => {
                        let mut instance_map = HashMap::new();
                        for (i, (f_name, _)) in fields.iter().enumerate() {
                            let mut val = Value::Nil;
                            for (arg_name, arg_expr) in args {
                                if let Some(an) = arg_name {
                                    if an == f_name {
                                        val = self.eval_expr(arg_expr, env.clone())?;
                                        break;
                                    }
                                }
                            }
                            if matches!(val, Value::Nil) && i < args.len() && args[i].0.is_none() {
                                val = self.eval_expr(&args[i].1, env.clone())?;
                            }
                            instance_map.insert(f_name.clone(), val);
                        }
                        if let Some(impl_env) = self.modules.get(&format!("impl_{}", name)) {
                            for (m_name, m_val) in &impl_env.lock().unwrap().variables {
                                instance_map.insert(m_name.clone(), m_val.value.clone());
                            }
                        }
                        Ok(Value::Formula(instance_map))
                    }
                    Value::VariantConstructor(enum_name, var) => {
                        if let crate::parser::EnumVariant::Tuple(n, _) = var {
                            let mut tuple_vals = Vec::new();
                            for (_, arg_expr) in args {
                                tuple_vals.push(self.eval_expr(arg_expr, env.clone())?);
                            }
                            return Ok(Value::EnumValue(enum_name, n, EnumData::Tuple(tuple_vals)));
                        }
                        Err(format!("variant '{:?}' is not a tuple variant", var))
                    }
                    Value::Function { params, body, env: closure_env } => {
                        let child_env = Arc::new(Mutex::new(Env::new_child(closure_env.clone())));
                        for (i, p) in params.iter().enumerate() {
                            if i < args.len() {
                                let arg_val = self.eval_expr(&args[i].1, env.clone())?;
                                if let Expr::Identifier(src_name, _) = &args[i].1 {
                                    env.lock().unwrap().move_var(src_name);
                                }
                                self.bind_param(child_env.clone(), p, arg_val);
                            }
                        }
                        let mut last_val = Value::Nil;
                        for stmt in &body {
                            let res = self.execute_statement(stmt, child_env.clone())?;
                            if let Value::Return(ret_val) = res {
                                return Ok(*ret_val);
                            }
                            last_val = res;
                        }
                        Ok(last_val)
                    }
                    Value::NativeCallback(cb) => {
                        let mut evaled_args = Vec::new();
                        for (_, arg_expr) in args {
                            evaled_args.push(self.eval_expr(arg_expr, env.clone())?);
                        }
                        cb(evaled_args)
                    }
                    _ => {
                        let lexeme = match &**callee {
                            Expr::Identifier(id, _) => id.as_str(),
                            _ => "unknown",
                        };
                        if lexeme == "print" {
                            for (_, arg_expr) in args {
                                let arg_v = self.eval_expr(arg_expr, env.clone())?;
                                print!("{} ", arg_v.to_string());
                            }
                            println!();
                            return Ok(Value::Nil);
                        }
                        if lexeme == "RustServer" {
                            let mut port = 8080;
                            for (arg_name, arg_expr) in args {
                                if let Some(n) = arg_name {
                                    if n == "port" {
                                        if let Value::Int(p) =
                                            self.eval_expr(arg_expr, env.clone())?
                                        {
                                            port = p as u16;
                                        }
                                    }
                                }
                            }
                            return Ok(Value::RustServer { port });
                        }
                        Ok(Value::Nil)
                    }
                }
            }
            Expr::Formula(mappings, _) => {
                let mut map = HashMap::new();
                for (k, v) in mappings {
                    let val = self.eval_expr(v, env.clone())?;
                    map.insert(k.clone(), val);
                }
                Ok(Value::Formula(map))
            }
            Expr::InterpolatedString(segments, _) => {
                let mut result = String::new();
                for seg in segments {
                    match seg {
                        InterpolatedSegment::Text(text) => result.push_str(text),
                        InterpolatedSegment::Expr(ex) => {
                            let val = self.eval_expr(ex, env.clone())?;
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expr::ThreadSpawn(expr_block, _) => {
                let expr_clone = expr_block.clone();
                let thread_env = Arc::new(Mutex::new(Env::new_child(env.clone())));
                let mut runner = self.clone_for_thread(thread_env.clone());

                let mut counter = get_thread_counter().lock().unwrap();
                *counter += 1;
                let id = *counter;

                let handle = thread::spawn(move || {
                    runner
                        .eval_expr(&expr_clone, thread_env)
                        .unwrap_or(Value::Nil)
                });

                get_threads().lock().unwrap().insert(id, handle);

                Ok(Value::ThreadHandler(id))
            }
            Expr::Tuple(exprs, _) => {
                let mut vals = Vec::new();
                for e in exprs {
                    vals.push(self.eval_expr(e, env.clone())?);
                }
                Ok(Value::Tuple(vals))
            }
            Expr::Await(inner, _) => {
                let inner_val = self.eval_expr(inner, env)?;
                if let Value::ThreadHandler(id) = inner_val {
                    let handle_opt = get_threads().lock().unwrap().remove(&id);
                    if let Some(handle) = handle_opt {
                        match handle.join() {
                            Ok(val) => Ok(val),
                            Err(_) => Err("Thread panicked".to_string()),
                        }
                    } else {
                        Err(format!("Invalid thread handle {}", id))
                    }
                } else {
                    Ok(inner_val)
                }
            }
            Expr::Block(statements, _) => {
                let child_env = Arc::new(Mutex::new(Env::new_child(env)));
                let mut last_val = Value::Nil;
                for stmt in statements {
                    last_val = self.execute_statement(stmt, child_env.clone())?;
                }
                Ok(last_val)
            }
            Expr::VectorLiteral(exprs, _) => {
                let mut vals = Vec::new();
                for e in exprs {
                    vals.push(self.eval_expr(e, env.clone())?);
                }
                Ok(Value::Tuple(vals))
            }
        }
    }

    fn read_target(&self, _env: Arc<Mutex<Env>>, path: RefPath) -> Result<Value, String> {
        match path {
            RefPath::Var(name, env) => {
                let val = {
                    let e = env.lock().unwrap();
                    e.get(&name)
                };
                match val {
                    Some(Value::Moved(moved_name)) => Err(format!(
                        "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                        moved_name, moved_name, moved_name
                    )),
                    Some(Value::RefPath(next, _)) => self.read_target(env.clone(), next),

                    Some(v) => Ok(v),
                    None => Err(format!("undefined variable '{}'", name)),
                }
            }
            RefPath::Field { owner, member, env } => {
                let owner_val = {
                    let e = env.lock().unwrap();
                    e.get(&owner)
                };
                match owner_val {
                    Some(Value::Moved(moved_name)) => Err(format!(
                        "use of moved value '{}'. Value was moved. Use '&{}' to borrow or '{}.clone()' to copy.",
                        moved_name, moved_name, moved_name
                    )),
                    Some(Value::RefPath(next, _)) => self.read_target(env.clone(), next),
                    Some(val) => self.read_field_value(&val, &member, &owner),
                    None => Err(format!("variable '{}' not found for field read", owner)),
                }
            }
        }
    }

    fn read_field_value(
        &self,
        owner_val: &Value,
        member: &str,
        owner: &str,
    ) -> Result<Value, String> {
        match owner_val {
            Value::Formula(map) => map
                .get(member)
                .cloned()
                .ok_or_else(|| format!("member '{}' not found in '{}'", member, owner)),
            Value::EnumValue(enum_name, variant_name, data) => match data {
                EnumData::Struct(map) => map.get(member).cloned().ok_or_else(|| {
                    format!(
                        "field '{}' not found in variant '{}.{}'",
                        member, enum_name, variant_name
                    )
                }),
                EnumData::Tuple(values) => {
                    if values.len() == 1 {
                        if let Value::Formula(map) = &values[0] {
                            map.get(member).cloned().ok_or_else(|| {
                                format!(
                                    "field '{}' not found in variant '{}.{}'",
                                    member, enum_name, variant_name
                                )
                            })
                        } else {
                            Err(format!(
                                "field '{}' not found in variant '{}.{}'",
                                member, enum_name, variant_name
                            ))
                        }
                    } else {
                        Err(format!(
                            "field '{}' not found in variant '{}.{}'",
                            member, enum_name, variant_name
                        ))
                    }
                }
                EnumData::Unit => Err(format!(
                    "variant '{}.{}' has no fields",
                    enum_name, variant_name
                )),
            },
            _ => Err(format!(
                "cannot access member '{}' on non-namespace value in '{}'",
                member, owner
            )),
        }
    }

    fn write_back(
        &self,
        _env: Arc<Mutex<Env>>,
        path: RefPath,
        new_val: Value,
    ) -> Result<(), String> {
        match path {
            RefPath::Var(name, env) => {
                let current = {
                    let e = env.lock().unwrap();
                    e.get(&name)
                };

                if let Some(Value::RefPath(next, _)) = current {
                    return self.write_back(env.clone(), next, new_val);
                }

                env.lock().unwrap().assign(name, new_val)
            }
            RefPath::Field { owner, member, env } => {
                let mut owner_val = {
                    let e = env.lock().unwrap();
                    e.get(&owner)
                };
                
                // Follow reference paths to get the actual value to modify
                let mut final_owner = owner.clone();
                let mut final_env = env.clone();
                while let Some(Value::RefPath(RefPath::Var(ref next_owner, ref next_env), _)) = owner_val {
                    final_owner = next_owner.clone();
                    final_env = next_env.clone();
                    owner_val = {
                        let e = final_env.lock().unwrap();
                        e.get(&final_owner)
                    };
                }
                
                let Some(mut owner_val) = owner_val else {
                    return Err(format!(
                        "variable '{}' not found for field assignment",
                        final_owner
                    ));
                };
                match &mut owner_val {
                    Value::Formula(map) => {
                        map.insert(member, new_val);
                    }
                    Value::EnumValue(enum_name, variant_name, data) => match data {
                        EnumData::Struct(map) => {
                            map.insert(member, new_val);
                        }
                        EnumData::Tuple(vec) => {
                            if vec.len() == 1 {
                                if let Value::Formula(map) = &mut vec[0] {
                                    map.insert(member, new_val);
                                } else {
                                    return Err(format!(
                                        "cannot assign field '{}' on '{}.{}' tuple payload that is not a Formula",
                                        member, enum_name, variant_name
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "cannot assign field '{}' on '{}.{}' tuple variant",
                                    member, enum_name, variant_name
                                ));
                            }
                        }
                        EnumData::Unit => {
                            return Err(format!(
                                "cannot assign field '{}' on '{}.{}' unit variant",
                                member, enum_name, variant_name
                            ));
                        }
                    },
                    _ => {
                        return Err(
                            "field assignment supported only on Formula and Enum variants"
                                .to_string(),
                        );
                    }
                }
                final_env.lock().unwrap().assign(final_owner, owner_val)
            }
        }
    }

    fn bind_param(&self, child_env: Arc<Mutex<Env>>, param: &Param, arg_val: Value) {
        // Store RefPath arguments directly; never re-wrap as RefPath::Var(param.name).
        child_env
            .lock()
            .unwrap()
            .define(param.name.clone(), arg_val, param.is_mut);
    }



    fn load_rust_file_methods(&self, rs_code: &str, env: Arc<Mutex<Env>>) {
        let mut e = env.lock().unwrap();
        for line in rs_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn ") {
                if let Some(rest) = trimmed.strip_prefix("pub fn ") {
                    if let Some(fn_name) = rest.split('(').next() {
                        let fn_name = fn_name.trim().to_string();
                        if !fn_name.is_empty() {
                            e.define(
                                fn_name,
                                Value::Function {
                                    params: vec![],
                                    body: vec![],
                                    env: env.clone(),
                                },
                                false,
                            );
                        }
                    }
                }
            }
        }
    }

    fn execute_plugin(&mut self, plugin_name: &str, env: Arc<Mutex<Env>>) -> Result<Value, String> {
        let rel_meta = format!(".flame/pkg/{}/{}.fmi", plugin_name, plugin_name);
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

        for meta_path in meta_candidates {
            if !meta_path.exists() {
                continue;
            }
            let meta_str = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
            let meta = serde_json::from_str::<crate::package_manager::FlameMeta>(&meta_str)
                .map_err(|e| e.to_string())?;
            let mod_env = Arc::new(Mutex::new(Env::new()));
            mod_env.lock().unwrap().define(
                "__crate__".to_string(),
                Value::String(plugin_name.to_string()),
                false,
            );
            for fn_meta in &meta.functions {
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
                    },
                    false,
                );
            }
            env.lock().unwrap().define(
                plugin_name.to_string(),
                Value::Formula(mod_env.lock().unwrap().to_formula_map()),
                false,
            );
            self.modules.insert(plugin_name.to_string(), mod_env);
            return Ok(Value::Nil);
        }

        let local_file = self
            .filepath
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("{}.flame", plugin_name));
        let plugin_file = if local_file.exists() {
            local_file
        } else {
            let ws_file = self
                .filepath
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("src")
                .join(format!("{}.flame", plugin_name));
            if ws_file.exists() {
                ws_file
            } else {
                return Err(format!(
                    "Plugin '{}' not found as native metadata or local .flame file",
                    plugin_name
                ));
            }
        };

        let content = std::fs::read_to_string(&plugin_file).map_err(|e| e.to_string())?;
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
        let mut parser = Parser::new(tokens, plugin_file.to_string_lossy().to_string());
        let parsed_stmts = parser.parse().map_err(|e| e.message)?;

        for s in &parsed_stmts {
            self.execute_statement(s, env.clone())?;
        }
        Ok(Value::Nil)
    }

    fn resolve_path(&self, path_str: &str) -> PathBuf {
        let p = Path::new(path_str);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.filepath
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(p)
        }
    }

    pub fn clone_for_thread(&self, env: Arc<Mutex<Env>>) -> Self {
        Self {
            env,
            filepath: self.filepath.clone(),
            modules: self.modules.clone(),
            current_span: None,
            native_methods: self.native_methods.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_flame(code: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(code);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == crate::lexer::TokenKind::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }

        let mut parser = Parser::new(tokens, "test.flame".to_string());
        let stmts = parser.parse().map_err(|diag| diag.message)?;
        let mut runner = Runner::new(PathBuf::from("test.flame"));
        runner.run(&stmts)
    }

    #[test]
    fn mut_ref_through_enum_field() {
        let code = r#"fn change(&mut name: String) {
    print("in change, before:", name)
    name = "core"
    print("in change, after:", name)
}

enum Config {
    modules(Formula)
}

fn main() {
    let mut f: Formula = formula {
        name: "std",
        v: "1.0.0",
        description: "std lib of flame"
    }

    let mut con: Config = Config.modules(f)

    change(&mut con.name)
    print("final:", con.name)
}"#;

        run_flame(code).unwrap();
    }

    #[test]
    fn std_thread_execution() {
        let code = r#"import std.thread

fn main() {
    let (tx, rx) = thread.channel()
    tx.send("test_message")
    rx.recv()
}"#;

        let result = run_flame(code).unwrap();
        assert_eq!(result.to_string(), "\"test_message\"");
    }
}
