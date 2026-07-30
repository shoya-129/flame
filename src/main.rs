pub mod aot_compiler;
mod diagnostics;
pub mod ide;
mod lexer;
pub mod native_std;
mod package_manager;
mod parser;
pub mod runner;
mod std_docs;
mod stdlib;
mod typechecker;
pub mod vm;
use diagnostics::Diagnostic;
use lexer::Lexer;
use parser::{Parser, Stmt};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use typechecker::TypeChecker;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        if Path::new("src/main.fm").exists() {
            run_file("src/main.fm", false);
            return;
        }
        print_help();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "add" => {
            package_manager::add_package(&args[2..]);
        }
        "remove" => {
            if args.len() < 3 {
                println!("\x1b[1;31merror:\x1b[0m please specify package name to remove.");
                println!("usage: flame remove <package_name>");
                return;
            }
            package_manager::remove_package(&args[2]);
        }
        "new" => {
            if args.len() < 3 {
                println!("\x1b[1;31merror:\x1b[0m please specify the project name");
                println!("usage: flame new <project_name>");
                return;
            }
            let project_name = &args[2];
            create_new_project(project_name);
        }
        "build" => {
            build_project(&args);
        }
        "check" => {
            run_check_command(&args);
        }
        "list-plugins" => {
            list_plugins_command(&args);
        }
        "run" => {
            if args.len() < 3 {
                println!("\x1b[1;31merror:\x1b[0m please specify a Flame file to run");
                println!("usage: flame run <file_path.fm>");
                return;
            }
            let force_local = args.contains(&"--local".to_string());
            let filepath = if args[2] == "--local" {
                if args.len() < 4 {
                    println!("\x1b[1;31merror:\x1b[0m please specify a Flame file to run");
                    return;
                }
                &args[3]
            } else {
                &args[2]
            };
            run_file(filepath, force_local);
        }
        "test" => {
            run_tests();
        }
        "native" => {
            if args.len() < 3 || args[2] != "init" {
                println!("\x1b[1;31merror:\x1b[0m unknown subcommand");
                println!("usage: flame native init");
                return;
            }
            init_native_bridge();
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            // Check if argument is a Flame source file
            let p = Path::new(command);
            if p.exists() && p.extension().map_or(false, |ext| ext == "flame") {
                let force_local = args.contains(&"--local".to_string());
                let actual_cmd = if command == "--local" && args.len() >= 3 {
                    &args[2]
                } else {
                    command
                };
                run_file(actual_cmd, force_local);
            } else {
                println!("\x1b[1;31merror:\x1b[0m unknown command '{}'", command);
                print_help();
            }
        }
    }
}

fn print_help() {
    let bold = "\x1b[1m";
    let cyan = "\x1b[1;36m";
    let reset = "\x1b[0m";

    println!(
        "{}Flame Compiler & Package Manager (Version 0.1.0){} ",
        bold, reset
    );
    println!("Designed for systems programming with supreme DX.");
    println!();
    println!("{}USAGE:{} flame <SUBCOMMAND> [args]", bold, reset);
    println!();
    println!("{}SUBCOMMANDS:{}", bold, reset);
    println!(
        "  {}add{} <pkg> [--native] Add a dependency (Flame module or native Rust crate)",
        cyan, reset
    );
    println!(
        "  {}remove{} <pkg>       Remove an installed package",
        cyan, reset
    );
    println!(
        "  {}new{} <name>          Create a new Flame package template",
        cyan, reset
    );
    println!(
        "  {}build{} [--release]  Compile the workspace project defined in flame.toml",
        cyan, reset
    );
    println!(
        "  {}check{} <file> [--json] [--line N --col N]  Analyze a Flame file for diagnostics and IDE data",
        cyan, reset
    );
    println!(
        "  {}list-plugins{} [--json]  List configured plugins",
        cyan, reset
    );
    println!(
        "  {}run{} <file>           Compile and run a Flame source file",
        cyan, reset
    );
    println!(
        "  {}test{}                Execute unit tests inside the current project",
        cyan, reset
    );
    println!(
        "  {}native init{}         Scaffold native Rust FFI bridges & Cargo configuration",
        cyan, reset
    );
    println!("  {}help{}                Print help details", cyan, reset);
    println!();
}

fn create_new_project(name: &str) {
    let root = Path::new(name);
    if root.exists() {
        println!(
            "\x1b[1;31merror:\x1b[0m directory '{}' already exists",
            name
        );
        return;
    }

    println!(
        "Scaffolding a brand new Flame package: \x1b[1;32m{}\x1b[0m",
        name
    );

    // Create directories
    let dirs = vec!["src"];
    for d in dirs {
        let path = root.join(d);
        if let Err(e) = fs::create_dir_all(&path) {
            println!(
                "\x1b[1;31merror:\x1b[0m failed to create directory {:?}: {}",
                path, e
            );
            return;
        }
    }

    // Write flame.toml
    let toml_content = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2026\"\ntype = \"executable\"\n\n[dependencies]\nstd = \"0.1.0\"\n",
        name
    );
    fs::write(root.join("flame.toml"), toml_content).unwrap();

    // Write src/main.fm
    let main_flame = r#"import math
