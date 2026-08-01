use crate::package_manager;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn build_aot_project(pkg_name: &str, profile: &str, native_deps: &[(String, String)], force_local: bool) {
    println!("\x1b[1;36m     Linking\x1b[0m native static object files...");

    let build_cache = Path::new(".flame").join("build-cache");
    let _ = fs::create_dir_all(&build_cache);
    let src_dir = build_cache.join("src");
    let _ = fs::create_dir_all(&src_dir);
    let _ = fs::write(src_dir.join("main.rs"), "fn main() {}");

    let current_exe = std::env::current_exe().unwrap();
    let mut is_local_dev = false;
    let mut flame_source_dir = std::path::PathBuf::new();
    
    if let (Some(parent1), Some(parent2), Some(parent3)) = (
        current_exe.parent(),
        current_exe.parent().and_then(|p| p.parent()),
        current_exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()),
    ) {
        if (parent1.ends_with("debug") || parent1.ends_with("release"))
            && parent2.ends_with("target")
            && parent3.join("Cargo.toml").exists()
        {
            let cargo_content = fs::read_to_string(parent3.join("Cargo.toml")).unwrap_or_default();
            if cargo_content.contains("name = \"flame\"") {
                is_local_dev = true;
                flame_source_dir = parent3.to_path_buf();
            }
        }
    }

    if let Ok(dev_path) = std::env::var("WREN_DEV_PATH") {
        is_local_dev = true;
        flame_source_dir = std::path::PathBuf::from(dev_path);
    }

    if force_local {
        is_local_dev = true;
        flame_source_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    }

    let flame_dep = if is_local_dev {
        format!(r#"flamelang = {{ path = "{}" }}"#, flame_source_dir.to_string_lossy().replace("\\", "/"))
    } else {
        format!(r#"flamelang = "{}""#, env!("CARGO_PKG_VERSION"))
    };

    let mut deps_str = String::new();
    for (name, version) in native_deps {
        if version.starts_with('{') {
            deps_str.push_str(&format!("{} = {}\n", name, version));
        } else {
            deps_str.push_str(&format!("{} = \"{}\"\n", name, version));
        }
    }

    let cargo_toml = format!(
        r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["rt-multi-thread", "macros", "time", "net", "sync"] }}
{flame_dep}
{deps}

[profile.dev]
split-debuginfo = "unpacked"
codegen-units = 256

[profile.release]
opt-level = 3
strip = true
lto = "fat"
codegen-units = 1
panic = "abort"
"#,
        pkg_name = pkg_name,
        flame_dep = flame_dep,
        deps = deps_str
    );

    fs::write(build_cache.join("Cargo.toml"), cargo_toml).unwrap();

    let mut main_rs = String::new();
    main_rs.push_str("#![allow(unused_variables, dead_code, unused_imports, non_snake_case)]\n");
    main_rs.push_str("use flamelang::runner::{Runner, CValue};\n");
    main_rs.push_str("use std::path::PathBuf;\n\n");
    main_rs.push_str("use flamelang::vm;\n");

    // Generate docs for dependencies to extract metadata
    for (name, version) in native_deps {
        let mut package_spec = name.clone();
        if !version.starts_with('{') && version != "*" {
            package_spec = format!("{}@{}", name, version);
        } else if version.starts_with('{') {
            if let Some(idx) = version.find("version = \"") {
                let rest = &version[idx + 11..];
                if let Some(end_idx) = rest.find("\"") {
                    let v = &rest[..end_idx];
                    if v != "*" {
                        package_spec = format!("{}@{}", name, v);
                    }
                }
            } else if let Some(idx) = version.find("version=\"") {
                let rest = &version[idx + 9..];
                if let Some(end_idx) = rest.find("\"") {
                    let v = &rest[..end_idx];
                    if v != "*" {
                        package_spec = format!("{}@{}", name, v);
                    }
                }
            }
        }
        let mut retry = true;
        
        while retry {
            retry = false;
            let output = Command::new("cargo")
                .args([
                    "+nightly",
                    "rustdoc",
                    "-p",
                    &package_spec,
                    "--",
                    "--output-format",
                    "json",
                    "-Zunstable-options",
                ])
                .current_dir(&build_cache)
                .output();

            if let Ok(out) = output {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    println!("WARNING: rustdoc failed for {}. Stderr: {}", package_spec, stderr);
                    if stderr.contains("is ambiguous") && stderr.contains("following specifications") {
                        // Extract the first specification
                        if let Some(spec_idx) = stderr.find("following specifications") {
                            let rest = &stderr[spec_idx..];
                            if let Some(spec_line) = rest.lines().nth(1) {
                                let spec = spec_line.trim();
                                println!("WARNING: Found ambiguous spec suggestion: '{}'", spec);
                                if !spec.is_empty() && spec != package_spec {
                                    package_spec = spec.to_string();
                                    retry = true;
                                    println!("WARNING: Retrying rustdoc for {}", package_spec);
                                }
                            }
                        }
                    }
                }
            }
        }

        let json_path = build_cache
            .join("target/doc")
            .join(format!("{}.json", name.replace("-", "_")));
        let meta_dir = Path::new(".flame").join("pkg").join(name);
        fs::create_dir_all(&meta_dir).unwrap();
        let meta_path = meta_dir.join(format!("{}.fmi", name));

        if json_path.exists() {
            let meta = crate::package_manager::parse_rustdoc_json(&json_path, name);
            if let Ok(meta_str) = serde_json::to_string_pretty(&meta) {
                fs::write(&meta_path, meta_str).unwrap();
            }
        }
    }

    for (name, _) in native_deps {
        let meta_path = Path::new(".flame")
            .join("pkg")
            .join(name)
            .join(format!("{}.fmi", name));
        if meta_path.exists() {
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<package_manager::FlameMeta>(&meta_content) {
                    main_rs.push_str(&format!("// Wrapper for crate {}\n", name));
                    main_rs.push_str(&format!("mod bridge_{} {{\n", name));
                    main_rs.push_str("    use super::*;\n");
                    main_rs.push_str("    #[allow(unused_imports, dead_code)]\n");
                    main_rs.push_str(&format!("    use {}::*;\n", name));
                    main_rs.push_str("    type NativeObject = std::ffi::c_void;\n");
                    let mut generated_methods = std::collections::HashSet::new();

                    for func in meta.functions {
                        if !generated_methods.insert(func.name.clone()) || should_skip_bridge_function(&func) {
                            continue;
                        }

                        let f_name = &func.flame_name;
                        main_rs.push_str(&format!(
                            "    pub fn {}(_args: *const CValue, _len: usize) -> CValue {{\n",
                            f_name
                        ));
                        // Start generated function body
                        main_rs.push_str("        let c_args = unsafe { std::slice::from_raw_parts(_args, _len) };\n");
                        let mut call_args = Vec::new();
                        for (idx, p) in func.params.iter().enumerate() {
                            let (ext_code, var_name) = generate_param_extraction(p, idx, &func.name);
                            main_rs.push_str(&ext_code);
                            call_args.push(var_name);
                        }

                        let args_str = call_args.join(", ");
                        let requires_prim = func.is_generic && (func.name == "gen" || func.params.is_empty());
                        let is_async = func.is_async || func.name.contains("listen") || func.name.contains("serve") || func.name.contains("run") || func.return_type.contains("Future") || func.return_type.contains("async");

                        if requires_prim {
                            let generic_idx = func.params.len();
                            main_rs.push_str(&format!("        let generic_type_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", generic_idx));
                            main_rs.push_str("        let generic_type = generic_type_cstr.to_str().unwrap_or_default();\n");
                            main_rs.push_str("        match generic_type {\n");
                            let primitives = vec!["i64", "f64", "bool"];
                            for prim in primitives {
                                main_rs.push_str(&format!("            \"{}\" => {{\n", prim));
                                main_rs.push_str(&format!("                let res = {}::{}::<{}>({});\n", name, func.name, prim, args_str));
                                if prim == "bool" {
                                    main_rs.push_str("                let mut cv = CValue::null();\n");
                                    main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Bool;\n");
                                    main_rs.push_str("                cv.bool_val = res;\n");
                                    main_rs.push_str("                return cv;\n");
                                } else if prim == "f32" || prim == "f64" {
                                    main_rs.push_str("                let mut cv = CValue::null();\n");
                                    main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Float;\n");
                                    main_rs.push_str("                cv.float_val = res as f64;\n");
                                    main_rs.push_str("                return cv;\n");
                                } else {
                                    main_rs.push_str("                let mut cv = CValue::null();\n");
                                    main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Int;\n");
                                    main_rs.push_str("                cv.int_val = res as i64;\n");
                                    main_rs.push_str("                return cv;\n");
                                }
                                main_rs.push_str("            }\n");
                            }
                            main_rs.push_str("            _ => return CValue::null(),\n");
                            main_rs.push_str("        }\n");
                        } else {
                            if is_async {
                                if func.name.contains("listen") || func.name.contains("serve") || func.name.contains("run") {
                                    main_rs.push_str("        flamelang::vm::set_event_loop_active(true);\n");
                                    main_rs.push_str(&format!("        std::thread::spawn(move || {{ let rt = tokio::runtime::Runtime::new().unwrap(); rt.block_on(async move {{ {}::{}({}).await }}); }});\n", name, func.name, args_str));
                                    main_rs.push_str("        CValue::null()\n");
                                } else {
                                    main_rs.push_str(&format!("        let res = tokio::runtime::Runtime::new().unwrap().block_on(async move {{ {}::{}({}).await }});\n", name, func.name, args_str));
                                    main_rs.push_str(&generate_return_conversion(&func.return_type, ""));
                                }
                            } else {
                                main_rs.push_str(&format!("        let res = {}::{}({});\n", name, func.name, args_str));
                                main_rs.push_str(&generate_return_conversion(&func.return_type, ""));
                            }
                        }
                        main_rs.push_str("    }\n");
                    }

                    for struct_meta in meta.structs {
                        let s_name = &struct_meta.name;
                        for func in struct_meta.methods {
                            let f_name = &func.flame_name;
                            let combined_name = format!("{}_{}", s_name, f_name);
                            if !generated_methods.insert(combined_name.clone()) || should_skip_bridge_function(&func) {
                                continue;
                            }

                            main_rs.push_str(&format!(
                                "    pub fn {}(_args: *const CValue, _len: usize) -> CValue {{\n",
                                combined_name
                            ));
                            main_rs.push_str("        let c_args = unsafe { std::slice::from_raw_parts(_args, _len) };\n");
                            let is_async = func.is_async || func.name.contains("listen") || func.name.contains("serve") || func.name.contains("run") || func.return_type.contains("Future") || func.return_type.contains("async");
                            if !func.is_static {
                                main_rs.push_str("        // Self is arg 0, cast from obj_ptr\n");
                                if func.receiver == Some("self".to_string()) || (is_async && (func.name.contains("listen") || func.name.contains("serve") || func.name.contains("run"))) {
                                    main_rs.push_str(&format!("        let obj = *unsafe {{ Box::from_raw(c_args[0].obj_ptr as *mut {}::{}) }};\n", name, s_name));
                                } else {
                                    main_rs.push_str(&format!("        let obj = unsafe {{ &mut *(c_args[0].obj_ptr as *mut {}::{}) }};\n", name, s_name));
                                }
                            }

                            let mut call_args = Vec::new();
                            for (idx, p) in func.params.iter().enumerate() {
                                let c_idx = if func.is_static { idx } else { idx + 1 };
                                let (ext_code, var_name) = generate_param_extraction(p, c_idx, &func.name);
                                main_rs.push_str(&ext_code);
                                call_args.push(var_name);
                            }

                            let args_str = call_args.join(", ");
                            let requires_prim = func.is_generic && (func.name == "gen" || func.params.is_empty());
                            if requires_prim {
                                let generic_idx = if func.is_static { func.params.len() } else { func.params.len() + 1 };
                                main_rs.push_str(&format!("        let generic_type_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", generic_idx));
                                main_rs.push_str("        let generic_type = generic_type_cstr.to_str().unwrap_or_default();\n");
                                main_rs.push_str("        match generic_type {\n");
                                let primitives = vec!["i64", "f64", "bool"];
                                for prim in primitives {
                                    main_rs.push_str(&format!("            \"{}\" => {{\n", prim));
                                    if func.is_static {
                                        main_rs.push_str(&format!(
                                            "                let res = {}::{}::{}::<{}>({});\n",
                                            name, s_name, func.name, prim, args_str
                                        ));
                                    } else {
                                        main_rs.push_str(&format!(
                                            "                let res = obj.{}::<{}>({});\n",
                                            func.name, prim, args_str
                                        ));
                                    }
                                    if prim == "bool" {
                                        main_rs.push_str("                let mut cv = CValue::null();\n");
                                        main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Bool;\n");
                                        main_rs.push_str("                cv.bool_val = res;\n");
                                        main_rs.push_str("                return cv;\n");
                                    } else if prim == "f32" || prim == "f64" {
                                        main_rs.push_str("                let mut cv = CValue::null();\n");
                                        main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Float;\n");
                                        main_rs.push_str("                cv.float_val = res as f64;\n");
                                        main_rs.push_str("                return cv;\n");
                                    } else {
                                        main_rs.push_str("                let mut cv = CValue::null();\n");
                                        main_rs.push_str("                cv.tag = flamelang::runner::CValueTag::Int;\n");
                                        main_rs.push_str("                cv.int_val = res as i64;\n");
                                        main_rs.push_str("                return cv;\n");
                                    }
                                    main_rs.push_str("            }\n");
                                }
                                main_rs.push_str("            _ => return CValue::null(),\n");
                                main_rs.push_str("        }\n");
                            } else {
                                let call_expr = if func.is_static {
                                    format!("{}::{}::{}({})", name, s_name, func.name, args_str)
                                } else {
                                    format!("obj.{}({})", func.name, args_str)
                                };

                                if is_async {
                                    if func.name.contains("listen") || func.name.contains("serve") || func.name.contains("run") {
                                        main_rs.push_str("        flamelang::vm::set_event_loop_active(true);\n");
                                        main_rs.push_str(&format!("        std::thread::spawn(move || {{ let rt = tokio::runtime::Runtime::new().unwrap(); rt.block_on(async move {{ {}.await }}); }});\n", call_expr));
                                        main_rs.push_str("        CValue::null()\n");
                                    } else {
                                        main_rs.push_str(&format!("        let res = tokio::runtime::Runtime::new().unwrap().block_on(async move {{ {}.await }});\n", call_expr));
                                        main_rs.push_str(&generate_return_conversion(&func.return_type, s_name));
                                    }
                                } else {
                                    main_rs.push_str(&format!("        let res = {};\n", call_expr));
                                    main_rs.push_str(&generate_return_conversion(&func.return_type, s_name));
                                }
                            }
                            main_rs.push_str("    }\n");
                        }
                    }
                    main_rs.push_str("}\n\n");
                }
            }
        }
    }

    main_rs.push_str("fn main() {\n");
    main_rs.push_str("    let mut runner = Runner::new(PathBuf::from(\"src/main.fm\"));\n");

    for (name, _) in native_deps {
        let meta_path = Path::new(".flame")
            .join("pkg")
            .join(name)
            .join(format!("{}.fmi", name));
        if meta_path.exists() {
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<package_manager::FlameMeta>(&meta_content) {
                    let mut generated_methods = std::collections::HashSet::new();
                    for func in meta.functions {
                        if !generated_methods.insert(func.name.clone()) || should_skip_bridge_function(&func) {
                            continue;
                        }
                        let f_name = &func.flame_name;
                        let sym = format!("flame_{}_{}", name, f_name);
                        main_rs.push_str(&format!("    runner.native_methods.insert(\"{sym}\".to_string(), bridge_{name}::{f_name} as fn(*const CValue, usize) -> CValue);\n", sym=sym, name=name, f_name=f_name));
                    }
                    for struct_meta in meta.structs {
                        let s_name = &struct_meta.name;
                        for func in struct_meta.methods {
                            let f_name = &func.flame_name;
                            let combined_name = format!("{}_{}", s_name, f_name);
                            if !generated_methods.insert(combined_name.clone()) || should_skip_bridge_function(&func) {
                                continue;
                            }
                            let sym = format!("flame_{}_{}_{}", name, s_name, f_name);
                            main_rs.push_str(&format!("    runner.native_methods.insert(\"{sym}\".to_string(), bridge_{name}::{s_name}_{f_name} as fn(*const CValue, usize) -> CValue);\n", sym=sym, name=name, s_name=s_name, f_name=f_name));
                            if name.to_lowercase() == s_name.to_lowercase() {
                                let alias_sym = format!("flame_{}_{}", name, f_name);
                                main_rs.push_str(&format!("    runner.native_methods.insert(\"{alias_sym}\".to_string(), bridge_{name}::{s_name}_{f_name} as fn(*const CValue, usize) -> CValue);\n", alias_sym=alias_sym, name=name, s_name=s_name, f_name=f_name));
                            }
                        }
                    }
                }
            }
        }
    }

    // We don't have execute_source right now, so we need to run file
    main_rs.push_str("    // Since execute_source does not exist, we just run_file from main.rs if we had it, but here we can just parse and run\n");
    main_rs
        .push_str("    // Read the package's source at runtime from current working directory\n");
    main_rs.push_str(
        "    let src = std::fs::read_to_string(\"src/main.fm\").unwrap_or_default();\n",
    );
    main_rs.push_str("    let mut lexer = flamelang::lexer::Lexer::new(&src);\n");
    main_rs.push_str("    let mut tokens = Vec::new();\n");
    main_rs.push_str("    loop {\n");
    main_rs.push_str("        let tok = lexer.next_token();\n");
    main_rs.push_str("        let is_eof = tok.kind == flamelang::lexer::TokenKind::EOF;\n");
    main_rs.push_str("        tokens.push(tok);\n");
    main_rs.push_str("        if is_eof { break; }\n");
    main_rs.push_str("    }\n");
    main_rs.push_str(
        "    let mut parser = flamelang::parser::Parser::new(tokens, \"src/main.fm\".to_string());\n",
    );
    main_rs.push_str("    match parser.parse() {\n");
    main_rs.push_str("        Ok(stmts) => {\n");
    main_rs.push_str("            let result = runner.run(&stmts);\n");
    main_rs.push_str("            vm::wait_for_all_threads();\n");
    main_rs.push_str("            if let Err(e) = result {\n");
    main_rs.push_str("                eprintln!(\"\\x1b[1;31mRuntime error:\\x1b[0m {}\", e);\n");
    main_rs.push_str("            }\n");
    main_rs.push_str("        }\n");
    main_rs.push_str("        Err(diag) => {\n");
    main_rs.push_str("            eprintln!(\"Parse error: {}\", diag.message);\n");
    main_rs.push_str("        }\n");
    main_rs.push_str("    }\n");
    main_rs.push_str("}\n");

    fs::write(build_cache.join("src/main.rs"), main_rs).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if profile == "release" {
        cmd.arg("--release");
    }
    cmd.current_dir(&build_cache);
    let output = cmd.output();

    let target_dir = Path::new("target").join(profile);
    fs::create_dir_all(&target_dir).unwrap();
    let exe_name = format!("{}{}", pkg_name, std::env::consts::EXE_SUFFIX);
    let target_exe = target_dir.join(&exe_name);

    if let Ok(out) = output {
        if out.status.success() {
            let compiled_exe = build_cache
                .join("target")
                .join(if profile == "release" {
                    "release"
                } else {
                    "debug"
                })
                .join(&exe_name);
            let _ = fs::copy(&compiled_exe, &target_exe);
            println!("\x1b[1;32m     Finished\x1b[0m building native executable!");
        } else {
            eprintln!("\x1b[1;31m     Error\x1b[0m failed to build native executable.");
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            std::process::exit(1);
        }
    } else {
        eprintln!("\x1b[1;31m     Error\x1b[0m could not invoke cargo build.");
        std::process::exit(1);
    }
}

