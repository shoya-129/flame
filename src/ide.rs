use crate::{JsonCompletion, JsonHover};
use regex::Regex;

const KEYWORDS: &[(&str, &str)] = &[
    ("let", "Declares a local variable. Example: `let x = 5`"),
    ("const", "Declares a constant. Example: `const x = 5`"),
    (
        "mut",
        "Keyword for mutable reference/variable. Example: `let mut x = 5`",
    ),
    ("fn", "Declares a function. Example: `fn do_something() {}`"),
    (
        "async",
        "Declares an asynchronous function. Example: `async fn do_something() {}`",
    ),
    (
        "struct",
        "Declares a struct. Example: `struct Point { x: Num, y: Num }`",
    ),
    (
        "enum",
        "Declares an enum. Example: `enum Color { Red, Green, Blue }`",
    ),
    (
        "trait",
        "Declares a trait (interface). Example: `trait Drawable { fn draw(); }`",
    ),
    (
        "impl",
        "Implements methods for a struct or enum. Example: `impl Point { fn new() -> Point {} }`",
    ),
    (
        "if",
        "Conditional execution. Example: `if condition { ... }`",
    ),
    (
        "else",
        "Alternative conditional execution. Example: `else { ... }`",
    ),
    (
        "while",
        "Looping construct based on condition. Example: `while condition { ... }`",
    ),
    (
        "loop",
        "Infinite looping construct. Example: `loop { ... }`",
    ),
    (
        "for",
        "Iterating construct. Example: `for i in 0..10 { ... }`",
    ),
    (
        "in",
        "Used in for-loops or containment checks. Example: `for x in list`",
    ),
    (
        "match",
        "Pattern matching. Example: `match x { 1 => ..., _ => ... }`",
    ),
    ("return", "Returns from a function. Example: `return 5`"),
    ("break", "Breaks out of a loop. Example: `break`"),
    (
        "continue",
        "Skips to the next iteration of a loop. Example: `continue`",
    ),
    (
        "defer",
        "Defers execution until the end of the scope. Example: `defer file.close()`",
    ),
    ("import", "Imports a module. Example: `import std.fs`"),
    (
        "native",
        "Native dependencies import prefix. Example: `import native.mysql`",
    ),
    (
        "std",
        "Standard library import prefix. Example: `import std.math`",
    ),
    (
        "thread",
        "Spawns a new background thread block. Example: `thread { thread.sleep(3000) }`",
    ),
    (
        "await",
        "Waits for an asynchronous task or thread to complete. Example: `await task`",
    ),
    (
        "print",
        "Prints to standard output. Example: `print(\"hello\")`",
    ),
    (
        "eprint",
        "Prints to standard error. Example: `eprint(\"error\")`",
    ),
    (
        "formula",
        "Declares a static map/dictionary structure. Example: `formula { key: \"value\" }`",
    ),
    (
        "Formula",
        "Built-in Type: A map-like literal data structure.",
    ),
    ("Int", "Built-in Type: A 64-bit signed integer."),
    ("Float", "Built-in Type: A 64-bit floating point number."),
    ("String", "Built-in Type: A UTF-8 text string."),
    ("Bool", "Built-in Type: A boolean value (true or false)."),
    ("Nil", "Built-in Type: Represents the absence of a value."),
    ("Vec", "Built-in Type: A dynamically-sized array."),
    (
        "ThreadHandler",
        "Built-in Type: A handle to a spawned background thread.",
    ),
    (
        "input",
        "```flame\nfn input(prompt: String) -> String\n```\nTakes a line of text input from standard input.\n\nExample:\n```flame\nlet name = input(\"Name: \")\n```",
    ),
    (
        "push",
        "Built-in Method: Appends an element to the back of a collection. Example: `arr.push(100)`",
    ),
    (
        "pop",
        "Built-in Method: Removes the last element from a collection and returns it. Example: `let last = arr.pop()`",
    ),
    (
        "len",
        "Built-in Method: Returns the number of elements in the collection. Example: `let l = arr.len()`",
    ),
    (
        "is_empty",
        "Built-in Method: Returns true if the collection contains no elements. Example: `if arr.is_empty() { ... }`",
    ),
    (
        "filter",
        "Built-in Method: Creates a new array containing elements that pass the provided test function. Example: `let filtered = arr.filter((x: Int) { return x > 10 })`",
    ),
    (
        "map",
        "Built-in Method: Creates a new array populated with the results of calling a provided function on every element. Example: `let mapped = arr.map((x: Int) { return x * 2 })`",
    ),
    (
        "annotation",
        "Declares a reusable custom annotation function. Example: `annotation Benchmark(name: String) -> Formula {}`",
    ),
    (
        "Cli",
        "```flame\n@Cli(name: String, version: String = \"0.1.0\", description: String = \"\")\n```\n**CLI Annotation**\nMarks an entry function or module as a CLI root. Use with `@Command`-annotated functions to describe subcommands, flags, and positional arguments for IDE/documentation tooling.",
    ),
    (
        "Command",
        "```flame\n@Command(name: String, about: String = \"\")\n```\n**CLI Annotation**\nMarks a function as a CLI subcommand handler. Parameters describe positional arguments and options; Bool parameters map naturally to flags.",
    ),
    (
        "Test",
        "```flame\n@Test(timeout: Int = 5000, skip: Bool = false, only: Bool = false, tags: Vector<String> = [])\n```\n**Annotated Function**\nMarks a function as a test.\nRuns only when `flame test` executes.\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Setup",
        "```flame\n@Setup\n```\n**Annotated Function**\nRuns before every test in the module (equivalent to `beforeEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Cleanup",
        "```flame\n@Cleanup\n```\n**Annotated Function**\nRuns after every test in the module (equivalent to `afterEach()`).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "BeforeAll",
        "```flame\n@BeforeAll\n```\n**Annotated Function**\nRuns exactly once before any test executes (e.g., database connection setup).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "AfterAll",
        "```flame\n@AfterAll\n```\n**Annotated Function**\nRuns exactly once after all tests complete (e.g., closing servers or cleaning temp files).\nIgnored during `flame run` and `flame build`.",
    ),
    (
        "Ignore",
        "```flame\n@Ignore\n```\n**Annotated Function**\nSkips test execution when running `flame test`.",
    ),
    (
        "Only",
        "```flame\n@Only\n```\n**Annotated Function**\nRestricts test execution to ONLY functions marked with `@Only` during `flame test`.",
    ),
    (
        "Parameterized",
        "```flame\n@Parameterized(arguments: Vector<Tuple>)\n```\n**Annotated Function**\nExpands a test into multiple independent test cases, passing each tuple element as parameters to the test function.",
    ),
    (
        "Benchmark",
        "```flame\n@Benchmark\n```\n**Annotated Function**\nExecutes the function as a high-precision performance benchmark during `flame test`, reporting average, minimum, and maximum execution times.",
    ),
    (
        "ExpectPanic",
        "```flame\n@ExpectPanic\n```\n**Annotated Function**\nAsserts that the test function MUST terminate with a panic or error; fails if the function completes successfully.",
    ),
    (
        "Cli",
        "```flame\n@Cli(name: String, version: String = \"1.0.0\")\n```\n**Annotated Function**\nAutomatically transforms the annotated function into a command-line interface (CLI) router. When this function is called, Flame parses `std::env::args()` and automatically dispatches execution to functions annotated with `@Command`.",
    ),
    (
        "Command",
        "```flame\n@Command(name: String)\n```\n**Annotated Function**\nRegisters the function as a CLI subcommand for the `@Cli` router. The router will automatically parse CLI flags/arguments based on this function's parameters and invoke it.",
    ),
    (
        "toInt",
        "```flame\nfn toInt(radix: Int = 10) -> Int\n```\nConverts a String value to a signed integer. Throws a runtime error if digits are invalid.",
    ),
    (
        "tryInt",
        "```flame\nfn tryInt(radix: Int = 10) -> Int | Nil\n```\nAttempts to parse a String value to an integer, returning `nil` if parsing fails.",
    ),
    (
        "toFloat",
        "```flame\nfn toFloat() -> Float\n```\nConverts a String value to a floating-point number.",
    ),
    (
        "tryFloat",
        "```flame\nfn tryFloat() -> Float | Nil\n```\nAttempts to parse a String value to a float, returning `nil` if parsing fails.",
    ),
    (
        "toDouble",
        "```flame\nfn toDouble() -> Float\n```\nConverts a String value to a double-precision floating-point number.",
    ),
    (
        "tryDouble",
        "```flame\nfn tryDouble() -> Float | Nil\n```\nAttempts to parse a String value to a double, returning `nil` if parsing fails.",
    ),
    (
        "toBool",
        "```flame\nfn toBool() -> Bool\n```\nConverts a String (\"true\", \"false\", \"1\", \"0\", etc.) to a boolean value.",
    ),
    (
        "tryBool",
        "```flame\nfn tryBool() -> Bool | Nil\n```\nAttempts to convert a String to a boolean value, returning `nil` if unrecognized.",
    ),
    (
        "toChar",
        "```flame\nfn toChar() -> String\n```\nReturns a string consisting of the first character.",
    ),
    (
        "toBytes",
        "```flame\nfn toBytes() -> Vector<Int>\n```\nConverts a UTF-8 String into an array of byte integers.",
    ),
    (
        "toString",
        "```flame\nfn toString(precision: Int = -1) -> String\n```\nConverts any value (integer, float, boolean, nil, byte array) into its String representation. For floats, specifying precision limits decimal digits.",
    ),
];