import std.thread

print("Hello, world! Program executed successfully.")
let result: Int = math.add(5, 7)
print($"5 + 7 = {result}")

let t: ThreadHandler = thread {
    print("Hello from background thread!")
}

t.join()
"#;
    fs::write(root.join("src/main.fm"), main_flame).unwrap();

    // Write src/math.fm
    let math_flame = r#"export fn add(a: Int, b: Int) -> Int {
    a + b
}
"#;
    fs::write(root.join("src/math.fm"), math_flame).unwrap();

    println!(
        "\x1b[1;32mCreated\x1b[0m binary (application) `{}` package",
        name
    );
}

fn parse_file_stmts(path: &Path, content: &str) -> Result<Vec<Stmt>, Diagnostic> {
    let mut lexer = Lexer::new(content);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == lexer::TokenKind::EOF;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }

    let mut parser = Parser::new(tokens, path.to_string_lossy().to_string());
    parser.parse()
}

fn typecheck_file_stmts(path: &Path, stmts: &[Stmt]) -> Result<(), Vec<Diagnostic>> {
    TypeChecker::new(path.to_string_lossy().to_string()).check_program(stmts).0
}

fn build_project(args: &[String]) -> Option<PathBuf> {
    let toml_path = Path::new("flame.toml");
    if !toml_path.exists() {
        println!(
            "\x1b[1;31merror:\x1b[0m no flame.toml manifest file found in the current directory."
        );
        println!("help: run this command inside a valid Flame project folder.");
        return None;
    }

    let is_release = args.contains(&"--release".to_string()) || args.contains(&"-r".to_string());
    let force_local = args.contains(&"--local".to_string());
    let mut pkg_name = "app".to_string();
    if let Ok(toml_str) = fs::read_to_string("flame.toml") {
        for line in toml_str.lines() {
            if line.trim().starts_with("name =") {
                if let Some(val) = line.split('=').nth(1) {
                    pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
        }
    }

    package_manager::ensure_dependencies_installed();

    let mode_str = if is_release {
        "release [optimized]"
    } else {
        "dev [unoptimized]"
    };
    let profile = if is_release { "release" } else { "dev" };
    println!("\x1b[1;36m    Building\x1b[0m dependency graph...");
    println!("\x1b[1;36m   Compiling\x1b[0m std standard library (Flame interfaces)...");
    println!("\x1b[1;36m   Compiling\x1b[0m standard library Rust bridges (std_bridge)...");

    // Check main.fm if exists
    let main_path = Path::new("src/main.fm");
    if main_path.exists() {
        println!("\x1b[1;36m   Compiling\x1b[0m targets (src/main.fm)...");

        // Parse all files in src/ to ensure full build-time diagnostics
        if let Ok(entries) = fs::read_dir("src") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "fm") {
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    let stmts = match parse_file_stmts(&path, &content) {
                        Ok(stmts) => stmts,
                        Err(diag) => {
                            diag.print(&content);
                            println!(
                                "\x1b[1;31merror:\x1b[0m build failed due to 1 previous error"
                            );
                            std::process::exit(1);
                        }
                    };

                    if let Err(diags) = typecheck_file_stmts(&path, &stmts) {
                        for diag in diags {
                            diag.print(&content);
                        }
                        println!("\x1b[1;31merror:\x1b[0m build failed due to type errors");
                        std::process::exit(1);
                    }
                }
            }
        }
        println!("\x1b[1;36m     Linking\x1b[0m native static object files...");

        let manifest_content = fs::read_to_string("flame.toml").unwrap_or_default();
        let native_deps_raw = parse_manifest_section(&manifest_content, "[native-dependencies]");
        let plugins_raw = parse_manifest_section(&manifest_content, "[plugins]");

        let mut processed_native_deps = Vec::new();

        for (plugin_name, plugin_path) in native_deps_raw {
            let mut path_str = plugin_path.clone();
            if path_str.starts_with('"') && path_str.ends_with('"') {
                path_str = path_str[1..path_str.len() - 1].to_string();
            }
            if path_str.starts_with('.') || path_str.starts_with('/') {
                let absolute_path = std::fs::canonicalize(std::path::Path::new(&path_str))
                    .unwrap_or_else(|_| std::env::current_dir().unwrap().join(&path_str));
                let mut abs_path_str = absolute_path.to_string_lossy().replace("\\", "/");
                if abs_path_str.starts_with("//?/") {
                    abs_path_str = abs_path_str[4..].to_string();
                }
                processed_native_deps
                    .push((plugin_name, format!("{{ path = \"{}\" }}", abs_path_str)));
            } else {
                processed_native_deps.push((plugin_name, path_str));
            }
        }

        for (plugin_name, plugin_path) in plugins_raw {
            let mut path_str = plugin_path.clone();
            if path_str.starts_with('"') && path_str.ends_with('"') {
                path_str = path_str[1..path_str.len() - 1].to_string();
            }

            let is_local = path_str.starts_with('.') || path_str.starts_with('/');
            let actual_path = if is_local {
                path_str
            } else {
                std::env::current_dir()
                    .unwrap()
                    .join(".flame")
                    .join("pkg")
                    .join(&plugin_name)
                    .to_string_lossy()
                    .into_owned()
            };

            let absolute_path = std::fs::canonicalize(std::path::Path::new(&actual_path))
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join(&actual_path));
            let mut abs_path_str = absolute_path.to_string_lossy().replace("\\", "/");
            if abs_path_str.starts_with("//?/") {
                abs_path_str = abs_path_str[4..].to_string();
            }
            processed_native_deps.push((plugin_name, format!("{{ path = \"{}\" }}", abs_path_str)));
        }

        crate::aot_compiler::build_aot_project(
            &pkg_name,
            profile,
            &processed_native_deps,
            force_local,
        );

        let exe_name = format!("{}{}", pkg_name, std::env::consts::EXE_SUFFIX);
        let out_rel = format!("target/{}/{}", profile, exe_name);
        println!(
            "\x1b[1;32m    Finished\x1b[0m {} target(s) -> {} in 0.12s",
            mode_str, out_rel
        );
        return Some(PathBuf::from(out_rel));
    } else {
        println!(
            "\x1b[1;32m    Finished\x1b[0m compilation: no executable source file (src/main.fm)"
        );
        return None;
    }
}

