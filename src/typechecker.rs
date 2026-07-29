use crate::diagnostics::Diagnostic;
use crate::lexer::Span;
use crate::parser::{BinaryOp, EnumVariant, Expr, FormulaValue, LiteralValue, Param, Stmt};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Int,
    Float,
    String,
    Bool,
    Nil,
    Tuple(Vec<Type>),
    Vector(Box<Type>),
    Formula(HashMap<String, Type>),
    Struct(String),
    Enum(String),
    EnumVariant {
        enum_name: String,
        variant_name: String,
        tuple_items: Vec<Type>,
        struct_fields: HashMap<String, Type>,
    },
    Named(String),
    Unknown,
    Reference {
        inner: Box<Type>,
        mutable: bool,
    },
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: Type,
    is_mut: bool,
}

#[derive(Debug, Clone)]
struct ParamInfo {
    name: String,
    ty: Type,
    is_ref: bool,
    is_mut: bool,
}

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<ParamInfo>,
    return_type: Type,
}

#[derive(Debug, Clone)]
struct StructInfo {
    fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
struct VariantInfo {
    tuple_items: Vec<Type>,
    struct_fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    variants: HashMap<String, VariantInfo>,
}

pub struct TypeChecker {
    filepath: String,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, VarInfo>>,
    functions: HashMap<String, FunctionSig>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    methods: HashMap<String, HashMap<String, FunctionSig>>,
    current_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new(filepath: String) -> Self {
        let mut checker = Self {
            filepath,
            diagnostics: Vec::new(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            methods: HashMap::new(),
            current_return_type: None,
        };
        checker.register_builtins();
        checker
    }

    pub fn check_program(mut self, stmts: &[Stmt]) -> Result<(), Vec<Diagnostic>> {
        self.collect_top_level_declarations(stmts);
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics)
        }
    }