fn should_skip_bridge_function(func: &crate::package_manager::FlameFunctionMeta) -> bool {
    func.return_type.contains("Iter")
        || func.return_type.contains("Iterator")
        || func.return_type.contains("NonNil")
        || func.name.contains("_iter")
        || func.name.starts_with("from_")
        || func.name.contains("unchecked")
        || func.params.iter().any(|p| {
            p.type_name.contains("Bytes")
                || p.type_name.contains("Variant")
                || p.type_name.contains("Version")
                || p.type_name.contains("Iter")
                || p.type_name.contains("StandardUniform")
                || p.type_name == "Uuid"
        })
}

fn generate_param_extraction(p: &crate::package_manager::FlameParamMeta, c_idx: usize, func_name: &str) -> (String, String) {
    let p_type = p.type_name.to_lowercase();
    let var_name = format!("arg{}", c_idx);
    let mut code = String::new();

    if p.is_callback || p_type.contains("callback") || p_type.contains("handler") || p.name.to_lowercase().contains("handler") || p.name.to_lowercase().contains("callback") || p.type_name.len() == 1 || p_type.contains("fn(") {
        code.push_str(&format!("        let fn_id{} = c_args[{}].int_val as u64;\n", c_idx, c_idx));
        if func_name == "post" || func_name == "put" || func_name == "patch" {
            code.push_str(&format!("        let {} = move |body: String| async move {{\n", var_name));
            code.push_str(&format!("            let cb = flamelang::vm::FlameCallback {{ function_id: fn_id{}, module_id: 0 }};\n", c_idx));
            code.push_str("            let arg_cv = flamelang::runner::CValue::from_string(&body);\n");
            code.push_str("            let res = flamelang::vm::enqueue_callback(cb, vec![arg_cv]).unwrap_or_else(|_| flamelang::vm::CValue::null());\n");
            code.push_str("            if res.tag == flamelang::runner::CValueTag::String && !res.string_ptr.is_null() {\n");
            code.push_str("                unsafe { std::ffi::CStr::from_ptr(res.string_ptr).to_string_lossy().into_owned() }\n");
            code.push_str("            } else {\n");
            code.push_str("                String::new()\n");
            code.push_str("            }\n");
            code.push_str("        };\n");
        } else {
            code.push_str(&format!("        let {} = move || async move {{\n", var_name));
            code.push_str(&format!("            let cb = flamelang::vm::FlameCallback {{ function_id: fn_id{}, module_id: 0 }};\n", c_idx));
            code.push_str("            let res = flamelang::vm::enqueue_callback(cb, vec![]).unwrap_or_else(|_| flamelang::vm::CValue::null());\n");
            code.push_str("            if res.tag == flamelang::runner::CValueTag::String && !res.string_ptr.is_null() {\n");
            code.push_str("                unsafe { std::ffi::CStr::from_ptr(res.string_ptr).to_string_lossy().into_owned() }\n");
            code.push_str("            } else {\n");
            code.push_str("                String::new()\n");
            code.push_str("            }\n");
            code.push_str("        };\n");
        }
    } else if p_type.contains("range") {
        code.push_str(&format!("        let {} = (c_args[{}].int_val as u32)..(c_args[{}].int_val2 as u32);\n", var_name, c_idx, c_idx));
    } else if p_type.contains("&str") || (p_type.contains("str") && !p_type.contains("string")) {
        code.push_str(&format!("        let {}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", var_name, c_idx));
        if p_type.contains("'static") {
            code.push_str(&format!("        let {}: &'static str = Box::leak({}_cstr.to_string_lossy().into_owned().into_boxed_str());\n", var_name, var_name));
        } else {
            code.push_str(&format!("        let {} = {}_cstr.to_str().unwrap_or_default();\n", var_name, var_name));
        }
    } else if p_type == "string" {
        code.push_str(&format!("        let {}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", var_name, c_idx));
        code.push_str(&format!("        let {} = {}_cstr.to_string_lossy().into_owned();\n", var_name, var_name));
    } else if p_type == "pathbuf" {
        code.push_str(&format!("        let {}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", var_name, c_idx));
        code.push_str(&format!("        let {} = std::path::PathBuf::from({}_cstr.to_string_lossy().into_owned());\n", var_name, var_name));
    } else if p_type == "&path" {
        code.push_str(&format!("        let {}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", var_name, c_idx));
        code.push_str(&format!("        let {}_path = std::path::Path::new({}_cstr.to_str().unwrap_or_default());\n", var_name, var_name));
        code.push_str(&format!("        let {} = {}_path;\n", var_name, var_name));
    } else if p_type == "bool" {
        code.push_str(&format!("        let {} = c_args[{}].bool_val;\n", var_name, c_idx));
    } else if p_type == "char" {
        code.push_str(&format!("        let {} = std::char::from_u32(c_args[{}].int_val as u32).unwrap_or(' ');\n", var_name, c_idx));
    } else if ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "usize", "isize"].contains(&p_type.as_str()) {
        code.push_str(&format!("        let {} = c_args[{}].int_val as {};\n", var_name, c_idx, p.type_name));
    } else if ["f32", "f64"].contains(&p_type.as_str()) {
        code.push_str(&format!("        let {} = c_args[{}].float_val as {};\n", var_name, c_idx, p.type_name));
    } else if p_type.starts_with("option<") {
        code.push_str(&format!("        let {} = if c_args[{}].tag == flamelang::runner::CValueTag::Null {{ None }} else {{ Some(c_args[{}].int_val) }};\n", var_name, c_idx, c_idx));
    } else if p_type.starts_with("vec<") {
        code.push_str(&format!("        let {} = Vec::new();\n", var_name));
    } else {
        if p.type_name.starts_with('&') || p_type.contains("object") || p.type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
            let clean_type = p.type_name.trim_start_matches('&').trim_start_matches("mut ").trim();
            code.push_str(&format!("        let {} = unsafe {{ &mut *(c_args[{}].obj_ptr as *mut {}) }};\n", var_name, c_idx, clean_type));
        } else {
            code.push_str(&format!("        let {} = c_args[{}].int_val;\n", var_name, c_idx));
        }
    }

    (code, var_name)
}