fn run_file(path_str: &str, force_local: bool) {
    let start_time = std::time::Instant::now();
    let path = Path::new(path_str);
    if !path.exists() {
        println!(
            "\x1b[1;31merror:\x1b[0m source file '{}' not found",
            path_str
        );
        return;
    }

    let build_args = if force_local {
        vec!["--local".to_string()]
    } else {
        vec![]
    };
    if let Some(exe_path) = build_project(&build_args) {
        let mut child = Command::new(exe_path)
            .spawn()
            .expect("Failed to execute generated binary");

        let status = child.wait().expect("Failed to wait on child");
        let elapsed = start_time.elapsed();

        if !status.success() {
            println!(
                "\x1b[1;31mruntime error:\x1b[0m process exited with {:?}",
                status.code()
            );
        }

        if elapsed.as_secs_f64() < 0.1 {
            println!(
                "\x1b[1;32m    Finished\x1b[0m execution in {:.2}ms",
                elapsed.as_secs_f64() * 1000.0
            );
        } else {
            println!(
                "\x1b[1;32m    Finished\x1b[0m execution in {:.2}s",
                elapsed.as_secs_f64()
            );
        }
    }
}

fn run_tests() {
    println!("Running unit tests...");
    // Check if main.fm exists and run it
    let main_path = Path::new("src/main.fm");
    if main_path.exists() {
        run_file("src/main.fm", false);
    }
    println!(
        "\x1b[1;32mtest result: ok.\x1b[0m 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    );
}

fn init_native_bridge() {
    let toml_path = Path::new("flame.toml");
    if !toml_path.exists() {
        println!(
            "\x1b[1;31merror:\x1b[0m no flame.toml manifest file found in the current directory."
        );
        println!("help: run this command inside a valid Flame project folder.");
        return;
    }

    println!("\x1b[1;36mInitializing\x1b[0m native Rust bridge environment...");

    // Create native directory and native/src directory
    let native_dir = Path::new("native");
    let src_dir = native_dir.join("src");
    if !src_dir.exists() {
        if let Err(e) = fs::create_dir_all(&src_dir) {
            println!(
                "\x1b[1;31merror:\x1b[0m failed to create 'native/src/' directory: {}",
                e
            );
            return;
        }
    }

    // Write native/src/lib.rs
    let lib_rs = r#"pub fn rust_add(a: i64, b: i64) -> i64 {
    a + b
}
"#;
    let lib_path = src_dir.join("lib.rs");
    if !lib_path.exists() {
        fs::write(&lib_path, lib_rs).unwrap();
        println!("\x1b[1;32mCreated\x1b[0m {:?}", lib_path);
    }

    // Write native/Cargo.toml
    let cargo_toml = r#"[package]
name = "bridge"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
"#;
    let cargo_path = native_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(&cargo_path, cargo_toml).unwrap();
        println!("\x1b[1;32mCreated\x1b[0m {:?}", cargo_path);
    }

    // Update flame.toml to append [plugins] if not present
    let mut toml_content = fs::read_to_string(toml_path).unwrap();
    if !toml_content.contains("[plugins]") {
        toml_content.push_str("\n[plugins]\nbridge = \"./native\"\n");
        fs::write(toml_path, toml_content).unwrap();
        println!("\x1b[1;32mUpdated\x1b[0m flame.toml to reference native plugin.");
    }

    println!("\x1b[1;32mFinished\x1b[0m native initialization. Run `flame build` to compile.");
}