pub fn get_keyword_completions(prefix: &str) -> Vec<JsonCompletion> {
    KEYWORDS
        .iter()
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
    KEYWORDS
        .iter()
        .find(|(kw, _)| *kw == word)
        .map(|(kw, doc)| {
            let formatted_doc;
            if doc.starts_with("```") {
                formatted_doc = doc.to_string();
            } else if let Some((desc, ex)) = doc.split_once("Example: `") {
                let clean_ex = ex.trim_end_matches('`');
                let kind = if desc.starts_with("Built-in Function:")
                    || desc.starts_with("Built-in Method:")
                {
                    "fn"
                } else {
                    "keyword"
                };
                formatted_doc = format!(
                    "```flame\n{} {}\n```\n{}\n\n**Example:**\n```flame\n{}\n```",
                    kind,
                    kw,
                    desc.trim(),
                    clean_ex
                );
            } else if doc.starts_with("Built-in Type:") {
                formatted_doc = format!("```flame\ntype {}\n```\n{}", kw, doc.trim());
            } else if doc.starts_with("Built-in Function:") || doc.starts_with("Built-in Method:") {
                formatted_doc = format!("```flame\nfn {}\n```\n{}", kw, doc.trim());
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
        structs.push(ScannedStruct {
            name,
            fields,
            methods: vec![],
        });
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
    let var_re =
        Regex::new(r"(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*(?:=\s*([a-zA-Z_]\w*))?").unwrap();
    for cap in var_re.captures_iter(content) {
        let name = cap[1].to_string();
        let typ = cap.get(2).map(|m| m.as_str().to_string());
        vars.push(ScannedVar { name, typ });
    }

    // Scan for variables with formula bodies to extract fields
    let formula_re =
        Regex::new(r"(?s)(?:let|const)(?:\s+mut)?\s+([a-zA-Z_]\w*)\s*=\s*formula\s*\{([^}]*)\}");
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
            vars.insert(
                0,
                ScannedVar {
                    name,
                    typ: Some(synthetic_type),
                },
            );
        }
    }

    // Scan for function and annotation decls: `fn name(a: Type, b: Type)` or `annotation name(...) -> Ret`
    let fn_decl_re = Regex::new(
        r"(fn|annotation)\s+([a-zA-Z_]\w*)\s*\(([^)]*)\)(?:\s*->\s*([a-zA-Z0-9_<>, \t]+))?",
    )
    .unwrap();
    let arg_re = Regex::new(r"([a-zA-Z_]\w*)\s*:\s*([a-zA-Z_]\w*)").unwrap();
    for cap in fn_decl_re.captures_iter(content) {
        let kind_kw = &cap[1];
        let name_str = &cap[2];
        let params_str = cap[3].trim();
        let ret_str = cap.get(4).map_or("()", |m| m.as_str().trim());

        let sig = if kind_kw == "annotation" {
            if ret_str == "()" {
                format!("annotation {}({})", name_str, params_str)
            } else {
                format!("annotation {}({}) -> {}", name_str, params_str, ret_str)
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
        });

        let args_body = &cap[3];
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

    let base_module = if base == "std" { parts.next()? } else { base };

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
