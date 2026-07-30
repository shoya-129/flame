use crate::package_manager;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn build_aot_project(pkg_name: &str, profile: &str, native_deps: &[(String, String)], force_local: bool) {
    println!("\x1b[1;36m     Linking\x1b[0m native static object files...");

    let build_cache = Path::new(".wren").join("build-cache");
    let _ = fs::create_dir_all(&build_cache);
    let _ = fs::create_dir_all(build_cache.join("src"));

    let current_exe = std::env::current_exe().unwrap();
    let mut is_local_dev = false;
    let mut wren_source_dir = std::path::PathBuf::new();
    
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
            if cargo_content.contains("name = \"wrenlang\"") {
                is_local_dev = true;
                wren_source_dir = parent3.to_path_buf();
            }
        }
    }

    if let Ok(dev_path) = std::env::var("WREN_DEV_PATH") {
        is_local_dev = true;
        wren_source_dir = std::path::PathBuf::from(dev_path);
    }

    if force_local {
        is_local_dev = true;
        wren_source_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    }

    let wrenlang_dep = if is_local_dev {
        format!(r#"wrenlang = {{ path = "{}" }}"#, wren_source_dir.to_string_lossy().replace("\\", "/"))
    } else {
        format!(r#"wrenlang = "{}""#, env!("CARGO_PKG_VERSION"))
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
{wrenlang_dep}
{deps}
"#,
        pkg_name = pkg_name,
        wrenlang_dep = wrenlang_dep,
        deps = deps_str
    );

    fs::write(build_cache.join("Cargo.toml"), cargo_toml).unwrap();

    let mut main_rs = String::new();
    main_rs.push_str("#![allow(unused_variables, dead_code, unused_imports, non_snake_case)]\n");
    main_rs.push_str("use wrenlang::runner::{Runner, CValue};\n");
    main_rs.push_str("use std::path::PathBuf;\n\n");
    main_rs.push_str("use wrenlang::vm;\n");

    // Generate docs for dependencies to extract metadata
    for (name, _) in native_deps {
        let _ = Command::new("cargo")
            .args([
                "+nightly",
                "rustdoc",
                "-p",
                name,
                "--",
                "--output-format",
                "json",
                "-Zunstable-options",
            ])
            .current_dir(&build_cache)
            .output();

        let json_path = build_cache
            .join("target/doc")
            .join(format!("{}.json", name.replace("-", "_")));
        let meta_dir = Path::new(".wren").join("pkg").join(name);
        fs::create_dir_all(&meta_dir).unwrap();
        let meta_path = meta_dir.join(format!("{}.wmeta", name));

        if json_path.exists() {
            let meta = crate::package_manager::parse_rustdoc_json(&json_path, name);
            if let Ok(meta_str) = serde_json::to_string_pretty(&meta) {
                fs::write(&meta_path, meta_str).unwrap();
            }
        }
    }

    for (name, _) in native_deps {
        let meta_path = Path::new(".wren")
            .join("pkg")
            .join(name)
            .join(format!("{}.wmeta", name));
        if meta_path.exists() {
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<package_manager::WrenMeta>(&meta_content) {
                    main_rs.push_str(&format!("// Wrapper for crate {}\n", name));
                    main_rs.push_str(&format!("mod bridge_{} {{\n", name));
                    main_rs.push_str("    use super::*;\n");
                    let mut generated_methods = std::collections::HashSet::new();

                    for func in meta.functions {
                        if !generated_methods.insert(func.name.clone()) {
                            continue;
                        }
                        let f_name = &func.wren_name;
                        main_rs.push_str(&format!(
                            "    pub fn {}(_args: *const CValue, _len: usize) -> CValue {{\n",
                            f_name
                        ));
                        // Start generated function body
                        main_rs.push_str("        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };\n");
                        let mut call_args = Vec::new();
                        for (idx, p) in func.params.iter().enumerate() {
                            let p_type = p.type_name.to_lowercase();
                            if p_type.contains("range") {
                                main_rs.push_str(&format!("        let arg{} = (c_args[{}].int_val as u32)..(c_args[{}].int_val2 as u32);\n", idx, idx, idx));
                            } else if p_type.contains("str") {
                                main_rs.push_str(&format!("        let arg{}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", idx, idx));
                                main_rs.push_str(&format!("        let arg{} = arg{}_cstr.to_str().unwrap_or_default();\n", idx, idx));
                            } else if p_type.contains("bool") {
                                main_rs.push_str(&format!(
                                    "        let arg{} = c_args[{}].bool_val;\n",
                                    idx, idx
                                ));
                                } else if p_type == "i32" || p_type == "i64" || p_type == "u32" || p_type == "u64" || p_type == "usize" || p_type == "i16" || p_type == "u16" || p_type == "i8" || p_type == "u8" || p_type == "u128" || p_type == "i128" {
                                    main_rs.push_str(&format!(
                                        "        let arg{} = c_args[{}].int_val as {};\n",
                                        idx, idx, p.type_name
                                    ));
                                } else {
                                    main_rs.push_str(&format!(
                                        "        let arg{} = c_args[{}].int_val;\n",
                                        idx, idx
                                    ));
                                }
                            call_args.push(format!("arg{}", idx));
                        }

                        let args_str = call_args.join(", ");

                        // Execute the call
                        if name == "uuid" && f_name == "new_v4" {
                            // Hardcoded fast path
                            main_rs.push_str("        let res = uuid::Uuid::new_v4();\n");
                        } else {
                            main_rs.push_str(&format!(
                                "        let res = {}::{}({});\n",
                                name, func.name, args_str
                            ));
                        }

                        // Convert result to CValue
                        let rt = func.return_type.to_lowercase();
                        if rt == "()" {
                            main_rs.push_str("        let mut cv = CValue::null();\n");
                            main_rs.push_str("        cv\n");
                        } else if rt.contains("string") || rt.contains("str") || rt.contains("uuid") {
                            main_rs.push_str("        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();\n");
                            main_rs.push_str("        let mut cv = CValue::null();\n");
                            main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::String;\n");
                            main_rs.push_str("        cv.string_ptr = c_str.into_raw();\n");
                            main_rs.push_str("        cv\n");
                        } else if rt == "i32" || rt == "i64" || rt == "u32" || rt == "u64" || rt == "usize" {
                            main_rs.push_str("        let mut cv = CValue::null();\n");
                            main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::Int;\n");
                            main_rs.push_str("        cv.int_val = res as i64;\n");
                            main_rs.push_str("        cv\n");
                        } else {
                            main_rs.push_str("        let boxed = Box::new(res);\n");
                            main_rs.push_str("        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;\n");
                            main_rs.push_str("        let mut cv = CValue::null();\n");
                            main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::NativeObject;\n");
                            main_rs.push_str("        cv.obj_ptr = ptr;\n");
                            main_rs.push_str("        cv\n");
                        }
                        main_rs.push_str("    }\n");
                    }

                    for struct_meta in meta.structs {
                        let s_name = &struct_meta.name;
                        for func in struct_meta.methods {
                            let f_name = &func.wren_name;
                            let combined_name = format!("{}_{}", s_name, f_name);
                            if !generated_methods.insert(combined_name.clone()) {
                                continue;
                            }

                            main_rs.push_str(&format!(
                                "    pub fn {}(_args: *const CValue, _len: usize) -> CValue {{\n",
                                combined_name
                            ));
                            main_rs.push_str("        let mut c_args = unsafe { std::slice::from_raw_parts(_args, _len) };\n");
                            if !func.is_static {
                                main_rs.push_str(&format!(
                                    "        // Self is arg 0, cast from obj_ptr\n"
                                ));
                                main_rs.push_str(&format!("        let obj = unsafe {{ &mut *(c_args[0].obj_ptr as *mut {}::{}) }};\n", name, s_name));
                            }

                            let mut call_args = Vec::new();
                            for (idx, p) in func.params.iter().enumerate() {
                                let c_idx = if func.is_static { idx } else { idx + 1 };
                                let p_type = p.type_name.to_lowercase();
                                if p_type.contains("range") {
                                    main_rs.push_str(&format!("        let arg{} = (c_args[{}].int_val as u32)..(c_args[{}].int_val2 as u32);\n", c_idx, c_idx, c_idx));
                                } else if p_type.contains("str") {
                                    main_rs.push_str(&format!("        let arg{}_cstr = unsafe {{ std::ffi::CStr::from_ptr(c_args[{}].string_ptr) }};\n", c_idx, c_idx));
                                    main_rs.push_str(&format!("        let arg{} = arg{}_cstr.to_str().unwrap_or_default();\n", c_idx, c_idx));
                                } else if p_type.contains("bool") {
                                    main_rs.push_str(&format!(
                                        "        let arg{} = c_args[{}].bool_val;\n",
                                        c_idx, c_idx
                                    ));
                                } else if p_type == "i32" || p_type == "i64" || p_type == "u32" || p_type == "u64" || p_type == "usize" || p_type == "i16" || p_type == "u16" || p_type == "i8" || p_type == "u8" || p_type == "u128" || p_type == "i128" {
                                    main_rs.push_str(&format!(
                                        "        let arg{} = c_args[{}].int_val as {};\n",
                                        c_idx, c_idx, p.type_name
                                    ));
                                } else {
                                    main_rs.push_str(&format!(
                                        "        let arg{} = c_args[{}].int_val;\n",
                                        c_idx, c_idx
                                    ));
                                }
                                call_args.push(format!("arg{}", c_idx));
                            }

                            let args_str = call_args.join(", ");
                            if func.is_static {
                                main_rs.push_str(&format!(
                                    "        let res = {}::{}::{}({});\n",
                                    name, s_name, func.name, args_str
                                ));
                            } else {
                                main_rs.push_str(&format!(
                                    "        let res = obj.{}({});\n",
                                    func.name, args_str
                                ));
                            }

                            // Convert result to CValue
                            let rt = func.return_type.to_lowercase();
                            if rt == "()" {
                                main_rs.push_str("        let mut cv = CValue::null();\n");
                                main_rs.push_str("        cv\n");
                            } else if rt.contains("string") || rt.contains("str") || rt.contains("uuid") || (rt == "self" && s_name == "Uuid") {
                                main_rs.push_str("        let c_str = std::ffi::CString::new(res.to_string()).unwrap_or_default();\n");
                                main_rs.push_str("        let mut cv = CValue::null();\n");
                                main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::String;\n");
                                main_rs.push_str("        cv.string_ptr = c_str.into_raw();\n");
                                main_rs.push_str("        cv\n");
                            } else if rt == "i32" || rt == "i64" || rt == "u32" || rt == "u64" || rt == "usize" {
                                main_rs.push_str("        let mut cv = CValue::null();\n");
                                main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::Int;\n");
                                main_rs.push_str("        cv.int_val = res as i64;\n");
                                main_rs.push_str("        cv\n");
                            } else {
                                main_rs.push_str("        let boxed = Box::new(res);\n");
                                main_rs.push_str("        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;\n");
                                main_rs.push_str("        let mut cv = CValue::null();\n");
                                main_rs.push_str("        cv.tag = wrenlang::runner::CValueTag::NativeObject;\n");
                                main_rs.push_str("        cv.obj_ptr = ptr;\n");
                                main_rs.push_str("        cv\n");
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
    main_rs.push_str("    let mut runner = Runner::new(PathBuf::from(\"src/main.wren\"));\n");

    for (name, _) in native_deps {
        let meta_path = Path::new(".wren")
            .join("pkg")
            .join(name)
            .join(format!("{}.wmeta", name));
        if meta_path.exists() {
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<package_manager::WrenMeta>(&meta_content) {
                    for func in meta.functions {
                        let f_name = &func.wren_name;
                        let sym = format!("wren_{}_{}", name, f_name);
                        main_rs.push_str(&format!("    runner.native_methods.insert(\"{sym}\".to_string(), bridge_{name}::{f_name} as fn(*const CValue, usize) -> CValue);\n", sym=sym, name=name, f_name=f_name));
                    }
                    for struct_meta in meta.structs {
                        let s_name = &struct_meta.name;
                        for func in struct_meta.methods {
                            let f_name = &func.wren_name;
                            let sym = format!("wren_{}_{}_{}", name, s_name, f_name);
                            main_rs.push_str(&format!("    runner.native_methods.insert(\"{sym}\".to_string(), bridge_{name}::{s_name}_{f_name} as fn(*const CValue, usize) -> CValue);\n", sym=sym, name=name, s_name=s_name, f_name=f_name));
                            if name.to_lowercase() == s_name.to_lowercase() {
                                let alias_sym = format!("wren_{}_{}", name, f_name);
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
        "    let src = std::fs::read_to_string(\"src/main.wren\").unwrap_or_default();\n",
    );
    main_rs.push_str("    let mut lexer = wrenlang::lexer::Lexer::new(&src);\n");
    main_rs.push_str("    let mut tokens = Vec::new();\n");
    main_rs.push_str("    loop {\n");
    main_rs.push_str("        let tok = lexer.next_token();\n");
    main_rs.push_str("        let is_eof = tok.kind == wrenlang::lexer::TokenKind::EOF;\n");
    main_rs.push_str("        tokens.push(tok);\n");
    main_rs.push_str("        if is_eof { break; }\n");
    main_rs.push_str("    }\n");
    main_rs.push_str(
        "    let mut parser = wrenlang::parser::Parser::new(tokens, \"src/main.wren\".to_string());\n",
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
        }
    } else {
        eprintln!("\x1b[1;31m     Error\x1b[0m could not invoke cargo build.");
    }
}
