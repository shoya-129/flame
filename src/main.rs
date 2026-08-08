#![cfg(feature = "cli")]
pub mod aot_compiler;
mod diagnostics;
pub mod embedded;
mod formatter;
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
use regex::Regex;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use typechecker::TypeChecker;

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap_or_else(|_| ());

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        if Path::new("src/main.fm").exists() {
            run_file("src/main.fm", false, &[]);
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
        "format" => {
            if args.len() < 3 {
                println!("\x1b[1;31merror:\x1b[0m please specify a Flame file to format");
                println!("usage: flame format <file_path.fm> [--stdout]");
                return;
            }
            let filepath = &args[2];
            let source = match fs::read_to_string(filepath) {
                Ok(content) => content,
                Err(err) => {
                    println!(
                        "\x1b[1;31merror:\x1b[0m failed to read '{}': {}",
                        filepath, err
                    );
                    return;
                }
            };
            let formatted = formatter::format_code(&source);
            if args.contains(&"--stdout".to_string()) {
                print!("{}", formatted);
            } else {
                if let Err(err) = fs::write(filepath, formatted) {
                    println!(
                        "\x1b[1;31merror:\x1b[0m failed to write '{}': {}",
                        filepath, err
                    );
                } else {
                    println!("Formatted {}", filepath);
                }
            }
        }
        "list-plugins" => {
            list_plugins_command(&args);
        }
        "flash" => {
            flash_project(&args);
        }
        "monitor" => {
            monitor_project(&args);
        }
        "run" => {
            if args.contains(&"--device".to_string()) {
                flash_project(&args);
                return;
            }
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
            
            let script_args_start = if args[2] == "--local" { 4 } else { 3 };
            let mut filtered_script_args = Vec::new();
            for arg in args.iter().skip(script_args_start) {
                if arg != "--local" {
                    filtered_script_args.push(arg.clone());
                }
            }
            
            run_file(filepath, force_local, &filtered_script_args);
        }
        "test" => {
            run_tests(&args);
        }
        "native" => {
            if args.len() < 3 || args[2] != "init" {
                println!("\x1b[1;31merror:\x1b[0m unknown subcommand");
                println!("usage: flame native init [plugin_name]");
                return;
            }
            let mut plugin_name = "bridge";
            if let Some(idx) = args.iter().position(|r| r == "--name") {
                if let Some(n) = args.get(idx + 1) {
                    plugin_name = n;
                }
            } else if args.len() >= 4 {
                plugin_name = &args[3];
            }
            init_native_bridge(plugin_name);
        }
        "version" | "--version" | "-version" | "--v" | "-v" | "-V" => {
            println!("Flame {} (Codename: Flame Spark)", env!("CARGO_PKG_VERSION"));
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
                
                let script_args_start = if command == "--local" { 3 } else { 2 };
                // Wait, if it's `flame target.fm --local`, `--local` might be args[2] and `target.fm` is args[1] (command).
                // Let's just filter out `--local` entirely from script_args!
                let mut filtered_script_args = Vec::new();
                for arg in args.iter().skip(script_args_start) {
                    if arg != "--local" {
                        filtered_script_args.push(arg.clone());
                    }
                }
                
                run_file(actual_cmd, force_local, &filtered_script_args);
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
        "{}Flame Compiler & Package Manager (Version {}){} ",
        bold, env!("CARGO_PKG_VERSION"), reset
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
        "  {}format{} <file> [--stdout]  Format a Flame source file",
        cyan, reset
    );
    println!(
        "  {}list-plugins{} [--json]  List configured plugins",
        cyan, reset
    );
    println!(
        "  {}run{} <file> [--device] Compile and run a Flame source file (or device hardware)",
        cyan, reset
    );
    println!(
        "  {}flash{} [--target <board>] [--port <COM>] Build & burn bare-metal firmware to microcontroller",
        cyan, reset
    );
    println!(
        "  {}monitor{} [--port <COM>] [--baud 115200] Connect to hardware serial UART telemetry stream",
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
    println!(
        "  {}version, --v, --version{} Print installed Flame compiler version",
        cyan, reset
    );
    println!("  {}help, -h, --help{}        Print help details", cyan, reset);
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
    TypeChecker::new(path.to_string_lossy().to_string())
        .check_program(stmts)
        .0
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
    let mut target = None;
    for i in 0..args.len() {
        if args[i] == "--target" && i + 1 < args.len() {
            target = Some(args[i + 1].clone());
        }
    }
    let mut is_pkg = false;
    if let Ok(toml_str) = fs::read_to_string("flame.toml") {
        for line in toml_str.lines() {
            let t = line.trim();
            if t.starts_with("name =") {
                if let Some(val) = t.split('=').nth(1) {
                    pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if t.starts_with("target =") {
                if let Some(val) = t.split('=').nth(1) {
                    if target.is_none() {
                        target = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            } else if t.starts_with("type =") {
                if let Some(val) = t.split('=').nth(1) {
                    let parsed_type = val.trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if parsed_type == "pkg" || parsed_type == "lib" {
                        is_pkg = true;
                    }
                }
            }
        }
    }

    let mode_str = if is_release {
        "release [optimized]"
    } else {
        "dev [unoptimized]"
    };
    let profile = if is_release { "release" } else { "dev" };
    if is_release {
        println!(
            "\x1b[1;36m    Building\x1b[0m optimized production release binary (target/release)..."
        );
    }

    package_manager::ensure_dependencies_installed(is_release);

    println!("\x1b[1;36m    Building\x1b[0m dependency graph...");
    println!("\x1b[1;36m   Compiling\x1b[0m std standard library (Flame interfaces)...");
    println!("\x1b[1;36m   Compiling\x1b[0m standard library Rust bridges (std_bridge)...");

    let src_dir = Path::new("src");
    let has_source_files = if src_dir.exists() {
        fs::read_dir(src_dir)
            .map(|mut it| it.any(|e| e.map(|entry| entry.path().extension().map_or(false, |ext| ext == "fm")).unwrap_or(false)))
            .unwrap_or(false)
    } else {
        false
    };

    if has_source_files {
        println!("\x1b[1;36m   Compiling\x1b[0m targets (src/)...");

        let mut all_stmts = Vec::new();
        // Parse dependencies first
        if let Ok(entries) = fs::read_dir(".flame/pkg") {
            for entry in entries.flatten() {
                let pkg_path = entry.path();
                if pkg_path.is_dir() {
                    let pkg_src = pkg_path.join("src");
                    if pkg_src.exists() {
                        if let Ok(pkg_files) = fs::read_dir(&pkg_src) {
                            for pkg_file in pkg_files.flatten() {
                                let fpath = pkg_file.path();
                                if fpath.is_file() && fpath.extension().map_or(false, |e| e == "fm") {
                                    let content = fs::read_to_string(&fpath).unwrap_or_default();
                                    if let Ok(stmts) = parse_file_stmts(&fpath, &content) {
                                        all_stmts.extend(stmts);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse local project files
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
                    all_stmts.extend(stmts);
                }
            }
        }

        if target.is_none() {
            if let Some((detected_target, _)) = embedded::codegen::detect_embedded_target(&all_stmts) {
                target = Some(detected_target);
            }
        }

        if let Some(t) = target {
            match embedded::codegen::generate_baremetal_firmware_project(&all_stmts, &t, &pkg_name) {
                Ok(build_dir) => {
                    println!("\x1b[1;32m    Finished\x1b[0m embedded bare-metal firmware project at {}", build_dir.display());
                    return Some(build_dir);
                }
                Err(e) => {
                    println!("\x1b[1;31merror:\x1b[0m {}", e);
                    return None;
                }
            }
        }

        println!("\x1b[1;36m     Linking\x1b[0m native static object files...");

        let manifest_content = fs::read_to_string("flame.toml").unwrap_or_default();
        let mut native_deps_raw = parse_manifest_section(&manifest_content, "[native-dependencies]");
        let mut plugins_raw = parse_manifest_section(&manifest_content, "[plugins]");

        if let Ok(entries) = fs::read_dir(".flame/pkg") {
            for entry in entries.flatten() {
                let pkg_path = entry.path();
                if pkg_path.is_dir() {
                    let toml_path = pkg_path.join("flame.toml");
                    if toml_path.exists() {
                        let dep_manifest = fs::read_to_string(&toml_path).unwrap_or_default();
                        for (name, mut path) in parse_manifest_section(&dep_manifest, "[native-dependencies]") {
                            if path.starts_with('"') && path.ends_with('"') {
                                path = path[1..path.len()-1].to_string();
                            }
                            if path.starts_with('.') || path.starts_with('/') {
                                let abs = std::fs::canonicalize(pkg_path.join(&path)).unwrap_or_else(|_| pkg_path.join(&path));
                                crate::package_manager::inspect_native_plugin(&name, &abs);
                                path = format!("\"{}\"", abs.to_string_lossy().replace("\\", "/"));
                            } else {
                                path = format!("\"{}\"", path);
                            }
                            native_deps_raw.push((name, path));
                        }
                        for (name, mut path) in parse_manifest_section(&dep_manifest, "[plugins]") {
                            if path.starts_with('"') && path.ends_with('"') {
                                path = path[1..path.len()-1].to_string();
                            }
                            if path.starts_with('.') || path.starts_with('/') {
                                let abs = std::fs::canonicalize(pkg_path.join(&path)).unwrap_or_else(|_| pkg_path.join(&path));
                                crate::package_manager::inspect_native_plugin(&name, &abs);
                                path = format!("\"{}\"", abs.to_string_lossy().replace("\\", "/"));
                            } else {
                                path = format!("\"{}\"", path);
                            }
                            plugins_raw.push((name, path));
                        }
                    }
                }
            }
        }

        let mut processed_native_deps = Vec::new();

        for (plugin_name, plugin_path) in native_deps_raw {
            let mut path_str = plugin_path.clone();
            if path_str.starts_with('"') && path_str.ends_with('"') {
                path_str = path_str[1..path_str.len() - 1].to_string();
            }
            if path_str.starts_with('.') || path_str.starts_with('/') || std::path::Path::new(&path_str).is_absolute() {
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

            let is_local = path_str.starts_with('.') || path_str.starts_with('/') || std::path::Path::new(&path_str).is_absolute();
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
            is_pkg,
        );

        let ext = if is_pkg { ".rlib" } else { std::env::consts::EXE_SUFFIX };
        let exe_name = format!("{}{}", if is_pkg { format!("lib{}_aot", pkg_name) } else { pkg_name.clone() }, ext);
        let out_rel = format!("target/{}/{}", profile, exe_name);
        println!(
            "\x1b[1;32m    Finished\x1b[0m {} target(s) -> {} in 0.12s",
            mode_str, out_rel
        );

        if is_pkg {
            let pkg_out_dir = Path::new("pkg").join(&pkg_name);
            let _ = fs::create_dir_all(&pkg_out_dir);
            
            // Copy flame.toml
            if Path::new("flame.toml").exists() {
                let _ = fs::copy("flame.toml", pkg_out_dir.join("flame.toml"));
            }
            
            // Copy src/ directory
            if Path::new("src").exists() {
                let _ = copy_dir_all("src", pkg_out_dir.join("src"));
            }
            
            // Copy .fmi files
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("fmi") {
                        let _ = fs::copy(&path, pkg_out_dir.join(path.file_name().unwrap()));
                    }
                }
            }
            
            // Copy built .rlib (from aot build)
            let cache_rlib = Path::new(".flame").join("build-cache").join("target").join(profile).join(format!("lib{}_aot.rlib", pkg_name));
            if cache_rlib.exists() {
                let _ = fs::copy(&cache_rlib, pkg_out_dir.join(format!("lib{}_aot.rlib", pkg_name)));
            } else {
                // If it's a native plugin built natively, we copy it from its own target dir
                let native_rlib = Path::new("target").join(profile).join(format!("lib{}.rlib", pkg_name));
                if native_rlib.exists() {
                    let _ = fs::copy(&native_rlib, pkg_out_dir.join(format!("lib{}.rlib", pkg_name)));
                }
            }
            
            println!(
                "\x1b[1;32m   Generated\x1b[0m package output directory -> target/{}/pkg/{}",
                profile, pkg_name
            );
        }
        return Some(PathBuf::from(out_rel));
    } else {
        println!(
            "\x1b[1;32m    Finished\x1b[0m compilation: no source files found in src/"
        );
        return None;
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn flash_project(args: &[String]) {
    let toml_path = Path::new("flame.toml");
    let mut pkg_name = "app".to_string();
    let mut toml_target = None;
    if toml_path.exists() {
        if let Ok(toml_str) = fs::read_to_string("flame.toml") {
            for line in toml_str.lines() {
                let t = line.trim();
                if t.starts_with("name =") {
                    if let Some(val) = t.split('=').nth(1) {
                        pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    }
                } else if t.starts_with("target =") {
                    if let Some(val) = t.split('=').nth(1) {
                        toml_target = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            }
        }
    }

    let mut port = None;
    let mut target = toml_target;
    for i in 0..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = Some(args[i + 1].as_str());
        } else if args[i] == "--target" && i + 1 < args.len() {
            target = Some(args[i + 1].to_string());
        }
    }

    let main_path = Path::new("src/main.fm");
    let content = if main_path.exists() {
        fs::read_to_string(main_path).unwrap_or_default()
    } else if args.len() >= 3 && !args[2].starts_with("-") && Path::new(&args[2]).exists() {
        fs::read_to_string(&args[2]).unwrap_or_default()
    } else {
        println!("\x1b[1;31merror:\x1b[0m no src/main.fm or Flame file found to flash.");
        return;
    };

    let stmts = match parse_file_stmts(main_path, &content) {
        Ok(s) => s,
        Err(e) => {
            e.print(&content);
            return;
        }
    };

    let mut _baud = 115200;
    if let Some((detected_target, detected_baud)) = embedded::codegen::detect_embedded_target(&stmts) {
        if target.is_none() {
            target = Some(detected_target);
        }
        _baud = detected_baud;
    }

    let target_str = target.unwrap_or_else(|| "arduino-uno".to_string());
    match embedded::codegen::generate_baremetal_firmware_project(&stmts, &target_str, &pkg_name) {
        Ok(build_dir) => {
            let _ = embedded::flasher::build_and_flash(&target_str, port, &build_dir, &pkg_name);
        }
        Err(err) => println!("\x1b[1;31merror:\x1b[0m {}", err),
    }
}

fn monitor_project(args: &[String]) {
    let mut port = None;
    let mut baud = 115200;
    for i in 0..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = Some(args[i + 1].as_str());
        } else if args[i] == "--baud" && i + 1 < args.len() {
            if let Ok(b) = args[i + 1].parse::<u32>() {
                baud = b;
            }
        }
    }
    embedded::flasher::open_serial_monitor(port, baud);
}

fn run_file(path_str: &str, force_local: bool, script_args: &[String]) {
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
            .args(script_args)
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

fn collect_fm_files(dir: &Path, list: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_fm_files(&p, list);
            } else if p.extension().and_then(|s| s.to_str()) == Some("fm") {
                list.push(p);
            }
        }
    }
}

fn has_test_annotations(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| {
        let check_stmt = if let Stmt::ExportDecl(inner, _) = stmt {
            inner.as_ref()
        } else {
            stmt
        };

        if let Stmt::FuncDecl { annotations, .. } = check_stmt {
            annotations.iter().any(|anno| {
                matches!(
                    anno.name.as_str(),
                    "Test"
                        | "Benchmark"
                        | "Parameterized"
                        | "ExpectPanic"
                        | "Ignore"
                        | "Only"
                        | "BeforeAll"
                        | "AfterAll"
                        | "Setup"
                        | "Cleanup"
                )
            })
        } else {
            false
        }
    })
}

fn run_tests(args: &[String]) {
    println!("\x1b[1;36mFlame Test & Benchmark Engine\x1b[0m");

    let mut files_to_test = Vec::new();
    if args.len() >= 3 && !args[2].starts_with('-') {
        let p = PathBuf::from(&args[2]);
        if p.exists() {
            if p.is_dir() {
                collect_fm_files(&p, &mut files_to_test);
            } else {
                files_to_test.push(p);
            }
        } else {
            println!(
                "\x1b[1;31merror:\x1b[0m test target '{}' does not exist.",
                args[2]
            );
            return;
        }
    } else {
        if Path::new("tests").exists() {
            collect_fm_files(Path::new("tests"), &mut files_to_test);
        }
        if Path::new("examples/tests").exists() {
            collect_fm_files(Path::new("examples/tests"), &mut files_to_test);
        }
        if Path::new("examples").exists() && !Path::new("examples/tests").exists() {
            collect_fm_files(Path::new("examples"), &mut files_to_test);
        }
        if Path::new("src").exists() {
            collect_fm_files(Path::new("src"), &mut files_to_test);
        }
        if files_to_test.is_empty() && Path::new("main.fm").exists() {
            files_to_test.push(PathBuf::from("main.fm"));
        }
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    files_to_test.retain(|p| seen.insert(p.clone()));

    if files_to_test.is_empty() {
        println!("No `.fm` test files found.");
        return;
    }

    let mut filtered_files = Vec::new();
    for path in files_to_test {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let stmts = match parse_file_stmts(&path, &content) {
            Ok(stmts) => stmts,
            Err(diag) => {
                println!(
                    "  \x1b[1;31mparse error in {}:{}:{}\x1b[0m: {}",
                    path.display(),
                    diag.span.line,
                    diag.span.col,
                    diag.message
                );
                continue;
            }
        };
        if has_test_annotations(&stmts) {
            filtered_files.push(path);
        }
    }
    files_to_test = filtered_files;

    if files_to_test.is_empty() {
        println!("No annotated test files found.");
        return;
    }

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_ignored = 0;
    let mut total_measured = 0;
    let mut total_filtered = 0;
    let total_start = std::time::Instant::now();

    for path in &files_to_test {
        println!("\nrunning tests in \x1b[1m{}\x1b[0m:", path.display());
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                println!("  \x1b[1;31mfatal:\x1b[0m failed to read file: {}", e);
                continue;
            }
        };

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
        let mut parser = Parser::new(tokens, path.to_string_lossy().to_string());
        let stmts = match parser.parse() {
            Ok(s) => s,
            Err(e) => {
                println!("  \x1b[1;31mparse error:\x1b[0m {}", e.message);
                total_failed += 1;
                continue;
            }
        };

        if !has_test_annotations(&stmts) {
            continue;
        }

        let mut runner = crate::runner::Runner::new(path.clone());
        runner.test_mode = true;
        let _ = runner.run(&stmts);

        let mut before_all = Vec::new();
        let mut after_all = Vec::new();
        let mut setup = Vec::new();
        let mut cleanup = Vec::new();
        let mut test_cases = Vec::new();
        let mut has_only_test = false;

        for stmt in &stmts {
            if let Stmt::FuncDecl {
                name, annotations, ..
            } = stmt
            {
                for anno in annotations {
                    match anno.name.as_str() {
                        "BeforeAll" => before_all.push(name.clone()),
                        "AfterAll" => after_all.push(name.clone()),
                        "Setup" => setup.push(name.clone()),
                        "Cleanup" => cleanup.push(name.clone()),
                        "Test" | "Benchmark" | "Parameterized" | "ExpectPanic" | "Ignore"
                        | "Only" => {
                            if !test_cases.contains(&name.clone()) {
                                test_cases.push(name.clone());
                            }
                            if anno.name == "Only"
                                || anno
                                    .args
                                    .iter()
                                    .any(|arg| arg.contains("only: true") || arg == "only: true")
                            {
                                has_only_test = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for func_name in &before_all {
            let func_opt = runner.env.lock().unwrap().get(func_name);
            if let Some(func_val) = func_opt {
                if let Err(e) = runner.invoke_callback_value(&func_val, vec![]) {
                    println!(
                        "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@BeforeAll\x1b[0m {}: {}",
                        func_name, e
                    );
                    total_failed += 1;
                    break;
                }
            }
        }

        for func_name in &test_cases {
            let mut is_ignore = false;
            let mut is_only = false;
            let mut is_benchmark = false;
            let mut is_expect_panic = false;
            let mut parameterized_args = None;

            for stmt in &stmts {
                if let Stmt::FuncDecl {
                    name, annotations, ..
                } = stmt
                {
                    if name == func_name {
                        for anno in annotations {
                            match anno.name.as_str() {
                                "Ignore" => is_ignore = true,
                                "Only" => is_only = true,
                                "Benchmark" => is_benchmark = true,
                                "ExpectPanic" => is_expect_panic = true,
                                "Parameterized" => {
                                    if !anno.args.is_empty() {
                                        parameterized_args = Some(anno.args[0].clone());
                                    }
                                }
                                "Test" => {
                                    if anno.args.iter().any(|arg| arg.contains("skip: true")) {
                                        is_ignore = true;
                                    }
                                    if anno.args.iter().any(|arg| arg.contains("only: true")) {
                                        is_only = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if has_only_test && !is_only {
                total_filtered += 1;
                continue;
            }

            if is_ignore {
                println!(
                    "  \x1b[33m[SKIP]\x1b[0m \x1b[1;36m@Ignore\x1b[0m {}",
                    func_name
                );
                total_ignored += 1;
                continue;
            }

            for setup_name in &setup {
                let setup_opt = runner.env.lock().unwrap().get(setup_name);
                if let Some(s_val) = setup_opt {
                    let _ = runner.invoke_callback_value(&s_val, vec![]);
                }
            }

            let test_func_opt = runner.env.lock().unwrap().get(func_name);
            if let Some(f_val) = test_func_opt {
                if is_benchmark {
                    let mut durations = Vec::new();
                    let mut benchmark_failed = false;
                    for _ in 0..25 {
                        let start = std::time::Instant::now();
                        if let Err(e) = runner.invoke_callback_value(&f_val, vec![]) {
                            println!(
                                "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Benchmark\x1b[0m {}: {}",
                                func_name, e
                            );
                            total_failed += 1;
                            benchmark_failed = true;
                            break;
                        }
                        durations.push(start.elapsed().as_secs_f64() * 1000.0);
                    }
                    if !benchmark_failed && !durations.is_empty() {
                        let avg = durations.iter().sum::<f64>() / durations.len() as f64;
                        let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max = durations.iter().fold(0.0_f64, |a, &b| a.max(b));
                        println!(
                            "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Benchmark\x1b[0m {}",
                            func_name
                        );
                        println!("    Benchmark: {}", func_name);
                        println!("    -----------");
                        println!("    average: {:.2} ms", avg);
                        println!("    min: {:.2} ms", min);
                        println!("    max: {:.2} ms", max);
                        total_measured += 1;
                    }
                } else if let Some(arg_str) = parameterized_args {
                    let mut l = Lexer::new(&arg_str);
                    let mut tok_vec = Vec::new();
                    loop {
                        let tok = l.next_token();
                        let e = tok.kind == lexer::TokenKind::EOF;
                        tok_vec.push(tok);
                        if e {
                            break;
                        }
                    }
                    let mut p = Parser::new(tok_vec, "param_arg".to_string());
                    if let Ok(expr) = p.parse_expr() {
                        let env_clone = runner.env.clone();
                        if let Ok(evaled) = runner.eval_expr(&expr, env_clone) {
                            let list = match evaled {
                                crate::vm::Value::Tuple(vec_val) => vec_val.clone(),
                                other => vec![other],
                            };
                            let mut all_ok = true;
                            let start = std::time::Instant::now();
                            for case in &list {
                                let call_args = match case {
                                    crate::vm::Value::Tuple(tup) => tup.clone(),
                                    single => vec![single.clone()],
                                };
                                if let Err(e) = runner.invoke_callback_value(&f_val, call_args) {
                                    println!(
                                        "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {} on argument {:?}: {}",
                                        func_name, case, e
                                    );
                                    all_ok = false;
                                    break;
                                }
                            }
                            if all_ok {
                                println!(
                                    "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {} ({} parameter cases in {:.2}ms)",
                                    func_name,
                                    list.len(),
                                    start.elapsed().as_secs_f64() * 1000.0
                                );
                                total_passed += 1;
                            } else {
                                total_failed += 1;
                            }
                        } else {
                            println!(
                                "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {}: failed to evaluate parameter argument expression",
                                func_name
                            );
                            total_failed += 1;
                        }
                    }
                } else {
                    let start = std::time::Instant::now();
                    let res = runner.invoke_callback_value(&f_val, vec![]);
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    if is_expect_panic {
                        match res {
                            Err(e) => {
                                println!(
                                    "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@ExpectPanic\x1b[0m {} (expected panic occurred in {:.2}ms: {})",
                                    func_name, elapsed, e
                                );
                                total_passed += 1;
                            }
                            Ok(_) => {
                                println!(
                                    "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@ExpectPanic\x1b[0m {}: function completed without expected error/panic!",
                                    func_name
                                );
                                total_failed += 1;
                            }
                        }
                    } else {
                        match res {
                            Ok(_) => {
                                println!(
                                    "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Test\x1b[0m {} ({:.2}ms)",
                                    func_name, elapsed
                                );
                                total_passed += 1;
                            }
                            Err(e) => {
                                println!(
                                    "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Test\x1b[0m {}: {}",
                                    func_name, e
                                );
                                total_failed += 1;
                            }
                        }
                    }
                }
            }

            for cleanup_name in &cleanup {
                let cleanup_opt = runner.env.lock().unwrap().get(cleanup_name);
                if let Some(c_val) = cleanup_opt {
                    let _ = runner.invoke_callback_value(&c_val, vec![]);
                }
            }
        }

        for func_name in &after_all {
            let after_opt = runner.env.lock().unwrap().get(func_name);
            if let Some(func_val) = after_opt {
                let _ = runner.invoke_callback_value(&func_val, vec![]);
            }
        }
    }

    let total_elapsed = total_start.elapsed().as_secs_f64() * 1000.0;
    let result_str = if total_failed == 0 {
        "\x1b[1;32mok.\x1b[0m"
    } else {
        "\x1b[1;31mFAILED.\x1b[0m"
    };
    println!(
        "\n\x1b[1;32mtest result:\x1b[0m {} {} passed; {} failed; {} ignored; {} measured; {} filtered out; finished in {:.2}ms",
        result_str,
        total_passed,
        total_failed,
        total_ignored,
        total_measured,
        total_filtered,
        total_elapsed
    );
}

fn init_native_bridge(plugin_name: &str) {
    let toml_path = Path::new("flame.toml");
    if !toml_path.exists() {
        println!(
            "\x1b[1;31merror:\x1b[0m no flame.toml manifest file found in the current directory."
        );
        println!("help: run this command inside a valid Flame project folder.");
        return;
    }

    println!(
        "\x1b[1;36mInitializing\x1b[0m native Rust plugin '{}' environment...",
        plugin_name
    );

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
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]

[profile.dev]
split-debuginfo = "unpacked"

[profile.release]
opt-level = 3
lto = "thin"
strip = true
codegen-units = 1
"#,
        plugin_name
    );
    let cargo_path = native_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(&cargo_path, cargo_toml).unwrap();
        println!("\x1b[1;32mCreated\x1b[0m {:?}", cargo_path);
    }

    // Update flame.toml to append [plugins] if not present
    let mut toml_content = fs::read_to_string(toml_path).unwrap();
    if !toml_content.contains(&format!("{} =", plugin_name))
        && !toml_content.contains(&format!("{}=", plugin_name))
    {
        if !toml_content.contains("[plugins]") {
            toml_content.push_str(&format!("\n[plugins]\n{} = \"./native\"\n", plugin_name));
        } else if let Some(idx) = toml_content.find("[plugins]") {
            let insert_pos = idx + "[plugins]".len();
            toml_content.insert_str(insert_pos, &format!("\n{} = \"./native\"", plugin_name));
        }
        fs::write(toml_path, toml_content).unwrap();
        println!(
            "\x1b[1;32mUpdated\x1b[0m flame.toml to reference native plugin '{}'.",
            plugin_name
        );
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
    let mut native_modules = parse_manifest_section(&manifest_content, "[native-dependencies]")
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    for (name, _) in parse_manifest_section(&manifest_content, "[plugins]") {
        if !native_modules.contains(&name) {
            native_modules.push(name);
        }
    }
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
                kind: "plugin".to_string(),
                detail: "native plugin".to_string(),
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

    let word_under_cursor_raw = extract_word_at_cursor(current_line, cursor_col);
    let word_under_cursor = word_under_cursor_raw.trim_start_matches('@').to_string();

    // Scan for variables and structs
    let (mut scanned_vars, scanned_structs) = ide::scan_document(&content);

    let imported_module_decls = load_imported_module_declarations(&manifest_dir, file);
    for stmt in &imported_module_decls {
        if let Some((name, params, return_type, is_annotation)) = match stmt {
            crate::parser::Stmt::FuncDecl {
                name,
                params,
                return_type,
                ..
            } => Some((name, params, return_type.as_deref(), false)),
            crate::parser::Stmt::AnnotationDecl {
                name,
                params,
                return_type,
                ..
            } => Some((name, params, return_type.as_deref(), true)),
            _ => None,
        } {
            let params_str = params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.type_name))
                .collect::<Vec<_>>()
                .join(", ");
            let ret_str = return_type.unwrap_or("Nil");
            let sig = if is_annotation {
                format!("annotation {}({}) -> {}", name, params_str, ret_str)
            } else {
                format!("fn {}({}) -> {}", name, params_str, ret_str)
            };

            scanned_vars.push(ide::ScannedVar {
                name: name.clone(),
                typ: Some(sig.clone()),
            });
            completions.push(JsonCompletion {
                label: name.clone(),
                kind: if is_annotation {
                    "annotation".to_string()
                } else {
                    "function".to_string()
                },
                detail: "imported module declaration".to_string(),
                documentation: Some(sig),
            });
        }
    }

    // Enrich scanned_vars with return types from native module function calls
    for mod_name in &native_modules {
        if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
            for func in &meta.functions {
                let pattern1 = format!("= {}.{}(", mod_name, func.flame_name);
                let pattern2 = format!("= await {}.{}(", mod_name, func.flame_name);
                for line in content.lines() {
                    if line.contains(&pattern1) || line.contains(&pattern2) {
                        if let Some(eq_idx) = line.find('=') {
                            let left = &line[..eq_idx].trim();
                            if let Some(var_name) = left.split_whitespace().last() {
                                if let Some(var) =
                                    scanned_vars.iter_mut().find(|v| v.name == var_name)
                                {
                                    var.typ = Some(func.return_type.clone());
                                } else {
                                    scanned_vars.push(ide::ScannedVar {
                                        name: var_name.to_string(),
                                        typ: Some(func.return_type.clone()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let (namespace, member_prefix) = extract_member_context(current_line, cursor_col);

    let mut exact_ast_hover = None;
    if let Some(tc) = &tc_opt {
        if let Some(l) = line {
            let mut best_span: Option<&crate::lexer::Span> = None;
            let mut best_ty_str = None;

            for (span, ty_str) in &tc.hover_info {
                let end_col = span.col + (span.end - span.start);
                if span.line == l && cursor_col >= span.col && cursor_col <= end_col {
                    if let Some(best) = best_span {
                        if (span.end - span.start) < (best.end - best.start) {
                            best_span = Some(span);
                            best_ty_str = Some(ty_str);
                        }
                    } else {
                        best_span = Some(span);
                        best_ty_str = Some(ty_str);
                    }
                }
            }

            if let Some(ty_str) = best_ty_str {
                if ty_str != "Unknown" {
                    if ty_str.starts_with("```") || ty_str.starts_with('@') || ty_str.contains('\n') {
                        exact_ast_hover = Some(JsonHover {
                            label: word_under_cursor.clone(),
                            documentation: Some(ty_str.clone()),
                        });
                    } else if ty_str.starts_with("fn(") {
                        if let Some(var) = scanned_vars.iter().find(|v| v.name == word_under_cursor) {
                            if let Some(t) = &var.typ {
                                if t.starts_with("fn ") {
                                    exact_ast_hover = Some(JsonHover {
                                        label: word_under_cursor.clone(),
                                        documentation: Some(format!("```flame\n{}\n```\nDefined in project", t)),
                                    });
                                }
                            }
                        }
                        if exact_ast_hover.is_none() && !word_under_cursor.is_empty() {
                            let sig = format!("fn {}{}", word_under_cursor, &ty_str[2..]);
                            exact_ast_hover = Some(JsonHover {
                                label: word_under_cursor.clone(),
                                documentation: Some(format!(
                                    "```flame\n{}\n```\nDefined in project",
                                    sig
                                )),
                            });
                        }
                    } else if !word_under_cursor.is_empty() {
                        let sig = format!("{}: {}", word_under_cursor, ty_str);
                        exact_ast_hover = Some(JsonHover {
                            label: word_under_cursor.clone(),
                            documentation: Some(format!(
                                "```flame\n{}\n```\nInferred type from AST",
                                sig
                            )),
                        });
                    }
                }
            }
        }
    }

    let mut scanned_var_hover = None;
    if !word_under_cursor.is_empty() {
        if let Some(var) = scanned_vars.iter().find(|v| v.name == word_under_cursor) {
            if let Some(t) = &var.typ {
                if t != "Unknown" {
                    let (code_block, source_msg) =
                        if t.starts_with("fn ") || t.starts_with("annotation ") {
                            (t.clone(), "Defined in project".to_string())
                        } else if let Some(mod_name) = native_modules.iter().find(|m| {
                            load_meta_from_project(&manifest_dir, m)
                                .map_or(false, |meta| meta.structs.iter().any(|s| s.name == *t))
                        }) {
                            (
                                format!("{}: {}", word_under_cursor, t),
                                format!("Struct type from native module '{}'", mod_name),
                            )
                        } else {
                            (
                                format!("{}: {}", word_under_cursor, t),
                                "Inferred type from AST".to_string(),
                            )
                        };
                    scanned_var_hover = Some(JsonHover {
                        label: word_under_cursor.clone(),
                        documentation: Some(format!(
                            "```flame\n{}\n```\n{}",
                            code_block, source_msg
                        )),
                    });
                }
            }
        }
    }

    let mut hover = None;

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
                        let params_str = function
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, p.type_name))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let sig = format!(
                            "fn {}({}) -> {}",
                            function.flame_name, params_str, function.return_type
                        );
                        let doc = function.docs.clone().unwrap_or_else(|| {
                            load_local_rust_doc(&manifest_dir, &namespace, &function.flame_name)
                                .unwrap_or_default()
                        });
                        JsonHover {
                            label: format!("{}.{}", namespace, function.flame_name),
                            documentation: Some(format!(
                                "```flame\n{}\n```\n{}\n\n**Return Type / Structure**: `{}`",
                                sig, doc, function.return_type
                            )),
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
                                let params_str = function
                                    .params
                                    .iter()
                                    .map(|p| format!("{}: {}", p.name, p.type_name))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let sig = format!(
                                    "fn {}({}) -> {}",
                                    function.flame_name, params_str, function.return_type
                                );
                                let doc = function.docs.clone().unwrap_or_else(|| {
                                    load_local_rust_doc(
                                        &manifest_dir,
                                        &namespace,
                                        &function.flame_name,
                                    )
                                    .unwrap_or_default()
                                });
                                hover_found = Some(JsonHover {
                                    label: format!("{}.{}", namespace, function.flame_name),
                                    documentation: Some(format!(
                                        "```flame\n{}\n```\n{}\n\n**Return Type / Structure**: `{}`",
                                        sig, doc, function.return_type
                                    )),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        } else if let Some(std_methods) = ide::get_std_module_methods(&namespace) {
            for method in &std_methods {
                if member_prefix
                    .as_deref()
                    .map_or(true, |prefix| method.starts_with(prefix))
                {
                    completions.push(JsonCompletion {
                        label: method.clone(),
                        kind: "function".to_string(),
                        detail: format!("std.{}", namespace),
                        documentation: crate::std_docs::get_std_function_doc(&namespace, method)
                            .map(|d| d.to_string()),
                    });
                }
            }

            if !word_under_cursor.is_empty() && std_methods.contains(&word_under_cursor) {
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
        } else if let Some(local_stmts) =
            load_local_module_declarations(&manifest_dir, file, &namespace)
        {
            let mut provided_completions = false;

            for stmt in &local_stmts {
                let func_info = match stmt {
                    crate::parser::Stmt::FuncDecl {
                        name,
                        params,
                        return_type,
                        ..
                    }
                    | crate::parser::Stmt::AnnotationDecl {
                        name,
                        params,
                        return_type,
                        ..
                    } => Some((name, params, return_type)),
                    crate::parser::Stmt::ExportDecl(inner, _) => {
                        if let crate::parser::Stmt::FuncDecl {
                            name,
                            params,
                            return_type,
                            ..
                        }
                        | crate::parser::Stmt::AnnotationDecl {
                            name,
                            params,
                            return_type,
                            ..
                        } = &**inner
                        {
                            Some((name, params, return_type))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some((name, params, return_type)) = func_info {
                    let param_strs = params
                        .iter()
                        .map(|p| {
                            format!(
                                "{}{}: {}",
                                if p.is_mut { "mut " } else { "" },
                                p.name,
                                p.type_name
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret_str = return_type.as_deref().unwrap_or("Nil");
                    let sig = format!("fn {}({}) -> {}", name, param_strs, ret_str);

                    if member_prefix
                        .as_deref()
                        .map_or(true, |prefix| name.starts_with(prefix))
                    {
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

            // If it's a native module directly (e.g. `flamer.`)
            if var_type.is_none() && native_modules.contains(&namespace) {
                if let Some(meta) = load_meta_from_project(&manifest_dir, &namespace) {
                    for function in &meta.functions {
                        if member_prefix
                            .as_deref()
                            .map(|p| function.flame_name.starts_with(p))
                            .unwrap_or(true)
                        {
                            let params_str = function
                                .params
                                .iter()
                                .map(|p| format!("{}: {}", p.name, p.type_name))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let sig = format!(
                                "fn {}({}) -> {}",
                                function.flame_name,
                                params_str,
                                function.return_type
                            );
                            
                            completions.push(JsonCompletion {
                                label: function.flame_name.clone(),
                                kind: "function".to_string(),
                                detail: format!("{} (from {})", function.flame_name, namespace),
                                documentation: Some(function.docs.clone().unwrap_or(sig.clone())),
                            });
                            provided_completions = true;
                        }

                        if !word_under_cursor.is_empty()
                            && function.flame_name == word_under_cursor
                        {
                            let params_str = function
                                .params
                                .iter()
                                .map(|p| format!("{}: {}", p.name, p.type_name))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let sig = format!(
                                "fn {}({}) -> {}",
                                function.flame_name,
                                params_str,
                                function.return_type
                            );
                            let doc = function.docs.clone().unwrap_or_else(|| "".to_string());
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}()", namespace, function.flame_name),
                                documentation: Some(format!(
                                    "```flame\n{}\n```\n{}\n\n**Return Type**: `{}`",
                                    sig, doc, function.return_type
                                )),
                            });
                        }
                    }
                    
                    for struct_meta in &meta.structs {
                        if member_prefix
                            .as_deref()
                            .map(|p| struct_meta.name.starts_with(p))
                            .unwrap_or(true)
                        {
                            completions.push(JsonCompletion {
                                label: struct_meta.name.clone(),
                                kind: "class".to_string(),
                                detail: format!("struct (from {})", namespace),
                                documentation: struct_meta.docs.clone(),
                            });
                            provided_completions = true;
                        }
                        
                        if !word_under_cursor.is_empty()
                            && struct_meta.name == word_under_cursor
                        {
                            let doc = struct_meta.docs.clone().unwrap_or_else(|| "".to_string());
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}", namespace, struct_meta.name),
                                documentation: Some(format!(
                                    "```flame\nstruct {}\n```\n{}",
                                    struct_meta.name, doc
                                )),
                            });
                        }
                    }
                }
            }

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

                // Also check native types like ThreadHandler or FlameServer across all modules
                if !provided_completions {
                    let modules_to_check = {
                        let mut mods = native_modules.clone();
                        if !mods.contains(&t) {
                            mods.push(t.clone());
                        }
                        mods
                    };
                    for mod_name in modules_to_check {
                        if let Some(meta) = load_meta_from_project(&manifest_dir, &mod_name) {
                            for struct_meta in &meta.structs {
                                if struct_meta.name == *t
                                    || struct_meta.name.to_lowercase() == t.to_lowercase()
                                {
                                    for function in &struct_meta.methods {
                                        if member_prefix
                                            .as_deref()
                                            .map(|p| function.flame_name.starts_with(p))
                                            .unwrap_or(true)
                                        {
                                            completions.push(JsonCompletion {
                                                label: function.flame_name.clone(),
                                                kind: "method".to_string(),
                                                detail: format!(
                                                    "{}::{} (from {})",
                                                    struct_meta.name, function.flame_name, mod_name
                                                ),
                                                documentation: function.docs.clone().or_else(
                                                    || {
                                                        load_local_rust_doc(
                                                            &manifest_dir,
                                                            &mod_name,
                                                            &function.flame_name,
                                                        )
                                                    },
                                                ),
                                            });
                                            provided_completions = true;
                                        }
                                        if !word_under_cursor.is_empty()
                                            && function.flame_name == word_under_cursor
                                        {
                                            let params_str = function
                                                .params
                                                .iter()
                                                .map(|p| format!("{}: {}", p.name, p.type_name))
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                            let sig = format!(
                                                "fn {}({}) -> {}",
                                                function.flame_name,
                                                params_str,
                                                function.return_type
                                            );
                                            let doc = function.docs.clone().unwrap_or_else(|| {
                                                load_local_rust_doc(
                                                    &manifest_dir,
                                                    &mod_name,
                                                    &function.flame_name,
                                                )
                                                .unwrap_or_default()
                                            });
                                            hover_found = Some(JsonHover {
                                                label: format!(
                                                    "{}::{}()",
                                                    struct_meta.name, function.flame_name
                                                ),
                                                documentation: Some(format!(
                                                    "```flame\n{}\n```\n{}\n\n**Return Type / Structure**: `{}`",
                                                    sig, doc, function.return_type
                                                )),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

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
                    (
                        "map",
                        "Transforms each element of the collection using the provided closure and returns a new collection.\n\nExample:\n```flame\narr.map((x) { return x * 2 })\n```",
                    ),
                    (
                        "filter",
                        "Returns a new collection containing only the elements for which the provided closure returns true.\n\nExample:\n```flame\narr.filter((x) { return x > 0 })\n```",
                    ),
                    ("mode", "Configures digital pin direction. Values: `\"OUTPUT\"`, `\"INPUT\"`, `\"INPUT_PULLUP\"`, `\"PWM\"` (Hardware Pin)"),
                    ("high", "Drives digital pin voltage to logical HIGH (Hardware Pin)"),
                    ("low", "Drives digital pin voltage to logical LOW (Hardware Pin)"),
                    ("toggle", "Flips digital pin voltage to opposite state (Hardware Pin)"),
                    ("read", "Reads digital/analog logic level or ADC raw value (Hardware Pin/ADC)"),
                    ("angle", "Sets absolute target rotation angle in degrees (Hardware Servo)"),
                    ("speed", "Sets throttle output as percentage (Hardware Motor)"),
                    ("forward", "Sets directional polarization to forward (Hardware Motor)"),
                    ("reverse", "Sets directional polarization to reverse (Hardware Motor)"),
                    ("stop", "Electro-dynamically brakes shaft to halt (Hardware Motor/Servo)"),
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
                    documentation: Some(format!(
                        "```flame\nmodule {}\n```\n{}",
                        word_under_cursor, doc
                    )),
                });
            } else if let Some(v) = scanned_vars.iter().find(|v| v.name == word_under_cursor) {
                if let Some(typ) = &v.typ {
                    if let Some(meta) = load_meta_from_project(&manifest_dir, typ) {
                        let struct_names = meta
                            .structs
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let desc = format!(
                            "```flame\nlet {}: native.{}\n```\n**Native Plugin Instance** (`{}`)\n\n**Structures in plugin**: `{}`",
                            v.name, typ, typ, struct_names
                        );
                        hover_found = Some(JsonHover {
                            label: format!("{}: native.{}", v.name, typ),
                            documentation: Some(desc),
                        });
                    }
                }
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
        } else if let Some(hf) = hover_found {
            hover = Some(hf);
        } else if !word_under_cursor.is_empty() {
            // Check if the bare word is a function/annotation from any native module
            for mod_name in &native_modules {
                if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
                    if let Some(function) = meta.functions.iter().find(|f| f.flame_name == word_under_cursor) {
                        let params_str = function
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, p.type_name))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let sig = format!(
                            "fn {}({}) -> {}",
                            function.flame_name,
                            params_str,
                            function.return_type
                        );
                        let doc = function.docs.clone().unwrap_or_else(|| "".to_string());
                        hover = Some(JsonHover {
                            label: format!("{}::{}()", mod_name, function.flame_name),
                            documentation: Some(format!(
                                "```flame\n{}\n```\n{}\n\n**Return Type**: `{}`",
                                sig, doc, function.return_type
                            )),
                        });
                        break;
                    }
                }
            }
        }
    };

    // Prioritize rich documentation (keywords, built-ins, standard library, decorators, native docs).
    // If no rich doc exists, fall back to exact AST hover from typechecker, then scanned var hover.
    if hover.is_none() {
        if let Some(ast) = exact_ast_hover {
            hover = Some(ast);
        } else {
            hover = scanned_var_hover;
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
        "embedded".to_string(),
    ]
}

fn extract_member_context(line: &str, col: usize) -> (Option<String>, Option<String>) {
    let upto = line.chars().take(col.saturating_sub(1)).collect::<String>();
    if let Some(dot_index) = upto.rfind('.') {
        let after_dot = &upto[dot_index + 1..];
        if !after_dot.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            return (None, None);
        }
        let left = upto[..dot_index].trim();
        let right = after_dot.to_string();
        return (
            left.split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|s| !s.is_empty())
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
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '"' || ch == '-' {
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
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '"' || ch == '-' {
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
    let path_parts = namespace
        .split('.')
        .map(|part| part.to_string())
        .collect::<Vec<_>>();
    let candidate = crate::stdlib::locate_import_file(Path::new(current_file), &path_parts)
        .or_else(|| {
            let direct = manifest_dir.join(format!("{}.fm", namespace));
            if direct.exists() { Some(direct) } else { None }
        })?;
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

fn load_imported_module_declarations(
    _manifest_dir: &Path,
    current_file: &str,
) -> Vec<crate::parser::Stmt> {
    let mut results = Vec::new();
    let content = fs::read_to_string(current_file).unwrap_or_default();
    let import_re = Regex::new(r"(?m)^import\s+([a-zA-Z_][\w]*(?:\.[a-zA-Z_][\w]*)*)").unwrap();

    for cap in import_re.captures_iter(&content) {
        let module_path = cap[1].to_string();
        if module_path == "std"
            || module_path == "native"
            || module_path.starts_with("std.")
            || module_path.starts_with("native.")
        {
            continue;
        }

        let path_parts: Vec<String> = module_path.split('.').map(|s| s.to_string()).collect();
        if let Some(file_path) =
            crate::stdlib::locate_import_file(Path::new(current_file), &path_parts)
        {
            if let Ok(module_content) = fs::read_to_string(&file_path) {
                let mut lexer = Lexer::new(&module_content);
                let mut tokens = Vec::new();
                loop {
                    let tok = lexer.next_token();
                    let is_eof = tok.kind == crate::lexer::TokenKind::EOF;
                    tokens.push(tok);
                    if is_eof {
                        break;
                    }
                }
                let mut parser = Parser::new(tokens, file_path.to_string_lossy().to_string());
                if let Ok(parsed_stmts) = parser.parse() {
                    for stmt in parsed_stmts {
                        if let crate::parser::Stmt::ExportDecl(inner, _) = stmt {
                            results.push(*inner);
                        }
                    }
                }
            }
        }
    }

    results
}

fn load_meta_from_project(
    manifest_dir: &Path,
    module_name: &str,
) -> Option<package_manager::FlameMeta> {
    let mut meta_path = manifest_dir
        .join(".flame")
        .join("pkg")
        .join(module_name)
        .join(format!("{}.fmi", module_name));
    if !meta_path.exists() {
        meta_path = manifest_dir
            .join(".flame")
            .join("pkg")
            .join("native")
            .join(format!("{}.fmi", module_name));
    }
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
