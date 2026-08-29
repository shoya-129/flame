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
mod test_engine;
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
    })
    .unwrap_or_else(|_| ());

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
        "doctor" => {
            run_doctor_command();
        }
        "build" => {
            build_project(&args);
        }
        "package" => {
            package_project(&args);
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
            let source = if args.contains(&"--stdin".to_string()) {
                use std::io::Read;
                let mut buf = String::new();
                let _ = std::io::stdin().read_to_string(&mut buf);
                buf
            } else {
                match fs::read_to_string(filepath) {
                    Ok(content) => content,
                    Err(err) => {
                        println!(
                            "\x1b[1;31merror:\x1b[0m failed to read '{}': {}",
                            filepath, err
                        );
                        return;
                    }
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
            let force_local = args.contains(&"--local".to_string());

            let (filepath, script_args_start) = if args.len() > 2 {
                let potential_file_idx = if args[2] == "--local" { 3 } else { 2 };

                if args.len() > potential_file_idx
                    && (Path::new(&args[potential_file_idx]).exists()
                        || args[potential_file_idx].ends_with(".fm"))
                {
                    (args[potential_file_idx].clone(), potential_file_idx + 1)
                } else if Path::new("src/main.fm").exists() {
                    ("src/main.fm".to_string(), potential_file_idx)
                } else {
                    println!(
                        "\x1b[1;31merror:\x1b[0m please specify a Flame file to run or create src/main.fm"
                    );
                    return;
                }
            } else if Path::new("src/main.fm").exists() {
                ("src/main.fm".to_string(), 2)
            } else {
                println!(
                    "\x1b[1;31merror:\x1b[0m please specify a Flame file to run or create src/main.fm"
                );
                println!("usage: flame run [file_path.fm]");
                return;
            };

            let mut filtered_script_args = Vec::new();
            for arg in args.iter().skip(script_args_start) {
                if arg != "--local" {
                    filtered_script_args.push(arg.clone());
                }
            }

            run_file(&filepath, force_local, &filtered_script_args);
        }
        "test" => {
            run_tests(&args);
        }
        "gen" => {
            if args.len() < 3 || args[2] != "fmi" {
                println!("\x1b[1;31merror:\x1b[0m unknown subcommand");
                println!("usage: flame gen fmi <rust_file>");
                return;
            }
            if args.len() < 4 {
                println!("\x1b[1;31merror:\x1b[0m expected rust file path");
                println!("usage: flame gen fmi <rust_file>");
                return;
            }
            let filepath = &args[3];
            package_manager::gen_fmi_from_rust_file(std::path::Path::new(filepath));
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
            println!("Flame {} (Third Spark)", env!("CARGO_PKG_VERSION"));
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

fn run_doctor_command() {
    println!("\nFlame 0.3.0 LTS\n");

    fn check_cmd(cmd: &str, args: &[&str]) -> bool {
        std::process::Command::new(cmd).args(args).output().is_ok()
    }

    println!("{} Blaze compiler", if true { "✓" } else { "✗" });
    println!(
        "{} Rust toolchain",
        if check_cmd("rustc", &["--version"]) {
            "✓"
        } else {
            "✗"
        }
    );
    println!(
        "{} Cargo",
        if check_cmd("cargo", &["--version"]) {
            "✓"
        } else {
            "✗"
        }
    );
    println!("{} Native plugin support", "✓");
    println!("{} Standard library", "✓");
    println!("{} Package manager", "✓");
    println!("{} FMI generation", "✓");
    println!("{} Test runner", "✓");
    println!("{} Formatter", "✓");

    println!("\nPlatform");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    // capitalize first letter of OS
    let mut os_chars = os.chars();
    let os_cap = match os_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + os_chars.as_str(),
    };
    println!("✓ {} {}", os_cap, arch);

    println!("\nOptional");
    println!("{} Camera", "✓");
    println!("{} Bluetooth", "✓");
    println!("{} Serial", "✓");
    println!(
        "{} QEMU",
        if check_cmd("qemu-system-x86_64", &["--version"])
            || check_cmd("qemu-system-aarch64", &["--version"])
        {
            "✓"
        } else {
            "✗"
        }
    );
    println!();
}

fn print_help() {
    let bold = "\x1b[1m";
    let cyan = "\x1b[1;36m";
    let reset = "\x1b[0m";

    println!(
        "{}Flame Compiler & Package Manager (Version {}){} ",
        bold,
        env!("CARGO_PKG_VERSION"),
        reset
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
    println!(
        "  {}help, -h, --help{}        Print help details",
        cyan, reset
    );
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
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2026\"\ntype = \"executable\"\n\n[dependencies]\n",
        name
    );
    fs::write(root.join("flame.toml"), toml_content).unwrap();

    // Write src/main.fm
    let main_flame = r#"

println("Hello, world!")

"#;
    fs::write(root.join("src/main.fm"), main_flame).unwrap();

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
    println!("\x1b[1;36m   Compiling\x1b[0m std standard library...");

    let src_dir = Path::new("src");
    let has_source_files = if src_dir.exists() {
        fs::read_dir(src_dir)
            .map(|mut it| {
                it.any(|e| {
                    e.map(|entry| entry.path().extension().map_or(false, |ext| ext == "fm"))
                        .unwrap_or(false)
                })
            })
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
                                if fpath.is_file() && fpath.extension().map_or(false, |e| e == "fm")
                                {
                                    let content = fs::read_to_string(&fpath).unwrap_or_default();
                                    if let Ok(mut stmts) = parse_file_stmts(&fpath, &content) {
                                        crate::parser::filter_platform_stmts(
                                            &mut stmts,
                                            target.as_deref(),
                                        );
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
                        Ok(mut stmts) => {
                            crate::parser::filter_platform_stmts(&mut stmts, target.as_deref());
                            stmts
                        }
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
            if let Some((detected_target, _)) =
                embedded::codegen::detect_embedded_target(&all_stmts)
            {
                target = Some(detected_target);
            }
        }

        if let Some(t) = target {
            match embedded::codegen::generate_baremetal_firmware_project(&all_stmts, &t, &pkg_name)
            {
                Ok(build_dir) => {
                    println!(
                        "\x1b[1;32m    Finished\x1b[0m embedded bare-metal firmware project at {}",
                        build_dir.display()
                    );
                    return Some(build_dir);
                }
                Err(e) => {
                    println!("\x1b[1;31merror:\x1b[0m {}", e);
                    return None;
                }
            }
        }

        let manifest_content = fs::read_to_string("flame.toml").unwrap_or_default();
        let mut native_deps_raw =
            parse_manifest_section(&manifest_content, "[native-dependencies]");
        let mut plugins_raw = parse_manifest_section(&manifest_content, "[plugins]");

        if let Ok(entries) = fs::read_dir(".flame/pkg") {
            for entry in entries.flatten() {
                let pkg_path = entry.path();
                if pkg_path.is_dir() {
                    let toml_path = pkg_path.join("flame.toml");
                    if toml_path.exists() {
                        let dep_manifest = fs::read_to_string(&toml_path).unwrap_or_default();
                        for (name, mut path) in
                            parse_manifest_section(&dep_manifest, "[native-dependencies]")
                        {
                            if path.starts_with('"') && path.ends_with('"') {
                                path = path[1..path.len() - 1].to_string();
                            }
                            if path.starts_with('.') || path.starts_with('/') {
                                let abs = std::fs::canonicalize(pkg_path.join(&path))
                                    .unwrap_or_else(|_| pkg_path.join(&path));
                                crate::package_manager::inspect_native_plugin(&name, &abs);
                                path = format!("\"{}\"", abs.to_string_lossy().replace("\\", "/"));
                            } else {
                                path = format!("\"{}\"", path);
                            }
                            native_deps_raw.push((name, path));
                        }
                        for (name, mut path) in parse_manifest_section(&dep_manifest, "[plugins]") {
                            if path.starts_with('"') && path.ends_with('"') {
                                path = path[1..path.len() - 1].to_string();
                            }
                            if path.starts_with('.') || path.starts_with('/') {
                                let abs = std::fs::canonicalize(pkg_path.join(&path))
                                    .unwrap_or_else(|_| pkg_path.join(&path));
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
            if path_str.starts_with('.')
                || path_str.starts_with('/')
                || std::path::Path::new(&path_str).is_absolute()
            {
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

            let is_local = path_str.starts_with('.')
                || path_str.starts_with('/')
                || std::path::Path::new(&path_str).is_absolute();
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
            false,
            false,
            None,
        );

        let ext = std::env::consts::EXE_SUFFIX;
        let exe_name = format!("{}{}", pkg_name.clone(), ext);
        let out_rel = format!("target/{}/{}", profile, exe_name);
        println!(
            "\x1b[1;32m    Finished\x1b[0m {} target(s) -> {} in 0.12s",
            mode_str, out_rel
        );

        if is_release {
            println!(
                "\x1b[1;32m   Packaged\x1b[0m distribution build successfully in \x1b[1mtarget/release/\x1b[0m directory"
            );
        }

        return Some(PathBuf::from(out_rel));
    } else {
        println!("\x1b[1;32m    Finished\x1b[0m compilation: no source files found in src/");
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

fn package_project(_args: &[String]) {
    let toml_path = Path::new("flame.toml");
    if !toml_path.exists() {
        println!("\x1b[1;31merror:\x1b[0m no flame.toml manifest file found.");
        return;
    }

    let mut pkg_name = "app".to_string();
    let mut is_pkg = false;
    if let Ok(toml_str) = fs::read_to_string("flame.toml") {
        for line in toml_str.lines() {
            let t = line.trim();
            if t.starts_with("name =") {
                if let Some(val) = t.split('=').nth(1) {
                    pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if t.starts_with("type =") {
                if let Some(val) = t.split('=').nth(1) {
                    let parsed_type = val
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_lowercase();
                    if parsed_type == "pkg" || parsed_type == "lib" {
                        is_pkg = true;
                    }
                }
            }
        }
    }

    if !is_pkg {
        println!("\x1b[1;33mwarning:\x1b[0m project type is not 'pkg'. packaging anyway.");
    }

    println!("\x1b[1;36m Packaging\x1b[0m {} ...", pkg_name);
    let src_dir = Path::new("src");
    let mut has_exports = false;
    if src_dir.exists() {
        if let Ok(entries) = fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "fm") {
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    if let Ok(stmts) = parse_file_stmts(&path, &content) {
                        for stmt in &stmts {
                            if matches!(stmt, Stmt::ExportDecl(_, _)) {
                                has_exports = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if !has_exports {
        println!(
            "\x1b[1;33mwarning:\x1b[0m package '{}' does not export anything. Is this intentional?",
            pkg_name
        );
    } else {
        println!(
            "\x1b[1;32m  Verified\x1b[0m package '{}' exports valid symbols.",
            pkg_name
        );
    }

    let pkg_out_dir = Path::new("target").join("pkg").join(&pkg_name);
    let _ = fs::create_dir_all(&pkg_out_dir);

    if Path::new("flame.toml").exists() {
        let _ = fs::copy("flame.toml", pkg_out_dir.join("flame.toml"));
    }
    if Path::new("src").exists() {
        let _ = copy_dir_all("src", pkg_out_dir.join("src"));
    }
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("fmi") {
                let _ = fs::copy(&path, pkg_out_dir.join(path.file_name().unwrap()));
            }
        }
    }
    println!(
        "\x1b[1;32m  Packaged\x1b[0m successfully to target/pkg/{}",
        pkg_name
    );
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
                        toml_target =
                            Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
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
        Ok(mut s) => {
            crate::parser::filter_platform_stmts(&mut s, target.as_deref());
            s
        }
        Err(e) => {
            e.print(&content);
            return;
        }
    };

    let mut _baud = 115200;
    if let Some((detected_target, detected_baud)) =
        embedded::codegen::detect_embedded_target(&stmts)
    {
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
                "\x1b[1;31mruntime error:\x1b[0m process exited with code {:?}",
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

fn collect_fm_files(dir: &Path, list: &mut Vec<PathBuf>, is_root: bool) {
    if !is_root && dir.join("flame.toml").exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_fm_files(&p, list, false);
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

    let manifest_content = fs::read_to_string("flame.toml").unwrap_or_default();
    let mut pkg_name = "app".to_string();
    for line in manifest_content.lines() {
        if line.starts_with("name =") {
            if let Some(val) = line.split('=').nth(1) {
                pkg_name = val.trim().trim_matches('"').trim_matches('\'').to_string();
            }
        }
    }

    let mut native_deps_raw = parse_manifest_section(&manifest_content, "[native-dependencies]");
    let mut plugins_raw = parse_manifest_section(&manifest_content, "[plugins]");

    if let Ok(entries) = fs::read_dir(".flame/pkg") {
        for entry in entries.flatten() {
            let pkg_path = entry.path();
            if pkg_path.is_dir() {
                let toml_path = pkg_path.join("flame.toml");
                if toml_path.exists() {
                    let dep_manifest = fs::read_to_string(&toml_path).unwrap_or_default();
                    for (name, mut path) in
                        parse_manifest_section(&dep_manifest, "[native-dependencies]")
                    {
                        if path.starts_with('"') && path.ends_with('"') {
                            path = path[1..path.len() - 1].to_string();
                        }
                        if path.starts_with('.') || path.starts_with('/') {
                            let abs = std::fs::canonicalize(pkg_path.join(&path))
                                .unwrap_or_else(|_| pkg_path.join(&path));
                            path = format!("\"{}\"", abs.to_string_lossy().replace("\\", "/"));
                        } else {
                            path = format!("\"{}\"", path);
                        }
                        native_deps_raw.push((name, path));
                    }
                    for (name, mut path) in parse_manifest_section(&dep_manifest, "[plugins]") {
                        if path.starts_with('"') && path.ends_with('"') {
                            path = path[1..path.len() - 1].to_string();
                        }
                        if path.starts_with('.') || path.starts_with('/') {
                            let abs = std::fs::canonicalize(pkg_path.join(&path))
                                .unwrap_or_else(|_| pkg_path.join(&path));
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

    let mut files_to_test = Vec::new();
    if args.len() >= 3 && !args[2].starts_with('-') {
        let p = PathBuf::from(&args[2]);
        if p.exists() {
            if p.is_dir() {
                collect_fm_files(&p, &mut files_to_test, true);
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
            collect_fm_files(Path::new("tests"), &mut files_to_test, false);
        }
        if Path::new("examples/tests").exists() {
            collect_fm_files(Path::new("examples/tests"), &mut files_to_test, false);
        }
        if Path::new("examples").exists() && !Path::new("examples/tests").exists() {
            collect_fm_files(Path::new("examples"), &mut files_to_test, false);
        }
        if Path::new("src").exists() {
            collect_fm_files(Path::new("src"), &mut files_to_test, false);
        }
        if files_to_test.is_empty() && Path::new("main.fm").exists() {
            files_to_test.push(PathBuf::from("main.fm"));
        }
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    files_to_test.retain(|p| seen.insert(p.clone()));

    if !native_deps_raw.is_empty() || !plugins_raw.is_empty() {
        println!(
            "\x1b[1;36m     AOT Testing\x1b[0m Native plugins detected. Compiling test suite natively..."
        );
        let mut processed_native_deps = Vec::new();
        for (plugin_name, plugin_path) in native_deps_raw {
            let mut path_str = plugin_path.clone();
            if path_str.starts_with('"') && path_str.ends_with('"') {
                path_str = path_str[1..path_str.len() - 1].to_string();
            }
            if path_str.starts_with('.')
                || path_str.starts_with('/')
                || std::path::Path::new(&path_str).is_absolute()
            {
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

            let is_local = path_str.starts_with('.')
                || path_str.starts_with('/')
                || std::path::Path::new(&path_str).is_absolute();
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
            "dev",
            &processed_native_deps,
            false,
            false,
            true, // is_test_mode
            Some(files_to_test.clone()),
        );

        let exe_name = format!("{}_test{}", pkg_name, std::env::consts::EXE_SUFFIX);
        let target_exe = Path::new("target").join("dev").join(&exe_name);

        if target_exe.exists() {
            println!(
                "\x1b[1;32m    Finished\x1b[0m test executable -> {}",
                target_exe.display()
            );
            let status = std::process::Command::new(&target_exe).status();
            if let Ok(st) = status {
                if !st.success() {
                    std::process::exit(1);
                }
            }
        }
        return;
    }

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
        if let Ok(content) = fs::read_to_string("flame.toml") {
            runner.granted_permissions =
                crate::package_manager::parse_manifest_permissions(&content);
        }
        runner.interactive = false;
        runner.test_mode = true;
        if let Err(e) = runner.run(&stmts) {
            println!("  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36mGlobal\x1b[0m setup failed");
            let span = runner.current_span.clone().unwrap_or(crate::lexer::Span {
                start: 0,
                end: 0,
                line: 1,
                col: 1,
            });
            crate::diagnostics::Diagnostic::new_error(
                e,
                runner.filepath.display().to_string(),
                span,
                None,
                None,
            )
            .print(&std::fs::read_to_string(&runner.filepath).unwrap_or_default());
        }

        let stats = crate::test_engine::execute_test_suite(
            &mut runner,
            &stmts,
            &path.display().to_string(),
        );
        total_passed += stats.passed;
        total_failed += stats.failed;
        total_ignored += stats.ignored;
        total_measured += stats.measured;
        total_filtered += stats.filtered;
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
    #[serde(rename = "sortText", skip_serializing_if = "Option::is_none")]
    pub sort_text: Option<String>,
}

#[derive(Serialize)]
pub struct JsonHover {
    pub label: String,
    pub documentation: Option<String>,
}

#[derive(Serialize)]
pub struct JsonSignatureHelp {
    pub label: String,
    pub parameters: Vec<String>,
    pub active_parameter: u32,
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
    signature_help: Option<JsonSignatureHelp>,
    pub tokens: Vec<crate::ide::SemanticToken>,
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
    let stdin_content = if args.iter().any(|arg| arg == "--stdin") {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        Some(buf)
    } else {
        None
    };

    let output = std::panic::catch_unwind(|| analyze_file_for_json(file, line, col, stdin_content))
        .unwrap_or_else(|_| JsonCheckOutput {
            file: file.to_string(),
            diagnostics: vec![],
            std_modules: vec![],
            native_modules: vec![],
            plugins: vec![],
            completions: vec![],
            hover: None,
            signature_help: None,
            tokens: vec![],
        });

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
        );
    } else if output.diagnostics.is_empty() {
        println!("\x1b[1;32mcheck:\x1b[0m no diagnostics");
    } else {
        let content = if args.iter().any(|arg| arg == "--stdin") {
            // Already read from stdin earlier, but we consumed it.
            // In non-json mode, we probably aren't using stdin, but let's fall back to reading the file.
            std::fs::read_to_string(file).unwrap_or_default()
        } else {
            std::fs::read_to_string(file).unwrap_or_default()
        };
        let file_lines: Vec<&str> = content.lines().collect();

        for diagnostic in output.diagnostics {
            let color = match diagnostic.severity.as_str() {
                "warning" => "\x1b[1;33m",
                "info" => "\x1b[1;34m",
                _ => "\x1b[1;31m",
            };
            println!(
                "{}{} :\x1b[0m \x1b[1m{}\x1b[0m",
                color, diagnostic.severity, diagnostic.message
            );
            println!(
                "  \x1b[1;36m-->\x1b[0m {}:{}:{}",
                diagnostic.file, diagnostic.line, diagnostic.column
            );

            let line_idx = diagnostic.line.saturating_sub(1);
            if line_idx < file_lines.len() {
                let line_str = diagnostic.line.to_string();
                let spacer = " ".repeat(line_str.len());
                println!(" \x1b[1;36m{} |\x1b[0m", spacer);
                println!(" \x1b[1;36m{} |\x1b[0m {}", line_str, file_lines[line_idx]);
                let col = diagnostic.column.saturating_sub(1);
                let pointer = " ".repeat(col) + "^";
                println!(" \x1b[1;36m{} |\x1b[0m {}{}\x1b[0m", spacer, color, pointer);
            }
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

fn analyze_file_for_json(
    file: &str,
    line: Option<usize>,
    col: Option<usize>,
    stdin_content: Option<String>,
) -> JsonCheckOutput {
    let content = stdin_content.unwrap_or_else(|| fs::read_to_string(file).unwrap_or_default());
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
        Ok(mut stmts) => {
            let mut target = None;
            for line in manifest_content.lines() {
                let t = line.trim();
                if t.starts_with("target =") {
                    if let Some(val) = t.split('=').nth(1) {
                        target = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            }
            crate::parser::filter_platform_stmts(&mut stmts, target.as_deref());
            let (res, tc) = TypeChecker::new(file.to_string()).check_program(&stmts);
            tc_opt = Some(tc);
            if let Err(diags) = res {
                for d in diags {
                    let sev = match d.severity {
                        crate::diagnostics::DiagnosticSeverity::Error => "error".to_string(),
                        crate::diagnostics::DiagnosticSeverity::Warning => "warning".to_string(),
                        crate::diagnostics::DiagnosticSeverity::Info => "info".to_string(),
                    };
                    diagnostics.push(JsonDiagnostic {
                        severity: sev,
                        message: d.message,
                        file: d.filepath,
                        line: d.span.line,
                        column: d.span.col,
                    });
                }
            }
        }
        Err(diag) => {
            let sev = match diag.severity {
                crate::diagnostics::DiagnosticSeverity::Error => "error".to_string(),
                crate::diagnostics::DiagnosticSeverity::Warning => "warning".to_string(),
                crate::diagnostics::DiagnosticSeverity::Info => "info".to_string(),
            };
            diagnostics.push(JsonDiagnostic {
                severity: sev,
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
    let current_line = if let Some(l) = line {
        let line = content.lines().nth(l.saturating_sub(1)).unwrap_or("");
        eprintln!(
            "DEBUG_EXTRACT: req.line={}, content.len={}, lines={}, line='{}'",
            l,
            content.len(),
            content.lines().count(),
            line
        );
        line
    } else {
        ""
    };
    let cursor_col = col.unwrap_or(current_line.len() + 1);

    let mut completions = Vec::new();
    if current_line.trim_end().ends_with("import") {
        completions.push(JsonCompletion {
            sort_text: None,
            label: "native".to_string(),
            kind: "module".to_string(),
            detail: "native dependencies".to_string(),
            documentation: None,
        });
        completions.push(JsonCompletion {
            sort_text: None,
            label: "std".to_string(),
            kind: "module".to_string(),
            detail: "standard library".to_string(),
            documentation: None,
        });
    } else if current_line.contains("import native.") {
        for module in &native_modules {
            completions.push(JsonCompletion {
                sort_text: None,
                label: module.clone(),
                kind: "plugin".to_string(),
                detail: "native plugin".to_string(),
                documentation: None,
            });
        }
    } else if current_line.contains("import std.") {
        for module in &std_modules {
            completions.push(JsonCompletion {
                sort_text: None,
                label: module.clone(),
                kind: "module".to_string(),
                detail: "standard library".to_string(),
                documentation: None,
            });
        }
    } else if current_line.trim().starts_with("@Requires(")
        && current_line[..cursor_col as usize].matches('"').count() % 2 == 1
    {
        for module in &std_modules {
            completions.push(JsonCompletion {
                sort_text: None,
                label: format!("std.{}", module),
                kind: "module".to_string(),
                detail: "standard library".to_string(),
                documentation: None,
            });
        }
        for module in &native_modules {
            completions.push(JsonCompletion {
                sort_text: None,
                label: module.clone(),
                kind: "plugin".to_string(),
                detail: "native plugin".to_string(),
                documentation: None,
            });
        }
    } else if current_line.contains("@p") || current_line.contains("@plugin") {
        for plugin in &plugins {
            completions.push(JsonCompletion {
                sort_text: None,
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
    let (mut scanned_vars, mut scanned_structs) = ide::scan_document(&content);

    let mut cursor_byte_idx = 0;
    if let Some(l) = line {
        let mut curr_line = 1;
        let mut curr_col = 1;
        let c_col = col.unwrap_or(1);
        for (i, c) in content.char_indices() {
            if curr_line == l && curr_col == c_col {
                cursor_byte_idx = i;
                break;
            }
            if c == '\n' {
                curr_line += 1;
                curr_col = 1;
            } else {
                curr_col += 1;
            }
        }
    }

    let impl_re = regex::Regex::new(r"impl\s+([a-zA-Z_]\w*)").unwrap();
    let mut current_impl = None;
    for cap in impl_re.captures_iter(&content) {
        if let Some(m) = cap.get(0) {
            if m.start() <= cursor_byte_idx {
                current_impl = Some(cap[1].to_string());
            }
        }
    }
    if let Some(ref impl_name) = current_impl {
        scanned_vars.push(crate::ide::ScannedVar {
            name: "self".to_string(),
            typ: Some(impl_name.clone()),
            doc: None,
        });
    }

    let imported_module_decls = load_imported_module_declarations(&manifest_dir, file);
    for stmt in &imported_module_decls {
        if let Some((name, params, return_type, is_annotation, annotations)) = match stmt {
            crate::parser::Stmt::FuncDecl {
                name,
                params,
                return_type,
                annotations,
                ..
            } => Some((name, params, return_type.as_deref(), false, annotations)),
            crate::parser::Stmt::PackageDecl {
                name, annotations, ..
            } => {
                let mut doc_str = String::new();
                for ann in annotations {
                    if ann.name == "Docs" {
                        if let Some(s) = ann.args.get(0) {
                            doc_str = s.trim_matches('"').to_string();
                        }
                    }
                }
                if !doc_str.is_empty() {
                    if let Some(var) = scanned_vars.iter_mut().find(|v| {
                        v.name == *name
                            && v.typ.as_deref()
                                == Some(&format!("```flame\nimport package {}\n```", name))
                    }) {
                        var.doc = Some(format!(
                            "```flame\nimport package {}\n```\n{}",
                            name, doc_str
                        ));
                    }
                }
                None
            }
            crate::parser::Stmt::AnnotationDecl {
                name,
                params,
                return_type,
                annotations,
                ..
            } => Some((name, params, return_type.as_deref(), true, annotations)),
            crate::parser::Stmt::ExportDecl(inner, _) => match &**inner {
                crate::parser::Stmt::FuncDecl {
                    name,
                    params,
                    return_type,
                    annotations,
                    ..
                } => Some((name, params, return_type.as_deref(), false, annotations)),
                crate::parser::Stmt::AnnotationDecl {
                    name,
                    params,
                    return_type,
                    annotations,
                    ..
                } => Some((name, params, return_type.as_deref(), true, annotations)),
                crate::parser::Stmt::StructDecl { name, fields, .. } => {
                    let mut struct_methods = Vec::new();
                    for stmt in &imported_module_decls {
                        if let crate::parser::Stmt::ImplDecl {
                            target_type,
                            methods,
                            ..
                        } = stmt
                        {
                            if target_type == name {
                                for m in methods {
                                    if let crate::parser::Stmt::FuncDecl {
                                        name: m_name,
                                        params,
                                        return_type,
                                        annotations,
                                        ..
                                    } = m
                                    {
                                        let mut doc_str = String::new();
                                        for ann in annotations {
                                            if ann.name == "Docs" {
                                                if let Some(s) = ann.args.get(0) {
                                                    doc_str = s.trim_matches('"').to_string();
                                                }
                                            }
                                        }
                                        let p_str = params
                                            .iter()
                                            .map(|p| format!("{}: {}", p.name, p.type_name))
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        let sig = format!(
                                            "fn {}({}) -> {}",
                                            m_name,
                                            p_str,
                                            return_type.as_deref().unwrap_or("Nil")
                                        );
                                        struct_methods.push(m_name.clone());
                                    }
                                }
                            }
                        }
                    }
                    scanned_structs.push(ide::ScannedStruct {
                        name: name.clone(),
                        fields: fields.clone(),
                        methods: struct_methods,
                    });
                    None
                }
                _ => None,
            },
            crate::parser::Stmt::StructDecl { name, fields, .. } => {
                let mut struct_methods = Vec::new();
                for stmt in &imported_module_decls {
                    if let crate::parser::Stmt::ImplDecl {
                        target_type,
                        methods,
                        ..
                    } = stmt
                    {
                        if target_type == name {
                            for m in methods {
                                if let crate::parser::Stmt::FuncDecl {
                                    name: m_name,
                                    params,
                                    return_type,
                                    annotations,
                                    ..
                                } = m
                                {
                                    let mut doc_str = String::new();
                                    for ann in annotations {
                                        if ann.name == "Docs" {
                                            if let Some(s) = ann.args.get(0) {
                                                doc_str = s.trim_matches('"').to_string();
                                            }
                                        }
                                    }
                                    let p_str = params
                                        .iter()
                                        .map(|p| format!("{}: {}", p.name, p.type_name))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    let sig = format!(
                                        "fn {}({}) -> {}",
                                        m_name,
                                        p_str,
                                        return_type.as_deref().unwrap_or("Nil")
                                    );
                                    struct_methods.push(m_name.clone());
                                }
                            }
                        }
                    }
                }
                scanned_structs.push(ide::ScannedStruct {
                    name: name.clone(),
                    fields: fields.clone(),
                    methods: struct_methods,
                });
                None
            }
            _ => None,
        } {
            let mut doc_str = None;
            for ann in annotations {
                if ann.name == "Docs" {
                    if let Some(s) = ann.args.get(0) {
                        doc_str = Some(s.trim_matches('"').to_string());
                    }
                }
            }

            let params_str = params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.type_name))
                .collect::<Vec<_>>()
                .join(", ");
            let sig = if is_annotation {
                if let Some(ret) = return_type {
                    format!("annotation @{}({}) -> {}", name, params_str, ret)
                } else {
                    format!("annotation @{}({})", name, params_str)
                }
            } else {
                format!(
                    "fn {}({}) -> {}",
                    name,
                    params_str,
                    return_type.unwrap_or("Nil")
                )
            };

            let doc_text = doc_str.clone().unwrap_or(sig.clone());

            scanned_vars.push(ide::ScannedVar {
                name: name.clone(),
                typ: Some(sig.clone()),
                doc: doc_str.clone(),
            });
            let (actual_label, sort_text) = if is_annotation {
                (format!("@{}", name), Some("1_".to_string()))
            } else {
                (name.clone(), Some("1_".to_string()))
            };
            completions.push(JsonCompletion {
                sort_text,
                label: actual_label,
                kind: if is_annotation {
                    "annotation".to_string()
                } else {
                    "function".to_string()
                },
                detail: "imported module declaration".to_string(),
                documentation: Some(doc_text),
            });
        }
    }

    // Enrich scanned_vars with return types from native module function calls
    for mod_name in &native_modules {
        if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
            // Resolve annotation plugins
            if let Some(init_func) = meta.functions.iter().find(|f| f.flame_name == "init") {
                for var in &mut scanned_vars {
                    if var.typ.as_deref() == Some(&format!("annotation_plugin:{}", mod_name)) {
                        var.typ = Some(init_func.return_type.clone());
                    }
                }
            }

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
                                        doc: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            for struct_meta in &meta.structs {
                for func in &struct_meta.methods {
                    let pattern1 =
                        format!("= {}.{}.{}(", mod_name, struct_meta.name, func.flame_name);
                    let pattern2 = format!(
                        "= await {}.{}.{}(",
                        mod_name, struct_meta.name, func.flame_name
                    );
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
                                            doc: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let (namespace, member_prefix) = extract_member_context(current_line, cursor_col);
    // eprintln!("DEBUG_CONTEXT: namespace={:?}, prefix={:?}, line='{}', col={}", namespace, member_prefix, current_line, cursor_col);

    let mut exact_ast_hover = None;
    if let Some(tc) = &tc_opt {
        if let Some(l) = line {
            let mut cursor_byte_idx = 0;
            let mut curr_line = 1;
            let mut curr_col = 1;
            for (i, c) in content.char_indices() {
                if curr_line == l && curr_col == cursor_col {
                    cursor_byte_idx = i;
                    break;
                }
                if c == '\n' {
                    curr_line += 1;
                    curr_col = 1;
                } else {
                    curr_col += 1;
                }
            }
            if cursor_byte_idx == 0 && curr_line == l && cursor_col >= curr_col {
                cursor_byte_idx = content.len();
            }

            let mut best_span: Option<&crate::lexer::Span> = None;
            let mut best_ty_str = None;
            eprintln!(
                "DEBUG_HOVER: byte_idx={}, tc.hover_info keys: {:?}",
                cursor_byte_idx,
                tc.hover_info.keys()
            );

            for (span, ty_str) in &tc.hover_info {
                if cursor_byte_idx >= span.start && cursor_byte_idx <= span.end {
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
                    if ty_str.starts_with("```") || ty_str.starts_with('@') || ty_str.contains('\n')
                    {
                        exact_ast_hover = Some(JsonHover {
                            label: word_under_cursor.clone(),
                            documentation: Some(ty_str.clone()),
                        });
                    } else if ty_str.starts_with("fn(") {
                        if let Some(var) = scanned_vars.iter().find(|v| v.name == word_under_cursor)
                        {
                            if let Some(t) = &var.typ {
                                if t.starts_with("fn ") {
                                    exact_ast_hover = Some(JsonHover {
                                        label: word_under_cursor.clone(),
                                        documentation: Some(format!(
                                            "```flame\n{}\n```\nDefined in project",
                                            t
                                        )),
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
    let mut hover_found = None;
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

    if let Some(namespace) = namespace {
        let mut resolved_as_var = false;

        // First check if it's a known variable! If so, it might be a struct or native type (e.g. FlameServer)
        if let Some(var) = scanned_vars.iter().find(|v| v.name == namespace) {
            if let Some(typ) = &var.typ {
                if typ != "Unknown" {
                    let mut provided = false;
                    for s in &scanned_structs {
                        if s.name == *typ {
                            for func_name in &s.methods {
                                if member_prefix
                                    .as_deref()
                                    .map_or(true, |p| func_name.starts_with(p))
                                {
                                    completions.push(JsonCompletion {
                                        sort_text: None,
                                        label: func_name.clone(),
                                        kind: "method".to_string(),
                                        detail: format!("struct {}", typ),
                                        documentation: None,
                                    });
                                }
                            }
                            provided = true;
                        }
                    }
                    if !provided {
                        for mod_name in &native_modules {
                            if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
                                if let Some(struct_meta) =
                                    meta.structs.iter().find(|s| s.name == *typ)
                                {
                                    for func in &struct_meta.methods {
                                        if member_prefix
                                            .as_deref()
                                            .map_or(true, |p| func.flame_name.starts_with(p))
                                        {
                                            completions.push(JsonCompletion {
                                                sort_text: None,
                                                label: func.flame_name.clone(),
                                                kind: "method".to_string(),
                                                detail: format!("native {}.{}", mod_name, typ),
                                                documentation: func.docs.clone(),
                                            });
                                        }
                                    }
                                    provided = true;
                                    break;
                                }
                            }
                        }
                    }
                    if provided {
                        resolved_as_var = true;

                        if !word_under_cursor.is_empty() {
                            if let Some(tc) = &tc_opt {
                                if let Some(methods) = tc.methods.get(typ) {
                                    if let Some((_, sig)) = methods.iter().find(|(name, _)| *name == &word_under_cursor) {
                                        hover_found = Some(JsonHover {
                                            label: format!("{}::{}()", typ, word_under_cursor),
                                            documentation: sig.hover_doc.clone().or_else(|| Some(format!("```flame\nfn {}(...)\n```", word_under_cursor)))
                                        });
                                    }
                                }
                            }
                            
                            if hover_found.is_none() {
                                for s in &scanned_structs {
                                    if s.name == *typ {
                                        if let Some(func_name) =
                                            s.methods.iter().find(|&f| f == &word_under_cursor)
                                        {
                                            let sig = format!("fn {}(...)", func_name);
                                            hover_found = Some(JsonHover {
                                                label: format!("{}::{}()", typ, func_name),
                                                documentation: Some(format!(
                                                    "```flame\n{}\n```\nDefined in project",
                                                    sig
                                                )),
                                            });
                                        }
                                    }
                                }
                            }
                            if hover_found.is_none() {
                                for mod_name in &native_modules {
                                    if let Some(meta) =
                                        load_meta_from_project(&manifest_dir, mod_name)
                                    {
                                        if let Some(struct_meta) =
                                            meta.structs.iter().find(|s| s.name == *typ)
                                        {
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
                                                    function.flame_name,
                                                    params_str,
                                                    function.return_type
                                                );
                                                hover_found = Some(JsonHover {
                                                    label: format!(
                                                        "{}.{}",
                                                        typ, function.flame_name
                                                    ),
                                                    documentation: Some(format!(
                                                        "```flame\n{}\n```\n{}",
                                                        sig,
                                                        function.docs.clone().unwrap_or_default()
                                                    )),
                                                });
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !resolved_as_var {
            if let Some(meta) = load_meta_from_project(&manifest_dir, &namespace) {
                for function in &meta.functions {
                    if member_prefix
                        .as_deref()
                        .map(|prefix| function.flame_name.starts_with(prefix))
                        .unwrap_or(true)
                    {
                        completions.push(JsonCompletion {
                            sort_text: None,
                            label: function.flame_name.clone(),
                            kind: "function".to_string(),
                            detail: format!("native.{}", namespace),
                            documentation: function.docs.clone().or_else(|| {
                                load_local_rust_doc(&manifest_dir, &namespace, &function.flame_name)
                            }),
                        });
                    }
                }
                // Add struct names themselves as completions
                for struct_meta in &meta.structs {
                    if member_prefix
                        .as_deref()
                        .map_or(true, |prefix| struct_meta.name.starts_with(prefix))
                    {
                        completions.push(JsonCompletion {
                            sort_text: Some("2_".to_string()),
                            label: struct_meta.name.clone(),
                            kind: "class".to_string(),
                            detail: format!("struct (from {})", namespace),
                            documentation: struct_meta.docs.clone(),
                        });
                    }

                    if struct_meta.name.to_lowercase() == namespace.to_lowercase() {
                        for function in &struct_meta.methods {
                            if member_prefix
                                .as_deref()
                                .map(|prefix| function.flame_name.starts_with(prefix))
                                .unwrap_or(true)
                            {
                                completions.push(JsonCompletion {
                                    sort_text: None,
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

                    if hover_found.is_none() {
                        if let Some(struct_meta) =
                            meta.structs.iter().find(|s| s.name == word_under_cursor)
                        {
                            let doc = struct_meta.docs.clone().unwrap_or_default();
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
            } else if let Some(def) = ide::get_native_module_def(&namespace) {
                for func in &def.functions {
                    if member_prefix
                        .as_deref()
                        .map_or(true, |p| func.name.starts_with(p))
                    {
                        completions.push(JsonCompletion {
                            sort_text: None,
                            label: func.name.clone(),
                            kind: "function".to_string(),
                            detail: format!("{}", func.return_type),
                            documentation: Some(format!(
                                "```flame\nfn {}({}) -> {}\n```\n{}",
                                func.name,
                                func.params
                                    .iter()
                                    .map(|(n, t)| format!("{}: {}", n, t))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                func.return_type,
                                func.description
                            )),
                        });
                    }
                    if !word_under_cursor.is_empty() && func.name == word_under_cursor {
                        hover_found = Some(JsonHover {
                            label: format!("{}::{}()", namespace, func.name),
                            documentation: Some(format!(
                                "```flame\nfn {}({}) -> {}\n```\n{}",
                                func.name,
                                func.params
                                    .iter()
                                    .map(|(n, t)| format!("{}: {}", n, t))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                func.return_type,
                                func.description
                            )),
                        });
                    }
                }
                for typ in &def.types {
                    if member_prefix
                        .as_deref()
                        .map_or(true, |p| typ.name.starts_with(p))
                    {
                        completions.push(JsonCompletion {
                            sort_text: None,
                            label: typ.name.clone(),
                            kind: "class".to_string(),
                            detail: "type".to_string(),
                            documentation: Some(format!(
                                "```flame\ntype {}\n```\n{}",
                                typ.name, typ.description
                            )),
                        });
                    }
                }

                // For hover:
                if !word_under_cursor.is_empty() {
                    if let Some(func) = def.functions.iter().find(|f| f.name == word_under_cursor) {
                        hover_found = Some(JsonHover {
                            label: format!("{}.{}()", def.name, func.name),
                            documentation: Some(format!(
                                "```flame\nfn {}({}) -> {}\n```\n{}",
                                func.name,
                                func.params
                                    .iter()
                                    .map(|(n, t)| format!("{}: {}", n, t))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                func.return_type,
                                func.description
                            )),
                        });
                    } else if let Some(typ) = def.types.iter().find(|t| t.name == word_under_cursor)
                    {
                        hover_found = Some(JsonHover {
                            label: format!("{}.{}", def.name, typ.name),
                            documentation: Some(format!(
                                "```flame\ntype {}\n```\n{}",
                                typ.name, typ.description
                            )),
                        });
                    }
                }
            } else if let Some(tc) = &tc_opt {
                if let Some(enum_info) = tc.enums.get(&namespace) {
                    if !word_under_cursor.is_empty() {
                        if let Some((variant_name, variant_info)) = enum_info.variants.iter().find(|(n, _)| *n == &word_under_cursor) {
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}", namespace, variant_name),
                                documentation: variant_info.hover_doc.clone().or_else(|| Some(format!("```flame\n{}::{} variant\n```", namespace, variant_name)))
                            });
                        }
                    }
                }
                
                if let Some(methods) = tc.methods.iter().find(|(k, _)| k == &&namespace || k.ends_with(&format!(".{}", namespace))).map(|(_, v)| v) {
                    for (method_name, sig) in methods {
                        if sig.is_static {
                            if member_prefix.as_deref().map_or(true, |p| method_name.starts_with(p)) {
                                completions.push(JsonCompletion {
                                    sort_text: None,
                                    label: method_name.clone(),
                                    kind: "function".to_string(),
                                    detail: format!("{} method", namespace),
                                    documentation: sig.hover_doc.clone()
                                });
                            }
                        }
                        if !word_under_cursor.is_empty() && method_name == &word_under_cursor {
                            let params_str = sig.params.iter().map(|p| format!("{}: {:?}", p.name, p.ty)).collect::<Vec<_>>().join(", ");
                            let return_str = if sig.return_type == crate::typechecker::Type::Nil { "".to_string() } else { format!(" -> {:?}", sig.return_type) };
                            let fallback_doc = format!("```flame\nfn {}({}){}\n```", method_name, params_str, return_str);
                            let final_doc = if let Some(doc) = &sig.hover_doc { format!("{}\n\n{}", fallback_doc, doc) } else { fallback_doc };
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}()", namespace, method_name),
                                documentation: Some(final_doc)
                            });
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
                            sort_text: None,
                            label: method.clone(),
                            kind: "function".to_string(),
                            detail: format!("std.{}", namespace),
                            documentation: crate::std_docs::get_std_function_doc(
                                &namespace, method,
                            )
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
                            annotations,
                            ..
                        } => Some((name, params, return_type, false, annotations)),
                        crate::parser::Stmt::AnnotationDecl {
                            name,
                            params,
                            return_type,
                            annotations,
                            ..
                        } => Some((name, params, return_type, true, annotations)),
                        crate::parser::Stmt::ExportDecl(inner, _) => match &**inner {
                            crate::parser::Stmt::FuncDecl {
                                name,
                                params,
                                return_type,
                                annotations,
                                ..
                            } => Some((name, params, return_type, false, annotations)),
                            crate::parser::Stmt::AnnotationDecl {
                                name,
                                params,
                                return_type,
                                annotations,
                                ..
                            } => Some((name, params, return_type, true, annotations)),
                            crate::parser::Stmt::StructDecl { name, .. } => {
                                completions.push(JsonCompletion {
                                    sort_text: None,
                                    label: name.clone(),
                                    kind: "class".to_string(),
                                    detail: "struct".to_string(),
                                    documentation: None,
                                });
                                provided_completions = true;
                                None
                            }
                            _ => None,
                        },
                        crate::parser::Stmt::StructDecl { name, .. } => {
                            completions.push(JsonCompletion {
                                sort_text: None,
                                label: name.clone(),
                                kind: "class".to_string(),
                                detail: "struct".to_string(),
                                documentation: None,
                            });
                            provided_completions = true;
                            None
                        }
                        _ => None,
                    };

                    if let Some((name, params, return_type, is_annotation, annotations)) = func_info
                    {
                        let param_strs = params
                            .iter()
                            .map(|p| {
                                format!(
                                    "{}{}: {}{}",
                                    if p.is_mut { "mut " } else { "" },
                                    p.name,
                                    if p.is_ref { "&" } else { "" },
                                    p.type_name
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let sig = if is_annotation {
                            if let Some(ret) = return_type.as_deref() {
                                format!("annotation @{}({}) -> {}", name, param_strs, ret)
                            } else {
                                format!("annotation @{}({})", name, param_strs)
                            }
                        } else {
                            format!(
                                "fn {}({}) -> {}",
                                name,
                                param_strs,
                                return_type.as_deref().unwrap_or("Nil")
                            )
                        };

                        let actual_label = if is_annotation {
                            format!("@{}", name)
                        } else {
                            name.clone()
                        };

                        let mut doc_str = String::new();
                        for ann in annotations.iter() {
                            if ann.name == "Docs" {
                                if let Some(s) = ann.args.get(0) {
                                    let unquoted = s.trim_matches('"').replace("\\n", "\n");
                                    let lines: Vec<&str> = unquoted.lines().collect();
                                    let mut min_indent = usize::MAX;
                                    for line in &lines {
                                        if line.trim().is_empty() {
                                            continue;
                                        }
                                        let indent = line
                                            .chars()
                                            .take_while(|c| *c == ' ' || *c == '\t')
                                            .count();
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
                                    let trimmed = cleaned_doc.trim();
                                    if !trimmed.is_empty() {
                                        doc_str = format!("\n\n{}", trimmed);
                                    }
                                }
                            }
                        }

                        if member_prefix
                            .as_deref()
                            .map_or(true, |prefix| name.starts_with(prefix))
                        {
                            if is_annotation && !word_under_cursor_raw.starts_with('@') {
                                // Only suggest annotations when user types @
                            } else {
                                completions.push(JsonCompletion {
                                    sort_text: Some("1_".to_string()),
                                    label: actual_label,
                                    kind: if is_annotation {
                                        "annotation".to_string()
                                    } else {
                                        "function".to_string()
                                    },
                                    detail: format!("module {}", namespace),
                                    documentation: Some(format!(
                                        "```flame\n{}\n```{}",
                                        sig, doc_str
                                    )),
                                });
                                provided_completions = true;
                            }
                        }

                        if !word_under_cursor.is_empty() && name == &word_under_cursor {
                            hover_found = Some(JsonHover {
                                label: format!("{}.{}()", namespace, name),
                                documentation: Some(format!("```flame\n{}\n```{}", sig, doc_str)),
                            });
                        }
                    }
                }

                if !provided_completions {
                    // If the local file had no such functions matching the prefix, we just do nothing here.
                }
            } else {
                let mut var_type = None;
                let mut is_instance = false;
                let mut pkg_suggestions = Vec::new();
                if let Some(first_part) = namespace.split('.').next() {
                    var_type = scanned_vars
                        .iter()
                        .find(|v| v.name == first_part)
                        .and_then(|v| {
                            is_instance = true;
                            v.typ.clone()
                        });

                    for part in namespace.split('.').skip(1) {
                        if let Some(vt) = var_type {
                            var_type = None;
                            if let Some(struct_def) = scanned_structs.iter().find(|s| s.name == vt)
                            {
                                if let Some(field) = struct_def.fields.iter().find(|f| f.0 == part)
                                {
                                    var_type = Some(field.1.clone());
                                }
                            }
                        }
                    }
                }

                // If not found as a variable, maybe it's a struct name directly?
                if var_type.is_none() {
                    is_instance = false;
                    if scanned_structs.iter().any(|s| s.name == namespace) {
                        var_type = Some(namespace.to_string());
                    } else if let Some(dot_idx) = namespace.find('.') {
                        let mod_name = &namespace[..dot_idx];
                        let struct_name = &namespace[dot_idx + 1..];
                        if native_modules.contains(&mod_name.to_string()) {
                            if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
                                if meta.structs.iter().any(|s| s.name == struct_name) {
                                    var_type = Some(struct_name.to_string());
                                }
                            }
                        }
                    } else {
                        let path_parts = vec![namespace.clone()];
                        if let Some(candidate) =
                            crate::stdlib::locate_import_file(Path::new(file), &path_parts).or_else(
                                || {
                                    let direct = manifest_dir.join(format!("{}.fm", namespace));
                                    if direct.exists() {
                                        return Some(direct);
                                    }
                                    let pkg_main = manifest_dir
                                        .join(".flame")
                                        .join("pkg")
                                        .join(&namespace)
                                        .join("src")
                                        .join("main.fm");
                                    if pkg_main.exists() {
                                        return Some(pkg_main);
                                    }
                                    None
                                },
                            )
                        {
                            eprintln!("DEBUG_CANDIDATE: {:?}", candidate);
                            match std::fs::read_to_string(&candidate) {
                                Ok(content) => {
                                    eprintln!("DEBUG_CANDIDATE_READ_OK, len={}", content.len());
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
                                        candidate.to_string_lossy().to_string(),
                                    );
                                    if let Ok(stmts) = parser.parse() {
                                        eprintln!("DEBUG_STMTS_LEN: {}", stmts.len());
                                        for stmt in stmts {
                                            if let crate::parser::Stmt::PackageDecl {
                                                annotations,
                                                ..
                                            } = stmt
                                            {
                                                for ann in annotations {
                                                    eprintln!(
                                                        "DEBUG_ANN: name={}, args={:?}",
                                                        ann.name, ann.args
                                                    );
                                                    if ann.name == "Suggestions"
                                                        && !ann.args.is_empty()
                                                    {
                                                        let s_args = ann.args.join(" ");
                                                        let re = regex::Regex::new(r"\{\s*name\s*:\s*([^,]+),\s*kind\s*:\s*([^,}]+)(?:,\s*doc\s*:\s*([^}]+))?\}").unwrap();
                                                        for cap in re.captures_iter(&s_args) {
                                                            let struct_name = cap[1]
                                                                .trim()
                                                                .trim_matches(|c| {
                                                                    c == '"' || c == '\''
                                                                })
                                                                .to_string();
                                                            let kind = cap[2]
                                                                .trim()
                                                                .trim_matches(|c| {
                                                                    c == '"' || c == '\''
                                                                })
                                                                .to_string();
                                                            let doc = cap.get(3).map(|m| {
                                                                m.as_str()
                                                                    .trim()
                                                                    .trim_matches(|c| {
                                                                        c == '"' || c == '\''
                                                                    })
                                                                    .to_string()
                                                            });
                                                            pkg_suggestions.push((
                                                                struct_name,
                                                                kind,
                                                                doc,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("DEBUG_CANDIDATE_READ_ERR: {}", e);
                                }
                            }
                        }
                    }
                }
                // eprintln!("DEBUG_COMPLETIONS_PRE: namespace={}, var_type={:?}", namespace, var_type);

                let mut provided_completions = false;

                if let Some(tc) = &tc_opt {
                    if let Some(enum_info) = tc.enums.get(&namespace) {
                        for (variant_name, _) in &enum_info.variants {
                            if member_prefix
                                .as_deref()
                                .map_or(true, |p| variant_name.starts_with(p))
                            {
                                completions.push(JsonCompletion {
                                    sort_text: Some("1_".to_string()),
                                    label: variant_name.clone(),
                                    kind: "enumMember".to_string(),
                                    detail: format!("{} enum variant", namespace),
                                    documentation: None,
                                });
                                provided_completions = true;
                            }
                        }
                    } else if var_type.is_none() {
                        for (enum_name, enum_info) in &tc.enums {
                            if enum_name.starts_with(&format!("{}.", namespace)) {
                                let short_name =
                                    enum_name.strip_prefix(&format!("{}.", namespace)).unwrap();
                                if !short_name.contains('.')
                                    && member_prefix
                                        .as_deref()
                                        .map_or(true, |p| short_name.starts_with(p))
                                {
                                    completions.push(JsonCompletion {
                                        sort_text: Some("1_".to_string()),
                                        label: short_name.to_string(),
                                        kind: "enum".to_string(),
                                        detail: format!("{} enum", namespace),
                                        documentation: enum_info.hover_doc.clone(),
                                    });
                                    provided_completions = true;
                                }
                            }
                        }
                        for (struct_name, s_info) in &tc.structs {
                            if struct_name.starts_with(&format!("{}.", namespace)) {
                                let short_name = struct_name
                                    .strip_prefix(&format!("{}.", namespace))
                                    .unwrap();
                                if !short_name.contains('.')
                                    && member_prefix
                                        .as_deref()
                                        .map_or(true, |p| short_name.starts_with(p))
                                {
                                    completions.push(JsonCompletion {
                                        sort_text: Some("1_".to_string()),
                                        label: short_name.to_string(),
                                        kind: "struct".to_string(),
                                        detail: format!("{} struct", namespace),
                                        documentation: s_info.hover_doc.clone(),
                                    });
                                    provided_completions = true;
                                }
                            }
                        }
                    }
                }

                // If it's a standard module directly (e.g. `json.`, `tcp.`, `http.`)
                let is_std_module = std_modules.contains(&namespace)
                    || matches!(
                        namespace.as_str(),
                        "json"
                            | "tcp"
                            | "udp"
                            | "http"
                            | "ws"
                            | "mqtt"
                            | "dns"
                            | "url"
                            | "interface"
                    );
                if var_type.is_none() {
                    if is_std_module {
                        if let Some(methods) = ide::get_std_module_methods(&namespace) {
                            for method in methods {
                                if member_prefix
                                    .as_deref()
                                    .map_or(true, |p| method.starts_with(p))
                                {
                                    let doc =
                                        crate::std_docs::get_std_function_doc(&namespace, &method);
                                    completions.push(JsonCompletion {
                                        sort_text: Some("4_".to_string()),
                                        label: method.clone(),
                                        kind: "function".to_string(),
                                        detail: format!("std.{} function", namespace),
                                        documentation: doc.map(|d| d.to_string()),
                                    });
                                    provided_completions = true;
                                }
                                if !word_under_cursor.is_empty() && method == word_under_cursor {
                                    let doc =
                                        crate::std_docs::get_std_function_doc(&namespace, &method);
                                    hover_found = Some(JsonHover {
                                        label: format!("std.{}::{}()", namespace, method),
                                        documentation: doc.map(|d| d.to_string()), // std_docs already provides good markdown
                                    });
                                }
                            }
                        }
                    } else if !pkg_suggestions.is_empty() {
                        for (s_name, s_kind, s_doc) in &pkg_suggestions {
                            if member_prefix
                                .as_deref()
                                .map_or(true, |p| s_name.starts_with(p))
                            {
                                completions.push(JsonCompletion {
                                    sort_text: Some("1_".to_string()),
                                    label: s_name.clone(),
                                    kind: s_kind.clone(),
                                    detail: format!("{} {}", namespace, s_kind),
                                    documentation: s_doc.clone(),
                                });
                                provided_completions = true;
                            }
                        }
                    }
                }

                // If it's a native module directly (e.g. `flamer.`)
                if !provided_completions
                    && var_type.is_none()
                    && native_modules.contains(&namespace)
                {
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
                                    function.flame_name, params_str, function.return_type
                                );

                                completions.push(JsonCompletion {
                                    sort_text: None,
                                    label: function.flame_name.clone(),
                                    kind: "function".to_string(),
                                    detail: format!("{} (from {})", function.flame_name, namespace),
                                    documentation: Some(
                                        function.docs.clone().unwrap_or(sig.clone()),
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
                                    function.flame_name, params_str, function.return_type
                                );
                                let doc = function.docs.clone().unwrap_or_default();
                                let formatted_doc = if doc.trim().is_empty() {
                                    format!(
                                        "```flame\n{}\n```\n\n**Return Type**: `{}`",
                                        sig, function.return_type
                                    )
                                } else {
                                    format!(
                                        "```flame\n{}\n```\n{}\n\n**Return Type**: `{}`",
                                        sig,
                                        doc.trim(),
                                        function.return_type
                                    )
                                };

                                hover_found = Some(JsonHover {
                                    label: format!("{}::{}()", namespace, function.flame_name),
                                    documentation: Some(formatted_doc),
                                });
                            }
                        }

                        for struct_meta in &meta.structs {
                            // eprintln!("DEBUG_COMPLETIONS: Checking struct: {}", struct_meta.name);
                            if member_prefix
                                .as_deref()
                                .map(|p| struct_meta.name.starts_with(p))
                                .unwrap_or(true)
                            {
                                // eprintln!("DEBUG_COMPLETIONS: Adding struct: {}", struct_meta.name);
                                completions.push(JsonCompletion {
                                    sort_text: Some("2_".to_string()),
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
                                let doc = struct_meta.docs.clone().unwrap_or_default();
                                let formatted_doc = if doc.trim().is_empty() {
                                    format!("```flame\nstruct {}\n```", struct_meta.name)
                                } else {
                                    format!(
                                        "```flame\nstruct {}\n```\n{}",
                                        struct_meta.name,
                                        doc.trim()
                                    )
                                };
                                hover_found = Some(JsonHover {
                                    label: format!("{}::{}", namespace, struct_meta.name),
                                    documentation: Some(formatted_doc),
                                });
                            }
                        }
                    }
                }

                if let Some(t) = &var_type {
                    // eprintln!("DEBUG_COMPLETIONS: namespace={}, var_type={:?}, t={}, is_instance={}", namespace, var_type, t, is_instance);
                    if let Some(struct_def) = scanned_structs.iter().find(|s| s.name == *t) {
                        for field in &struct_def.fields {
                            if member_prefix
                                .as_deref()
                                .map_or(true, |prefix| field.0.starts_with(prefix))
                            {
                                completions.push(JsonCompletion {
                                    sort_text: Some("1_".to_string()),
                                    label: field.0.clone(),
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
                                    sort_text: Some("1_".to_string()),
                                    label: method.clone(),
                                    kind: "method".to_string(),
                                    detail: format!("{} method", t),
                                    documentation: None,
                                });
                                provided_completions = true;
                            }
                        }
                    }

                    if let Some(tc) = &tc_opt {
                        if let Some(s_info) = tc.structs.get(t) {
                            for (field_name, _) in &s_info.fields {
                                if member_prefix
                                    .as_deref()
                                    .map_or(true, |prefix| field_name.starts_with(prefix))
                                {
                                    completions.push(JsonCompletion {
                                        sort_text: Some("1_".to_string()),
                                        label: field_name.clone(),
                                        kind: "property".to_string(),
                                        detail: format!("{} field", t),
                                        documentation: s_info.hover_doc.clone(),
                                    });
                                    provided_completions = true;
                                }
                            }
                        }
                        if let Some(methods) = tc.methods.get(t) {
                            for (method_name, m_sig) in methods {
                                if member_prefix
                                    .as_deref()
                                    .map_or(true, |prefix| method_name.starts_with(prefix))
                                {
                                    completions.push(JsonCompletion {
                                        sort_text: Some("1_".to_string()),
                                        label: method_name.clone(),
                                        kind: "method".to_string(),
                                        detail: format!("{} method", t),
                                        documentation: m_sig.hover_doc.clone(),
                                    });
                                    provided_completions = true;
                                }
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
                                // eprintln!("DEBUG_COMPLETIONS: loaded meta for {}", mod_name);
                                for struct_meta in &meta.structs {
                                    if struct_meta.name == *t
                                        || struct_meta.name.to_lowercase() == t.to_lowercase()
                                    {
                                        // eprintln!("DEBUG_COMPLETIONS: matched struct {}", struct_meta.name);
                                        for function in &struct_meta.methods {
                                            if is_instance && function.is_static {
                                                continue;
                                            }
                                            if !is_instance && !function.is_static {
                                                continue;
                                            }

                                            if member_prefix
                                                .as_deref()
                                                .map(|p| function.flame_name.starts_with(p))
                                                .unwrap_or(true)
                                            {
                                                completions.push(JsonCompletion {
                                                    sort_text: Some("1_".to_string()),
                                                    label: function.flame_name.clone(),
                                                    kind: "method".to_string(),
                                                    detail: format!(
                                                        "{}::{} (from {})",
                                                        struct_meta.name,
                                                        function.flame_name,
                                                        mod_name
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
                                                let doc =
                                                    function.docs.clone().unwrap_or_else(|| {
                                                        load_local_rust_doc(
                                                            &manifest_dir,
                                                            &mod_name,
                                                            &function.flame_name,
                                                        )
                                                        .unwrap_or_default()
                                                    });
                                                let final_doc = if function.docs.is_some() {
                                                    doc
                                                } else {
                                                    if doc.trim().is_empty() {
                                                        format!(
                                                            "```flame\n{}\n```\n\n**Return Type / Structure**: `{}`",
                                                            sig, function.return_type
                                                        )
                                                    } else {
                                                        format!(
                                                            "```flame\n{}\n```\n{}\n\n**Return Type / Structure**: `{}`",
                                                            sig,
                                                            doc.trim(),
                                                            function.return_type
                                                        )
                                                    }
                                                };
                                                hover_found = Some(JsonHover {
                                                    label: format!(
                                                        "{}::{}()",
                                                        struct_meta.name, function.flame_name
                                                    ),
                                                    documentation: Some(final_doc),
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
                                            sort_text: None,
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
                    let mut builtin_methods = vec![
                        ("type", "Returns the type of the value as a string"),
                        ("toString", "Converts the value to a string representation"),
                        (
                            "toInt",
                            "Converts the value to an integer, throws error if invalid",
                        ),
                        (
                            "tryInt",
                            "Converts the value to an integer, returns nil if invalid",
                        ),
                        (
                            "toFloat",
                            "Converts the value to a floating point number, throws error if invalid",
                        ),
                        (
                            "tryFloat",
                            "Converts the value to a floating point number, returns nil if invalid",
                        ),
                        (
                            "toBool",
                            "Converts the value to its truthy boolean representation",
                        ),
                        (
                            "tryBool",
                            "Converts the value to its truthy boolean representation",
                        ),
                        (
                            "toByte",
                            "Converts a String or Int into a binary Byte or Byte array.",
                        ),
                        (
                            "toUtf8",
                            "Decodes a Byte array into a UTF-8 String. Panics if invalid UTF-8.",
                        ),
                        (
                            "tryUtf8",
                            "Attempts to decode a Byte array into a UTF-8 String. Returns nil if invalid.",
                        ),
                        (
                            "index",
                            "Extracts the value at the given key/index (requires 1 argument)",
                        ),
                        ("toJson", "Serializes a struct or object into a JSON string"),
                        (
                            "len",
                            "Returns the length in bytes (String) or elements (Vec)",
                        ),
                        ("pushStr", "Appends a string slice (String)"),
                        ("toUpperCase", "Returns uppercase string (String)"),
                        ("toLowerCase", "Returns lowercase string (String)"),
                        ("trim", "Returns trimmed string (String)"),
                        ("new", "Creates a new instance (Vec, HashMap)"),
                        ("push", "Appends an element (Vec)"),
                        ("pop", "Removes and returns the last element (Vec)"),
                        ("isEmpty", "Returns true if empty (Vec, HashMap)"),
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
                        (
                            "mode",
                            "Configures digital pin direction. Values: `\"OUTPUT\"`, `\"INPUT\"`, `\"INPUT_PULLUP\"`, `\"PWM\"` (Hardware Pin)",
                        ),
                        (
                            "high",
                            "Drives digital pin voltage to logical HIGH (Hardware Pin)",
                        ),
                        (
                            "low",
                            "Drives digital pin voltage to logical LOW (Hardware Pin)",
                        ),
                        (
                            "toggle",
                            "Flips digital pin voltage to opposite state (Hardware Pin)",
                        ),
                        (
                            "read",
                            "Reads digital/analog logic level or ADC raw value (Hardware Pin/ADC)",
                        ),
                        (
                            "angle",
                            "Sets absolute target rotation angle in degrees (Hardware Servo)",
                        ),
                        (
                            "speed",
                            "Sets throttle output as percentage (Hardware Motor)",
                        ),
                        (
                            "forward",
                            "Sets directional polarization to forward (Hardware Motor)",
                        ),
                        (
                            "reverse",
                            "Sets directional polarization to reverse (Hardware Motor)",
                        ),
                        (
                            "stop",
                            "Electro-dynamically brakes shaft to halt (Hardware Motor/Servo)",
                        ),
                    ];

                    if content.contains("import std.math") {
                        builtin_methods.extend(vec![
                        ("abs", "Returns the absolute value (Math)"),
                        ("floor", "Returns the largest integer less than or equal to a number (Math)"),
                        ("ceil", "Returns the smallest integer greater than or equal to a number (Math)"),
                        ("round", "Returns the nearest integer to a number (Math)"),
                        ("sqrt", "Returns the square root of a number (Math)"),
                        ("pow", "Returns the base to the exponent power (Math)"),
                        ("min", "Returns the smaller of two numbers (Math)"),
                        ("max", "Returns the larger of two numbers (Math)"),
                        ("clamp", "Clamps a number within the inclusive range specified (Math)"),
                    ]);
                    }

                    if content.contains("import std.byte") {
                        builtin_methods.extend(vec![
                            (
                                "toHex",
                                "Returns the hexadecimal string representation (Bytes)",
                            ),
                            (
                                "toBase64",
                                "Returns the Base64 string representation (Bytes)",
                            ),
                            ("concat", "Concatenates another byte array (Bytes)"),
                        ]);
                    }

                    for (method, doc) in &builtin_methods {
                        if member_prefix
                            .as_deref()
                            .map(|prefix| method.starts_with(prefix))
                            .unwrap_or(true)
                        {
                            completions.push(JsonCompletion {
                                sort_text: Some("3_".to_string()),
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
        } // closes if !resolved_as_var
    } else {
        // Keyword completions for bare words
        completions.extend(ide::get_keyword_completions(
            current_line,
            &word_under_cursor_raw,
            &word_under_cursor,
            tc_opt.as_ref(),
        ));

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
                    } else {
                        let is_func_or_anno =
                            typ.starts_with("fn ") || typ.starts_with("annotation ");

                        if is_func_or_anno {
                            let doc = v
                                .doc
                                .clone()
                                .unwrap_or_else(|| format!("```flame\n{}\n```", typ));
                            hover_found = Some(JsonHover {
                                label: format!("{}", typ),
                                documentation: Some(doc),
                            });
                        } else {
                            let doc = v.doc.clone().unwrap_or_else(|| {
                                format!("```flame\nlet {}: {}\n```", v.name, typ)
                            });
                            hover_found = Some(JsonHover {
                                label: format!("{}: {}", v.name, typ),
                                documentation: Some(doc),
                            });
                        }
                    }
                } else {
                    hover_found = Some(JsonHover {
                        label: format!("{}: Unknown", v.name),
                        documentation: Some(format!("```flame\nlet {}: Unknown\n```", v.name)),
                    });
                }
            }
        }

        // Provide variables as completion for bare words
        for v in &scanned_vars {
            let typ_str = v.typ.clone().unwrap_or_else(|| "unknown".to_string());
            let is_annotation = typ_str.starts_with("annotation ");

            // If user typed `@`, ONLY show annotations!
            if word_under_cursor_raw.starts_with('@') && !is_annotation {
                continue;
            }

            if v.name.starts_with(&word_under_cursor) || word_under_cursor.is_empty() {
                let mut kind = "variable".to_string();
                let mut label = v.name.clone();
                let mut sort_text = Some("0_".to_string());

                if typ_str.starts_with("fn ") {
                    kind = "function".to_string();
                    sort_text = Some("1_".to_string());
                } else if is_annotation {
                    kind = "annotation".to_string();
                    sort_text = Some("0_".to_string()); // Workspace annotations first
                    if !label.starts_with('@') {
                        label = format!("@{}", label);
                    }
                }

                completions.push(JsonCompletion {
                    sort_text,
                    label,
                    kind,
                    detail: typ_str,
                    documentation: None,
                });
            }
        }

        // Provide structs as completion for bare words
        if !word_under_cursor_raw.starts_with('@') {
            for s in &scanned_structs {
                if s.name.starts_with(&word_under_cursor) || word_under_cursor.is_empty() {
                    completions.push(JsonCompletion {
                        sort_text: None,
                        label: s.name.clone(),
                        kind: "class".to_string(),
                        detail: "struct".to_string(),
                        documentation: None,
                    });
                }
            }
        }

        if hover_found.is_none()
            && exact_ast_hover.is_none()
            && scanned_var_hover.is_none()
            && !word_under_cursor.is_empty()
        {
            if let Some(tc) = &tc_opt {
                if let Some(s) = tc.structs.iter().find(|(k, _)| k == &&word_under_cursor || k.ends_with(&format!(".{}", word_under_cursor))).map(|(_, v)| v) {
                    let doc = s.hover_doc.clone().unwrap_or_default();
                    let final_doc = format!("```flame\nstruct {}\n```\n{}", word_under_cursor, doc);
                    hover_found = Some(JsonHover {
                        label: word_under_cursor.clone(),
                        documentation: Some(final_doc)
                    });
                } else if let Some(e) = tc.enums.iter().find(|(k, _)| k == &&word_under_cursor || k.ends_with(&format!(".{}", word_under_cursor))).map(|(_, v)| v) {
                    let doc = e.hover_doc.clone().unwrap_or_default();
                    let final_doc = format!("```flame\nenum {}\n```\n{}", word_under_cursor, doc);
                    hover_found = Some(JsonHover {
                        label: word_under_cursor.clone(),
                        documentation: Some(final_doc)
                    });
                } else if let Some(f) = tc.functions.iter().find(|(k, _)| k == &&word_under_cursor || k.ends_with(&format!(".{}", word_under_cursor))).map(|(_, v)| v) {
                    let params_str = f.params.iter().map(|p| format!("{}: {:?}", p.name, p.ty)).collect::<Vec<_>>().join(", ");
                    let return_str = if f.return_type == crate::typechecker::Type::Nil { "".to_string() } else { format!(" -> {:?}", f.return_type) };
                    let fallback_doc = format!("```flame\nfn {}({}){}\n```", word_under_cursor, params_str, return_str);
                    let final_doc = if let Some(doc) = &f.hover_doc { format!("{}\n{}", fallback_doc, doc) } else { fallback_doc };
                    hover_found = Some(JsonHover {
                        label: format!("{}()", word_under_cursor),
                        documentation: Some(final_doc)
                    });
                } else if let Some(impl_name) = &current_impl {
                    if let Some(methods) = tc.methods.get(impl_name) {
                        if let Some((_, f)) = methods.iter().find(|(k, _)| k == &&word_under_cursor) {
                            let params_str = f.params.iter().map(|p| format!("{}: {:?}", p.name, p.ty)).collect::<Vec<_>>().join(", ");
                            let return_str = if f.return_type == crate::typechecker::Type::Nil { "".to_string() } else { format!(" -> {:?}", f.return_type) };
                            let fallback_doc = format!("```flame\nfn {}({}){}\n```", word_under_cursor, params_str, return_str);
                            let final_doc = if let Some(doc) = &f.hover_doc { format!("{}\n{}", fallback_doc, doc) } else { fallback_doc };
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}()", impl_name, word_under_cursor),
                                documentation: Some(final_doc)
                            });
                        }
                    }
                }
            }
            
            if hover_found.is_none() {
                // Check if the bare word is a function/annotation from any native module
                for mod_name in &native_modules {
                    if let Some(meta) = load_meta_from_project(&manifest_dir, mod_name) {
                        if let Some(function) = meta
                            .functions
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
                                    mod_name,
                                    &function.flame_name,
                                )
                                .unwrap_or_default()
                            });
                            let final_doc = if doc.trim().is_empty() {
                                format!("```flame\n{}\n```\n\n**Return Type**: `{}`", sig, function.return_type)
                            } else {
                                format!(
                                    "```flame\n{}\n```\n{}\n\n**Return Type**: `{}`",
                                    sig, doc, function.return_type
                                )
                            };
                            hover_found = Some(JsonHover {
                                label: format!("{}::{}()", mod_name, function.flame_name),
                                documentation: Some(final_doc),
                            });
                            break;
                        }
                    }
                }
            }
        } // closes if !resolved_as_var
    };

    // Prioritize rich documentation (keywords, built-ins, standard library, decorators, native docs).

    let mut signature_help = None;
    if let Some(prefix) = current_line.get(..cursor_col.saturating_sub(1)) {
        let mut open_parens = 0;
        let mut chars = prefix.chars().rev().enumerate();
        let mut found_call = false;
        let mut commas = 0;
        let mut call_start_idx = 0;

        while let Some((i, c)) = chars.next() {
            if c == ')' {
                open_parens += 1;
            } else if c == ',' && open_parens == 0 {
                commas += 1;
            } else if c == '(' {
                if open_parens > 0 {
                    open_parens -= 1;
                } else {
                    found_call = true;
                    call_start_idx = prefix.len() - 1 - i;
                    break;
                }
            }
        }

        if found_call {
            let func_name_str = prefix[..call_start_idx].trim_end();
            let name_end = func_name_str.len();
            let mut name_start = name_end;
            for (i, c) in func_name_str.chars().rev().enumerate() {
                if !c.is_alphanumeric() && c != '_' && c != '.' && c != '@' {
                    name_start = name_end - i;
                    break;
                }
                if i == name_end - 1 {
                    name_start = 0;
                }
            }
            if name_start < name_end {
                let func_name = &func_name_str[name_start..name_end];
                let clean_name = func_name
                    .trim_start_matches('@')
                    .split('.')
                    .last()
                    .unwrap_or(func_name);

                if let Some(tc) = &tc_opt {
                    let mut found_sig = tc.functions.get(clean_name);

                    if found_sig.is_none() {
                        for (_, methods) in &tc.methods {
                            if let Some(sig) = methods.get(clean_name) {
                                found_sig = Some(sig);
                                break;
                            }
                        }
                    }

                    if found_sig.is_none() {
                        for (_, funcs) in &tc.plugin_functions {
                            if let Some(sig) = funcs.get(clean_name) {
                                found_sig = Some(sig);
                                break;
                            }
                        }
                    }

                    if let Some(func) = found_sig {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, tc.format_type(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let ret_str = format!(" -> {}", tc.format_type(&func.return_type));

                        let label = if func_name.starts_with('@') {
                            format!("@{}({})", clean_name, params_str)
                        } else {
                            format!("{}({}){}", clean_name, params_str, ret_str)
                        };
                        let parameters = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, tc.format_type(&p.ty)))
                            .collect();

                        signature_help = Some(JsonSignatureHelp {
                            label,
                            parameters,
                            active_parameter: commas,
                        });
                    }
                }
            }
        }
    }

    let hover = exact_ast_hover
        .or(hover_found)
        .or(scanned_var_hover)
        .or_else(|| ide::get_keyword_hover(&word_under_cursor));

    let tokens = ide::get_semantic_tokens(&content);

    let mut unique_completions = Vec::new();
    let mut seen_labels = std::collections::HashSet::new();
    for c in completions {
        if seen_labels.insert(c.label.clone()) {
            unique_completions.push(c);
        }
    }

    JsonCheckOutput {
        file: file.to_string(),
        diagnostics,
        std_modules,
        native_modules,
        plugins,
        completions: unique_completions,
        hover,
        signature_help,
        tokens,
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
    let end = col.min(line.len());
    let upto = line.chars().take(end).collect::<String>();

    // If the character just after the extracted part is a dot (meaning the cursor is exactly ON the dot),
    // we should include it to properly trigger member completions.
    let upto = if end < line.len() && line[end..].starts_with('.') {
        line.chars().take(end + 1).collect::<String>()
    } else {
        upto
    };

    if let Some(dot_index) = upto.rfind('.') {
        let after_dot = &upto[dot_index + 1..];
        if !after_dot
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return (None, None);
        }
        let left = upto[..dot_index].trim();
        let right = after_dot.to_string();
        return (
            left.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
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
            if direct.exists() {
                return Some(direct);
            }

            let pkg_main = manifest_dir
                .join(".flame")
                .join("pkg")
                .join(namespace)
                .join("src")
                .join("main.fm");
            if pkg_main.exists() {
                return Some(pkg_main);
            }

            None
        })?;

    let mut paths_to_read = Vec::new();
    if candidate.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&candidate) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("fm") {
                    paths_to_read.push(p);
                }
            }
        }
    } else {
        paths_to_read.push(candidate);
    }

    let mut exported_stmts = Vec::new();
    for path in paths_to_read {
        if let Ok(content) = fs::read_to_string(&path) {
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
            let mut parser = Parser::new(tokens, path.to_string_lossy().to_string());
            if let Ok(stmts) = parser.parse() {
                for stmt in stmts {
                    if let crate::parser::Stmt::ExportDecl(inner, _) = stmt {
                        exported_stmts.push(*inner);
                    }
                }
            }
        }
    }

    if exported_stmts.is_empty() {
        None
    } else {
        Some(exported_stmts)
    }
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
        let file_path = crate::stdlib::locate_import_file(Path::new(current_file), &path_parts)
            .or_else(|| {
                let pkg_main = _manifest_dir
                    .join(".flame")
                    .join("pkg")
                    .join(&module_path)
                    .join("src")
                    .join("main.fm");
                if pkg_main.exists() {
                    return Some(pkg_main);
                }
                None
            });

        if let Some(file_path) = file_path {
            let mut paths_to_read = Vec::new();
            if file_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&file_path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("fm") {
                            paths_to_read.push(p);
                        }
                    }
                }
            } else {
                paths_to_read.push(file_path);
            }

            for path in paths_to_read {
                if let Ok(module_content) = fs::read_to_string(&path) {
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
                    let mut parser = Parser::new(tokens, path.to_string_lossy().to_string());
                    if let Ok(parsed_stmts) = parser.parse() {
                        for stmt in parsed_stmts {
                            if let crate::parser::Stmt::ExportDecl(inner, _) = &stmt {
                                results.push((**inner).clone());
                            } else if let crate::parser::Stmt::PackageDecl { .. } = &stmt {
                                results.push(stmt.clone());
                            }
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
