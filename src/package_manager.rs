use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WrenParamMeta {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WrenFunctionMeta {
    pub name: String,
    pub wren_name: String,
    pub params: Vec<WrenParamMeta>,
    pub return_type: String,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WrenStructMeta {
    pub name: String,
    pub methods: Vec<WrenFunctionMeta>,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WrenMeta {
    pub module: String,
    pub kind: String,
    pub lib: Option<String>,
    pub functions: Vec<WrenFunctionMeta>,
    pub structs: Vec<WrenStructMeta>,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSpec {
    pub name: String,
    pub source: String,
    pub version: Option<String>,
    pub is_local: bool,
}

fn parse_section_entries(content: &str, section_name: &str) -> Vec<(String, String)> {
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

fn parse_plugin_entries(content: &str) -> Vec<(String, String)> {
    parse_section_entries(content, "[plugins]")
}

pub fn list_plugins() -> Vec<PluginSpec> {
    let toml_path = Path::new("wren.toml");
    if !toml_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(toml_path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    parse_plugin_entries(&content)
        .into_iter()
        .map(|(name, source)| {
            let version = if source == "*" || source.is_empty() {
                None
            } else if let Some((_, version)) = source.rsplit_once('@') {
                Some(version.to_string())
            } else {
                None
            };
            let is_local = source.starts_with('.') || source.starts_with('/') || source == "*";
            PluginSpec {
                name,
                source,
                version,
                is_local,
            }
        })
        .collect()
}

pub fn add_package(args: &[String]) {
    if args.is_empty() {
        println!("\x1b[1;31merror:\x1b[0m please specify package name to add.");
        println!("usage: wren add <package_name> | wren add @plugin <plugin_name_or_url@version>");
        return;
    }

    let is_plugin = args[0] == "@plugin";
    let raw_target = if is_plugin {
        if args.len() < 2 {
            println!("\x1b[1;31merror:\x1b[0m please specify plugin name or source.");
            return;
        }
        &args[1]
    } else {
        &args[0]
    };
    let is_native = args.contains(&"--native".to_string());

    let plugin_name = raw_target
        .rsplit('/')
        .next()
        .unwrap_or(raw_target)
        .split('@')
        .next()
        .unwrap_or(raw_target)
        .trim_end_matches(".git")
        .to_string();
    let manifest_key = &plugin_name;
    let manifest_value = if raw_target == manifest_key {
        "*"
    } else {
        raw_target
    };
    let section = if is_plugin {
        "[plugins]"
    } else if is_native {
        "[native-dependencies]"
    } else {
        "[dependencies]"
    };

    println!(
        "\x1b[1;36m    Adding\x1b[0m {} '{}'...",
        if is_plugin {
            "plugin"
        } else if is_native {
            "native dependency"
        } else {
            "dependency"
        },
        manifest_key
    );

    let toml_path = Path::new("wren.toml");
    if toml_path.exists() {
        let mut content = fs::read_to_string(toml_path).unwrap_or_default();
        if !content.contains(section) {
            content.push_str(&format!("\n{}\n", section));
        }
        if !content.contains(&format!("{} =", manifest_key)) {
            content.push_str(&format!("{} = \"{}\"\n", manifest_key, manifest_value));
            let _ = fs::write(toml_path, content);
        }
    }

    if is_plugin {
        let local_plugin = Path::new(manifest_key);
        if local_plugin.join("Cargo.toml").exists() {
            println!("\x1b[1;36m   Compiling\x1b[0m native plugin '{}'...", manifest_key);
            let _ = std::process::Command::new("cargo")
                .arg("build")
                .current_dir(local_plugin)
                .output();
            inspect_native_plugin(manifest_key, local_plugin);
        }
    }

    println!(
        "\x1b[1;32m   Installed\x1b[0m {} '{}' successfully.",
        if is_plugin {
            "plugin"
        } else if is_native {
            "native dependency"
        } else {
            "package"
        },
        manifest_key
    );
}

pub fn remove_package(pkg_name: &str) {
    let toml_path = Path::new("wren.toml");
    if toml_path.exists() {
        if let Ok(content) = fs::read_to_string(toml_path) {
            let lines: Vec<&str> = content
                .lines()
                .filter(|line| !line.trim().starts_with(&format!("{} =", pkg_name)))
                .collect();
            let _ = fs::write(toml_path, lines.join("\n"));
        }
    }

    let pkg_dir = Path::new(".wren").join("pkg").join(pkg_name);
    if pkg_dir.exists() {
        let _ = fs::remove_dir_all(pkg_dir);
    }
    println!("\x1b[1;32m     Removed\x1b[0m package '{}'", pkg_name);
}

pub fn ensure_dependencies_installed() {
    // Compile std_bridge directly since it uses #[wren_export] now
    let std_bridge_path = Path::new("wren-stdlib").join("native").join("std_bridge");
    if std_bridge_path.exists() {
        let _ = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(&std_bridge_path)
            .output();
    }
    let toml_path = Path::new("wren.toml");
    if !toml_path.exists() {
        return;
    }

    let content = match fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(_) => return,
    };


    // Parse all relevant sections for dependencies
    let deps = parse_section_entries(&content, "[dependencies]");
    let native_deps = parse_section_entries(&content, "[native-dependencies]");
    let plugins = parse_section_entries(&content, "[plugins]");
    
    // Combine native deps and plugins for compilation
    let mut native_to_compile = native_deps;
    native_to_compile.extend(plugins);
    
    // Helper to fetch remote dependency
    let fetch_remote = |target: &str, source: &str| -> String {
        let is_local = source.starts_with('.') || source.starts_with('/') || source == "*";
        if is_local {
            if source == "*" { target.to_string() } else { source.to_string() }
        } else {
            let pkg_dir = Path::new(".wren").join("pkg");
            let target_dir = pkg_dir.join(&target);
            
            if !source.starts_with("http") {
                // If it's a version number and not a URL, we skip remote fetching for now
                // as there's no central registry. (e.g. `std = "0.1.0"`)
                return target.to_string();
            }
            
            if !target_dir.exists() {
                let _ = fs::create_dir_all(&pkg_dir);
                println!("\x1b[1;36m   Fetching\x1b[0m '{}' from {}...", target, source);
                
                let mut url = source.to_string();
                let mut version = None;
                if let Some(idx) = url.rfind('@') {
                    if idx > url.rfind('/').unwrap_or(0) {
                        version = Some(url[idx + 1..].to_string());
                        url = url[..idx].to_string();
                    }
                }
                let download_url = if let Some(v) = &version {
                    format!("{}/archive/refs/tags/{}.zip", url.trim_end_matches('/'), v)
                } else {
                    format!("{}/archive/refs/heads/main.zip", url.trim_end_matches('/'))
                };

                let download_zip = |u: &str| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                    let client = reqwest::blocking::Client::builder()
                        .user_agent("Wrenlang-Package-Manager")
                        .build()?;
                    let resp = client.get(u).send()?;
                    if resp.status().is_success() {
                        Ok(resp.bytes()?.to_vec())
                    } else {
                        Err(format!("HTTP {}", resp.status()).into())
                    }
                };

                let mut bytes = download_zip(&download_url);
                if bytes.is_err() && version.is_none() {
                    // Fallback to master
                    let fallback = format!("{}/archive/refs/heads/master.zip", url.trim_end_matches('/'));
                    bytes = download_zip(&fallback);
                }

                if let Ok(data) = bytes {
                    let cursor = std::io::Cursor::new(data);
                    if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).unwrap();
                            let outpath = match file.enclosed_name() {
                                Some(path) => path.to_owned(),
                                None => continue,
                            };

                            let mut components = outpath.components();
                            components.next(); // skip root dir
                            let rel_path: std::path::PathBuf = components.collect();
                            
                            if rel_path.as_os_str().is_empty() {
                                continue;
                            }
                            
                            let target_path = target_dir.join(&rel_path);

                            if (*file.name()).ends_with('/') {
                                let _ = fs::create_dir_all(&target_path);
                            } else {
                                if let Some(p) = target_path.parent() {
                                    if !p.exists() {
                                        let _ = fs::create_dir_all(&p);
                                    }
                                }
                                let mut outfile = fs::File::create(&target_path).unwrap();
                                std::io::copy(&mut file, &mut outfile).unwrap();
                            }
                        }
                    }
                } else {
                    println!("\x1b[1;31merror:\x1b[0m failed to fetch plugin '{}'", target);
                }
            }
            target_dir.to_string_lossy().into_owned()
        }
    };

    // Fetch pure Wren dependencies
    for (target, source) in deps {
        fetch_remote(&target, &source);
    }

    // Fetch and compile native dependencies & plugins
    for (target, source) in native_to_compile {
        let plugin_path_str = fetch_remote(&target, &source);
        let plugin_path = Path::new(&plugin_path_str);
        
        if plugin_path.join("Cargo.toml").exists() {
            println!("\x1b[1;36m   Compiling\x1b[0m native plugin '{}'...", target);
            let output = std::process::Command::new("cargo")
                .arg("build")
                .current_dir(plugin_path)
                .output();
            if let Ok(out) = output {
                if !out.status.success() {
                    println!("Failed to compile {}: {}", target, String::from_utf8_lossy(&out.stderr));
                }
            }
            inspect_native_plugin(&target, plugin_path);
        }
    }
}

pub fn inspect_native_plugin(target: &str, plugin_path: &Path) {
    println!(
        "\x1b[1;36m  Inspecting\x1b[0m native Rust plugin '{}' via cargo rustdoc...",
        target
    );

    let output = std::process::Command::new("cargo")
        .args([
            "+nightly",
            "rustdoc",
            "--",
            "--output-format",
            "json",
            "-Zunstable-options",
        ])
        .current_dir(plugin_path)
        .output();

    if let Ok(output) = output {
        if !output.status.success() {
            println!("Rustdoc error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }


    let rustdoc_json_path = plugin_path
        .join("target")
        .join("doc")
        .join(format!("{}.json", target.replace("-", "_")));

    let meta = parse_rustdoc_json(&rustdoc_json_path, target);

    let pkg_dir = Path::new(".wren").join("pkg").join(target);
    let _ = fs::create_dir_all(&pkg_dir);
    if let Ok(meta_str) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(pkg_dir.join(format!("{}.wmeta", target)), meta_str);
    }
}

pub fn parse_rustdoc_json(rustdoc_json_path: &Path, target: &str) -> WrenMeta {
    let mut functions = Vec::new();
    let mut structs = Vec::new();

    if let Ok(json_str) = fs::read_to_string(rustdoc_json_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let paths = v.get("paths").and_then(|p| p.as_object());
            if let Some(index) = v.get("index").and_then(|i| i.as_object()) {
                for (id, item) in index {
                    if item.get("visibility").and_then(|v| v.as_str()) != Some("public") {
                        continue;
                    }
                    let name = match item.get("name").and_then(|n| n.as_str()) {
                        Some(n) => n,
                        None => continue,
                    };
                    if let Some(inner) = item.get("inner").and_then(|i| i.as_object()) {
                        if inner.contains_key("function") {
                            let mut is_top_level = false;
                            if let Some(p_obj) = paths.and_then(|p| p.get(id)).and_then(|p| p.as_object()) {
                                if let Some(path_arr) = p_obj.get("path").and_then(|pa| pa.as_array()) {
                                    if path_arr.len() == 2 {
                                        is_top_level = true;
                                    }
                                }
                            }
                            if !is_top_level {
                                continue;
                            }
                            let mut param_types = vec![];
                            let mut return_type = "NativeObject".to_string();

                            if let Some(sig) = inner
                                .get("function")
                                .and_then(|f| f.as_object())
                                .and_then(|f| f.get("sig"))
                                .and_then(|s| s.as_object())
                            {
                                if let Some(inputs) = sig.get("inputs").and_then(|i| i.as_array()) {
                                    for input in inputs {
                                        if let Some(arr) = input.as_array() {
                                            if arr.len() == 2 {
                                                param_types.push(parse_type(&arr[1]));
                                            }
                                        }
                                    }
                                }
                                if let Some(output) = sig.get("output") {
                                    return_type = parse_type(output);
                                } else {
                                    return_type = "()".to_string();
                                }
                            }
                            
                            let has_unsupported_param = param_types.iter().any(|pt| {
                                let p = pt.to_lowercase();
                                !p.contains("str") && !p.contains("bool") && !p.contains("range") && 
                                p != "i8" && p != "i16" && p != "i32" && p != "i64" && p != "i128" && p != "isize" &&
                                p != "u8" && p != "u16" && p != "u32" && p != "u64" && p != "u128" && p != "usize"
                            });
                            
                            if return_type.contains("(") || param_types.iter().any(|pt| pt.contains("impl ")) || has_unsupported_param {
                                continue;
                            }

                            functions.push(WrenFunctionMeta {
                                name: name.to_string(),
                                wren_name: name.to_string(),
                                params: param_types
                                    .iter()
                                    .enumerate()
                                    .map(|(i, pt)| WrenParamMeta {
                                        name: format!("arg{}", i),
                                        type_name: pt.clone(),
                                    })
                                    .collect(),
                                return_type: return_type.clone(),
                                is_static: true,
                                docs: item
                                    .get("docs")
                                    .and_then(|d| d.as_str())
                                    .map(|d| d.trim().to_string())
                                    .filter(|d| !d.is_empty()),
                            });
                        } else if inner.contains_key("struct") {
                            if name == "Hyphenated" || name == "Simple" || name == "Urn" || name == "Braced" || name == "ThreadLocalContext" {
                                continue;
                            }
                            let mut s_methods = vec![];
                            if let Some(impls) = inner.get("struct").and_then(|s| s.get("impls")).and_then(|i| i.as_array()) {
                                for impl_id in impls {
                                    let impl_id_str = if let Some(s) = impl_id.as_str() { s.to_string() } else { impl_id.to_string() };
                                    if let Some(impl_item) = index.get(&impl_id_str).and_then(|i| i.as_object()) {
                                        if let Some(impl_inner) = impl_item.get("inner").and_then(|i| i.as_object()) {
                                                if let Some(impl_block) = impl_inner.get("impl").and_then(|i| i.as_object()) {
                                                    if let Some(items) = impl_block.get("items").and_then(|i| i.as_array()) {
                                                        for m_id in items {
                                                            let m_id_str = if let Some(s) = m_id.as_str() { s.to_string() } else { m_id.to_string() };
                                                            if let Some(m_item) = index.get(&m_id_str).and_then(|i| i.as_object()) {
                                                                if m_item.get("visibility").and_then(|v| v.as_str()) != Some("public") {
                                                                    continue;
                                                                }
                                                                if let Some(m_name) = m_item.get("name").and_then(|n| n.as_str()) {
                                                                    if let Some(m_inner) = m_item.get("inner").and_then(|i| i.as_object()) {
                                                                        if m_inner.contains_key("function") {
                                                                                let mut m_param_types = vec![];
                                                                                let mut m_return_type = "NativeObject".to_string();
                                                                                
                                                                                let mut is_static = true;
                                                                                let mut consumes_self = false;
                                                                                if let Some(sig) = m_inner.get("function").and_then(|f| f.as_object()).and_then(|f| f.get("sig")).and_then(|s| s.as_object()) {
                                                                                    if let Some(inputs) = sig.get("inputs").and_then(|i| i.as_array()) {
                                                                                        for input in inputs {
                                                                                            if let Some(arr) = input.as_array() {
                                                                                                if arr.len() == 2 {
                                                                                                    let param_name = arr[0].as_str().unwrap_or_default();
                                                                                                    if param_name == "self" {
                                                                                                        is_static = false;
                                                                                                        if let Some(g) = arr[1].as_object().and_then(|o| o.get("generic")).and_then(|v| v.as_str()) {
                                                                                                            if g == "Self" { consumes_self = true; }
                                                                                                        }
                                                                                                        continue;
                                                                                                    }
                                                                                                    m_param_types.push(parse_type(&arr[1]));
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    if let Some(output) = sig.get("output") {
                                                                                        m_return_type = parse_type(output);
                                                                                    } else {
                                                                                        m_return_type = "()".to_string();
                                                                                    }
                                                                                }
                                                                                let has_unsupported_param = m_param_types.iter().any(|pt| {
                                                                                    let p = pt.to_lowercase();
                                                                                    !p.contains("str") && !p.contains("bool") && !p.contains("range") && 
                                                                                    p != "i8" && p != "i16" && p != "i32" && p != "i64" && p != "i128" && p != "isize" &&
                                                                                    p != "u8" && p != "u16" && p != "u32" && p != "u64" && p != "u128" && p != "usize"
                                                                                });
                                                                                if consumes_self || m_return_type.contains("(") || m_param_types.iter().any(|pt| pt.contains("impl ")) || has_unsupported_param {
                                                                                    continue;
                                                                                }
                                                                                
                                                                                s_methods.push(WrenFunctionMeta {
                                                                                    name: m_name.to_string(),
                                                                                    wren_name: m_name.to_string(),
                                                                                    params: m_param_types.iter().enumerate().map(|(idx, pt)| WrenParamMeta {
                                                                                        name: format!("arg{}", idx),
                                                                                        type_name: pt.clone(),
                                                                                    }).collect(),
                                                                                    return_type: m_return_type,
                                                                                    is_static,
                                                                                    docs: m_item
                                                                                        .get("docs")
                                                                                        .and_then(|d| d.as_str())
                                                                                        .map(|d| d.trim().to_string())
                                                                                        .filter(|d| !d.is_empty()),
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
                                        }
                                    }
                            structs.push(WrenStructMeta {
                                name: name.to_string(),
                                methods: s_methods,
                                docs: item
                                    .get("docs")
                                    .and_then(|d| d.as_str())
                                    .map(|d| d.trim().to_string())
                                    .filter(|d| !d.is_empty()),
                            });
                        }
                    }
                }
            }
        }
    } else {
        println!(
            "WARNING: Rustdoc generation failed for '{}'. Is cargo +nightly installed?",
            target
        );
    }

    let lib_filename = if cfg!(target_os = "windows") {
        format!("{}.dll", target)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", target)
    } else {
        format!("lib{}.so", target)
    };

    WrenMeta {
        module: target.to_string(),
        kind: "native".to_string(),
        lib: Some(lib_filename),
        functions,
        structs,
        docs: None,
    }
}

fn parse_type(ty: &serde_json::Value) -> String {
    if let Some(prim) = ty.get("primitive").and_then(|p| p.as_str()) {
        return prim.to_string();
    }
    if let Some(res) = ty.get("resolved_path").and_then(|p| p.as_object()) {
        if let Some(name) = res.get("name").or_else(|| res.get("path")).and_then(|n| n.as_str()) {
            return name.to_string();
        }
    }
    if ty.get("tuple").is_some() {
        return "(tuple)".to_string();
    }
    if ty.get("impl_trait").is_some() {
        return "impl trait".to_string();
    }
    if let Some(generic) = ty.get("generic").and_then(|g| g.as_str()) {
        if generic == "Self" {
            return "Self".to_string();
        }
    }
    "NativeObject".to_string()
}