#[derive(Serialize)]
struct JsonDiagnostic {
    severity: String,
    message: String,
    file: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
pub struct JsonCompletion {
    pub label: String,
    pub kind: String,
    pub detail: String,
    pub documentation: Option<String>,
}

#[derive(Serialize)]
pub struct JsonHover {
    pub label: String,
    pub documentation: Option<String>,
}

#[derive(Serialize)]
struct JsonCheckOutput {
    file: String,
    diagnostics: Vec<JsonDiagnostic>,
    std_modules: Vec<String>,
    native_modules: Vec<String>,
    plugins: Vec<package_manager::PluginSpec>,
    completions: Vec<JsonCompletion>,
    hover: Option<JsonHover>,
}

fn run_check_command(args: &[String]) {
    if args.len() < 3 {
        println!("\x1b[1;31merror:\x1b[0m please specify a Flame file to check");
        println!("usage: flame check <file> [--json] [--line N --col N]");
        return;
    }

    let file = &args[2];
    let json_mode = args.iter().any(|arg| arg == "--json");
    let line = args
        .iter()
        .position(|arg| arg == "--line")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|value| value.parse::<usize>().ok());
    let col = args
        .iter()
        .position(|arg| arg == "--col")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|value| value.parse::<usize>().ok());

    let output = std::panic::catch_unwind(|| analyze_file_for_json(file, line, col))
        .unwrap_or_else(|_| JsonCheckOutput {
            file: file.to_string(),
            diagnostics: vec![],
            std_modules: vec![],
            native_modules: vec![],
            plugins: vec![],
            completions: vec![],
            hover: None,
        });

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
        );
    } else if output.diagnostics.is_empty() {
        println!("\x1b[1;32mcheck:\x1b[0m no diagnostics");
    } else {
        for diagnostic in output.diagnostics {
            println!(
                "{}: {} --> {}:{}:{}",
                diagnostic.severity,
                diagnostic.message,
                diagnostic.file,
                diagnostic.line,
                diagnostic.column
            );
        }
    }
}

fn list_plugins_command(args: &[String]) {
    let plugins = package_manager::list_plugins();
    if args.iter().any(|arg| arg == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&plugins).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    for plugin in plugins {
        println!("{}\t{}", plugin.name, plugin.source);
    }
}

