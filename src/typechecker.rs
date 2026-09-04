use crate::diagnostics::Diagnostic;
use crate::lexer::Span;
use crate::parser::{Annotation, BinaryOp, EnumVariant, Expr, LiteralValue, Param, Stmt, UnaryOp};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Nil,
    Byte,
    Tuple(Vec<Type>),
    Vector(Box<Type>),
    Formula(HashMap<String, Type>, HashMap<String, String>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Enum(String),
    EnumVariant {
        enum_name: String,
        variant_name: String,
        tuple_items: Vec<Type>,
        struct_fields: HashMap<String, Type>,
    },
    Named(String),
    Quantity(HashMap<String, i32>),
    Unit(HashMap<String, i32>),
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
    hover_doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Type,
    pub is_ref: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub params: Vec<ParamInfo>,
    pub return_type: Type,
    pub hover_doc: Option<String>,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: Vec<(String, Type)>,
    pub hover_doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub tuple_items: Vec<Type>,
    pub struct_fields: Vec<(String, Type)>,
    pub hover_doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub variants: HashMap<String, VariantInfo>,
    pub hover_doc: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandInfo {
    pub name: String,
    pub about: Option<String>,
    pub func_name: String,
    pub params: Vec<ParamInfo>,
    pub hover_doc: String,
    pub span: Span,
}

pub struct TypeChecker {
    filepath: String,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, VarInfo>>,
    pub functions: HashMap<String, FunctionSig>,
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub methods: HashMap<String, HashMap<String, FunctionSig>>,
    pub commands: HashMap<String, CommandInfo>,
    current_return_type: Option<Type>,
    pub hover_info: HashMap<Span, String>,
    pub modules: HashSet<String>,
    pub module_docs: HashMap<String, String>,
    pub plugins: HashSet<String>,
    pub plugin_methods: HashMap<String, HashMap<String, Type>>,
    pub plugin_functions: HashMap<String, HashMap<String, FunctionSig>>,
    pub annotations: HashSet<String>,
    pub is_importing: bool,
    pub defined_functions_in_file: HashSet<String>,
}

impl TypeChecker {
    pub fn insert_hover_info(&mut self, span: crate::lexer::Span, info: String) {
        if !self.is_importing {
            self.hover_info.insert(span, info);
        }
    }

    pub fn new(filepath: String) -> Self {
        let mut checker = Self {
            filepath,
            diagnostics: Vec::new(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            methods: HashMap::new(),
            commands: HashMap::new(),
            current_return_type: None,
            hover_info: HashMap::new(),
            modules: HashSet::new(),
            module_docs: HashMap::new(),
            plugins: HashSet::new(),
            plugin_methods: HashMap::new(),
            plugin_functions: HashMap::new(),
            annotations: HashSet::new(),
            is_importing: false,
            defined_functions_in_file: HashSet::new(),
        };
        checker.register_builtins();
        checker
    }

    pub fn check_program(mut self, stmts: &[Stmt]) -> (Result<(), Vec<Diagnostic>>, Self) {
        self.collect_top_level_declarations(stmts);
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        let res = if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics.clone())
        };
        (res, self)
    }

    fn process_annotations(&mut self, annotations: &[Annotation]) -> Option<String> {
        let mut docs = Vec::new();
        for ann in annotations {
            if ann.name == "Docs" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Docs(text: String)\n```\n\nAdds documentation to declarations. This documentation will appear when hovering over the declared item.\n\n**Example:**\n```flame\n@Docs(\"Adds two numbers\")\nfn add(a: Int, b: Int) -> Int {\n    return a + b\n}\n```".to_string()
                );

                if !ann.args.is_empty() {
                    let mut raw = ann.args[0].clone();
                    if raw.starts_with('"') && raw.ends_with('"') {
                        raw = raw[1..raw.len() - 1].to_string();
                    }
                    let unquoted = raw.replace("\\n", "\n");
                    let lines: Vec<&str> = unquoted.lines().collect();
                    let mut min_indent = usize::MAX;
                    for line in &lines {
                        if line.trim().is_empty() {
                            continue;
                        }
                        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                        if indent < min_indent {
                            min_indent = indent;
                        }
                    }
                    let mut cleaned_doc = String::new();
                    for line in &lines {
                        if line.trim().is_empty() {
                            cleaned_doc.push('\n');
                        } else {
                            let indent = if min_indent == usize::MAX {
                                0
                            } else {
                                min_indent
                            };
                            let slice_start = std::cmp::min(indent, line.len());
                            cleaned_doc.push_str(&line[slice_start..]);
                            cleaned_doc.push('\n');
                        }
                    }
                    docs.push(cleaned_doc.trim().to_string());
                }
            } else if ann.name == "Test" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Test\n```\n\nMarks a function as a unit test. It will be executed by the test runner.".to_string()
                );
                docs.push("**@Test Function**\nThis function is a unit test case.".to_string());
            } else if ann.name == "Requires" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Requires(...modules: String)\n```\n\nSpecifies module dependencies required by this function.".to_string()
                );
                docs.push(format!(
                    "**Requires Dependencies:** `{}`",
                    ann.args.join(", ")
                ));
            } else if ann.name == "Permission" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Permission(...permissions: String)\n```\n\nRequests specific runtime permissions for this function.".to_string()
                );
                docs.push(format!(
                    "**Required Permissions:** `{}`",
                    ann.args.join(", ")
                ));
            } else if ann.name == "Suggestions" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Suggestions([{name: String, kind: String}])\n```\n\nProvides custom suggestions for IDE autocompletion when typing the package name.".to_string()
                );
            } else if ann.name == "Command" {
                self.insert_hover_info(
                    ann.name_span.clone(),
                    "```flame\n@Command(name: String, about: String)\n```\n\nDeclares a CLI subcommand handled by this function.".to_string()
                );
                let mut about = None;
                for (idx, arg) in ann.args.iter().enumerate() {
                    let trimmed = arg.trim();
                    if trimmed.starts_with("about:") || trimmed.starts_with("about=") || trimmed.starts_with("about :") || trimmed.starts_with("about =") {
                        let val = if let Some((_, v)) = trimmed.split_once(':') { v } else if let Some((_, v)) = trimmed.split_once('=') { v } else { trimmed };
                        about = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                    } else if trimmed.starts_with("description:") || trimmed.starts_with("description=") {
                        let val = if let Some((_, v)) = trimmed.split_once(':') { v } else if let Some((_, v)) = trimmed.split_once('=') { v } else { trimmed };
                        about = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                    } else if !trimmed.starts_with("name:") && !trimmed.starts_with("name=") && idx == 1 && about.is_none() {
                        about = Some(trimmed.trim_matches('"').trim_matches('\'').to_string());
                    }
                }
                if let Some(ab) = about {
                    docs.push(ab);
                }
            } else if let Some(func) = self.functions.get(&ann.name).cloned() {
                if let Some(doc) = func.hover_doc {
                    self.insert_hover_info(ann.name_span.clone(), doc);
                }
            }
        }
        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n\n---\n\n"))
        }
    }

    fn register_builtins(&mut self) {
        self.functions.insert(
            "print".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some("Prints a value to standard output without a newline.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "eprint".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some("Prints a value to standard error without a newline.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "println".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some(
                    "Prints a value to standard output, followed by a newline.".to_string(),
                ),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "panic".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "message".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some(
                    "Terminates the program immediately with an error message.".to_string(),
                ),
                return_type: Type::Unknown,
            },
        );
        self.functions.insert(
            "assert".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "condition".to_string(),
                    ty: Type::Bool,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some("Asserts that a condition is true. Panics if false.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "RustServer".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "value".to_string(),
                    ty: Type::Unknown,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some("Creates a new native Rust server handle.".to_string()),
                return_type: Type::Named("ServerHandle".to_string()),
            },
        );
        self.functions.insert(
            "input".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![ParamInfo {
                    name: "prompt".to_string(),
                    ty: Type::String,
                    is_ref: false,
                    is_mut: false,
                }],
                hover_doc: Some(
                    "Prompts the user for input from standard input and returns the read string."
                        .to_string(),
                ),
                return_type: Type::String,
            },
        );
        self.functions.insert(
            "assertEq".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![
                    ParamInfo {
                        name: "actual".to_string(),
                        ty: Type::Unknown,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "expected".to_string(),
                        ty: Type::Unknown,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "msg".to_string(),
                        ty: Type::String,
                        is_ref: false,
                        is_mut: false,
                    },
                ],
                hover_doc: Some("Asserts that two values are equal. Panics with the provided message if they are not.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "assertNe".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![
                    ParamInfo {
                        name: "actual".to_string(),
                        ty: Type::Unknown,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "expected".to_string(),
                        ty: Type::Unknown,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "msg".to_string(),
                        ty: Type::String,
                        is_ref: false,
                        is_mut: false,
                    },
                ],
                hover_doc: Some("Asserts that two values are not equal. Panics with the provided message if they are.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "assert_true".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![
                    ParamInfo {
                        name: "cond".to_string(),
                        ty: Type::Bool,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "msg".to_string(),
                        ty: Type::String,
                        is_ref: false,
                        is_mut: false,
                    },
                ],
                hover_doc: Some("Asserts that a boolean condition is true. Panics with the provided message if false.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "assert_false".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![
                    ParamInfo {
                        name: "cond".to_string(),
                        ty: Type::Bool,
                        is_ref: false,
                        is_mut: false,
                    },
                    ParamInfo {
                        name: "msg".to_string(),
                        ty: Type::String,
                        is_ref: false,
                        is_mut: false,
                    },
                ],
                hover_doc: Some("Asserts that a boolean condition is false. Panics with the provided message if true.".to_string()),
                return_type: Type::Nil,
            },
        );
        self.functions.insert(
            "mock_api".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![],
                hover_doc: Some("Mocks an API endpoint for testing purposes.".to_string()),
                return_type: Type::Unknown,
            },
        );
        self.functions.insert(
            "mock_data".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![],
                hover_doc: Some("Returns mock data for testing purposes.".to_string()),
                return_type: Type::Unknown,
            },
        );
        self.functions.insert(
            "mock_function".to_string(),
            FunctionSig {
                is_static: false,
                params: vec![],
                hover_doc: Some("Returns a mock function for testing purposes.".to_string()),
                return_type: Type::Nil,
            },
        );

        // Built-in Enums
        let mut result_variants = HashMap::new();
        result_variants.insert("Ok".to_string(), VariantInfo { tuple_items: vec![Type::Unknown], struct_fields: vec![], hover_doc: Some("The successful variant of `Result`, containing the value.\n\n### Example\n```flame\nlet res = Ok(42)\n```".to_string()) });
        result_variants.insert("Err".to_string(), VariantInfo { tuple_items: vec![Type::Unknown], struct_fields: vec![], hover_doc: Some("The error variant of `Result`, containing the error data.\n\n### Example\n```flame\nlet err = Err(\"Something went wrong\")\n```".to_string()) });
        self.enums.insert("Result".to_string(), EnumInfo {
            variants: result_variants,
            hover_doc: Some("`Result` is a generic type that represents either success (`Ok`) or failure (`Err`).\nIt is commonly used for error handling instead of exceptions.\n\n### Example\n```flame\nfn divide(a: Int, b: Int) -> Result<Int, Error> {\n    if b == 0 {\n        return Err(Error { code: 1, message: \"Divide by zero\" })\n    }\n    return Ok(a / b)\n}\n```".to_string()),
        });

        let mut option_variants = HashMap::new();
        option_variants.insert("Some".to_string(), VariantInfo { tuple_items: vec![Type::Unknown], struct_fields: vec![], hover_doc: Some("Contains a value in an `Option`.\n\n### Example\n```flame\nlet val = Some(\"data\")\n```".to_string()) });
        option_variants.insert("None".to_string(), VariantInfo { tuple_items: vec![], struct_fields: vec![], hover_doc: Some("Indicates no value in an `Option`.\n\n### Example\n```flame\nlet empty = None\n```".to_string()) });
        self.enums.insert("Option".to_string(), EnumInfo {
            variants: option_variants,
            hover_doc: Some("`Option` is a generic type that represents an optional value: every `Option` is either `Some` and contains a value, or `None`, and does not.\n\n### Example\n```flame\nlet val = Some(42)\nlet empty = None\n```".to_string()),
        });

        // Built-in Structs
        self.structs.insert("Error".to_string(), StructInfo {
            fields: vec![
                ("message".to_string(), Type::String),
                ("code".to_string(), Type::Named("Int".to_string())),
            ],
            hover_doc: Some("`Error` is a built-in type that represents a standard runtime error.\n\n### Example\n```flame\nlet err = Error { message: \"Not found\", code: 404 }\n```".to_string()),
        });
    }

    fn collect_top_level_declarations(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::StructDecl {
                    name,
                    fields,
                    annotations,
                    span: _,
                    name_span,
                } => {
                    let mut hover_str = format!("```flame\nstruct {}\n```", name);
                    let hover_doc = self.process_annotations(annotations);
                    if let Some(doc) = &hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }
                    self.insert_hover_info(name_span.clone(), hover_str);
                    let fields = fields
                        .iter()
                        .map(|(field_name, type_name)| {
                            (field_name.clone(), self.parse_type_name(type_name))
                        })
                        .collect();
                    self.structs
                        .insert(name.clone(), StructInfo { fields, hover_doc });
                }
                Stmt::EnumDecl {
                    name,
                    variants,
                    annotations,
                    span: _,
                    name_span,
                } => {
                    let mut hover_str = format!("```flame\nenum {}\n```", name);
                    let hover_doc = self.process_annotations(annotations);
                    if let Some(doc) = &hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }
                    self.insert_hover_info(name_span.clone(), hover_str);
                    let mut map = HashMap::new();
                    for variant in variants {
                        match variant {
                            EnumVariant::Unit(variant_name) => {
                                map.insert(
                                    variant_name.clone(),
                                    VariantInfo {
                                        tuple_items: Vec::new(),
                                        struct_fields: Vec::new(),
                                        hover_doc: None,
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
                                        hover_doc: None,
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
                                        hover_doc: None,
                                    },
                                );
                            }
                        }
                    }
                    self.enums.insert(
                        name.clone(),
                        EnumInfo {
                            variants: map,
                            hover_doc,
                        },
                    );
                }
                Stmt::FuncDecl {
                    name,
                    params,
                    return_type,
                    annotations,
                    span,
                    ..
                } => {
                    let hover_doc = self.process_annotations(annotations);

                    let mut hover_str = format!("```flame\nfn {}(", name);
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            hover_str.push_str(", ");
                        }
                        let ref_mut = match (p.is_ref, p.is_mut) {
                            (true, true) => "ref mut ",
                            (true, false) => "ref ",
                            (false, true) => "mut ",
                            _ => "",
                        };
                        hover_str.push_str(&format!("{}{}: {}", ref_mut, p.name, p.type_name));
                    }
                    hover_str.push_str(")");
                    if let Some(ret) = return_type {
                        hover_str.push_str(&format!(" -> {}", ret));
                    }
                    hover_str.push_str("\n```");

                    if let Some(doc) = &hover_doc {
                        hover_str.push_str(&format!("\n\n{}", doc));
                    }

                    // Estimate the span of the function name
                    let mut name_span = span.clone();
                    name_span.col += 3; // 'fn ' length
                    name_span.end = name_span.start + name.len();
                    self.insert_hover_info(name_span, hover_str.clone());
                    if let Some(cmd_info) =
                        self.parse_command_annotation(name, annotations, params, span)
                    {
                        self.commands.insert(cmd_info.name.clone(), cmd_info);
                    }
                    let is_builtin_file = self.filepath.ends_with("builtins.fm");
                    if self.defined_functions_in_file.contains(name)
                        || (self.functions.contains_key(name) && !is_builtin_file)
                    {
                        self.diagnostics
                            .push(crate::diagnostics::Diagnostic::new_error(
                                format!(
                                    "Duplicate function definition: '{}' is already defined",
                                    name
                                ),
                                self.filepath.clone(),
                                span.clone(),
                                None,
                                None,
                            ));
                    }
                    self.defined_functions_in_file.insert(name.clone());
                    self.functions.insert(
                        name.clone(),
                        FunctionSig {
                            is_static: false,
                            params: params
                                .iter()
                                .map(|param| ParamInfo {
                                    name: param.name.clone(),
                                    ty: self.parse_type_name(&param.type_name),
                                    is_ref: param.is_ref,
                                    is_mut: param.is_mut,
                                })
                                .collect(),
                            hover_doc: hover_doc,
                            return_type: return_type
                                .as_ref()
                                .map(|ret| self.parse_type_name(ret))
                                .unwrap_or(Type::Nil),
                        },
                    );
                }
                Stmt::PackageDecl { .. } => {}
                Stmt::AnnotationDecl {
                    name,
                    params,
                    return_type,
                    annotations,
                    span: _,
                    name_span,
                    ..
                } => {
                    let mut hover_str = format!("```flame\nannotation @{}(", name);
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            hover_str.push_str(", ");
                        }
                        let ref_mut = match (p.is_ref, p.is_mut) {
                            (true, true) => "ref mut ",
                            (true, false) => "ref ",
                            (false, true) => "mut ",
                            _ => "",
                        };
                        hover_str.push_str(&format!("{}{}: {}", ref_mut, p.name, p.type_name));
                    }
                    if let Some(ret) = return_type {
                        hover_str.push_str(&format!(") -> {}\n```", ret));
                    } else {
                        hover_str.push_str(")\n```");
                    }

                    let hover_doc = self.process_annotations(annotations);
                    if let Some(doc) = &hover_doc {
                        hover_str.push_str(&format!("\n\n{}", doc));
                    }
                    self.insert_hover_info(name_span.clone(), hover_str.clone());

                    self.annotations.insert(name.clone());
                    self.functions.insert(
                        name.clone(),
                        FunctionSig {
                            is_static: false,
                            params: params
                                .iter()
                                .map(|param| ParamInfo {
                                    name: param.name.clone(),
                                    ty: self.parse_type_name(&param.type_name),
                                    is_ref: param.is_ref,
                                    is_mut: param.is_mut,
                                })
                                .collect(),
                            hover_doc: hover_doc,
                            return_type: return_type
                                .as_ref()
                                .map(|ret| self.parse_type_name(ret))
                                .unwrap_or(Type::Nil),
                        },
                    );
                }
                Stmt::ImplDecl {
                    target_type,
                    trait_name,
                    methods,
                    annotations,
                    span: _,
                    name_span,
                } => {
                    let mut hover_str = if let Some(tr) = &trait_name {
                        format!("```flame\nimpl {} for {}\n```", tr, target_type)
                    } else {
                        format!("```flame\nimpl {}\n```", target_type)
                    };
                    let hover_doc = self.process_annotations(annotations);
                    if let Some(doc) = hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }
                    self.insert_hover_info(name_span.clone(), hover_str);
                    for method in methods {
                        if let Stmt::FuncDecl {
                            name,
                            params,
                            return_type,
                            annotations,
                            ..
                        } = method
                        {
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

                            let is_static = !params.first().map_or(false, |p| p.name == "self");
                            let hover_doc = self.process_annotations(annotations);
                            self.methods.entry(target_type.clone()).or_default().insert(
                                name.clone(),
                                FunctionSig {
                                    is_static,
                                    params: params_info,
                                    hover_doc,
                                    return_type: ret_type,
                                },
                            );
                        }
                    }
                }
                Stmt::ImportDecl { path, .. } => {
                    if let Some(mod_name) = path.last() {
                        if path.first().map_or(false, |p| p == "native" || p == "std") {
                            self.plugins.insert(mod_name.clone());
                            self.modules.insert(mod_name.clone());
                            if path.first().map_or(false, |p| p == "native") {
                                let mut methods = HashMap::new();
                                let mut p = std::path::Path::new(&self.filepath);
                                let mut fmi_path = None;
                                while let Some(parent) = p.parent() {
                                    let candidate = parent
                                        .join(".flame")
                                        .join("pkg")
                                        .join(mod_name)
                                        .join(format!("{}.fmi", mod_name));
                                    if candidate.exists() {
                                        fmi_path = Some(candidate);
                                        break;
                                    }
                                    p = parent;
                                }

                                if let Some(fmi) = fmi_path {
                                    if let Ok(content) = std::fs::read_to_string(&fmi) {
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(&content)
                                        {
                                            let mut p_funcs = HashMap::new();
                                            if let Some(funcs) =
                                                json.get("functions").and_then(|f| f.as_array())
                                            {
                                                for func in funcs {
                                                    if let (Some(name), Some(ret_str)) = (
                                                        func.get("flame_name")
                                                            .and_then(|n| n.as_str())
                                                            .or(func
                                                                .get("name")
                                                                .and_then(|n| n.as_str())),
                                                        func.get("return_type")
                                                            .and_then(|r| r.as_str()),
                                                    ) {
                                                        let ret_ty = self.parse_type_name(ret_str);
                                                        methods.insert(
                                                            name.to_string(),
                                                            ret_ty.clone(),
                                                        );

                                                        let mut params = Vec::new();
                                                        if let Some(ps) = func
                                                            .get("params")
                                                            .and_then(|p| p.as_array())
                                                        {
                                                            for p in ps {
                                                                if let (
                                                                    Some(p_name),
                                                                    Some(p_ty_str),
                                                                ) = (
                                                                    p.get("name")
                                                                        .and_then(|n| n.as_str()),
                                                                    p.get("type_name")
                                                                        .and_then(|t| t.as_str()),
                                                                ) {
                                                                    params.push(ParamInfo {
                                                                        name: p_name.to_string(),
                                                                        ty: self.parse_type_name(
                                                                            p_ty_str,
                                                                        ),
                                                                        is_ref: p
                                                                            .get("is_ref")
                                                                            .and_then(|r| {
                                                                                r.as_bool()
                                                                            })
                                                                            .unwrap_or(false),
                                                                        is_mut: p
                                                                            .get("is_mut")
                                                                            .and_then(|m| {
                                                                                m.as_bool()
                                                                            })
                                                                            .unwrap_or(false),
                                                                    });
                                                                }
                                                            }
                                                        }

                                                        let doc = func
                                                            .get("docs")
                                                            .and_then(|d| d.as_str())
                                                            .map(|s| s.to_string());

                                                        p_funcs.insert(
                                                            name.to_string(),
                                                            FunctionSig {
                                                                is_static: true,
                                                                params,
                                                                return_type: ret_ty,
                                                                hover_doc: doc,
                                                            },
                                                        );
                                                    }
                                                }
                                            }
                                            self.plugin_functions.insert(mod_name.clone(), p_funcs);
                                            if let Some(structs) =
                                                json.get("structs").and_then(|s| s.as_array())
                                            {
                                                for s in structs {
                                                    if let Some(struct_name) =
                                                        s.get("name").and_then(|n| n.as_str())
                                                    {
                                                        methods.insert(
                                                            struct_name.to_string(),
                                                            Type::Struct(struct_name.to_string()),
                                                        );

                                                        self.structs.insert(struct_name.to_string(), StructInfo {
                                                            fields: Vec::new(),
                                                            hover_doc: Some("**Native Plugin Struct**".to_string()),
                                                        });

                                                        if let Some(s_methods) = s
                                                            .get("methods")
                                                            .and_then(|m| m.as_array())
                                                        {
                                                            let mut struct_methods = HashMap::new();
                                                            for m in s_methods {
                                                                if let (
                                                                    Some(m_name),
                                                                    Some(ret_str),
                                                                ) = (
                                                                    m.get("flame_name")
                                                                        .and_then(|n| n.as_str())
                                                                        .or(m
                                                                            .get("name")
                                                                            .and_then(|n| {
                                                                                n.as_str()
                                                                            })),
                                                                    m.get("return_type")
                                                                        .and_then(|r| r.as_str()),
                                                                ) {
                                                                    let is_static = m
                                                                        .get("is_static")
                                                                        .and_then(|s| s.as_bool())
                                                                        .unwrap_or(false);
                                                                    let mut params = Vec::new();
                                                                    if !is_static {
                                                                        params.push(ParamInfo {
                                                                            name: "self"
                                                                                .to_string(),
                                                                            ty: Type::Struct(
                                                                                struct_name
                                                                                    .to_string(),
                                                                            ),
                                                                            is_ref: true,
                                                                            is_mut: true,
                                                                        });
                                                                    }
                                                                    if let Some(ps) = m
                                                                        .get("params")
                                                                        .and_then(|p| p.as_array())
                                                                    {
                                                                        for p in ps {
                                                                            if let (Some(p_name), Some(p_ty_str)) = (p.get("name").and_then(|n| n.as_str()), p.get("type_name").and_then(|t| t.as_str())) {
                                                                                params.push(ParamInfo {
                                                                                    name: p_name.to_string(),
                                                                                    ty: self.parse_type_name(p_ty_str),
                                                                                    is_ref: p.get("is_ref").and_then(|r| r.as_bool()).unwrap_or(false),
                                                                                    is_mut: p.get("is_mut").and_then(|m| m.as_bool()).unwrap_or(false),
                                                                                });
                                                                            }
                                                                        }
                                                                    }

                                                                    let doc = m
                                                                        .get("docs")
                                                                        .and_then(|d| d.as_str())
                                                                        .map(|s| s.to_string());
                                                                    struct_methods.insert(
                                                                        m_name.to_string(),
                                                                        FunctionSig {
                                                                            is_static,
                                                                            params,
                                                                            return_type: self
                                                                                .parse_type_name(
                                                                                    ret_str,
                                                                                ),
                                                                            hover_doc: doc,
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                            self.methods.insert(
                                                                struct_name.to_string(),
                                                                struct_methods,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Fallback to parsing native/src/lib.rs if .fmi not found
                                    let p = std::path::Path::new(&self.filepath);
                                    if let Some(parent) = p.parent() {
                                        if let Some(root) = parent.parent() {
                                            let lib_rs =
                                                root.join("native").join("src").join("lib.rs");
                                            if let Ok(content) = std::fs::read_to_string(&lib_rs) {
                                                for line in content.lines() {
                                                    let line = line.trim();
                                                    if line.starts_with("pub fn ") {
                                                        let rest = &line["pub fn ".len()..];
                                                        if let Some(paren) = rest.find('(') {
                                                            let name =
                                                                rest[..paren].trim().to_string();
                                                            let mut ret_ty = Type::Unknown;
                                                            if let Some(arrow) = rest.find("->") {
                                                                let after_arrow =
                                                                    rest[arrow + 2..].trim();
                                                                let end = after_arrow
                                                                    .find('{')
                                                                    .unwrap_or(after_arrow.len());
                                                                let ret_str =
                                                                    after_arrow[..end].trim();
                                                                if ret_str == "i64"
                                                                    || ret_str == "i32"
                                                                    || ret_str == "usize"
                                                                {
                                                                    ret_ty = Type::Int;
                                                                } else if ret_str == "f64"
                                                                    || ret_str == "f32"
                                                                {
                                                                    ret_ty = Type::Float;
                                                                } else if ret_str == "bool" {
                                                                    ret_ty = Type::Bool;
                                                                } else if ret_str == "String" {
                                                                    ret_ty = Type::String;
                                                                }
                                                            }
                                                            methods.insert(name, ret_ty);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                self.plugin_methods.insert(mod_name.clone(), methods);
                            }
                        } else {
                            self.modules.insert(mod_name.clone());
                            if let Some(file_path) = crate::stdlib::locate_import_file(
                                std::path::Path::new(&self.filepath),
                                path,
                            ) {
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let mut lexer = crate::lexer::Lexer::new(&content);
                                    let mut tokens = Vec::new();
                                    loop {
                                        let tok = lexer.next_token();
                                        let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                                        tokens.push(tok);
                                        if is_eof {
                                            break;
                                        }
                                    }
                                    let mut parser = crate::parser::Parser::new(
                                        tokens,
                                        file_path.to_string_lossy().to_string(),
                                    );
                                    if let Ok(parsed_stmts) = parser.parse() {
                                        let prev = self.is_importing;
                                        self.is_importing = true;
                                        self.collect_top_level_declarations(&parsed_stmts);
                                        self.is_importing = prev;
                                    }
                                }
                            }
                        }
                    }
                }
                Stmt::ExportDecl(inner, _) => {
                    self.collect_top_level_declarations(std::slice::from_ref(inner.as_ref()));
                }
                _ => {}
            }
        }
    }

    fn get_std_module_type(&self, mod_name: &str) -> Type {
        match mod_name {
            "time" => {
                let mut map = HashMap::new();
                let mut docs = HashMap::new();

                let mut ts_map = HashMap::new();
                ts_map.insert("millis".to_string(), Type::Int);
                ts_map.insert(
                    "toMillis".to_string(),
                    Type::Function(vec![], Box::new(Type::Int)),
                );
                ts_map.insert(
                    "toSeconds".to_string(),
                    Type::Function(vec![], Box::new(Type::Int)),
                );
                ts_map.insert(
                    "toString".to_string(),
                    Type::Function(vec![], Box::new(Type::String)),
                );
                let ts_ty = Type::Formula(ts_map, HashMap::new());

                map.insert("now".to_string(), Type::Function(vec![], Box::new(ts_ty)));
                if let Some(doc) = crate::std_docs::get_std_function_doc("std.time", "now") {
                    docs.insert("now".to_string(), doc.to_string());
                }

                Type::Formula(map, docs)
            }
            "unit" => {
                let mut map = HashMap::new();
                let mut docs = HashMap::new();

                let eq_fn = Type::Function(
                    vec![Type::Int, Type::Int, Type::Int],
                    Box::new(Type::Named("Unit".to_string())),
                );

                map.insert("Equation".to_string(), eq_fn);
                map.insert(
                    "meter".to_string(),
                    Type::Quantity(HashMap::from([("m".to_string(), 1)])),
                );
                map.insert(
                    "second".to_string(),
                    Type::Quantity(HashMap::from([("s".to_string(), 1)])),
                );
                map.insert(
                    "kilogram".to_string(),
                    Type::Quantity(HashMap::from([("kg".to_string(), 1)])),
                );

                if let Some(doc) = crate::std_docs::get_std_function_doc("std.unit", "Equation") {
                    docs.insert("Equation".to_string(), doc.to_string());
                }
                docs.insert(
                    "meter".to_string(),
                    "```flame\nunit.meter: Quantity\n```\nThe SI base unit for length.".to_string(),
                );
                docs.insert(
                    "second".to_string(),
                    "```flame\nunit.second: Quantity\n```\nThe SI base unit for time.".to_string(),
                );
                docs.insert(
                    "kilogram".to_string(),
                    "```flame\nunit.kilogram: Quantity\n```\nThe SI base unit for mass."
                        .to_string(),
                );

                Type::Formula(map, docs)
            }
            "math" => {
                let mut map = HashMap::new();
                let mut docs = HashMap::new();

                let float_fn = Type::Function(vec![Type::Unknown], Box::new(Type::Float));
                let float_fn2 =
                    Type::Function(vec![Type::Unknown, Type::Unknown], Box::new(Type::Float));

                map.insert(
                    "pi".to_string(),
                    Type::Function(vec![], Box::new(Type::Float)),
                );
                map.insert(
                    "e".to_string(),
                    Type::Function(vec![], Box::new(Type::Float)),
                );
                map.insert(
                    "inf".to_string(),
                    Type::Function(vec![], Box::new(Type::Float)),
                );
                map.insert("abs".to_string(), float_fn.clone());
                map.insert("sin".to_string(), float_fn.clone());
                map.insert("cos".to_string(), float_fn.clone());
                map.insert("sqrt".to_string(), float_fn.clone());
                map.insert("pow".to_string(), float_fn2.clone());
                map.insert("min".to_string(), float_fn2.clone());
                map.insert("max".to_string(), float_fn2.clone());
                map.insert("round".to_string(), float_fn.clone());
                map.insert("floor".to_string(), float_fn.clone());
                map.insert("ceil".to_string(), float_fn.clone());

                for name in [
                    "pi", "e", "inf", "abs", "sin", "cos", "sqrt", "pow", "min", "max",
                    "round", "floor", "ceil",
                ] {
                    if let Some(doc) = crate::std_docs::get_std_function_doc("std.math", name) {
                        docs.insert(name.to_string(), doc.to_string());
                    }
                }

                Type::Formula(map, docs)
            }
            "http" => {
                let mut map = HashMap::new();
                let mut docs = HashMap::new();

                let mut resp_map = HashMap::new();
                resp_map.insert("status".to_string(), Type::Int);
                resp_map.insert("ok".to_string(), Type::Bool);
                resp_map.insert(
                    "text".to_string(),
                    Type::Function(vec![], Box::new(Type::String)),
                );
                resp_map.insert(
                    "json".to_string(),
                    Type::Function(vec![], Box::new(Type::Unknown)),
                );
                let resp_ty = Type::Formula(resp_map, HashMap::new());

                map.insert(
                    "get".to_string(),
                    Type::Function(vec![Type::String], Box::new(resp_ty.clone())),
                );
                if let Some(doc) = crate::std_docs::get_std_function_doc("std.http", "get") {
                    docs.insert("get".to_string(), doc.to_string());
                }
                map.insert(
                    "post".to_string(),
                    Type::Function(vec![Type::String, Type::Unknown], Box::new(resp_ty.clone())),
                );
                if let Some(doc) = crate::std_docs::get_std_function_doc("std.http", "post") {
                    docs.insert("post".to_string(), doc.to_string());
                }
                Type::Formula(map, docs)
            }
            _ => Type::Named(format!("module:{}", mod_name)),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ImportDecl { path, span, .. } => {
                if let Some(last) = path.last() {
                    let is_native = path.first().map_or(false, |p| p == "native");
                    let kind_str = if is_native {
                        format!("plugin:{}", last)
                    } else {
                        format!("module:{}", last)
                    };

                    let path_str = path.join(".");
                    let mut is_package = false;
                    let mut package_docs = None;
                    let mut suggestions = Vec::new();
                    if !path.first().map_or(false, |p| p == "native" || p == "std") {
                        if let Some(file_path) = crate::stdlib::locate_import_file(
                            std::path::Path::new(&self.filepath),
                            path,
                        ) {
                            let p_str = file_path.to_string_lossy();
                            if p_str.contains(".flame/pkg/") || p_str.contains(".flame\\pkg\\") {
                                is_package = true;
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let mut lexer = crate::lexer::Lexer::new(&content);
                                    let mut tokens = Vec::new();
                                    loop {
                                        let tok = lexer.next_token();
                                        let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                                        tokens.push(tok);
                                        if is_eof {
                                            break;
                                        }
                                    }
                                    let mut parser = crate::parser::Parser::new(
                                        tokens,
                                        file_path.to_string_lossy().to_string(),
                                    );
                                    if let Ok(stmts) = parser.parse() {
                                        for stmt in stmts {
                                            if let crate::parser::Stmt::PackageDecl {
                                                annotations,
                                                ..
                                            } = stmt
                                            {
                                                package_docs =
                                                    self.process_annotations(&annotations);
                                                for ann in &annotations {
                                                    if ann.name == "Suggestions" {
                                                        if let Some(s) = ann.args.first() {
                                                            let s_trimmed = s.trim_matches(|c| {
                                                                c == '"' || c == '[' || c == ']'
                                                            });
                                                            let parts: Vec<&str> = s_trimmed
                                                                .split(',')
                                                                .map(|p| p.trim().trim_matches('"'))
                                                                .collect();
                                                            let name = parts[0].to_string();
                                                            let kind = if parts.len() > 1 {
                                                                parts[1].to_string()
                                                            } else {
                                                                "object".to_string()
                                                            };
                                                            suggestions.push((name, kind));
                                                        }
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let hover_str = if is_native {
                        format!("```flame\nimport {}\n```\n**Native Plugin**", path_str)
                    } else if path.first().map_or(false, |p| p == "std") {
                        format!(
                            "```flame\nimport {}\n```\n**Standard Library Module**",
                            path_str
                        )
                    } else if is_package {
                        let h = if let Some(ref docs) = package_docs {
                            format!("```flame\nimport package {}\n```\n\n{}", path_str, docs)
                        } else {
                            format!("```flame\nimport package {}\n```", path_str)
                        };
                        h
                    } else {
                        format!("```flame\nimport {}\n```\n**Local Module**", path_str)
                    };

                    let ty = if path.first().map_or(false, |p| p == "std") {
                        self.get_std_module_type(last)
                    } else if let Some((s, _)) = suggestions
                        .iter()
                        .find(|(_, k)| k == "object")
                        .or_else(|| suggestions.first())
                    {
                        Type::Named(s.clone())
                    } else {
                        Type::Named(kind_str)
                    };

                    let mut final_hover = hover_str.clone();
                    let mut final_ty = ty.clone();

                    if let Some(existing) = self.lookup_var(&last).cloned() {
                        let is_existing_native =
                            matches!(existing.ty, Type::Named(ref n) if n.starts_with("plugin:"));
                        let is_new_native = is_native;

                        if is_existing_native && !is_new_native {
                            final_ty = existing.ty.clone();
                            if is_package && package_docs.is_some() {
                                final_hover = format!(
                                    "```flame\nimport package {}\n```\n\n{}",
                                    path_str,
                                    package_docs.clone().unwrap()
                                );
                            } else if let Some(existing_doc) = existing.hover_doc {
                                final_hover = existing_doc;
                            }
                        } else if !is_existing_native && is_new_native {
                            if let Some(existing_doc) = existing.hover_doc {
                                final_hover = existing_doc;
                            }
                        }
                    }

                    self.insert_hover_info(span.clone(), final_hover.clone());
                    if let Some(first_part) = path.first() {
                        self.module_docs
                            .insert(first_part.to_string(), final_hover.clone());
                    }

                    self.define_var(
                        last.clone(),
                        VarInfo {
                            ty: final_ty,
                            is_mut: false,
                            hover_doc: Some(final_hover),
                        },
                    );
                    if !path.first().map_or(false, |p| p == "native" || p == "std") {
                        if let Some(file_path) = crate::stdlib::locate_import_file(
                            std::path::Path::new(&self.filepath),
                            path,
                        ) {
                            let mut paths_to_read = Vec::new();
                            if file_path.is_dir() {
                                if let Ok(entries) = std::fs::read_dir(&file_path) {
                                    for entry in entries.flatten() {
                                        let p = entry.path();
                                        if p.is_file()
                                            && p.extension().and_then(|s| s.to_str()) == Some("fm")
                                        {
                                            paths_to_read.push(p);
                                        }
                                    }
                                }
                            } else {
                                paths_to_read.push(file_path);
                            }

                            for path_to_read in paths_to_read {
                                if let Ok(content) = std::fs::read_to_string(&path_to_read) {
                                    let mut lexer = crate::lexer::Lexer::new(&content);
                                    let mut tokens = Vec::new();
                                    loop {
                                        let tok = lexer.next_token();
                                        let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                                        tokens.push(tok);
                                        if is_eof {
                                            break;
                                        }
                                    }
                                    let mut parser = crate::parser::Parser::new(
                                        tokens,
                                        path_to_read.to_string_lossy().to_string(),
                                    );
                                    if let Ok(parsed_stmts) = parser.parse() {
                                        let prev = self.is_importing;
                                        self.is_importing = true;
                                        for s in &parsed_stmts {
                                            if let Stmt::ImplDecl {
                                                target_type,
                                                methods,
                                                ..
                                            } = s
                                            {
                                                let prefixed_target =
                                                    format!("{}.{}", last, target_type);
                                                if !self.structs.contains_key(&prefixed_target) {
                                                    self.structs.insert(
                                                        prefixed_target.clone(),
                                                        StructInfo {
                                                            fields: Vec::new(),
                                                            hover_doc: None,
                                                        },
                                                    );
                                                }
                                                for m in methods {
                                                    if let Stmt::FuncDecl {
                                                        name,
                                                        params,
                                                        return_type,
                                                        ..
                                                    } = m
                                                    {
                                                        let is_static = !params
                                                            .first()
                                                            .map_or(false, |p| p.name == "self");
                                                        let p_info = params
                                                            .iter()
                                                            .map(|p| ParamInfo {
                                                                name: p.name.clone(),
                                                                ty: self
                                                                    .parse_type_name(&p.type_name),
                                                                is_ref: p.is_ref,
                                                                is_mut: p.is_mut,
                                                            })
                                                            .collect();
                                                        let r_type = return_type
                                                            .as_ref()
                                                            .map(|t| self.parse_type_name(t))
                                                            .unwrap_or(Type::Nil);
                                                        self.methods
                                                            .entry(prefixed_target.clone())
                                                            .or_default()
                                                            .insert(
                                                                name.clone(),
                                                                FunctionSig {
                                                                    params: p_info,
                                                                    return_type: r_type,
                                                                    is_static,
                                                                    hover_doc: None,
                                                                },
                                                            );
                                                    }
                                                }
                                            }
                                            if let Stmt::ExportDecl(inner, _) = s {
                                                match inner.as_ref() {
                                                    Stmt::LetDecl {
                                                        name, annotations, ..
                                                    }
                                                    | Stmt::ConstDecl {
                                                        name, annotations, ..
                                                    } => {
                                                        let hover_doc =
                                                            self.process_annotations(annotations);
                                                        self.define_var(
                                                            format!("{}.{}", last, name),
                                                            VarInfo {
                                                                ty: Type::Unknown,
                                                                is_mut: false,
                                                                hover_doc,
                                                            },
                                                        );
                                                    }
                                                    Stmt::StructDecl {
                                                        name,
                                                        fields,
                                                        annotations,
                                                        ..
                                                    } => {
                                                        let hover_doc =
                                                            self.process_annotations(annotations);
                                                        let mut struct_fields = Vec::new();
                                                        for (f_name, f_type) in fields {
                                                            struct_fields.push((
                                                                f_name.clone(),
                                                                self.parse_type_name(f_type),
                                                            ));
                                                        }
                                                        self.structs.insert(
                                                            format!("{}.{}", last, name),
                                                            StructInfo {
                                                                fields: struct_fields,
                                                                hover_doc,
                                                            },
                                                        );
                                                    }
                                                    Stmt::EnumDecl {
                                                        name,
                                                        variants,
                                                        annotations,
                                                        ..
                                                    } => {
                                                        let hover_doc =
                                                            self.process_annotations(annotations);
                                                        let mut enum_variants = HashMap::new();
                                                        for var in variants {
                                                            match var {
                                                                crate::parser::EnumVariant::Unit(n) => {
                                                                    enum_variants.insert(n.clone(), VariantInfo {
                                                                        tuple_items: vec![],
                                                                        struct_fields: Vec::new(),
                                                                        hover_doc: None,
                                                                    });
                                                                }
                                                                crate::parser::EnumVariant::Tuple(n, items) => {
                                                                    enum_variants.insert(n.clone(), VariantInfo {
                                                                        tuple_items: items.iter().map(|item| self.parse_type_name(item)).collect(),
                                                                        struct_fields: Vec::new(),
                                                                        hover_doc: None,
                                                                    });
                                                                }
                                                                crate::parser::EnumVariant::Struct(n, fields) => {
                                                                    let mut struct_fields = Vec::new();
                                                                    for (f_name, f_type) in fields {
                                                                        struct_fields.push((f_name.clone(), self.parse_type_name(f_type)));
                                                                    }
                                                                    enum_variants.insert(n.clone(), VariantInfo {
                                                                        tuple_items: vec![],
                                                                        struct_fields,
                                                                        hover_doc: None,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                        self.enums.insert(
                                                            format!("{}.{}", last, name),
                                                            EnumInfo {
                                                                variants: enum_variants,
                                                                hover_doc,
                                                            },
                                                        );
                                                    }
                                                    Stmt::FuncDecl {
                                                        name,
                                                        params,
                                                        return_type,
                                                        annotations,
                                                        ..
                                                    } => {
                                                        let hover_doc =
                                                            self.process_annotations(annotations);
                                                        let p_info = params
                                                            .iter()
                                                            .map(|p| ParamInfo {
                                                                name: p.name.clone(),
                                                                ty: self
                                                                    .parse_type_name(&p.type_name),
                                                                is_ref: p.is_ref,
                                                                is_mut: p.is_mut,
                                                            })
                                                            .collect();
                                                        let r_type = return_type
                                                            .as_ref()
                                                            .map(|t| self.parse_type_name(t))
                                                            .unwrap_or(Type::Nil);
                                                        self.functions.insert(
                                                            format!("{}.{}", last, name),
                                                            FunctionSig {
                                                                params: p_info,
                                                                return_type: r_type,
                                                                is_static: false,
                                                                hover_doc,
                                                            },
                                                        );
                                                    }
                                                    Stmt::ImplDecl {
                                                        target_type,
                                                        methods,
                                                        ..
                                                    } => {
                                                        let prefixed_target =
                                                            format!("{}.{}", last, target_type);
                                                        if !self
                                                            .structs
                                                            .contains_key(&prefixed_target)
                                                        {
                                                            self.structs.insert(
                                                                prefixed_target.clone(),
                                                                StructInfo {
                                                                    fields: Vec::new(),
                                                                    hover_doc: None,
                                                                },
                                                            );
                                                        }
                                                        for m in methods {
                                                            if let Stmt::FuncDecl {
                                                                name,
                                                                params,
                                                                return_type,
                                                                annotations,
                                                                ..
                                                            } = m
                                                            {
                                                                let hover_doc = self
                                                                    .process_annotations(
                                                                        annotations,
                                                                    );
                                                                let is_static = !params
                                                                    .first()
                                                                    .map_or(false, |p| {
                                                                        p.name == "self"
                                                                    });
                                                                let p_info = params
                                                                    .iter()
                                                                    .map(|p| ParamInfo {
                                                                        name: p.name.clone(),
                                                                        ty: self.parse_type_name(
                                                                            &p.type_name,
                                                                        ),
                                                                        is_ref: p.is_ref,
                                                                        is_mut: p.is_mut,
                                                                    })
                                                                    .collect();
                                                                let r_type = return_type
                                                                    .as_ref()
                                                                    .map(|t| {
                                                                        self.parse_type_name(t)
                                                                    })
                                                                    .unwrap_or(Type::Nil);
                                                                self.methods
                                                                    .entry(prefixed_target.clone())
                                                                    .or_default()
                                                                    .insert(
                                                                        name.clone(),
                                                                        FunctionSig {
                                                                            params: p_info,
                                                                            return_type: r_type,
                                                                            is_static,
                                                                            hover_doc,
                                                                        },
                                                                    );
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        self.is_importing = prev;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Stmt::ExportDecl(inner, _) => self.check_stmt(inner),
            Stmt::LetDecl {
                name,
                is_mut,
                type_ann,
                value,
                annotations,
                span,
                name_span,
                ..
            }
            | Stmt::ConstDecl {
                name,
                is_mut,
                type_ann,
                value,
                annotations,
                span,
                name_span,
                ..
            } => {
                let value_ty = self.infer_expr_type(value);
                let declared_ty = type_ann
                    .as_ref()
                    .map(|type_name| self.parse_type_name(type_name));

                if let Some(expected) = &declared_ty {
                    self.expect_assignable(expected, &value_ty, span, "variable initializer");
                }

                if !name.starts_with('{') && !name.starts_with('(') {
                    let stored_ty = match (&declared_ty, &value_ty) {
                        (Some(Type::Formula(_, _)), Type::Formula(map, _)) => {
                            Type::Formula(map.clone(), HashMap::new())
                        }
                        (Some(Type::Enum(expected)), Type::EnumVariant { enum_name, .. })
                            if expected == enum_name =>
                        {
                            value_ty.clone()
                        }
                        (Some(expected), _) => expected.clone(),
                        (None, Type::EnumVariant { enum_name, .. }) => {
                            Type::Enum(enum_name.clone())
                        }
                        (None, ty) => ty.clone(),
                    };

                    let type_str = match &stored_ty {
                        Type::Named(n) => n.clone(),
                        Type::Int => "Int".to_string(),
                        Type::Float => "Float".to_string(),
                        Type::String => "String".to_string(),
                        Type::Bool => "Bool".to_string(),
                        Type::Nil => "Nil".to_string(),
                        t => format!("{:?}", t),
                    };
                    let decl_kw = if matches!(stmt, Stmt::ConstDecl { .. }) {
                        "const"
                    } else if *is_mut {
                        "let mut"
                    } else {
                        "let"
                    };
                    let mut hover_str =
                        format!("```flame\n{} {}: {}\n```", decl_kw, name, type_str);

                    let hover_doc = self.process_annotations(annotations);

                    if let Some(doc) = hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }

                    self.insert_hover_info(name_span.clone(), hover_str.clone());

                    self.define_var(
                        name.clone(),
                        VarInfo {
                            ty: stored_ty,
                            is_mut: *is_mut,
                            hover_doc: Some(hover_str.clone()),
                        },
                    );
                } else if name.starts_with('(') && name.ends_with(')') {
                    // It's a tuple destructuring assignment, e.g. "(tx, rx)" or "(b: 1, c: 2)"
                    let inner_names = name[1..name.len() - 1]
                        .split(',')
                        .map(|s| s.trim())
                        .collect::<Vec<_>>();

                    if let Type::Tuple(types) = &value_ty {
                        for (i, inner_name) in inner_names.iter().enumerate() {
                            if inner_name.is_empty() || *inner_name == "_" {
                                continue;
                            }
                            // Handle "(b: 1)" style destructuring
                            let actual_name = inner_name.split(':').next().unwrap().trim();
                            let v_ty = types.get(i).cloned().unwrap_or(Type::Unknown);
                            self.define_var(
                                actual_name.to_string(),
                                VarInfo {
                                    ty: v_ty,
                                    is_mut: *is_mut,
                                    hover_doc: None,
                                },
                            );
                        }
                    } else {
                        // Fallback: If it's an unknown type or not a tuple
                        for inner_name in inner_names {
                            if inner_name.is_empty() || inner_name == "_" {
                                continue;
                            }
                            let actual_name = inner_name.split(':').next().unwrap().trim();
                            self.define_var(
                                actual_name.to_string(),
                                VarInfo {
                                    ty: value_ty.clone(),
                                    is_mut: *is_mut,
                                    hover_doc: None,
                                },
                            );
                        }
                    }
                } else if name.starts_with('{') && name.ends_with('}') {
                    // It's an object destructuring assignment, e.g. "{status, data}"
                    let inner_names = name
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>();

                    if let Type::Formula(map, _) = &value_ty {
                        for v_name in inner_names {
                            if v_name != "_" {
                                let v_ty = map.get(&v_name).cloned().unwrap_or(Type::Unknown);
                                self.define_var(
                                    v_name.clone(),
                                    VarInfo {
                                        ty: v_ty,
                                        is_mut: *is_mut,
                                        hover_doc: None,
                                    },
                                );
                            }
                        }
                    } else if let Type::Struct(struct_name) = &value_ty {
                        for v_name in inner_names {
                            if v_name != "_" {
                                let mut field_ty = Type::Unknown;
                                if let Some(info) = self.structs.get(struct_name) {
                                    if let Some((_, ty)) =
                                        info.fields.iter().find(|(n, _)| n == &v_name)
                                    {
                                        field_ty = ty.clone();
                                    }
                                }
                                self.define_var(
                                    v_name.clone(),
                                    VarInfo {
                                        ty: field_ty,
                                        is_mut: *is_mut,
                                        hover_doc: None,
                                    },
                                );
                            }
                        }
                    } else {
                        // Fallback
                        for v_name in inner_names {
                            if v_name != "_" {
                                self.define_var(
                                    v_name.clone(),
                                    VarInfo {
                                        ty: Type::Unknown,
                                        is_mut: *is_mut,
                                        hover_doc: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            Stmt::FuncDecl {
                name,
                params,
                return_type,
                body,
                annotations,
                span: _span,
                name_span,
                ..
            } => {
                let func_type = Type::Function(
                    params
                        .iter()
                        .map(|param| self.parse_type_name(&param.type_name))
                        .collect(),
                    Box::new(
                        return_type
                            .as_ref()
                            .map(|ret| self.parse_type_name(ret))
                            .unwrap_or(Type::Nil),
                    ),
                );
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_str = if let Some(ret) = &return_type {
                    format!(" -> {}", ret)
                } else {
                    "".to_string()
                };
                let mut hover_str =
                    format!("```flame\nfn {}({}){}\n```", name, params_str, ret_str);
                let hover_doc = self.process_annotations(annotations);

                if let Some(doc) = hover_doc {
                    hover_str = format!("{}\n\n{}", hover_str, doc);
                }

                self.insert_hover_info(name_span.clone(), hover_str.clone());

                self.define_var(
                    name.clone(),
                    VarInfo {
                        ty: func_type,
                        is_mut: false,
                        hover_doc: Some(hover_str.clone()),
                    },
                );

                let prev_return = self.current_return_type.clone();
                self.current_return_type = Some(
                    return_type
                        .as_ref()
                        .map(|ret| self.parse_type_name(ret))
                        .unwrap_or(Type::Nil),
                );

                self.push_scope();
                for anno in annotations {
                    if anno.name == "Requires" {
                        for arg in &anno.args {
                            if arg.starts_with('"') && arg.ends_with('"') {
                                let mod_name = arg[1..arg.len() - 1].to_string();
                                let parts: Vec<String> =
                                    mod_name.split('.').map(|s| s.to_string()).collect();
                                self.check_stmt(&Stmt::ImportDecl {
                                    path: parts,
                                    glob: false,
                                    span: anno.span.clone(),
                                });
                            }
                        }
                    }

                    let mut ret_ty = if let Some(sig) = self.functions.get(&anno.name) {
                        sig.return_type.clone()
                    } else if let Some(funcs) = self.plugin_functions.get(&anno.name.to_lowercase())
                    {
                        if let Some(sig) = funcs.get("init") {
                            sig.return_type.clone()
                        } else {
                            Type::Unknown
                        }
                    } else {
                        Type::Unknown
                    };
                    if let Type::Named(name) = &ret_ty {
                        if self.structs.contains_key(name) {
                            ret_ty = Type::Struct(name.clone());
                        }
                    }
                    self.define_var(
                        anno.name.clone(),
                        VarInfo {
                            ty: ret_ty.clone(),
                            is_mut: true,
                            hover_doc: None,
                        },
                    );
                    self.define_var(
                        anno.name.to_lowercase(),
                        VarInfo {
                            ty: ret_ty,
                            is_mut: true,
                            hover_doc: None,
                        },
                    );
                }
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
                            hover_doc: None,
                        },
                    );
                }
                if let Some(body_stmts) = body {
                    for stmt in body_stmts {
                        self.check_stmt(stmt);
                    }
                }
                self.pop_scope();
                self.current_return_type = prev_return;
            }
            Stmt::AnnotationDecl {
                name,
                params,
                return_type,
                body,
                annotations,
                span: _span,
                name_span,
            } => {
                if let Some(first_char) = name.chars().next() {
                    if !first_char.is_uppercase() {
                        self.diagnostics.push(Diagnostic::new_error(
                            format!("Annotation names should start with an uppercase letter, found '{}'", name),
                            self.filepath.clone(),
                            name_span.clone(),
                            None,
                            None
                        ));
                    }
                }
                let func_type = Type::Function(
                    params
                        .iter()
                        .map(|param| self.parse_type_name(&param.type_name))
                        .collect(),
                    Box::new(
                        return_type
                            .as_ref()
                            .map(|ret| self.parse_type_name(ret))
                            .unwrap_or(Type::Nil),
                    ),
                );

                let params_str = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut hover_str = if let Some(ret) = &return_type {
                    format!(
                        "```flame\nannotation @{}({}) -> {}\n```",
                        name, params_str, ret
                    )
                } else {
                    format!("```flame\nannotation @{}({})\n```", name, params_str)
                };
                let hover_doc = self.process_annotations(annotations);

                if let Some(doc) = hover_doc {
                    hover_str = format!("{}\n\n{}", hover_str, doc);
                }

                self.insert_hover_info(name_span.clone(), hover_str);

                self.define_var(
                    name.clone(),
                    VarInfo {
                        ty: func_type,
                        is_mut: false,
                        hover_doc: None,
                    },
                );

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
                            hover_doc: None,
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
                        hover_doc: None,
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
            Stmt::MatchStmt {
                target,
                arms,
                span: _,
            } => {
                let target_ty = self.infer_expr_type(target);
                let is_cli_match = match &target_ty {
                    Type::Named(name) => name == "Cli",
                    Type::Struct(name) => name == "Cli",
                    _ => match target {
                        Expr::Identifier(id, _) => id == "cli",
                        _ => false,
                    },
                };

                if is_cli_match {
                    for arm in arms {
                        let is_wildcard = arm.patterns.iter().any(|p| p == "_");
                        let is_help = arm.patterns.iter().any(|p| p == "help");
                        let cmd_match = arm
                            .patterns
                            .iter()
                            .find_map(|p| self.commands.get(p).map(|c| (p, c.clone())));

                        if is_wildcard {
                            let wildcard_doc = "```flame\n_ => ...\n```\n**Wildcard Match Arm**\nMatches any unrecognized CLI command.".to_string();
                            self.insert_hover_info(arm.pattern_span.clone(), wildcard_doc);
                            self.push_scope();
                            self.infer_expr_type(&arm.body);
                            self.pop_scope();
                        } else if is_help {
                            let help_doc = if let Some(cmd) = self.commands.get("help") {
                                cmd.hover_doc.clone()
                            } else {
                                "```flame\n@Command(name: \"help\", about: \"Print help message\")\n```\n**CLI Subcommand**: `help`\n\nPrint help message".to_string()
                            };
                            self.insert_hover_info(arm.pattern_span.clone(), help_doc);
                            self.push_scope();
                            self.infer_expr_type(&arm.body);
                            self.pop_scope();
                        } else if let Some((_pat, cmd)) = cmd_match {
                            self.insert_hover_info(arm.pattern_span.clone(), cmd.hover_doc.clone());
                            self.push_scope();
                            for field in &arm.destructure {
                                if let Some(param) = cmd.params.iter().find(|p| &p.name == field) {
                                    self.define_var(
                                        field.clone(),
                                        VarInfo {
                                            ty: param.ty.clone(),
                                            is_mut: false,
                                            hover_doc: None,
                                        },
                                    );
                                } else {
                                    self.define_var(
                                        field.clone(),
                                        VarInfo {
                                            ty: Type::Unknown,
                                            is_mut: false,
                                            hover_doc: None,
                                        },
                                    );
                                }
                            }
                            self.infer_expr_type(&arm.body);
                            self.pop_scope();
                        } else {
                            // Command was not defined with @Command -> emit error!
                            let pat = arm.patterns.first().unwrap_or(&String::new()).clone();
                            self.error(
                                format!("unknown command '{}': no matching function annotated with @Command(name: \"{}\") was found", pat, pat),
                                arm.pattern_span.clone(),
                                Some(format!("unknown command '{}'", pat)),
                                Some(format!("define a function annotated with '@Command(name: \"{}\")' to handle this command", pat)),
                            );
                            self.push_scope();
                            for field in &arm.destructure {
                                self.define_var(
                                    field.clone(),
                                    VarInfo {
                                        ty: Type::Unknown,
                                        is_mut: false,
                                        hover_doc: None,
                                    },
                                );
                            }
                            self.infer_expr_type(&arm.body);
                            self.pop_scope();
                        }
                    }
                } else {
                    for arm in arms {
                        self.push_scope();
                        for field in &arm.destructure {
                            self.define_var(
                                field.clone(),
                                VarInfo {
                                    ty: Type::Unknown,
                                    is_mut: false,
                                    hover_doc: None,
                                },
                            );
                        }
                        self.infer_expr_type(&arm.body);
                        self.pop_scope();
                    }
                }
            }
            Stmt::StructDecl { .. }
            | Stmt::EnumDecl { .. }
            | Stmt::TraitDecl { .. }
            | Stmt::ImplDecl { .. }
            | Stmt::Break(_)
            | Stmt::Continue(_)
            | Stmt::PackageDecl { .. }
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
                let inferred = if let Some(var) = self.lookup_var(name).cloned() {
                    if let Some(doc) = &var.hover_doc {
                        self.insert_hover_info(span.clone(), doc.clone());
                    }
                    var.ty
                } else if let Some(struct_info) = self.structs.get(name).cloned() {
                    if let Some(doc) = struct_info.hover_doc {
                        self.insert_hover_info(span.clone(), doc);
                    }
                    Type::Named(name.clone())
                } else if let Some(enum_info) = self.enums.get(name).cloned() {
                    if let Some(doc) = enum_info.hover_doc {
                        self.insert_hover_info(span.clone(), doc);
                    }
                    Type::Enum(name.clone())
                } else if let Some(func) = self.functions.get(name) {
                    let params_str: Vec<String> = func
                        .params
                        .iter()
                        .map(|p| {
                            let mut mods = String::new();
                            if p.is_ref {
                                mods.push('&');
                            }
                            if p.is_mut {
                                mods.push_str("mut ");
                            }
                            let type_str = match &p.ty {
                                Type::Named(n) => n.clone(),
                                Type::Int => "Int".to_string(),
                                Type::Float => "Float".to_string(),
                                Type::String => "String".to_string(),
                                Type::Bool => "Bool".to_string(),
                                t => format!("{:?}", t),
                            };
                            format!(
                                "{}{}: {}{}",
                                if p.is_mut && !p.is_ref { "mut " } else { "" },
                                p.name,
                                mods,
                                type_str
                            )
                        })
                        .collect();
                    let ret_str = match &func.return_type {
                        Type::Named(n) => n.clone(),
                        Type::Int => "Int".to_string(),
                        Type::Float => "Float".to_string(),
                        Type::String => "String".to_string(),
                        Type::Bool => "Bool".to_string(),
                        Type::Nil => "Nil".to_string(),
                        t => format!("{:?}", t),
                    };
                    Type::Named(format!("fn({}) -> {}", params_str.join(", "), ret_str))
                } else if self.plugins.contains(name) {
                    self.insert_hover_info(
                        span.clone(),
                        format!("```flame\nplugin {}\n```\n**Native Plugin**", name),
                    );
                    Type::Named(format!("plugin:{}", name))
                } else if self.modules.contains(name) {
                    if let Some(doc) = self.module_docs.get(name) {
                        self.insert_hover_info(span.clone(), doc.clone());
                    }
                    Type::Named(format!("module:{}", name))
                } else {
                    let mut found_variant = None;
                    for (enum_name, enum_info) in &self.enums {
                        if let Some(variant) = enum_info.variants.get(name) {
                            found_variant = Some((enum_name.clone(), variant.clone()));
                            break;
                        }
                    }
                    if let Some((enum_name, variant)) = found_variant {
                        if let Some(doc) = &variant.hover_doc {
                            self.insert_hover_info(span.clone(), doc.clone());
                        }
                        if variant.struct_fields.is_empty() && variant.tuple_items.is_empty() {
                            Type::EnumVariant {
                                enum_name,
                                variant_name: name.clone(),
                                tuple_items: vec![],
                                struct_fields: HashMap::new(),
                            }
                        } else {
                            // It's a constructor for a tuple or struct variant
                            let params_str: Vec<String> = variant
                                .tuple_items
                                .iter()
                                .map(|t| format!("{:?}", t))
                                .collect();
                            Type::Named(format!("fn({}) -> EnumVariant", params_str.join(", ")))
                        }
                    } else {
                        self.error(
                            format!("undefined identifier '{}'", name),
                            span.clone(),
                            Some("This name is not declared in the current scope".to_string()),
                            None,
                        );
                        Type::Unknown
                    }
                };

                let hover_str = if let Some(var) = self.lookup_var(name) {
                    if let Some(doc) = &var.hover_doc {
                        doc.clone()
                    } else if let (Type::Function(_, _), Some(func_sig)) =
                        (&var.ty, self.functions.get(name))
                    {
                        let mut params_str = Vec::new();
                        for p in &func_sig.params {
                            let ty_str = self.format_type(&p.ty);
                            params_str.push(format!("{}: {}", p.name, ty_str));
                        }
                        if let Some(doc) = &func_sig.hover_doc {
                            doc.clone()
                        } else {
                            let ret_ty_str = self.format_type(&func_sig.return_type);
                            format!(
                                "```flame\nfn {}({}) -> {}\n```",
                                name,
                                params_str.join(", "),
                                ret_ty_str
                            )
                        }
                    } else {
                        let type_str = self.format_type(&inferred);
                        let sig_str = if var.is_mut {
                            format!("```flame\nlet mut {}: {}\n```", name, type_str)
                        } else {
                            format!("```flame\nlet {}: {}\n```", name, type_str)
                        };
                        sig_str
                    }
                } else if self.plugins.contains(name) {
                    "plugin".to_string()
                } else if self.modules.contains(name) {
                    if let Some(doc) = self.module_docs.get(name) {
                        self.insert_hover_info(span.clone(), doc.clone());
                    }
                    format!("module:{}", name)
                } else {
                    let mut variant_doc = None;
                    for (enum_name, enum_info) in &self.enums {
                        if let Some(variant) = enum_info.variants.get(name) {
                            if let Some(doc) = &variant.hover_doc {
                                variant_doc = Some(format!(
                                    "```flame\n{}::{}\n```\n{}",
                                    enum_name, name, doc
                                ));
                            } else {
                                let params_str: Vec<String> = variant
                                    .tuple_items
                                    .iter()
                                    .map(|t| self.format_type(t))
                                    .collect();
                                variant_doc = Some(format!(
                                    "```flame\n{}({}) -> {}\n```",
                                    name,
                                    params_str.join(", "),
                                    enum_name
                                ));
                            }
                            break;
                        }
                    }
                    if let Some(doc) = variant_doc {
                        doc
                    } else if let Some(struct_info) = self.structs.get(name) {
                        if let Some(doc) = &struct_info.hover_doc {
                            format!("```flame\nstruct {}\n```\n\n{}", name, doc)
                        } else {
                            format!("```flame\nstruct {}\n```", name)
                        }
                    } else if let Some(enum_info) = self.enums.get(name) {
                        if let Some(doc) = &enum_info.hover_doc {
                            format!("```flame\nenum {}\n```\n\n{}", name, doc)
                        } else {
                            format!("```flame\nenum {}\n```", name)
                        }
                    } else if let Some(func_sig) = self.functions.get(name) {
                        let mut params_str = Vec::new();
                        for p in &func_sig.params {
                            let is_already_ref = matches!(p.ty, Type::Reference { .. });
                            let mut mods = String::new();
                            if p.is_ref && !is_already_ref {
                                mods.push('&');
                            }
                            if p.is_mut && !is_already_ref {
                                mods.push_str("mut ");
                            }
                            params_str.push(format!(
                                "{}{}: {}{}",
                                if p.is_mut && !p.is_ref { "mut " } else { "" },
                                p.name,
                                mods,
                                self.format_type(&p.ty)
                            ));
                        }
                        let ret_str = if func_sig.return_type == Type::Nil {
                            "".to_string()
                        } else {
                            format!(" -> {}", self.format_type(&func_sig.return_type))
                        };
                        let mut s = format!(
                            "```flame\nfn {}({}){}\n```",
                            name,
                            params_str.join(", "),
                            ret_str
                        );
                        if let Some(doc) = &func_sig.hover_doc {
                            s = format!("{}\n\n{}", s, doc);
                        }
                        s
                    } else if let Type::Named(s) = &inferred {
                        s.clone()
                    } else {
                        self.format_type(&inferred)
                    }
                };

                self.insert_hover_info(span.clone(), hover_str);
                inferred
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
                let mut docs = HashMap::new();
                for (k, v, span, annotations) in pairs {
                    let ty = self.infer_expr_type(v);
                    let signature = match v {
                        Expr::Closure {
                            params,
                            return_type,
                            ..
                        } => {
                            let params_str = params
                                .iter()
                                .map(|p| format!("{}: {}", p.name, p.type_name))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let ret_str = if let Some(ret) = return_type {
                                format!(" -> {}", ret)
                            } else {
                                "".to_string()
                            };
                            format!("```flame\nfn {}({}){}\n```", k, params_str, ret_str)
                        }
                        _ => format!("```flame\n{}: {}\n```", k, self.format_type(&ty)),
                    };
                    let hover_doc = self.process_annotations(annotations);
                    let full_doc = if let Some(doc) = hover_doc {
                        format!("{}\n\n{}", signature, doc)
                    } else {
                        signature
                    };
                    self.insert_hover_info(span.clone(), full_doc.clone());
                    docs.insert(k.clone(), full_doc);
                    map.insert(k.clone(), ty);
                }
                Type::Formula(map, docs)
            }
            Expr::Object(pairs, _) => {
                let mut map = HashMap::new();
                for (k, v, annotations) in pairs {
                    let ty = self.infer_expr_type(v);
                    let _ = self.process_annotations(annotations);
                    map.insert(k.clone(), ty);
                }
                Type::Formula(map, HashMap::new()) // We treat Object and Formula as structurally equivalent in types for now, or we can use a new Type::Object. Let's use Type::Formula since it's a dynamic map
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
            Expr::ThreadSpawn(_, _) => Type::Named("ThreadHandler".to_string()),
            Expr::Block(stmts, _) => {
                self.push_scope();
                for stmt in stmts {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                Type::Nil
            }
            Expr::Unary(op, inner, span) => match op {
                UnaryOp::Neg => {
                    let ty = self.infer_expr_type(inner);
                    if self.is_numeric(&ty) {
                        ty
                    } else {
                        self.error(
                            "cannot apply unary '-' to non-numeric type".to_string(),
                            span.clone(),
                            None,
                            None,
                        );
                        Type::Unknown
                    }
                }
                UnaryOp::Not => {
                    let _ = self.infer_expr_type(inner);
                    Type::Bool
                }
                UnaryOp::NonNullAssert => {
                    let ty = self.infer_expr_type(inner);
                    ty
                }
                UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                    let ty = self.infer_expr_type(inner);
                    if !self.is_numeric(&ty) {
                        self.error(
                            "cannot increment/decrement non-numeric type".to_string(),
                            span.clone(),
                            None,
                            None,
                        );
                    }
                    ty
                }
            },
            Expr::SafeDot(inner, member, span) => self.infer_dot_type(inner, member, span),
            Expr::Binary(left, op, right, span) => self.infer_binary_type(left, op, right, span),
            Expr::Dot(inner, member, span) => self.infer_dot_type(inner, member, span),
            Expr::StructInit(inner, fields, span) => {
                self.infer_struct_init_type(inner, fields, span)
            }
            Expr::Index(inner, idx, span) => self.infer_index_type(inner, idx, span),
            Expr::Closure {
                params,
                return_type,
                body,
                annotations,
                span,
                ..
            } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_str = if let Some(ret) = &return_type {
                    format!(" -> {}", ret)
                } else {
                    "".to_string()
                };
                let mut hover_str = format!("```flame\n|{}|{}\n```", params_str, ret_str);

                let hover_doc = self.process_annotations(annotations);

                if let Some(doc) = hover_doc {
                    hover_str = format!("{}\n\n{}", hover_str, doc);
                }

                self.insert_hover_info(span.clone(), hover_str);

                let prev_return = self.current_return_type.clone();
                let ret_ty = return_type
                    .as_ref()
                    .map(|ret| self.parse_type_name(ret))
                    .unwrap_or(Type::Unknown);
                self.current_return_type = Some(ret_ty.clone());

                self.push_scope();
                let mut param_types = Vec::new();
                for param in params {
                    let p_ty = self.parse_type_name(&param.type_name);
                    param_types.push(p_ty.clone());
                    self.define_var(
                        param.name.clone(),
                        VarInfo {
                            ty: p_ty.clone(),
                            is_mut: param.is_mut,
                            hover_doc: None,
                        },
                    );
                }
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
                self.current_return_type = prev_return;
                Type::Function(param_types, Box::new(ret_ty))
            }
            Expr::Call(callee, args, span) => self.infer_call_type(callee, args, span),
        }
    }

    fn infer_binary_type(&mut self, left: &Expr, op: &BinaryOp, right: &Expr, span: &Span) -> Type {
        if matches!(
            op,
            BinaryOp::Assign
                | BinaryOp::PlusAssign
                | BinaryOp::MinusAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign
                | BinaryOp::ModAssign
                | BinaryOp::BitAndAssign
                | BinaryOp::BitOrAssign
                | BinaryOp::BitXorAssign
                | BinaryOp::ShlAssign
                | BinaryOp::ShrAssign
        ) {
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
                    if *op != BinaryOp::Assign {
                        if !self.is_numeric(&var.ty) || !self.is_numeric(&rhs_ty) {
                            if matches!(op, BinaryOp::PlusAssign)
                                && matches!(var.ty, Type::String)
                                && matches!(rhs_ty, Type::String)
                            {
                                // String concatenation
                            } else if matches!(var.ty, Type::Unknown)
                                || matches!(rhs_ty, Type::Unknown)
                            {
                                // Ignore mismatch if type is Unknown
                            } else {
                                self.error_binary_mismatch(op, &var.ty, &rhs_ty, span);
                            }
                        }
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

                if *op != BinaryOp::Assign {
                    if !self.is_numeric(&lhs_ty) || !self.is_numeric(&rhs_ty) {
                        if matches!(op, BinaryOp::PlusAssign)
                            && matches!(lhs_ty, Type::String)
                            && matches!(rhs_ty, Type::String)
                        {
                            // String concatenation
                        } else if matches!(lhs_ty, Type::Unknown) || matches!(rhs_ty, Type::Unknown)
                        {
                            // Ignore mismatch if type is Unknown
                        } else {
                            self.error_binary_mismatch(op, &lhs_ty, &rhs_ty, span);
                        }
                    }
                }

                self.expect_assignable(&lhs_ty, &rhs_ty, span, "field assignment");
                return lhs_ty;
            } else if let Expr::Index(inner, idx, _) = left {
                let lhs_ty = self.infer_index_type(inner, idx, span);
                let rhs_ty = self.infer_expr_type(right);
                if *op != BinaryOp::Assign {
                    if !self.is_numeric(&lhs_ty) || !self.is_numeric(&rhs_ty) {
                        if matches!(lhs_ty, Type::Unknown) || matches!(rhs_ty, Type::Unknown) {
                            // ignore
                        } else {
                            self.error_binary_mismatch(op, &lhs_ty, &rhs_ty, span);
                        }
                    }
                }
                self.expect_assignable(&lhs_ty, &rhs_ty, span, "index assignment");
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

        let mut left_ty = self.infer_expr_type(left);
        let mut right_ty = self.infer_expr_type(right);

        if let Type::Function(params, ret) = &left_ty {
            if params.is_empty() {
                left_ty = *ret.clone();
            }
        }
        if let Type::Function(params, ret) = &right_ty {
            if params.is_empty() {
                right_ty = *ret.clone();
            }
        }
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
                } else if let (Type::Quantity(m1), Type::Quantity(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Unit(m1), Type::Unit(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Unit(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Quantity(m1), Type::Unit(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Unit(m1), Type::Quantity(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if matches!(left_ty, Type::Named(ref n) if n == "Quantity" || n == "Unit")
                    && matches!(right_ty, Type::Named(ref m) if m == "Quantity" || m == "Unit")
                {
                    Type::Named("Quantity".to_string())
                } else if matches!(left_ty, Type::Unknown) || matches!(right_ty, Type::Unknown) {
                    if matches!(left_ty, Type::String) || matches!(right_ty, Type::String) {
                        Type::String
                    } else {
                        Type::Unknown
                    }
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Sub => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else if let (Type::Quantity(m1), Type::Quantity(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Unit(m1), Type::Unit(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Unit(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Quantity(m1), Type::Unit(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if let (Type::Unit(m1), Type::Quantity(m2)) = (&left_ty, &right_ty) {
                    if m1 == m2 {
                        Type::Quantity(m1.clone())
                    } else {
                        self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                        Type::Unknown
                    }
                } else if matches!(left_ty, Type::Named(ref n) if n == "Quantity" || n == "Unit")
                    && matches!(right_ty, Type::Named(ref m) if m == "Quantity" || m == "Unit")
                {
                    Type::Named("Quantity".to_string())
                } else if matches!(left_ty, Type::Unknown) || matches!(right_ty, Type::Unknown) {
                    Type::Unknown
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Mul | BinaryOp::Div => {
                let is_q_or_u = |t: &Type| {
                    if let Type::Named(n) = t {
                        n == "Quantity" || n == "Unit"
                    } else {
                        matches!(t, Type::Quantity(_) | Type::Unit(_))
                    }
                };
                let get_dims = |t: &Type| -> HashMap<String, i32> {
                    match t {
                        Type::Quantity(map) | Type::Unit(map) => map.clone(),
                        _ => HashMap::new(),
                    }
                };
                let compute_dims =
                    |m1: &HashMap<String, i32>, m2: &HashMap<String, i32>, is_div: bool| {
                        let mut res = m1.clone();
                        for (k, v) in m2 {
                            let current = res.entry(k.clone()).or_insert(0);
                            if is_div {
                                *current -= v;
                            } else {
                                *current += v;
                            }
                            if *current == 0 {
                                res.remove(k);
                            }
                        }
                        res
                    };
                let invert_dims = |m: &HashMap<String, i32>| {
                    let mut res = HashMap::new();
                    for (k, v) in m {
                        res.insert(k.clone(), -v);
                    }
                    res
                };

                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else if self.is_numeric(&left_ty) && is_q_or_u(&right_ty) {
                    let dims = get_dims(&right_ty);
                    if matches!(*op, BinaryOp::Div) {
                        Type::Quantity(invert_dims(&dims))
                    } else {
                        Type::Quantity(dims)
                    }
                } else if is_q_or_u(&left_ty) && self.is_numeric(&right_ty) {
                    Type::Quantity(get_dims(&left_ty))
                } else if is_q_or_u(&left_ty) && is_q_or_u(&right_ty) {
                    let dims = compute_dims(
                        &get_dims(&left_ty),
                        &get_dims(&right_ty),
                        matches!(*op, BinaryOp::Div),
                    );
                    Type::Quantity(dims)
                } else if matches!(left_ty, Type::Unknown) || matches!(right_ty, Type::Unknown) {
                    Type::Unknown
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Mod => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else if matches!(left_ty, Type::Unknown) || matches!(right_ty, Type::Unknown) {
                    Type::Unknown
                } else {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                if !matches!(left_ty, Type::Nil)
                    && !matches!(right_ty, Type::Nil)
                    && !matches!(left_ty, Type::Unknown)
                    && !matches!(right_ty, Type::Unknown)
                    && !self.is_compatible(&left_ty, &right_ty)
                {
                    self.error_binary_mismatch(op, &left_ty, &right_ty, span);
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    Type::Bool
                } else if matches!(left_ty, Type::Unknown) || matches!(right_ty, Type::Unknown) {
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
            BinaryOp::NilCoalesce => {
                if matches!(left_ty, Type::Nil) {
                    right_ty
                } else {
                    left_ty
                }
            }
            BinaryOp::BitXor => {
                if let Type::Quantity(dims) | Type::Unit(dims) = &left_ty {
                    Type::Quantity(dims.clone())
                } else if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                    if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else {
                    Type::Int
                }
            }
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::Shl
            | BinaryOp::Shr => Type::Int,
            BinaryOp::Range => {
                self.expect_assignable(&Type::Int, &left_ty, span, "range start");
                self.expect_assignable(&Type::Int, &right_ty, span, "range end");
                Type::Named("Range".to_string())
            }
            BinaryOp::Assign
            | BinaryOp::PlusAssign
            | BinaryOp::MinusAssign
            | BinaryOp::MulAssign
            | BinaryOp::DivAssign
            | BinaryOp::ModAssign
            | BinaryOp::BitAndAssign
            | BinaryOp::BitOrAssign
            | BinaryOp::BitXorAssign
            | BinaryOp::ShlAssign
            | BinaryOp::ShrAssign => Type::Unknown,
        }
    }

    fn infer_index_type(&mut self, inner: &Expr, idx: &Expr, span: &Span) -> Type {
        let inner_ty = self.infer_expr_type(inner);
        let idx_ty = self.infer_expr_type(idx);
        if !matches!(idx_ty, Type::Int | Type::Unknown) {
            self.error(
                format!(
                    "expected integer index, found {}",
                    self.format_type(&idx_ty)
                ),
                idx.span(),
                None,
                None,
            );
        }
        match inner_ty {
            Type::Vector(elem) => *elem,
            Type::Tuple(elems) => {
                if let Expr::Literal(crate::parser::LiteralValue::Int(i), _) = idx {
                    if *i >= 0 && (*i as usize) < elems.len() {
                        return elems[*i as usize].clone();
                    } else {
                        self.error(
                            format!(
                                "tuple index out of bounds: {} for tuple of length {}",
                                i,
                                elems.len()
                            ),
                            span.clone(),
                            None,
                            None,
                        );
                    }
                }
                Type::Unknown
            }
            Type::String => Type::String,
            Type::Byte => Type::Byte,
            Type::Unknown => Type::Unknown,
            Type::Reference {
                inner: ref_inner, ..
            } => match *ref_inner {
                Type::Vector(elem) => *elem,
                _ => Type::Unknown,
            },
            _ => {
                self.error(
                    format!("cannot index into type {}", self.format_type(&inner_ty)),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
        }
    }

    fn infer_dot_type(&mut self, inner: &Expr, member: &str, span: &Span) -> Type {
        // Intercept Flame-defined package fields first
        if let Expr::Identifier(mod_name, _) = inner {
            let prefixed_member = format!("{}.{}", mod_name, member);

            if self.structs.contains_key(&prefixed_member) {
                if let Some(info) = self.structs.get(&prefixed_member) {
                    let mut doc_str = format!("```flame\nstruct {}\n```", member);
                    if let Some(d) = &info.hover_doc {
                        doc_str = format!("{}\n\n{}", doc_str, d);
                    }
                    self.insert_hover_info(span.clone(), doc_str);
                }
                return Type::Struct(prefixed_member);
            }
            if self.enums.contains_key(&prefixed_member) {
                if let Some(info) = self.enums.get(&prefixed_member) {
                    let mut doc_str = format!("```flame\nenum {}\n```", member);
                    if let Some(d) = &info.hover_doc {
                        doc_str = format!("{}\n\n{}", doc_str, d);
                    }
                    self.insert_hover_info(span.clone(), doc_str);
                }
                return Type::Enum(prefixed_member);
            }
            if let Some(sig) = self.functions.get(&prefixed_member).cloned() {
                let params_str = sig
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_str = if sig.return_type == Type::Nil {
                    "".to_string()
                } else {
                    format!(" -> {}", self.format_type(&sig.return_type))
                };
                let fallback = format!("```flame\nfn {}({}){}\n```", member, params_str, ret_str);
                if let Some(doc) = &sig.hover_doc {
                    self.insert_hover_info(span.clone(), format!("{}\n\n{}", fallback, doc));
                } else {
                    self.insert_hover_info(span.clone(), fallback);
                }

                let mut p_tys = Vec::new();
                for p in &sig.params {
                    p_tys.push(p.ty.clone());
                }
                return Type::Function(p_tys, Box::new(sig.return_type));
            }
        }

        let mut inner_ty = self.infer_expr_type(inner);
        if let Type::Reference {
            inner: ref_inner, ..
        } = inner_ty
        {
            inner_ty = *ref_inner;
        }
        match member {
            "toString" | "toChar" | "trim" | "toUpperCase" | "toLowerCase" | "replace" | "join"
            | "push_str" | "push" | "pop" | "clear" | "remove" | "insert" | "slice"
            | "substring" => return Type::String,
            "toInt" | "tryInt" | "len" => return Type::Int,
            "toFloat" | "tryFloat" => return Type::Float,
            "toBool" | "tryBool" | "contains" | "startsWith" | "endsWith" | "isEmpty" => {
                return Type::Bool;
            }
            "toByte" | "to_byte" => {
                return Type::Byte;
            }
            "split" | "keys" | "values" => {
                return Type::Vector(Box::new(Type::Unknown));
            }
            "clone" => return inner_ty.clone(),
            "assertEq" => return Type::Nil,
            _ => {}
        }
        match inner_ty {
            Type::Named(name) if name.starts_with("plugin:") || name.starts_with("module:") => {
                let prefix = if name.starts_with("plugin:") {
                    &name["plugin:".len()..]
                } else {
                    &name["module:".len()..]
                };

                // Check for Flame-defined functions via indirect access
                let prefixed_member = format!("{}.{}", prefix, member);
                if let Some(sig) = self.functions.get(&prefixed_member).cloned() {
                    let params_str = sig
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret_str = if sig.return_type == Type::Nil {
                        "".to_string()
                    } else {
                        format!(" -> {}", self.format_type(&sig.return_type))
                    };
                    let fallback =
                        format!("```flame\nfn {}({}){}\n```", member, params_str, ret_str);
                    if let Some(doc) = &sig.hover_doc {
                        self.insert_hover_info(span.clone(), format!("{}\n\n{}", fallback, doc));
                    } else {
                        self.insert_hover_info(span.clone(), fallback);
                    }

                    let mut p_tys = Vec::new();
                    for p in &sig.params {
                        p_tys.push(p.ty.clone());
                    }
                    return Type::Function(p_tys, Box::new(sig.return_type));
                }

                if name.starts_with("plugin:") {
                    if let Some(funcs) = self.plugin_functions.get(prefix) {
                        if let Some(sig) = funcs.get(member).cloned() {
                            let params_str = sig
                                .params
                                .iter()
                                .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let ret_str = if sig.return_type == Type::Nil {
                                "".to_string()
                            } else {
                                format!(" -> {}", self.format_type(&sig.return_type))
                            };
                            let fallback =
                                format!("```flame\nfn {}({}){}\n```", member, params_str, ret_str);
                            if let Some(doc) = &sig.hover_doc {
                                self.insert_hover_info(
                                    span.clone(),
                                    format!("{}\n{}", fallback, doc),
                                );
                            } else {
                                self.insert_hover_info(span.clone(), fallback);
                            }
                            return Type::Named("Function".into());
                        }
                    }
                    if let Some(methods) = self.plugin_methods.get(prefix) {
                        if let Some(ty) = methods.get(member) {
                            return ty.clone();
                        }
                    }
                } else {
                    let ret_ty = match (prefix, member) {
                        ("fs", "read") => Type::String,
                        ("fs", "readDir") => Type::Vector(Box::new(Type::String)),
                        ("fs", "readBytes") => Type::Byte,
                        ("fs", "open") => Type::Unknown,
                        ("thread", "sleep") => Type::Nil,
                        ("thread", "channel") => Type::Tuple(vec![Type::Unknown, Type::Unknown]),
                        ("byte", "readBytes") => Type::Byte,
                        ("byte", "readByte") => Type::Byte,
                        ("byte", "readByteAt") => Type::Byte,
                        _ => Type::Unknown,
                    };
                    if !matches!(ret_ty, Type::Unknown) {
                        return ret_ty;
                    }
                }
                Type::Unknown
            }
            Type::Enum(enum_name) => {
                if let Some(info) = self.enums.get(&enum_name).cloned() {
                    if let Some(variant) = info.variants.get(member) {
                        if let Some(doc) = &variant.hover_doc {
                            self.insert_hover_info(span.clone(), doc.clone());
                        }
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
                if let [Type::Formula(fmap, _)] = tuple_items.as_slice() {
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
                    if let Some(sig) = methods.get(member) {
                        let mut params_str = Vec::new();
                        for p in &sig.params {
                            let is_already_ref = matches!(p.ty, Type::Reference { .. });
                            let mut mods = String::new();
                            if p.is_ref && !is_already_ref {
                                mods.push('&');
                            }
                            if p.is_mut && !is_already_ref {
                                mods.push_str("mut ");
                            }
                            params_str.push(format!(
                                "{}{}: {}{}",
                                if p.is_mut && !p.is_ref { "mut " } else { "" },
                                p.name,
                                mods,
                                self.format_type(&p.ty)
                            ));
                        }
                        let ret_str = if sig.return_type == Type::Nil {
                            "".to_string()
                        } else {
                            format!(" -> {}", self.format_type(&sig.return_type))
                        };
                        let mut hover_str = format!(
                            "```flame\nfn {}({}){}\n```",
                            member,
                            params_str.join(", "),
                            ret_str
                        );
                        if let Some(doc) = &sig.hover_doc {
                            hover_str = format!("{}\n\n{}", hover_str, doc);
                        }
                        self.insert_hover_info(span.clone(), hover_str);

                        return Type::Named("Function".into());
                    }
                }

                self.error(
                    format!(
                        "struct '{}' has no field or method '{}'",
                        struct_name, member
                    ),
                    span.clone(),
                    None,
                    None,
                );
                Type::Unknown
            }
            Type::Formula(ref fmap, ref docs) => {
                let ty = fmap.get(member).cloned().unwrap_or(Type::Unknown);
                if let Some(doc) = docs.get(member) {
                    self.insert_hover_info(span.clone(), doc.clone());
                }
                ty
            }
            Type::Quantity(_) | Type::Unit(_) => {
                if member == "value" {
                    Type::Float
                } else {
                    self.error(
                        format!(
                            "type '{}' has no field or method '{}'",
                            self.format_type(&inner_ty),
                            member
                        ),
                        span.clone(),
                        None,
                        None,
                    );
                    Type::Unknown
                }
            }
            Type::Vector(_)
            | Type::String
            | Type::Int
            | Type::Float
            | Type::Bool
            | Type::Tuple(_)
            | Type::Byte => Type::Named("Function".into()),
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
                for (_, field_expr) in fields {
                    self.infer_expr_type(field_expr);
                }
                if let Expr::Identifier(name, _) = inner {
                    Type::Struct(name.clone())
                } else if let Type::Named(name) = base_ty {
                    Type::Struct(name)
                } else {
                    base_ty
                }
            }
        }
    }

    fn infer_call_type(
        &mut self,
        callee: &Expr,
        args: &[(Option<String>, Expr)],
        span: &Span,
    ) -> Type {
        let callee_ty = self.infer_expr_type(callee);
        if let Type::Function(params, ret) = &callee_ty {
            if args.len() != params.len() {
                self.error(
                    format!(
                        "closure expects {} argument(s), got {}",
                        params.len(),
                        args.len()
                    ),
                    span.clone(),
                    None,
                    None,
                );
            }
            for (idx, expected) in params.iter().enumerate() {
                if let Some((_, arg)) = args.get(idx) {
                    let actual = self.infer_expr_type(arg);
                    self.expect_assignable(expected, &actual, &arg.span(), "closure argument");
                }
            }

            // Statically evaluate `unit.Equation` to capture exact units
            if let Expr::Dot(_, member, _) = callee {
                if member == "Equation" && params.len() == 3 {
                    let mut is_literal = true;
                    let mut vals = vec![];
                    for arg in args {
                        if let Expr::Literal(LiteralValue::Int(v), _) = &arg.1 {
                            vals.push(*v as i32);
                        } else if let Expr::Unary(UnaryOp::Neg, inner_arg, _) = &arg.1 {
                            if let Expr::Literal(LiteralValue::Int(v), _) = &**inner_arg {
                                vals.push(-(*v as i32));
                            } else {
                                is_literal = false;
                            }
                        } else {
                            is_literal = false;
                        }
                    }
                    if is_literal && vals.len() == 3 {
                        let mut map = HashMap::new();
                        if vals[0] != 0 {
                            map.insert("kg".to_string(), vals[0]);
                        }
                        if vals[1] != 0 {
                            map.insert("m".to_string(), vals[1]);
                        }
                        if vals[2] != 0 {
                            map.insert("s".to_string(), vals[2]);
                        }
                        return Type::Unit(map);
                    }
                }
            }

            return *ret.clone();
        }

        if let Expr::Identifier(name, id_span) = callee {
            if let Some(sig) = self.functions.get(name).cloned() {
                self.check_call_args(&sig.params, args, span, name);

                let mut params_str = Vec::new();
                for p in &sig.params {
                    let is_already_ref = matches!(p.ty, Type::Reference { .. });
                    let mut mods = String::new();
                    if p.is_ref && !is_already_ref {
                        mods.push('&');
                    }
                    if p.is_mut && !is_already_ref {
                        mods.push_str("mut ");
                    }
                    params_str.push(format!(
                        "{}{}: {}{}",
                        if p.is_mut && !p.is_ref { "mut " } else { "" },
                        p.name,
                        mods,
                        self.format_type(&p.ty)
                    ));
                }

                let ret_str = if sig.return_type == Type::Nil {
                    "".to_string()
                } else {
                    format!(" -> {}", self.format_type(&sig.return_type))
                };
                let mut hover_str = format!(
                    "```flame\nfn {}({}){}\n```",
                    name,
                    params_str.join(", "),
                    ret_str
                );
                if let Some(doc) = &sig.hover_doc {
                    hover_str = format!("{}\n\n{}", hover_str, doc);
                }
                self.insert_hover_info(id_span.clone(), hover_str);

                return sig.return_type;
            }

            if let Some(struct_info) = self.structs.get(name).cloned() {
                self.check_struct_constructor_args(name, &struct_info, args, span);
                let mut hover_str = format!("```flame\nstruct {}\n```", name);
                if let Some(doc) = struct_info.hover_doc {
                    hover_str = format!("{}\n\n{}", hover_str, doc);
                }
                self.insert_hover_info(id_span.clone(), hover_str);
                return Type::Struct(name.clone());
            }

            // Check if it's a globally available enum variant (like Ok, Err, Some, None)
            let mut found_variant = None;
            for (enum_name, enum_info) in &self.enums {
                if let Some(variant) = enum_info.variants.get(name) {
                    found_variant = Some((enum_name.clone(), variant.clone()));
                    break;
                }
            }

            if let Some((enum_name, variant)) = found_variant {
                if variant.struct_fields.is_empty() && !variant.tuple_items.is_empty() {
                    if variant.tuple_items.len() != args.len() {
                        self.error(
                            format!(
                                "enum constructor '{}' expects {} argument(s), got {}",
                                name,
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
                    let mut hover_str = format!("```flame\n{}::{}\n```", enum_name, name);
                    if let Some(doc) = variant.hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }
                    self.insert_hover_info(id_span.clone(), hover_str);
                    return Type::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant_name: name.clone(),
                        tuple_items: variant.tuple_items.clone(),
                        struct_fields: HashMap::new(),
                    };
                }
            }
        }

        if let Expr::Dot(inner, member, _) = callee {
            let mut inner_ty = self.infer_expr_type(inner);
            if let Type::Reference {
                inner: ref_inner, ..
            } = inner_ty
            {
                inner_ty = *ref_inner;
            }

            match member.as_str() {
                "toString" => return Type::String,
                "toInt" | "tryInt" => return Type::Int,
                "toFloat" | "toDouble" | "tryFloat" => return Type::Float,
                "toBool" | "tryBool" => return Type::Bool,
                "toChar" => return Type::String,
                "toByte" => return Type::Byte,
                "assertEq" => return Type::Nil,
                _ => {}
            }

            if let Type::Named(name) = &inner_ty {
                if name.starts_with("plugin:") || name.starts_with("module:") {
                    let prefix = if name.starts_with("plugin:") {
                        &name["plugin:".len()..]
                    } else {
                        &name["module:".len()..]
                    };

                    let prefixed_member = format!("{}.{}", prefix, member);
                    if let Some(sig) = self.functions.get(&prefixed_member).cloned() {
                        self.check_call_args(&sig.params, args, span, member);
                        let params_str = sig
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let ret_str = if sig.return_type == Type::Nil {
                            "".to_string()
                        } else {
                            format!(" -> {}", self.format_type(&sig.return_type))
                        };
                        let fallback =
                            format!("```flame\nfn {}({}){}\n```", member, params_str, ret_str);
                        if let Some(doc) = &sig.hover_doc {
                            self.insert_hover_info(
                                span.clone(),
                                format!("{}\n\n{}", fallback, doc),
                            );
                        } else {
                            self.insert_hover_info(span.clone(), fallback);
                        }
                        return sig.return_type.clone();
                    }

                    if name.starts_with("plugin:") {
                        if let Some(funcs) = self.plugin_functions.get(prefix) {
                            if let Some(sig) = funcs.get(member).cloned() {
                                self.check_call_args(&sig.params, args, span, member);
                                let params_str = sig
                                    .params
                                    .iter()
                                    .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let ret_str = if sig.return_type == Type::Nil {
                                    "".to_string()
                                } else {
                                    format!(" -> {}", self.format_type(&sig.return_type))
                                };
                                let fallback = format!(
                                    "```flame\nfn {}({}){}\n```",
                                    member, params_str, ret_str
                                );
                                if let Some(doc) = &sig.hover_doc {
                                    if doc.trim().starts_with("```") {
                                        self.insert_hover_info(span.clone(), doc.clone());
                                    } else {
                                        self.insert_hover_info(
                                            span.clone(),
                                            format!("{}\n\n{}", fallback, doc),
                                        );
                                    }
                                } else {
                                    self.insert_hover_info(span.clone(), fallback);
                                }
                                return sig.return_type.clone();
                            }
                        }
                        if let Some(methods) = self.plugin_methods.get(prefix) {
                            if let Some(ty) = methods.get(member) {
                                return ty.clone();
                            }
                        }
                    } else {
                        let ret_ty = match (prefix, member.as_str()) {
                            ("fs", "read") => Type::String,
                            ("fs", "readDir") => Type::Vector(Box::new(Type::String)),
                            ("fs", "readBytes") => Type::Byte,
                            ("fs", "open") => Type::Unknown,
                            ("thread", "sleep") => Type::Nil,
                            ("thread", "channel") => {
                                Type::Tuple(vec![Type::Unknown, Type::Unknown])
                            }
                            ("byte", "readBytes") => Type::Byte,
                            ("byte", "readByte") => Type::Byte,
                            ("byte", "readByteAt") => Type::Byte,
                            _ => Type::Unknown,
                        };
                        if !matches!(ret_ty, Type::Unknown) {
                            return ret_ty;
                        }
                    }
                }
            }

            if let Type::Struct(struct_name) = &inner_ty {
                let sig_opt = self
                    .methods
                    .get(struct_name)
                    .and_then(|methods| methods.get(member))
                    .cloned();

                if let Some(sig) = sig_opt {
                    let params_to_check = if sig.is_static {
                        &sig.params[..]
                    } else if !sig.params.is_empty() {
                        &sig.params[1..]
                    } else {
                        &[]
                    };
                    self.check_call_args(params_to_check, args, span, member);

                    let mut params_str = Vec::new();
                    for p in &sig.params {
                        let is_already_ref = matches!(p.ty, Type::Reference { .. });
                        let mut mods = String::new();
                        if p.is_ref && !is_already_ref {
                            mods.push('&');
                        }
                        if p.is_mut && !is_already_ref {
                            mods.push_str("mut ");
                        }
                        params_str.push(format!(
                            "{}{}: {}{}",
                            if p.is_mut && !p.is_ref { "mut " } else { "" },
                            p.name,
                            mods,
                            self.format_type(&p.ty)
                        ));
                    }
                    let ret_str = if sig.return_type == Type::Nil {
                        "".to_string()
                    } else {
                        format!(" -> {}", self.format_type(&sig.return_type))
                    };
                    let mut hover_str = format!(
                        "```flame\nfn {}({}){}\n```",
                        member,
                        params_str.join(", "),
                        ret_str
                    );
                    if let Some(doc) = &sig.hover_doc {
                        hover_str = format!("{}\n\n{}", hover_str, doc);
                    }
                    self.insert_hover_info(span.clone(), hover_str);

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

            if let Type::Byte = &inner_ty {
                match member.as_str() {
                    "toHex" | "toBase64" | "toUtf8" | "tryUtf8" => {
                        self.check_call_args(&[], args, span, member);
                        return match member.as_str() {
                            "toHex" | "toBase64" | "toUtf8" => Type::String,
                            "tryUtf8" => Type::Named("String?".to_string()),
                            _ => Type::Unknown,
                        };
                    }
                    "concat" => {
                        self.check_call_args(
                            &[ParamInfo {
                                name: "other".into(),
                                ty: Type::Byte,
                                is_ref: false,
                                is_mut: false,
                            }],
                            args,
                            span,
                            member,
                        );
                        return Type::Byte;
                    }
                    "len" => {
                        self.check_call_args(&[], args, span, member);
                        return Type::Int;
                    }
                    "type" => {
                        self.check_call_args(&[], args, span, member);
                        return Type::String;
                    }
                    _ => {
                        self.error(
                            format!("Bytes has no method '{}'", member),
                            span.clone(),
                            None,
                            None,
                        );
                        return Type::Unknown;
                    }
                }
            }

            if let Type::Vector(element_ty) = &inner_ty {
                match member.as_str() {
                    "push" => {
                        self.check_call_args(
                            &[ParamInfo {
                                name: "item".into(),
                                ty: *element_ty.clone(),
                                is_ref: false,
                                is_mut: false,
                            }],
                            args,
                            span,
                            member,
                        );
                        return Type::Nil;
                    }
                    "pop" => {
                        self.check_call_args(&[], args, span, member);
                        return *element_ty.clone();
                    }
                    "len" => {
                        self.check_call_args(&[], args, span, member);
                        return Type::Int;
                    }
                    "filter" => {
                        let cb_ty = Type::Function(vec![*element_ty.clone()], Box::new(Type::Bool));
                        self.check_call_args(
                            &[ParamInfo {
                                name: "cb".into(),
                                ty: cb_ty,
                                is_ref: false,
                                is_mut: false,
                            }],
                            args,
                            span,
                            member,
                        );
                        return Type::Vector(element_ty.clone());
                    }
                    "map" => {
                        if let Some((_, arg)) = args.get(0) {
                            let arg_ty = self.infer_expr_type(arg);
                            if let Type::Function(_, ret) = arg_ty {
                                return Type::Vector(ret);
                            }
                        }
                        return Type::Unknown;
                    }
                    "type" | "toHex" | "toBase64" | "concat" | "assertEq" => {
                        return Type::Unknown;
                    }
                    _ => {
                        self.error(
                            format!("array has no method '{}'", member),
                            span.clone(),
                            None,
                            None,
                        );
                        return Type::Unknown;
                    }
                }
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
                    if matches!(actual, Type::String) && matches!(param.ty, Type::String) {
                        self.expect_assignable(
                            &param.ty,
                            &actual,
                            &arg.span(),
                            "function argument (string by value to ref)",
                        );
                    } else {
                        if param.is_mut {
                            if let Type::Reference { mutable, .. } = &actual {
                                if !mutable {
                                    self.error(
                                        format!(
                                            "parameter '{}' requires '&mut' but argument is not mutable",
                                            param.name
                                        ),
                                        arg.span(),
                                        None,
                                        None,
                                    );
                                }
                            }
                        }
                        self.expect_assignable(
                            &param.ty,
                            &actual,
                            &arg.span(),
                            "function argument (by reference)",
                        );
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
        if let Type::Named(name) = expected {
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                return true;
            }
        }
        if expected == actual {
            return true;
        }
        // Allow Type::String to be assigned to &'staticstr
        if matches!(actual, Type::String) {
            if let Type::Reference { inner, .. } = expected {
                if let Type::Named(name) = &**inner {
                    if name == "'staticstr" || name == "'static str" {
                        return true;
                    }
                }
            } else if let Type::Named(name) = expected {
                if name == "&'staticstr" || name == "&'static str" {
                    return true;
                }
            }
        }

        // Fallback for cases where structural equality fails but formatted strings match.
        if self.format_type(expected) == self.format_type(actual) {
            return true;
        }

        match (expected, actual) {
            (Type::Float, Type::Int) => true,
            (Type::Byte, Type::Int) => true,
            (Type::Int, Type::Byte) => true,
            (Type::Enum(expected_name), Type::EnumVariant { enum_name, .. }) => {
                expected_name == enum_name
            }
            (Type::EnumVariant { enum_name: e1, .. }, Type::EnumVariant { enum_name: e2, .. }) => {
                e1 == e2
            }
            (Type::EnumVariant { enum_name, .. }, Type::Enum(actual_name)) => {
                enum_name == actual_name
            }
            (Type::Enum(e1), Type::Enum(e2)) => e1 == e2,
            (Type::Tuple(expected_items), Type::Function(a_params, _))
                if expected_items.is_empty() && a_params.is_empty() =>
            {
                true
            }
            (Type::Named(expected_name), Type::Struct(actual_name))
            | (Type::Named(expected_name), Type::Enum(actual_name)) => {
                expected_name == actual_name
                    || expected_name.starts_with(&format!("{}<", actual_name))
            }
            (Type::Struct(expected_name), Type::Named(actual_name))
            | (Type::Enum(expected_name), Type::Named(actual_name)) => {
                expected_name == actual_name
                    || actual_name.starts_with(&format!("{}<", expected_name))
            }
            (Type::Named(expected_name), Type::EnumVariant { enum_name, .. }) => {
                expected_name == enum_name || expected_name.starts_with(&format!("{}<", enum_name))
            }
            (Type::Byte, Type::Byte) => true,
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
            (Type::Reference { inner, .. }, actual) => self.is_compatible(inner, actual),
            (expected, Type::Reference { inner, .. }) => self.is_compatible(expected, inner),
            (Type::Named(expected_name), Type::Named(actual_name)) => {
                expected_name == actual_name
                    || expected_name.split('<').next() == actual_name.split('<').next()
            }
            (Type::Formula(_, _), Type::Formula(_, _)) => true,
            (Type::Function(e_params, e_ret), Type::Function(a_params, a_ret)) => {
                e_params.len() == a_params.len()
                    && e_params
                        .iter()
                        .zip(a_params.iter())
                        .all(|(expected, actual)| self.is_compatible(expected, actual))
                    && self.is_compatible(e_ret, a_ret)
            }
            _ => false,
        }
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }

    fn parse_type_name(&self, type_name: &str) -> Type {
        let trimmed = type_name.trim();
        if trimmed == "Object" || trimmed == "Formula" {
            return Type::Formula(HashMap::new(), HashMap::new());
        }
        if trimmed == "&str"
            || trimmed == "&'static str"
            || trimmed == "&'staticstr"
            || trimmed == "str"
            || trimmed == "'static str"
            || trimmed == "'staticstr"
        {
            return Type::String;
        }
        if let Some(rest) = trimmed.strip_prefix("&mut ") {
            return Type::Reference {
                inner: Box::new(self.parse_type_name(rest)),
                mutable: true,
            };
        }
        if let Some(rest) = trimmed.strip_prefix('&') {
            return Type::Reference {
                inner: Box::new(self.parse_type_name(rest)),
                mutable: false,
            };
        }
        if let Some(rest) = trimmed.strip_suffix('?') {
            return self.parse_type_name(rest);
        }
        match trimmed {
            "Int" | "I32" | "I64" | "U32" | "U64" | "i32" | "i64" | "u32" | "u64" => Type::Int,
            "Float" | "F32" | "F64" | "f32" | "f64" => Type::Float,
            "String" | "string" | "str" | "'static str" => Type::String,
            "Bool" | "bool" => Type::Bool,
            "Nil" | "nil" => Type::Nil,
            "Byte" | "Bytes" | "u8" | "U8" => Type::Byte,
            "Formula" | "Object" => Type::Formula(HashMap::new(), HashMap::new()),
            _ if trimmed.len() == 1 && trimmed.chars().next().unwrap().is_uppercase() => {
                Type::Named(trimmed.to_string())
            }
            _ if trimmed.contains("->") => {
                if let Some((left, right)) = trimmed.split_once("->") {
                    let left_type = self.parse_type_name(left.trim());
                    let right_type = self.parse_type_name(right.trim());
                    let params = match left_type {
                        Type::Tuple(items) => items,
                        Type::Unknown | Type::Nil => Vec::new(),
                        other => vec![other], // e.g. Int -> String
                    };
                    return Type::Function(params, Box::new(right_type));
                }
                Type::Named(trimmed.to_string())
            }
            _ if trimmed.starts_with('[') && trimmed.ends_with(']') => {
                let inner = &trimmed[1..trimmed.len() - 1];
                Type::Vector(Box::new(self.parse_type_name(inner)))
            }
            _ if trimmed.starts_with("Vec<") && trimmed.ends_with('>') => {
                let inner = &trimmed[4..trimmed.len() - 1];
                Type::Vector(Box::new(self.parse_type_name(inner)))
            }
            _ if trimmed.starts_with('(') && trimmed.ends_with(')') => {
                let inner = &trimmed[1..trimmed.len() - 1];
                if inner.trim().is_empty() {
                    Type::Tuple(Vec::new())
                } else {
                    let mut parts = Vec::new();
                    let mut current = String::new();
                    let mut depth = 0;
                    for c in inner.chars() {
                        if c == '(' || c == '[' || c == '<' {
                            depth += 1;
                            current.push(c);
                        } else if c == ')' || c == ']' || c == '>' {
                            depth -= 1;
                            current.push(c);
                        } else if c == ',' && depth == 0 {
                            parts.push(current.trim().to_string());
                            current.clear();
                        } else {
                            current.push(c);
                        }
                    }
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    Type::Tuple(
                        parts
                            .into_iter()
                            .map(|part| self.parse_type_name(&part))
                            .collect(),
                    )
                }
            }
            _ if self.structs.contains_key(trimmed) => Type::Struct(trimmed.to_string()),
            _ if self.enums.contains_key(trimmed) => Type::Enum(trimmed.to_string()),
            _ => Type::Named(trimmed.to_string()),
        }
    }

    pub fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::String => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Nil => "Nil".to_string(),
            Type::Byte => "Byte".to_string(),
            Type::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .map(|item| self.format_type(item))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Vector(item) => format!("[{}]", self.format_type(item)),
            Type::Quantity(map) => {
                let mut terms = Vec::new();
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for k in keys {
                    let v = map[k];
                    if v == 1 {
                        terms.push(k.clone());
                    } else {
                        terms.push(format!("{}^{}", k, v));
                    }
                }
                if terms.is_empty() {
                    "Quantity".to_string()
                } else {
                    format!("Quantity <{}>", terms.join(" * "))
                }
            }
            Type::Unit(map) => {
                let mut terms = Vec::new();
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for k in keys {
                    let v = map[k];
                    if v == 1 {
                        terms.push(k.clone());
                    } else {
                        terms.push(format!("{}^{}", k, v));
                    }
                }
                if terms.is_empty() {
                    "Unit".to_string()
                } else {
                    format!("Unit <{}>", terms.join(" * "))
                }
            }
            Type::Formula(_, _) => "Formula".to_string(),
            Type::Function(params, ret) => {
                let params_str = params
                    .iter()
                    .map(|p| self.format_type(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, self.format_type(ret))
            }
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

    fn parse_command_annotation(
        &self,
        func_name: &str,
        annotations: &[crate::parser::Annotation],
        params: &[crate::parser::Param],
        func_span: &Span,
    ) -> Option<CommandInfo> {
        let cmd_anno = annotations.iter().find(|a| a.name == "Command")?;
        let mut cmd_name = None;
        let mut about = None;

        for (idx, arg) in cmd_anno.args.iter().enumerate() {
            let trimmed = arg.trim();
            if trimmed.starts_with("name:")
                || trimmed.starts_with("name :")
                || trimmed.starts_with("name=")
                || trimmed.starts_with("name =")
            {
                let val = if let Some((_, v)) = trimmed.split_once(':') {
                    v
                } else if let Some((_, v)) = trimmed.split_once('=') {
                    v
                } else {
                    trimmed
                };
                cmd_name = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            } else if trimmed.starts_with("about:")
                || trimmed.starts_with("about :")
                || trimmed.starts_with("about=")
                || trimmed.starts_with("about =")
            {
                let val = if let Some((_, v)) = trimmed.split_once(':') {
                    v
                } else if let Some((_, v)) = trimmed.split_once('=') {
                    v
                } else {
                    trimmed
                };
                about = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            } else if trimmed.starts_with("description:")
                || trimmed.starts_with("description :")
                || trimmed.starts_with("description=")
                || trimmed.starts_with("description =")
            {
                let val = if let Some((_, v)) = trimmed.split_once(':') {
                    v
                } else if let Some((_, v)) = trimmed.split_once('=') {
                    v
                } else {
                    trimmed
                };
                about = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            } else {
                let unquoted = trimmed.trim_matches('"').trim_matches('\'').to_string();
                if idx == 0 && cmd_name.is_none() {
                    cmd_name = Some(unquoted);
                } else if idx == 1 && about.is_none() {
                    about = Some(unquoted);
                }
            }
        }

        let final_name = cmd_name.unwrap_or_else(|| func_name.to_string());
        let anno_sig = match &about {
            Some(ab) => format!("@Command(name: \"{}\", about: \"{}\")", final_name, ab),
            None => format!("@Command(name: \"{}\")", final_name),
        };

        let mut doc = format!(
            "```flame\n{}\n```\n**CLI Subcommand**: `{}`",
            anno_sig, final_name
        );
        if let Some(ab) = &about {
            doc.push_str(&format!("\n\n{}", ab));
        }

        if !params.is_empty() {
            doc.push_str("\n\n**Arguments & Flags:**");
            for p in params {
                let default_str = if let Some(def) = &p.default_val {
                    format!(" = {}", format_expr_simple(def))
                } else {
                    String::new()
                };
                doc.push_str(&format!(
                    "\n- `--{}`: `{}`{}",
                    p.name, p.type_name, default_str
                ));
            }
        }

        Some(CommandInfo {
            name: final_name,
            about,
            func_name: func_name.to_string(),
            params: params
                .iter()
                .map(|p| ParamInfo {
                    name: p.name.clone(),
                    ty: self.parse_type_name(&p.type_name),
                    is_ref: p.is_ref,
                    is_mut: p.is_mut,
                })
                .collect(),
            hover_doc: doc,
            span: func_span.clone(),
        })
    }
}

fn format_expr_simple(expr: &Expr) -> String {
    match expr {
        Expr::Literal(LiteralValue::Int(i), _) => i.to_string(),
        Expr::Literal(LiteralValue::Float(f), _) => f.to_string(),
        Expr::Literal(LiteralValue::String(s), _) => format!("\"{}\"", s),
        Expr::Literal(LiteralValue::Bool(b), _) => b.to_string(),
        Expr::Literal(LiteralValue::Nil, _) => "nil".to_string(),
        Expr::Identifier(id, _) => id.clone(),
        Expr::VectorLiteral(items, _) => {
            let inner = items
                .iter()
                .map(format_expr_simple)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        }
        _ => "...".to_string(),
    }
}