fn generate_return_conversion(return_type: &str, s_name: &str) -> String {
    let rt = return_type.to_lowercase();
    let mut code = String::new();
    if rt == "()" {
        code.push_str("        let mut cv = CValue::null();\n        cv\n");
    } else if rt == "string" || rt == "&str" || rt.contains("uuid") || (rt == "self" && s_name == "Uuid") {
        code.push_str("        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();\n");
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::String;\n");
        code.push_str("        cv.string_ptr = c_str.into_raw();\n");
        code.push_str("        cv\n");
    } else if ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "usize", "isize"].contains(&rt.as_str()) {
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::Int;\n");
        code.push_str("        cv.int_val = res as i64;\n");
        code.push_str("        cv\n");
    } else if rt == "bool" {
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::Bool;\n");
        code.push_str("        cv.bool_val = res;\n");
        code.push_str("        cv\n");
    } else if ["f32", "f64"].contains(&rt.as_str()) {
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::Float;\n");
        code.push_str("        cv.float_val = res as f64;\n");
        code.push_str("        cv\n");
    } else if rt == "char" {
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::Int;\n");
        code.push_str("        cv.int_val = res as u32 as i64;\n");
        code.push_str("        cv\n");
    } else if rt == "pathbuf" || rt == "&path" {
        code.push_str("        let c_str = std::ffi::CString::new(res.to_str().unwrap_or_default()).unwrap_or_default();\n");
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::String;\n");
        code.push_str("        cv.string_ptr = c_str.into_raw();\n");
        code.push_str("        cv\n");
    } else if rt.starts_with("option<") {
        code.push_str("        match res {\n");
        code.push_str("            Some(val) => {\n");
        code.push_str("                let boxed = Box::new(val);\n");
        code.push_str("                let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;\n");
        code.push_str("                let mut cv = CValue::null();\n");
        code.push_str("                cv.tag = flamelang::runner::CValueTag::NativeObject;\n");
        code.push_str("                cv.obj_ptr = ptr;\n");
        code.push_str("                cv\n");
        code.push_str("            }\n");
        code.push_str("            None => CValue::null(),\n");
        code.push_str("        }\n");
    } else if rt.starts_with("result<") || rt.contains("::result::") || rt.starts_with("std::io::result") {
        code.push_str("        match res {\n");
        code.push_str("            Ok(val) => {\n");
        code.push_str("                let boxed = Box::new(val);\n");
        code.push_str("                let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;\n");
        code.push_str("                let mut cv = CValue::null();\n");
        code.push_str("                cv.tag = flamelang::runner::CValueTag::NativeObject;\n");
        code.push_str("                cv.obj_ptr = ptr;\n");
        code.push_str("                cv\n");
        code.push_str("            }\n");
        code.push_str("            Err(e) => {\n");
        code.push_str("                eprintln!(\"Runtime bridge exception: {:?}\", e);\n");
        code.push_str("                CValue::null()\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
    } else {
        code.push_str("        let boxed = Box::new(res);\n");
        code.push_str("        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;\n");
        code.push_str("        let mut cv = CValue::null();\n");
        code.push_str("        cv.tag = flamelang::runner::CValueTag::NativeObject;\n");
        code.push_str("        cv.obj_ptr = ptr;\n");
        code.push_str("        cv\n");
    }
    code
}