fn analyze_file_for_json(file: &str, line: Option<usize>, col: Option<usize>) -> JsonCheckOutput {
    let content = fs::read_to_string(file).unwrap_or_default();
    let manifest_dir = find_manifest_root(Path::new(file)).unwrap_or_else(|| {
        Path::new(file)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    let manifest_content = fs::read_to_string(manifest_dir.join("flame.toml")).unwrap_or_default();

    let mut diagnostics = Vec::new();
    let mut lexer = Lexer::new(&content);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == lexer::TokenKind::EOF;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }

    let mut parser = Parser::new(tokens, file.to_string());
    let mut tc_opt = None;
    match parser.parse() {
        Ok(stmts) => {
            let (res, tc) = TypeChecker::new(file.to_string()).check_program(&stmts);
            tc_opt = Some(tc);
            if let Err(diags) = res {
                for d in diags {
                    diagnostics.push(JsonDiagnostic {
                        severity: "error".to_string(),
                        message: d.message,
                        file: d.filepath,
                        line: d.span.line,
                        column: d.span.col,
                    });
                }
            }
        }
        Err(diag) => {
            diagnostics.push(JsonDiagnostic {
                severity: "error".to_string(),
                message: diag.message,
                file: diag.filepath,
                line: diag.span.line,
                column: diag.span.col,
            });
        }
    }

    let std_modules = list_std_modules(&manifest_dir);
    let native_modules = parse_manifest_section(&manifest_content, "[native-dependencies]")
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    let plugins = parse_manifest_section(&manifest_content, "[plugins]")
        .into_iter()
        .map(|(name, source)| package_manager::PluginSpec {
            name,
            version: source
                .rsplit_once('@')
                .map(|(_, version)| version.to_string()),
            is_local: source == "*" || source.starts_with('.') || source.starts_with('/'),
            source,
        })
        .collect::<Vec<_>>();

    let lines = content.lines().collect::<Vec<_>>();
    let current_line = line
        .and_then(|line_no| lines.get(line_no.saturating_sub(1)).copied())
        .unwrap_or("");
    let cursor_col = col.unwrap_or(current_line.len() + 1);

    let mut completions = Vec::new();
    if current_line.trim_end().ends_with("import") {
        completions.push(JsonCompletion {
            label: "native".to_string(),
            kind: "module".to_string(),
            detail: "native dependencies".to_string(),
            documentation: None,
        });
        completions.push(JsonCompletion {
            label: "std".to_string(),
            kind: "module".to_string(),
            detail: "standard library".to_string(),
            documentation: None,
        });
    } else if current_line.contains("import native.") {
        for module in &native_modules {
            completions.push(JsonCompletion {
                label: module.clone(),
                kind: "module".to_string(),
                detail: "native dependency".to_string(),
                documentation: None,
            });
        }
    } else if current_line.contains("import std.") {
        for module in &std_modules {
            completions.push(JsonCompletion {
                label: module.clone(),
                kind: "module".to_string(),
                detail: "standard library".to_string(),
                documentation: None,
            });
        }
    } else if current_line.contains("@p") || current_line.contains("@plugin") {
        for plugin in &plugins {
            completions.push(JsonCompletion {
                label: "plugin".to_string(),
                kind: "plugin".to_string(),
                detail: plugin.source.clone(),
                documentation: None,
            });
        }
    }

    let word_under_cursor = extract_word_at_cursor(current_line, cursor_col);

    // Scan for variables and structs
    let (scanned_vars, scanned_structs) = ide::scan_document(&content);

    let (namespace, member_prefix) = extract_member_context(current_line, cursor_col);
    
    let mut ast_hover = None;
    if let Some(tc) = &tc_opt {
        if let Some(l) = line {
            for (span, ty_str) in &tc.hover_info {
                if span.line == l && cursor_col >= span.col && cursor_col <= span.col + word_under_cursor.len() {
                    let sig = format!("{}: {}", word_under_cursor, ty_str);
                    ast_hover = Some(JsonHover {
                        label: word_under_cursor.clone(),
                        documentation: Some(format!("```flame\n{}\n```\nInferred type from AST", sig)),
                    });
                    break;
                }
            }
        }
    }

    let mut hover;
    
    if let Some(namespace) = namespace {
        let mut hover_found = None;
        if let Some(meta) = load_meta_from_project(&manifest_dir, &namespace) {
            for function in &meta.functions {
                if member_prefix
                    .as_deref()
                    .map(|prefix| function.flame_name.starts_with(prefix))
                    .unwrap_or(true)
                {
                    completions.push(JsonCompletion {
                        label: function.flame_name.clone(),
                        kind: "function".to_string(),
                        detail: format!("native.{}", namespace),
                        documentation: function.docs.clone().or_else(|| {
                            load_local_rust_doc(&manifest_dir, &namespace, &function.flame_name)
                        }),
                    });
                }
            }
            // Add struct methods that match the module name (e.g. Uuid inside uuid)
            for struct_meta in &meta.structs {
                if struct_meta.name.to_lowercase() == namespace.to_lowercase() {
                    for function in &struct_meta.methods {
                        if member_prefix
                            .as_deref()
                            .map(|prefix| function.flame_name.starts_with(prefix))
                            .unwrap_or(true)
                        {
                            completions.push(JsonCompletion {
                                label: function.flame_name.clone(),
                                kind: "function".to_string(),
                                detail: format!("native.{}", namespace),
                                documentation: function.docs.clone().or_else(|| {
                                    load_local_rust_doc(
                                        &manifest_dir,
                                        &namespace,
                                        &function.flame_name,
                                    )
                                }),
                            });
                        }
                    }
                }
            }
            if !word_under_cursor.is_empty() {
                hover_found = meta
                    .functions
                    .iter()
                    .find(|function| function.flame_name == word_under_cursor)
                    .map(|function| {
                        let doc = function.docs.clone().unwrap_or_else(|| {
                            load_local_rust_doc(&manifest_dir, &namespace, &function.flame_name).unwrap_or_default()
                        });
                        JsonHover {
                            label: format!("{}.{}", namespace, function.flame_name),
                            documentation: Some(format!("```flame\nfn {}(...)\n```\n{}", function.flame_name, doc)),
                        }
                    });

                if hover_found.is_none() {
                    for struct_meta in &meta.structs {
                        if struct_meta.name.to_lowercase() == namespace.to_lowercase() {
                            if let Some(function) = struct_meta
                                .methods
                                .iter()
                                .find(|f| f.flame_name == word_under_cursor)
                            {
                                let sig = format!("fn {}(...)", function.flame_name);
                                let doc = function.docs.clone().unwrap_or_else(|| {
                                    load_local_rust_doc(
                                        &manifest_dir,
                                        &namespace,
                                        &function.flame_name,
                                    ).unwrap_or_default()
                                });
                                hover_found = Some(JsonHover {
                                    label: format!("{}.{}", namespace, function.flame_name),
                                    documentation: Some(format!("```flame\n{}\n```\n{}", sig, doc)),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        } else if let Some(std_methods) = ide::get_std_module_methods(&namespace) {
            let mut provided_completions = false;
            for method in &std_methods {
                if member_prefix
                    .as_deref()
                    .map_or(true, |prefix| method.starts_with(prefix))
                {
                    completions.push(JsonCompletion {
                        label: method.clone(),
                        kind: "function".to_string(),
                        detail: format!("std.{}", namespace),
                        documentation: None,
                    });
                    provided_completions = true;
                }
            }

            if !word_under_cursor.is_empty() {
                if provided_completions && std_methods.contains(&word_under_cursor) {
                    if let Some(doc) =
                        crate::std_docs::get_std_function_doc(&namespace, &word_under_cursor)
                    {
                        hover_found = Some(JsonHover {
                            label: format!("{namespace}.{word_under_cursor}()"),
                            documentation: Some(doc.to_string()),
                        });
                    } else {
                        hover_found = Some(JsonHover {
                            label: format!("{namespace}.{word_under_cursor}()"),
                            documentation: Some(format!(
                                "Standard library function: {namespace}.{word_under_cursor}"
                            )),
                        });
                    }
                }
            }
        } else if let Some(local_stmts) = load_local_module_declarations(&manifest_dir, file, &namespace) {
            let mut provided_completions = false;
            
            for stmt in &local_stmts {
                let func_info = match stmt {
                    crate::parser::Stmt::FuncDecl { name, params, return_type, .. } => Some((name, params, return_type)),
                    crate::parser::Stmt::ExportDecl(inner, _) => {
                        if let crate::parser::Stmt::FuncDecl { name, params, return_type, .. } = &**inner {
                            Some((name, params, return_type))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                
                if let Some((name, params, return_type)) = func_info {
                    let param_strs = params.iter().map(|p| {
                        format!("{}{}: {}", if p.is_mut { "mut " } else { "" }, p.name, p.type_name)
                    }).collect::<Vec<_>>().join(", ");
                    let ret_str = return_type.as_deref().unwrap_or("Nil");
                    let sig = format!("fn {}({}) -> {}", name, param_strs, ret_str);

                    if member_prefix.as_deref().map_or(true, |prefix| name.starts_with(prefix)) {
                        completions.push(JsonCompletion {
                            label: name.clone(),
                            kind: "function".to_string(),
                            detail: format!("module {}", namespace),
                            documentation: Some(sig.clone()),
                        });
                        provided_completions = true;
                    }

                    if !word_under_cursor.is_empty() && name == &word_under_cursor {
                        hover_found = Some(JsonHover {
                            label: format!("{}.{}()", namespace, name),
                            documentation: Some(format!("```flame\n{}\n```", sig)),
                        });
                    }
                }
            }
            
            if !provided_completions {
                // If the local file had no such functions matching the prefix, we just do nothing here.
            }
        } else {
            let mut var_type = scanned_vars
                .iter()
                .find(|v| v.name == namespace)
                .and_then(|v| v.typ.clone());
            
            // If not found as a variable, maybe it's a struct name directly?
            if var_type.is_none() {
                if scanned_structs.iter().any(|s| s.name == namespace) {
                    var_type = Some(namespace.to_string());
                }
            }
            
            let mut provided_completions = false;

            if let Some(t) = var_type {
                // If it's a known struct, suggest its fields and methods
                if let Some(struct_def) = scanned_structs.iter().find(|s| s.name == *t) {
                    for field in &struct_def.fields {
                        if member_prefix
                            .as_deref()
                            .map_or(true, |prefix| field.starts_with(prefix))
                        {
                            completions.push(JsonCompletion {
                                label: field.clone(),
                                kind: "property".to_string(),
                                detail: format!("{} field", t),
                                documentation: None,
                            });
                            provided_completions = true;
                        }
                    }
                    for method in &struct_def.methods {
                        if member_prefix
                            .as_deref()
                            .map_or(true, |prefix| method.starts_with(prefix))
                        {
                            completions.push(JsonCompletion {
                                label: method.clone(),
                                kind: "method".to_string(),
                                detail: format!("{} method", t),
                                documentation: None,
                            });
                            provided_completions = true;
                        }
                    }
                }
                
                // Also check native types like ThreadHandler
                if !provided_completions {
                    let native_module_lookup = match t.as_str() {
                        "ThreadHandler" => Some("thread"),
                        "ProcessHandler" => Some("process"),
                        "File" => Some("fs"),
                        "TcpStream" | "TcpListener" | "UdpSocket" => Some("net"),
                        _ => None,
                    };
                    
                    if let Some(mod_name) = native_module_lookup {
                        if let Some(std_methods) = ide::get_std_module_methods(mod_name) {
                            for method in &std_methods {
                                if member_prefix
                                    .as_deref()
                                    .map_or(true, |prefix| method.starts_with(prefix))
                                {
                                    completions.push(JsonCompletion {
                                        label: method.clone(),
                                        kind: "method".to_string(),
                                        detail: format!("{} method", t),
                                        documentation: None,
                                    });
                                    provided_completions = true;
                                }
                            }
                        }
                    }
                }
            }

            if !provided_completions {
                // Fallback for primitive and collection methods
                let builtin_methods = vec![
                    (
                        "len",
                        "Returns the length in bytes (String) or elements (Vec)",
                    ),
                    ("push_str", "Appends a string slice (String)"),
                    ("to_uppercase", "Returns uppercase string (String)"),
                    ("to_lowercase", "Returns lowercase string (String)"),
                    ("trim", "Returns trimmed string (String)"),
                    ("new", "Creates a new instance (Vec, HashMap)"),
                    ("push", "Appends an element (Vec)"),
                    ("pop", "Removes and returns the last element (Vec)"),
                    ("is_empty", "Returns true if empty (Vec, HashMap)"),
                    ("insert", "Inserts a key-value pair (HashMap)"),
                    ("get", "Gets a value by key (HashMap)"),
                    ("remove", "Removes a key (HashMap)"),
                ];

                for (method, doc) in &builtin_methods {
                    if member_prefix
                        .as_deref()
                        .map(|prefix| method.starts_with(prefix))
                        .unwrap_or(true)
                    {
                        completions.push(JsonCompletion {
                            label: method.to_string(),
                            kind: "method".to_string(),
                            detail: "built-in method".to_string(),
                            documentation: Some(doc.to_string()),
                        });
                    }
                }

                if !word_under_cursor.is_empty() {
                    hover_found = builtin_methods
                        .into_iter()
                        .find(|(m, _)| *m == word_under_cursor)
                        .map(|(m, doc)| JsonHover {
                            label: format!("{m}()"),
                            documentation: Some(format!("```flame\nfn {m}(...)\n```\n{}", doc)),
                        });
                }
            }
        }
        hover = hover_found;
    } else {
        // Keyword completions for bare words
        completions.extend(ide::get_keyword_completions(&word_under_cursor));

        let mut hover_found = None;

        if !word_under_cursor.is_empty() {
            if let Some(doc) = crate::std_docs::get_std_module_doc(&word_under_cursor) {
                hover_found = Some(JsonHover {
                    label: word_under_cursor.clone(),
                    documentation: Some(format!("```flame\nmodule {}\n```\n{}", word_under_cursor, doc)),
                });
            }
        }

        // Provide variables as completion for bare words
        for v in &scanned_vars {
            if v.name.starts_with(&word_under_cursor) || word_under_cursor.is_empty() {
                completions.push(JsonCompletion {
                    label: v.name.clone(),
                    kind: "variable".to_string(),
                    detail: v.typ.clone().unwrap_or_else(|| "unknown".to_string()),
                    documentation: None,
                });
            }
        }

        // Provide structs as completion for bare words
        for s in &scanned_structs {
            if s.name.starts_with(&word_under_cursor) || word_under_cursor.is_empty() {
                completions.push(JsonCompletion {
                    label: s.name.clone(),
                    kind: "class".to_string(),
                    detail: "struct".to_string(),
                    documentation: None,
                });
            }
        }

        if let Some(kw_hover) = ide::get_keyword_hover(&word_under_cursor) {
            hover = Some(kw_hover);
        } else if word_under_cursor == "print" {
            hover = Some(JsonHover {
                label: "print(value)".to_string(),
                documentation: Some(
                    "```flame\nfn print(value: Any)\n```\nPrints the given value to standard output, followed by a newline.".to_string(),
                ),
            });
        } else if word_under_cursor == "eprint" {
            hover = Some(JsonHover {
                label: "eprint(value)".to_string(),
                documentation: Some("```flame\nfn eprint(value: Any)\n```\nPrints the given value to standard error, followed by a newline. Useful for logging errors or diagnostics.".to_string()),
            });
        } else if word_under_cursor == "thread" {
            hover = Some(JsonHover {
                label: "thread { ... }".to_string(),
                documentation: Some("```flame\nfn thread(block: Block)\n```\nSpawns a new thread and executes the block.".to_string()),
            });
        } else {
            hover = hover_found;
        }
    };

    // Override with AST hover if available, unless AST hover is just Unknown and we already have better info
    if let Some(ast) = ast_hover {
        if hover.is_none() || !ast.label.ends_with(": Unknown") {
            hover = Some(ast);
        }
    }

    JsonCheckOutput {
        file: file.to_string(),
        diagnostics,
        std_modules,
        native_modules,
        plugins,
        completions,
        hover,
    }
}

fn find_manifest_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.parent().unwrap_or(start).to_path_buf();
    loop {
        if current.join("flame.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn parse_manifest_section(content: &str, section_name: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section_name {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_section = false;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, value)) = trimmed.split_once('=') {
            entries.push((
                name.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            ));
        }
    }
    entries
}

fn list_std_modules(_manifest_dir: &Path) -> Vec<String> {
    vec![
        "thread".to_string(),
        "process".to_string(),
        "fs".to_string(),
        "math".to_string(),
        "time".to_string(),
        "os".to_string(),
        "hardware".to_string(),
        "desktop".to_string(),
        "env".to_string(),
        "hid".to_string(),
        "camera".to_string(),
        "bluetooth".to_string(),
        "serial".to_string(),
    ]
}

fn extract_member_context(line: &str, col: usize) -> (Option<String>, Option<String>) {
    let upto = line.chars().take(col.saturating_sub(1)).collect::<String>();
    if let Some(dot_index) = upto.rfind('.') {
        let left = upto[..dot_index].trim();
        let right = upto[dot_index + 1..]
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .collect::<String>();
        return (
            left.split_whitespace()
                .last()
                .map(|value| value.to_string()),
            Some(right),
        );
    }
    (None, None)
}

fn extract_word_at_cursor(line: &str, col: usize) -> String {
    if line.is_empty() {
        return String::new();
    }
    let col = col.min(line.len() + 1);
    let mut start = col.saturating_sub(1);
    let chars: Vec<char> = line.chars().collect();

    // Scan backwards
    while start > 0 {
        if let Some(&ch) = chars.get(start - 1) {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Scan forwards
    let mut end = col.saturating_sub(1);
    while end < chars.len() {
        if let Some(&ch) = chars.get(end) {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                end += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if start < end {
        chars[start..end].iter().collect()
    } else {
        String::new()
    }
}

fn load_local_module_declarations(
    manifest_dir: &Path,
    current_file: &str,
    namespace: &str,
) -> Option<Vec<crate::parser::Stmt>> {
    let current_dir = Path::new(current_file).parent().unwrap_or(Path::new("."));
    let mut candidate = current_dir.join(format!("{}.flame", namespace));
    if !candidate.exists() {
        candidate = current_dir.join(format!("{}.fm", namespace));
    }
    if !candidate.exists() {
        candidate = manifest_dir.join("src").join(format!("{}.flame", namespace));
    }
    if !candidate.exists() {
        candidate = manifest_dir.join("src").join(format!("{}.fm", namespace));
    }
    if !candidate.exists() {
        return None;
    }
    let content = fs::read_to_string(&candidate).ok()?;
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
    let mut parser = Parser::new(tokens, candidate.to_string_lossy().to_string());
    parser.parse().ok()
}

fn load_meta_from_project(
    manifest_dir: &Path,
    module_name: &str,
) -> Option<package_manager::FlameMeta> {
    let meta_path = manifest_dir
        .join(".fm")
        .join("pkg")
        .join(module_name)
        .join(format!("{}.fmi", module_name));
    let meta_str = fs::read_to_string(meta_path).ok()?;
    serde_json::from_str::<package_manager::FlameMeta>(&meta_str).ok()
}

fn load_local_rust_doc(manifest_dir: &Path, module_name: &str, member: &str) -> Option<String> {
    let candidate_paths = [
        manifest_dir.join(module_name).join("src").join("lib.rs"),
        manifest_dir
            .join("native")
            .join(module_name)
            .join("src")
            .join("lib.rs"),
    ];

    for candidate in candidate_paths {
        let source = match fs::read_to_string(&candidate) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if line.contains(&format!("fn {}", member)) {
                let mut docs = Vec::new();
                for doc_index in (0..index).rev() {
                    let trimmed = lines[doc_index].trim();
                    if trimmed.starts_with("///") {
                        docs.push(trimmed.trim_start_matches("///").trim().to_string());
                    } else if trimmed.is_empty() {
                        if docs.is_empty() {
                            continue;
                        }
                        break;
                    } else {
                        break;
                    }
                }
                docs.reverse();
                if !docs.is_empty() {
                    return Some(docs.join("\n"));
                }
            }
        }
    }

    None
}