    fn register_builtins(&mut self) {
        self.functions.insert(
            "print".to_string(),
            FunctionSig {
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "eprint".to_string(),
            FunctionSig {
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "RustServer".to_string(),
            FunctionSig {
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                return_type: Type::Named("ServerHandle".to_string()),
            },
        );
    }

    fn collect_top_level_declarations(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::StructDecl { name, fields, .. } => {
                    let fields = fields
                        .iter()
                        .map(|(field_name, type_name)| {
                            (field_name.clone(), self.parse_type_name(type_name))
                        })
                        .collect();
                    self.structs.insert(name.clone(), StructInfo { fields });
                }
                Stmt::EnumDecl { name, variants, .. } => {
                    let mut map = HashMap::new();
                    for variant in variants {
                        match variant {
                            EnumVariant::Unit(variant_name) => {
                                map.insert(
                                    variant_name.clone(),
                                    VariantInfo {
                                        tuple_items: Vec::new(),
                                        struct_fields: Vec::new(),
                                    },
                                );
                            }
                            EnumVariant::Tuple(variant_name, items) => {
                                map.insert(
                                    variant_name.clone(),
                                    VariantInfo {
                                        tuple_items: items
                                            .iter()
                                            .map(|item| self.parse_type_name(item))
                                            .collect(),
                                        struct_fields: Vec::new(),
                                    },
                                );
                            }
                            EnumVariant::Struct(variant_name, fields) => {
                                map.insert(
                                    variant_name.clone(),
                                    VariantInfo {
                                        tuple_items: Vec::new(),
                                        struct_fields: fields
                                            .iter()
                                            .map(|(field_name, type_name)| {
                                                (
                                                    field_name.clone(),
                                                    self.parse_type_name(type_name),
                                                )
                                            })
                                            .collect(),
                                    },
                                );
                            }
                        }
                    }
                    self.enums.insert(name.clone(), EnumInfo { variants: map });
                }
                Stmt::FuncDecl {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    self.functions.insert(
                        name.clone(),
                        FunctionSig {
                            params: params
                                .iter()
                                .map(|param| ParamInfo {
                                    name: param.name.clone(),
                                    ty: self.parse_type_name(&param.type_name),
                                    is_ref: param.is_ref,
                                    is_mut: param.is_mut,
                                })
                                .collect(),
                            return_type: return_type
                                .as_ref()
                                .map(|ret| self.parse_type_name(ret))
                                .unwrap_or(Type::Nil),
                        },
                    );
                }
                Stmt::ImplDecl { target_type, methods, .. } => {
                    for method in methods {
                        if let Stmt::FuncDecl { name, params, return_type, .. } = method {
                            let params_info: Vec<ParamInfo> = params
                                .iter()
                                .map(|param| ParamInfo {
                                    name: param.name.clone(),
                                    ty: self.parse_type_name(&param.type_name),
                                    is_ref: param.is_ref,
                                    is_mut: param.is_mut,
                                })
                                .collect();
                                
                            let ret_type = return_type
                                .as_ref()
                                .map(|ret| self.parse_type_name(ret))
                                .unwrap_or(Type::Nil);
                                
                            self.methods
                                .entry(target_type.clone())
                                .or_default()
                                .insert(
                                    name.clone(),
                                    FunctionSig {
                                        params: params_info,
                                        return_type: ret_type,
                                    },
                                );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ImportDecl { path, .. } => {
                if let Some(last) = path.last() {
                    self.define_var(
                        last.clone(),
                        VarInfo {
                            ty: Type::Unknown,
                            is_mut: false,
                        },
                    );
                }
            }
            Stmt::ExportDecl(inner, _) => self.check_stmt(inner),
            Stmt::LetDecl {
                name,
                is_mut,
                type_ann,
                value,
                span,
            }
            | Stmt::ConstDecl {
                name,
                is_mut,
                type_ann,
                value,
                span,
            } => {
                let value_ty = self.infer_expr_type(value);
                let declared_ty = type_ann
                    .as_ref()
                    .map(|type_name| self.parse_type_name(type_name));

                if let Some(expected) = &declared_ty {
                    self.expect_assignable(expected, &value_ty, span, "variable initializer");
                }

                if !name.starts_with('(') {
                    let stored_ty = match (&declared_ty, &value_ty) {
                        (Some(Type::Enum(expected)), Type::EnumVariant { enum_name, .. })
                            if expected == enum_name =>
                        {
                            value_ty.clone()
                        }

                        (Some(expected), _) => expected.clone(),

                        (None, _) => value_ty.clone(),
                    };

                    self.define_var(
                        name.clone(),
                        VarInfo {
                            ty: stored_ty,
                            is_mut: *is_mut,
                        },
                    );
                }
            }
            Stmt::FuncDecl {
                params,
                return_type,
                body,
                ..
            } => {
                let prev_return = self.current_return_type.clone();
                self.current_return_type = Some(
                    return_type
                        .as_ref()
                        .map(|ret| self.parse_type_name(ret))
                        .unwrap_or(Type::Nil),
                );

                self.push_scope();
                for Param {
                    name,
                    type_name,
                    is_mut,
                    ..
                } in params
                {
                    self.define_var(
                        name.clone(),
                        VarInfo {
                            ty: self.parse_type_name(type_name),
                            is_mut: *is_mut,
                        },
                    );
                }
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                self.current_return_type = prev_return;
            }
            Stmt::IfStmt {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_ty = self.infer_expr_type(cond);
                self.expect_assignable(&Type::Bool, &cond_ty, &cond.span(), "if condition");

                self.push_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt);
                }
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    for stmt in else_branch {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }
            }
            Stmt::WhileStmt { cond, body, .. } => {
                let cond_ty = self.infer_expr_type(cond);
                self.expect_assignable(&Type::Bool, &cond_ty, &cond.span(), "while condition");
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }
            Stmt::ForStmt {
                var_name,
                iterable,
                body,
                ..
            } => {
                let item_ty = match self.infer_expr_type(iterable) {
                    Type::Tuple(items) => items.first().cloned().unwrap_or(Type::Unknown),
                    Type::Vector(item) => (*item).clone(),
                    _ => Type::Unknown,
                };
                self.push_scope();
                self.define_var(
                    var_name.clone(),
                    VarInfo {
                        ty: item_ty,
                        is_mut: false,
                    },
                );
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }
            Stmt::LoopStmt { body, .. } => {
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }
            Stmt::ReturnStmt(value, span) => {
                let actual = value
                    .as_ref()
                    .map(|expr| self.infer_expr_type(expr))
                    .unwrap_or(Type::Nil);
                if let Some(expected) = self.current_return_type.clone() {
                    self.expect_assignable(&expected, &actual, span, "return value");
                }
            }
            Stmt::ExprStmt(expr) => {
                self.infer_expr_type(expr);
            }
            Stmt::DeferStmt(inner, _) => self.check_stmt(inner),
            Stmt::MatchStmt { target, arms, .. } => {
                let _target_ty = self.infer_expr_type(target);
                for arm in arms {
                    self.infer_expr_type(&arm.body);
                }
            }
            Stmt::StructDecl { .. }
            | Stmt::EnumDecl { .. }
            | Stmt::TraitDecl { .. }
            | Stmt::ImplDecl { .. }
            | Stmt::Break(_)
            | Stmt::Continue(_)
            | Stmt::PluginDecl { .. } => {}
        }
    }

    fn infer_expr_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                LiteralValue::Int(_) => Type::Int,
                LiteralValue::Float(_) => Type::Float,
                LiteralValue::String(_) => Type::String,
                LiteralValue::Bool(_) => Type::Bool,
                LiteralValue::Nil => Type::Nil,
            },
            Expr::Identifier(name, span) => {
                if let Some(var) = self.lookup_var(name) {
                    var.ty.clone()
                } else if self.structs.contains_key(name) {
                    Type::Named(name.clone())
                } else if self.enums.contains_key(name) {
                    Type::Enum(name.clone())
                } else if self.functions.contains_key(name) {
                    Type::Named("Function".to_string())
                } else {
                    self.error(
                        format!("undefined identifier '{}'", name),
                        span.clone(),
                        Some("This name is not declared in the current scope".to_string()),
                        None,
                    );
                    Type::Unknown
                }
            }
            Expr::Tuple(items, _) => Type::Tuple(
                items
                    .iter()
                    .map(|item| self.infer_expr_type(item))
                    .collect(),
            ),
            Expr::VectorLiteral(items, _) => {
                let mut unique_types = HashSet::new();
                let mut inferred = Vec::new();
                for item in items {
                    let item_ty = self.infer_expr_type(item);
                    unique_types.insert(self.type_key(&item_ty));
                    inferred.push(item_ty);
                }
                if unique_types.len() > 1 {
                    self.error(
                        "vector literal contains incompatible element types".to_string(),
                        expr.span(),
                        Some("All elements in a vector should have the same type".to_string()),
                        None,
                    );
                }
                Type::Vector(Box::new(inferred.first().cloned().unwrap_or(Type::Unknown)))
            }
            Expr::Formula(pairs, _) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), self.infer_formula_value_type(v));
                }
                Type::Formula(map)
            }
            Expr::InterpolatedString(segments, span) => {
                for segment in segments {
                    if let crate::parser::InterpolatedSegment::Expr(inner) = segment {
                        self.infer_expr_type(inner);
                    }
                }
                let _ = span;
                Type::String
            }
            Expr::Borrow(inner, is_mut, _) => Type::Reference {
                inner: Box::new(self.infer_expr_type(inner)),
                mutable: *is_mut,
            },
            Expr::Await(inner, _) => self.infer_expr_type(inner),
            Expr::ThreadSpawn(_, _) => Type::Named("ThreadHandle".to_string()),
            Expr::Block(stmts, _) => {
                self.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                Type::Nil
            }
            Expr::Binary(left, op, right, span) => self.infer_binary_type(left, op, right, span),
            Expr::Dot(inner, member, span) => self.infer_dot_type(inner, member, span),
            Expr::StructInit(inner, fields, span) => {
                self.infer_struct_init_type(inner, fields, span)
            }
            Expr::Call(callee, args, span) => self.infer_call_type(callee, args, span),
        }
    }

    fn infer_binary_type(&mut self, left: &Expr, op: &BinaryOp, right: &Expr, span: &Span) -> Type {
        if matches!(op, BinaryOp::Assign) {
            if let Expr::Identifier(name, left_span) = left {
                let rhs_ty = self.infer_expr_type(right);
                if let Some(var) = self.lookup_var(name).cloned() {
                    if !var.is_mut {
                        self.error(
                            format!("cannot assign to immutable variable '{}'", name),
                            left_span.clone(),
                            Some(
                                "Declare it with 'let mut' if reassignment is intended".to_string(),
                            ),
                            None,
                        );
                    }
                    self.expect_assignable(&var.ty, &rhs_ty, span, "assignment");
                } else {
                    self.error(
                        format!("cannot assign to undefined variable '{}'", name),
                        left_span.clone(),
                        None,
                        None,
                    );
                }
                return rhs_ty;
            } else if let Expr::Dot(inner, member, _) = left {
                // con.name = expr: resolve the field type and ensure RHS matches
                let lhs_ty = self.infer_dot_type(inner, member, span);
                let rhs_ty = self.infer_expr_type(right);
                self.expect_assignable(&lhs_ty, &rhs_ty, span, "field assignment");
                return rhs_ty;
            }

            self.error(
                "left-hand side of assignment must be an identifier or field".to_string(),
                span.clone(),
                None,
                None,
            );
            return Type::Unknown;
        }

        let left_ty = self.infer_expr_type(left);
        let right_ty = self.infer_expr_type(right);
        match op {
            BinaryOp::Add => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else if matches!(left_ty, Type::String) && matches!(right_ty, Type::String) {
                    Type::String
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                if !self.is_compatible(&left_ty, &right_ty) {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    Type::Bool
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Bool
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                self.expect_assignable(&Type::Bool, &left_ty, span, "logical expression");
                self.expect_assignable(&Type::Bool, &right_ty, span, "logical expression");
                Type::Bool
            }
            BinaryOp::Range => {
                self.expect_assignable(&Type::Int, &left_ty, span, "range start");
                self.expect_assignable(&Type::Int, &right_ty, span, "range end");
                Type::Named("Range".to_string())
            }
            BinaryOp::Assign => Type::Unknown,
        }
    }

    fn infer_dot_type(&mut self, inner: &Expr, member: &str, span: &Span) -> Type {
        let mut inner_ty = self.infer_expr_type(inner);
        if let Type::Reference { inner: ref_inner, .. } = inner_ty {
            inner_ty = *ref_inner;
        }
        match inner_ty {
            Type::Enum(enum_name) => {
                if let Some(info) = self.enums.get(&enum_name) {
                    if let Some(variant) = info.variants.get(member) {
                        let struct_fields = variant
                            .struct_fields
                            .iter()
                            .map(|(name, ty)| (name.clone(), ty.clone()))
                            .collect();
                        return Type::EnumVariant {
                            enum_name,
                            variant_name: member.to_string(),
                            tuple_items: variant.tuple_items.clone(),
                            struct_fields,
                        };
                    }
                }
                self.error(
                    format!("enum '{}' has no variant '{}'", enum_name, member),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
            Type::EnumVariant {
                enum_name,
                variant_name,
                tuple_items,
                struct_fields,
            } => {
                if let Some(field_ty) = struct_fields.get(member) {
                    return field_ty.clone();
                }

                // Delegate member access to single-item tuple Formula payloads.
                if let [Type::Formula(fmap)] = tuple_items.as_slice() {
                    if let Some(t) = fmap.get(member) {
                        return t.clone();
                    } else {
                        return Type::Unknown;
                    }
                }

                self.error(
                    format!(
                        "variant '{}.{}' has no field '{}'",
                        enum_name, variant_name, member
                    ),
                    span.clone(),
                    None,
                    None,
                );

                Type::Unknown
            }

            Type::Struct(struct_name) => {
                if let Some(info) = self.structs.get(&struct_name) {
                    if let Some((_, ty)) = info.fields.iter().find(|(name, _)| name == member) {
                        return ty.clone();
                    }
                }
                
                if let Some(methods) = self.methods.get(&struct_name) {
                    if methods.contains_key(member) {
                        return Type::Named("Function".into());
                    }
                }
                
                self.error(
                    format!("struct '{}' has no field or method '{}'", struct_name, member),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
            Type::Formula(ref fmap) => fmap.get(member).cloned().unwrap_or(Type::Unknown),
            Type::Unknown | Type::Named(_) => Type::Unknown,
            other => {
                self.error(
                    format!(
                        "cannot access member '{}' on value of type {}",
                        member,
                        self.format_type(&other)
                    ),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
        }
    }

    fn infer_struct_init_type(
        &mut self,
        inner: &Expr,
        fields: &[(String, Expr)],
        span: &Span,
    ) -> Type {
        let base_ty = self.infer_expr_type(inner);
        match base_ty {
            Type::EnumVariant {
                enum_name,
                variant_name,
                tuple_items,
                struct_fields,
            } => {
                if !tuple_items.is_empty() {
                    self.error(
                        format!(
                            "variant '{}.{}' is a tuple variant, not a struct variant",
                            enum_name, variant_name
                        ),
                        span.clone(),
                        None,
                        None,
                    );
                    return Type::Unknown;
                }

                let mut seen = HashSet::new();
                for (field_name, field_expr) in fields {
                    seen.insert(field_name.clone());
                    let actual = self.infer_expr_type(field_expr);
                    if let Some(expected) = struct_fields.get(field_name) {
                        self.expect_assignable(
                            expected,
                            &actual,
                            &field_expr.span(),
                            "enum field initializer",
                        );
                    } else {
                        self.error(
                            format!(
                                "unknown field '{}' for variant '{}.{}'",
                                field_name, enum_name, variant_name
                            ),
                            field_expr.span(),
                            None,
                            None,
                        );
                    }
                }

                for required in struct_fields.keys() {
                    if !seen.contains(required) {
                        self.error(
                            format!(
                                "missing field '{}' for variant '{}.{}'",
                                required, enum_name, variant_name
                            ),
                            span.clone(),
                            None,
                            None,
                        );
                    }
                }

                Type::EnumVariant {
                    enum_name,
                    variant_name,
                    tuple_items: Vec::new(),
                    struct_fields,
                }
            }
            _ => {
                self.error(
                    "struct-style initialization requires an enum struct variant constructor"
                        .to_string(),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
        }
    }

    fn infer_call_type(
        &mut self,
        callee: &Expr,
        args: &[(Option<String>, Expr)],
        span: &Span,
    ) -> Type {
        if let Expr::Identifier(name, _) = callee {
            if let Some(sig) = self.functions.get(name).cloned() {
                self.check_call_args(&sig.params, args, span, name);
                return sig.return_type;
            }

            if let Some(struct_info) = self.structs.get(name).cloned() {
                self.check_struct_constructor_args(name, &struct_info, args, span);
                return Type::Struct(name.clone());
            }
        }

        if let Expr::Dot(inner, member, _) = callee {
            let mut inner_ty = self.infer_expr_type(inner);
            if let Type::Reference { inner: ref_inner, .. } = inner_ty {
                inner_ty = *ref_inner;
            }
            
            if let Type::Struct(struct_name) = &inner_ty {
                let sig_opt = self
                    .methods
                    .get(struct_name)
                    .and_then(|methods| methods.get(member))
                    .cloned();
                    
                if let Some(sig) = sig_opt {
                    let params_to_check = if !sig.params.is_empty() {
                        &sig.params[1..]
                    } else {
                        &[]
                    };
                    self.check_call_args(params_to_check, args, span, member);
                    return sig.return_type;
                }
                
                self.error(
                    format!("struct '{}' has no method '{}'", struct_name, member),
                    span.clone(),
                    None,
                    None,
                );
                return Type::Unknown;
            }

            if let Type::Enum(enum_name) = inner_ty {
                if let Some(variant) = self
                    .enums
                    .get(&enum_name)
                    .and_then(|ei| ei.variants.get(member))
                    .cloned()
                {
                    if variant.struct_fields.is_empty() && !variant.tuple_items.is_empty() {
                        if variant.tuple_items.len() != args.len() {
                            self.error(
                                format!(
                                    "enum constructor '{}.{}' expects {} argument(s), got {}",
                                    enum_name,
                                    member,
                                    variant.tuple_items.len(),
                                    args.len()
                                ),
                                span.clone(),
                                None,
                                None,
                            );
                        }
                        for (idx, expected) in variant.tuple_items.iter().enumerate() {
                            if let Some((_, arg)) = args.get(idx) {
                                let actual = self.infer_expr_type(arg);
                                self.expect_assignable(
                                    expected,
                                    &actual,
                                    &arg.span(),
                                    "enum constructor argument",
                                );
                            }
                        }
                        return Type::EnumVariant {
                            enum_name,
                            variant_name: member.clone(),
                            tuple_items: variant.tuple_items.clone(),
                            struct_fields: HashMap::new(),
                        };
                    }
                }
            }
        }

        for (_, arg) in args {
            self.infer_expr_type(arg);
        }
        let _ = self.infer_expr_type(callee);
        Type::Unknown
    }

    fn check_call_args(
        &mut self,
        params: &[ParamInfo],
        args: &[(Option<String>, Expr)],
        span: &Span,
        name: &str,
    ) {
        if name != "print" && name != "eprint" && args.len() != params.len() {
            self.error(
                format!(
                    "function '{}' expects {} argument(s), got {}",
                    name,
                    params.len(),
                    args.len()
                ),
                span.clone(),
                None,
                None,
            );
        }

        for (idx, (_, arg)) in args.iter().enumerate() {
            let actual = self.infer_expr_type(arg);
            if name == "print" || name == "eprint" {
                continue;
            }
            if let Some(param) = params.get(idx) {
                if param.is_ref {
                    match &actual {
                        Type::Reference { inner, mutable } => {
                            if param.is_mut && !mutable {
                                self.error(
                                    format!(
                                        "parameter '{}' requires '&mut {}' but argument is not mutable",
                                        param.name,
                                        self.format_type(&param.ty)
                                    ),
                                    arg.span(),
                                    None,
                                    None,
                                );
                            }
                            self.expect_assignable(
                                &param.ty,
                                inner,
                                &arg.span(),
                                "function argument (by reference)",
                            );
                        }
                        _ => {
                            self.error(
                                format!(
                                    "parameter '{}' expects a reference to {}",
                                    param.name,
                                    self.format_type(&param.ty)
                                ),
                                arg.span(),
                                None,
                                None,
                            );
                        }
                    }
                } else {
                    self.expect_assignable(&param.ty, &actual, &arg.span(), "function argument");
                }
            }
        }
    }

    fn check_struct_constructor_args(
        &mut self,
        name: &str,
        struct_info: &StructInfo,
        args: &[(Option<String>, Expr)],
        span: &Span,
    ) {
        let named = args.iter().any(|(arg_name, _)| arg_name.is_some());
        if named {
            for (field_name, field_ty) in &struct_info.fields {
                if let Some((_, expr)) = args.iter().find(|(arg_name, _)| {
                    arg_name.as_ref().map(|n| n == field_name).unwrap_or(false)
                }) {
                    let actual = self.infer_expr_type(expr);
                    self.expect_assignable(
                        field_ty,
                        &actual,
                        &expr.span(),
                        "struct field initializer",
                    );
                } else {
                    self.error(
                        format!("missing field '{}' for struct '{}'", field_name, name),
                        span.clone(),
                        None,
                        None,
                    );
                }
            }
            for (arg_name, expr) in args {
                if let Some(arg_name) = arg_name {
                    if !struct_info
                        .fields
                        .iter()
                        .any(|(field_name, _)| field_name == arg_name)
                    {
                        self.error(
                            format!("unknown field '{}' for struct '{}'", arg_name, name),
                            expr.span(),
                            None,
                            None,
                        );
                    }
                }
            }
        } else {
            if args.len() != struct_info.fields.len() {
                self.error(
                    format!(
                        "struct '{}' expects {} argument(s), got {}",
                        name,
                        struct_info.fields.len(),
                        args.len()
                    ),
                    span.clone(),
                    None,
                    None,
                );
            }
            for (idx, (_, expr)) in args.iter().enumerate() {
                if let Some((_, expected)) = struct_info.fields.get(idx) {
                    let actual = self.infer_expr_type(expr);
                    self.expect_assignable(
                        expected,
                        &actual,
                        &expr.span(),
                        "struct constructor argument",
                    );
                }
            }
        }
    }

    fn expect_assignable(&mut self, expected: &Type, actual: &Type, span: &Span, context: &str) {
        if !self.is_compatible(expected, actual) {
            self.error(
                format!(
                    "type mismatch in {}: expected {}, found {}",
                    context,
                    self.format_type(expected),
                    self.format_type(actual)
                ),
                span.clone(),
                None,
                None,
            );
        }
    }

    fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if matches!(expected, Type::Unknown) || matches!(actual, Type::Unknown) {
            return true;
        }
        if expected == actual {
            return true;
        }

        match (expected, actual) {
            (Type::Float, Type::Int) => true,
            (Type::Enum(expected_name), Type::EnumVariant { enum_name, .. }) => {
                expected_name == enum_name
            }
            (Type::Named(expected_name), Type::Struct(actual_name))
            | (Type::Named(expected_name), Type::Enum(actual_name)) => expected_name == actual_name,
            (Type::Struct(expected_name), Type::Named(actual_name))
            | (Type::Enum(expected_name), Type::Named(actual_name)) => expected_name == actual_name,
            (Type::Named(expected_name), Type::EnumVariant { enum_name, .. }) => {
                expected_name == enum_name
            }
            (Type::Vector(expected_item), Type::Vector(actual_item)) => {
                self.is_compatible(expected_item, actual_item)
            }
            (Type::Tuple(expected_items), Type::Tuple(actual_items)) => {
                expected_items.len() == actual_items.len()
                    && expected_items
                        .iter()
                        .zip(actual_items.iter())
                        .all(|(expected, actual)| self.is_compatible(expected, actual))
            }
            (
                Type::Reference {
                    inner: expected,
                    mutable: em,
                },
                Type::Reference {
                    inner: actual,
                    mutable: am,
                },
            ) => em == am && self.is_compatible(expected, actual),
            (Type::Named(expected_name), Type::Named(actual_name)) => expected_name == actual_name,
            (Type::Formula(_), Type::Formula(_)) => true,
            _ => false,
        }
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }

    fn parse_type_name(&self, type_name: &str) -> Type {
        let trimmed = type_name.trim();
        match trimmed {
            "Int" | "I32" | "I64" | "U32" | "U64" => Type::Int,
            "Float" | "F32" | "F64" => Type::Float,
            "String" => Type::String,
            "Bool" => Type::Bool,
            "Nil" | "nil" => Type::Nil,
            "Formula" => Type::Formula(HashMap::new()),
            _ if trimmed.starts_with('[') && trimmed.ends_with(']') => {
                let inner = &trimmed[1..trimmed.len() - 1];
                Type::Vector(Box::new(self.parse_type_name(inner)))
            }
            _ if trimmed.starts_with('(') && trimmed.ends_with(')') => {
                let inner = &trimmed[1..trimmed.len() - 1];
                if inner.trim().is_empty() {
                    Type::Tuple(Vec::new())
                } else {
                    Type::Tuple(
                        inner
                            .split(',')
                            .map(|part| self.parse_type_name(part.trim()))
                            .collect(),
                    )
                }
            }
            _ if self.structs.contains_key(trimmed) => Type::Struct(trimmed.to_string()),
            _ if self.enums.contains_key(trimmed) => Type::Enum(trimmed.to_string()),
            _ => Type::Named(trimmed.to_string()),
        }
    }

    fn infer_formula_value_type(&self, val: &FormulaValue) -> Type {
        match val {
            FormulaValue::Literal(lit) => match lit {
                LiteralValue::Int(_) => Type::Int,
                LiteralValue::Float(_) => Type::Float,
                LiteralValue::String(_) => Type::String,
                LiteralValue::Bool(_) => Type::Bool,
                LiteralValue::Nil => Type::Nil,
            },
            FormulaValue::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), self.infer_formula_value_type(v));
                }
                Type::Formula(map)
            }
            FormulaValue::List(items) => {
                let mut iter = items.iter().map(|i| self.infer_formula_value_type(i));
                if let Some(first) = iter.next() {
                    let same = iter.all(|t| t == first);
                    if same {
                        Type::Vector(Box::new(first))
                    } else {
                        Type::Vector(Box::new(Type::Unknown))
                    }
                } else {
                    Type::Vector(Box::new(Type::Unknown))
                }
            }
        }
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::String => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Nil => "Nil".to_string(),
            Type::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .map(|item| self.format_type(item))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Vector(item) => format!("[{}]", self.format_type(item)),
            Type::Formula(_) => "Formula".to_string(),
            Type::Struct(name) | Type::Enum(name) | Type::Named(name) => name.clone(),
            Type::EnumVariant {
                enum_name,
                variant_name,
                ..
            } => format!("{}.{}", enum_name, variant_name),
            Type::Unknown => "Unknown".to_string(),
            Type::Reference { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", self.format_type(inner))
                } else {
                    format!("&{}", self.format_type(inner))
                }
            }
        }
    }

    fn error_binary_mismatch(&mut self, op: &BinaryOp, left: &Type, right: &Type, span: &Span) {
        self.error(
            format!(
                "operator {:?} cannot be applied to {} and {}",
                op,
                self.format_type(left),
                self.format_type(right)
            ),
            span.clone(),
            None,
            None,
        );
    }

    fn type_key(&self, ty: &Type) -> String {
        self.format_type(ty)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: String, info: VarInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn error(
        &mut self,
        message: String,
        span: Span,
        label: Option<String>,
        suggestion: Option<String>,
    ) {
        self.diagnostics.push(Diagnostic::new_error(
            message,
            self.filepath.clone(),
            span,
            label,
            suggestion,
        ));
    }
}
