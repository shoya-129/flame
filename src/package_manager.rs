use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameParamMeta {
    pub name: String,
    pub type_name: String,
    #[serde(default)]
    pub is_callback: bool,
    #[serde(default)]
    pub is_ref: bool,
    #[serde(default)]
    pub is_mut: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameFunctionMeta {
    pub name: String,
    pub flame_name: String,
    pub params: Vec<FlameParamMeta>,
    pub return_type: String,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_generic: bool,
    #[serde(default)]
    pub is_async: bool,
    #[serde(default)]
    pub is_constructor: bool,
    #[serde(default)]
    pub persistent_runtime: bool,
    #[serde(default)]
    pub receiver: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameStructMeta {
    pub name: String,
    pub methods: Vec<FlameFunctionMeta>,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameMeta {
    pub module: String,
    pub kind: String,
    pub lib: Option<String>,
    pub functions: Vec<FlameFunctionMeta>,
    pub structs: Vec<FlameStructMeta>,
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

pub fn parse_manifest_permissions(content: &str) -> std::collections::HashSet<String> {
    let mut perms = std::collections::HashSet::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[permissions]" {
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
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim().to_string();
            let val = trimmed[eq_idx+1..].trim();
            if val == "true" {
                perms.insert(key);
            }
        } else {
            perms.insert(trimmed.to_string());
        }
    }
    perms
}

pub fn list_plugins() -> Vec<PluginSpec> {
    let toml_path = Path::new("flame.toml");
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

#[cfg(feature = "cli")]
pub fn add_package(args: &[String]) {
    if args.is_empty() {
        println!("\x1b[1;31merror:\x1b[0m please specify package name to add.");
        println!(
            "usage: flame add <package_name> | flame add --plugin <path> --name <plugin_name>"
        );
        return;
    }

    let is_plugin = args.contains(&"--plugin".to_string()) || args.contains(&"@plugin".to_string());
    let is_native = args.contains(&"--native".to_string());

    let (manifest_key, manifest_value, section) = if is_plugin {
        let plugin_idx = match args.iter().position(|r| r == "--plugin" || r == "@plugin") {
            Some(idx) => idx,
            None => {
                println!(
                    "\x1b[1;31merror:\x1b[0m --plugin requires a file path argument (e.g. --plugin ./native)."
                );
                return;
            }
        };
        let plugin_path = match args.get(plugin_idx + 1) {
            Some(p) if !p.starts_with("--") => p.clone(),
            _ => {
                println!(
                    "\x1b[1;31merror:\x1b[0m --plugin requires a valid file path argument (e.g. --plugin ./native)."
                );
                return;
            }
        };
        let plugin_name = if let Some(name_idx) = args.iter().position(|r| r == "--name") {
            match args.get(name_idx + 1) {
                Some(n) if !n.starts_with("--") => n.clone(),
                _ => {
                    println!(
                        "\x1b[1;31merror:\x1b[0m --name requires a valid name argument (e.g. --name server)."
                    );
                    return;
                }
            }
        } else {
            let cargo_toml_path = Path::new(&plugin_path).join("Cargo.toml");
            let mut extracted = None;
            if cargo_toml_path.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("name") && trimmed.contains('=') {
                            if let Some(val) = trimmed.split('=').nth(1) {
                                let clean = val.trim().trim_matches('"').trim_matches('\'');
                                if !clean.is_empty() {
                                    extracted = Some(clean.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            extracted.unwrap_or_else(|| {
                Path::new(&plugin_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("plugin")
                    .to_string()
            })
        };
        (plugin_name, plugin_path, "[plugins]")
    } else {
        let raw_target = &args[0];
        let name = raw_target
            .rsplit('/')
            .next()
            .unwrap_or(raw_target)
            .split('@')
            .next()
            .unwrap_or(raw_target)
            .trim_end_matches(".git")
            .to_string();
        let val = if raw_target == &name {
            "*".to_string()
        } else {
            raw_target.clone()
        };
        let sec = if is_native {
            "[native-dependencies]"
        } else {
            "[dependencies]"
        };
        (name, val, sec)
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

    let toml_path = Path::new("flame.toml");
    if toml_path.exists() {
        let mut content = fs::read_to_string(toml_path).unwrap_or_default();
        if !content.contains(&format!("{} =", manifest_key))
            && !content.contains(&format!("{}=", manifest_key))
        {
            if !content.contains(section) {
                content.push_str(&format!("\n{}\n", section));
                content.push_str(&format!("{} = \"{}\"\n", manifest_key, manifest_value));
            } else {
                if let Some(idx) = content.find(section) {
                    let insert_pos = idx + section.len();
                    content.insert_str(
                        insert_pos,
                        &format!("\n{} = \"{}\"", manifest_key, manifest_value),
                    );
                }
            }
            let _ = fs::write(toml_path, content);
        }
    }

    // Perform git cloning for github.com URLs
    if !is_plugin && manifest_value.starts_with("github.com/") {
        let mut parts = manifest_value.split('@');
        let repo_url = format!("https://{}", parts.next().unwrap());
        let version = parts.next();

        let pkg_dir = Path::new(".flame").join("pkg").join(&manifest_key);
        if pkg_dir.exists() {
            println!("\x1b[1;33m   Warning:\x1b[0m package '{}' is already downloaded. To update, remove it first.", manifest_key);
        } else {
            println!(
                "\x1b[1;36m   Fetching\x1b[0m {} from {}...",
                manifest_key, repo_url
            );
            let _ = fs::create_dir_all(".flame/pkg");

            let mut cmd = std::process::Command::new("git");
            cmd.arg("clone").arg(&repo_url).arg(&pkg_dir);
            if let Some(tag) = version {
                cmd.arg("--branch").arg(tag);
            }

            let result = cmd.output();
            if let Ok(output) = result {
                if !output.status.success() {
                    println!("\x1b[1;31merror:\x1b[0m failed to clone repository. Make sure git is installed and the repository exists.");
                    if let Ok(err_str) = String::from_utf8(output.stderr) {
                        println!("{}", err_str);
                    }
                }
            } else {
                println!("\x1b[1;31merror:\x1b[0m failed to execute git clone command.");
            }
        }
    }

    if is_plugin {
        let local_plugin = Path::new(&manifest_value);
        if local_plugin.join("Cargo.toml").exists() {
            println!(
                "\x1b[1;36m   Compiling\x1b[0m native plugin '{}'...",
                manifest_key
            );
            let _ = std::process::Command::new("cargo")
                .arg("build")
                .current_dir(local_plugin)
                .output();
            inspect_native_plugin(&manifest_key, local_plugin);
        } else {
            println!(
                "\x1b[1;33m   Warning:\x1b[0m local plugin path '{}' does not contain Cargo.toml yet.",
                manifest_value
            );
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

#[cfg(feature = "cli")]
pub fn remove_package(pkg_name: &str) {
    let toml_path = Path::new("flame.toml");
    if toml_path.exists() {
        if let Ok(content) = fs::read_to_string(toml_path) {
            let lines: Vec<&str> = content
                .lines()
                .filter(|line| !line.trim().starts_with(&format!("{} =", pkg_name)))
                .collect();
            let _ = fs::write(toml_path, lines.join("\n"));
        }
    }

    let pkg_dir = Path::new(".flame").join("pkg").join(pkg_name);
    if pkg_dir.exists() {
        let _ = fs::remove_dir_all(pkg_dir);
    }
    println!("\x1b[1;32m     Removed\x1b[0m package '{}'", pkg_name);
}

#[cfg(feature = "cli")]
pub fn ensure_dependencies_installed(is_release: bool) {
    // Compile std_bridge directly since it uses #[flame_export] now
    let std_bridge_path = Path::new("flame-stdlib").join("native").join("std_bridge");
    if std_bridge_path.exists() {
        let profile_dir = if is_release { "release" } else { "debug" };
        let lib_out = std_bridge_path
            .join("target")
            .join(profile_dir)
            .join("libstd_bridge.rlib");
        let mut needs_build = true;
        if lib_out.exists() {
            if let Ok(out_meta) = fs::metadata(&lib_out) {
                if let Ok(out_mtime) = out_meta.modified() {
                    if let Ok(src_meta) = fs::metadata(std_bridge_path.join("src").join("lib.rs")) {
                        if let Ok(src_mtime) = src_meta.modified() {
                            if out_mtime > src_mtime {
                                needs_build = false;
                            }
                        }
                    }
                }
            }
        }
        if needs_build {
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            if is_release {
                cmd.arg("--release");
            }
            let _ = cmd.current_dir(&std_bridge_path).output();
        }
    }
    let toml_path = Path::new("flame.toml");
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
            if source == "*" {
                target.to_string()
            } else {
                source.to_string()
            }
        } else {
            let pkg_dir = Path::new(".flame").join("pkg");
            let target_dir = pkg_dir.join(&target);

            if !source.starts_with("http") {
                // If it's a version number and not a URL, we skip remote fetching for now
                // as there's no central registry. (e.g. `std = "0.1.0"`)
                return target.to_string();
            }

            if !target_dir.exists() {
                let _ = fs::create_dir_all(&pkg_dir);
                println!(
                    "\x1b[1;36m   Fetching\x1b[0m '{}' from {}...",
                    target, source
                );

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
                        .user_agent("Flamelang-Package-Manager")
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
                    let fallback = format!(
                        "{}/archive/refs/heads/master.zip",
                        url.trim_end_matches('/')
                    );
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
                    println!(
                        "\x1b[1;31merror:\x1b[0m failed to fetch plugin '{}'",
                        target
                    );
                }
            }
            target_dir.to_string_lossy().into_owned()
        }
    };

    // Fetch pure Flame dependencies
    for (target, source) in deps {
        fetch_remote(&target, &source);
    }

    // Fetch and compile native dependencies & plugins
    for (target, source) in native_to_compile {
        let plugin_path_str = fetch_remote(&target, &source);
        let plugin_path = Path::new(&plugin_path_str);

        if plugin_path.join("Cargo.toml").exists() {
            println!(
                "\x1b[1;36m   Compiling\x1b[0m native plugin '{}' ({})...",
                target,
                if is_release {
                    "release [optimized]"
                } else {
                    "dev [unoptimized]"
                }
            );
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            if is_release {
                cmd.arg("--release");
            }
            let output = cmd.current_dir(plugin_path).output();
            if let Ok(out) = output {
                if !out.status.success() {
                    println!(
                        "Failed to compile {}: {}",
                        target,
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
            }
            inspect_native_plugin(&target, plugin_path);
        }
    }
}

pub fn inspect_native_plugin(target: &str, plugin_path: &Path) {
    let pkg_dir = Path::new(".flame").join("pkg").join(target);
    let fmi_path = pkg_dir.join(format!("{}.fmi", target));

    let mut needs_update = true;
    if fmi_path.exists() {
        if let Ok(fmi_meta) = fs::metadata(&fmi_path) {
            if let Ok(fmi_mtime) = fmi_meta.modified() {
                needs_update = false;

                // Check src/lib.rs
                if let Ok(src_meta) = fs::metadata(plugin_path.join("src").join("lib.rs")) {
                    if let Ok(src_mtime) = src_meta.modified() {
                        if src_mtime > fmi_mtime {
                            needs_update = true;
                        }
                    }
                }

                // Check Cargo.toml
                if let Ok(cargo_meta) = fs::metadata(plugin_path.join("Cargo.toml")) {
                    if let Ok(cargo_mtime) = cargo_meta.modified() {
                        if cargo_mtime > fmi_mtime {
                            needs_update = true;
                        }
                    }
                }
            }
        }
    }

    if !needs_update {
        return;
    }

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

    let mut meta = parse_rustdoc_json(&rustdoc_json_path, target);
    enrich_with_syn(&mut meta, plugin_path);

    let pkg_dir = Path::new(".flame").join("pkg").join(target);
    let _ = fs::create_dir_all(&pkg_dir);
    if let Ok(meta_str) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(pkg_dir.join(format!("{}.fmi", target)), meta_str);
    }
}

pub fn parse_rustdoc_json(rustdoc_json_path: &Path, target: &str) -> FlameMeta {
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
                            if let Some(p_obj) =
                                paths.and_then(|p| p.get(id)).and_then(|p| p.as_object())
                            {
                                if let Some(path_arr) =
                                    p_obj.get("path").and_then(|pa| pa.as_array())
                                {
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
                            let mut is_generic = false;
                            let mut is_async = false;
                            let mut has_bounds = false;
                            if let Some(func_obj) =
                                inner.get("function").and_then(|f| f.as_object())
                            {
                                if let Some(generics) =
                                    func_obj.get("generics").and_then(|g| g.as_object())
                                {
                                    if let Some(params) =
                                        generics.get("params").and_then(|p| p.as_array())
                                    {
                                        if !params.is_empty() {
                                            if params.len() == 1 {
                                                if let Some(kind) = params[0]
                                                    .get("kind")
                                                    .and_then(|k| k.as_object())
                                                {
                                                    if let Some(type_obj) =
                                                        kind.get("type").and_then(|t| t.as_object())
                                                    {
                                                        if let Some(bounds) = type_obj
                                                            .get("bounds")
                                                            .and_then(|b| b.as_array())
                                                        {
                                                            if !bounds.is_empty() {
                                                                has_bounds = true;
                                                            }
                                                        }
                                                    }
                                                }
                                                is_generic = true;
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }
                                if has_bounds {
                                    continue;
                                }
                                if let Some(header) =
                                    func_obj.get("header").and_then(|h| h.as_object())
                                {
                                    is_async = header
                                        .get("is_async")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                }
                                if let Some(sig) = func_obj.get("sig").and_then(|s| s.as_object()) {
                                    if let Some(inputs) =
                                        sig.get("inputs").and_then(|i| i.as_array())
                                    {
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
                            }

                            if let Some(func_obj) =
                                inner.get("function").and_then(|f| f.as_object())
                            {
                                if let Some(generics) =
                                    func_obj.get("generics").and_then(|g| g.as_object())
                                {
                                    if let Some(params) =
                                        generics.get("params").and_then(|p| p.as_array())
                                    {
                                        if !params.is_empty() {
                                            is_generic = true;
                                        }
                                    }
                                }
                            }

                            let is_constructor = return_type == target;
                            functions.push(FlameFunctionMeta {
                                name: name.to_string(),
                                flame_name: name.to_string(),
                                is_generic,
                                is_async,
                                is_constructor,
                                requires: vec![],
                                permissions: vec![],
                                persistent_runtime: false,
                                receiver: None,
                                params: param_types
                                    .iter()
                                    .enumerate()
                                    .map(|(i, pt)| FlameParamMeta {
                                        name: format!("arg{}", i),
                                        type_name: pt.clone(),
                                        is_callback: pt == "Callback" || pt == "FlameCallback",
                                        is_ref: false,
                                        is_mut: false,
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
                            if name == "Hyphenated"
                                || name == "Simple"
                                || name == "Urn"
                                || name == "Braced"
                                || name == "ThreadLocalContext"
                                || name == "WeightedIndex"
                                || name == "Bernoulli"
                                || name == "StepRng"
                                || name == "ReseedingRng"
                                || name == "Choose"
                                || name == "ThreadRng"
                            {
                                continue;
                            }
                            let mut s_methods = vec![];
                            if let Some(impls) = inner
                                .get("struct")
                                .and_then(|s| s.get("impls"))
                                .and_then(|i| i.as_array())
                            {
                                for impl_id in impls {
                                    let impl_id_str = if let Some(s) = impl_id.as_str() {
                                        s.to_string()
                                    } else {
                                        impl_id.to_string()
                                    };
                                    if let Some(impl_item) =
                                        index.get(&impl_id_str).and_then(|i| i.as_object())
                                    {
                                        if let Some(impl_inner) =
                                            impl_item.get("inner").and_then(|i| i.as_object())
                                        {
                                            if let Some(impl_block) =
                                                impl_inner.get("impl").and_then(|i| i.as_object())
                                            {
                                                if let Some(items) = impl_block
                                                    .get("items")
                                                    .and_then(|i| i.as_array())
                                                {
                                                    for m_id in items {
                                                        let m_id_str =
                                                            if let Some(s) = m_id.as_str() {
                                                                s.to_string()
                                                            } else {
                                                                m_id.to_string()
                                                            };
                                                        if let Some(m_item) = index
                                                            .get(&m_id_str)
                                                            .and_then(|i| i.as_object())
                                                        {
                                                            if m_item
                                                                .get("visibility")
                                                                .and_then(|v| v.as_str())
                                                                != Some("public")
                                                            {
                                                                continue;
                                                            }
                                                            if let Some(m_name) = m_item
                                                                .get("name")
                                                                .and_then(|n| n.as_str())
                                                            {
                                                                if let Some(m_inner) = m_item
                                                                    .get("inner")
                                                                    .and_then(|i| i.as_object())
                                                                {
                                                                    if m_inner
                                                                        .contains_key("function")
                                                                    {
                                                                        let mut m_param_types =
                                                                            vec![];
                                                                        let mut m_return_type =
                                                                            "NativeObject"
                                                                                .to_string();

                                                                        let mut is_static = true;
                                                                        let mut consumes_self =
                                                                            false;
                                                                        let mut is_generic = false;
                                                                        let mut is_async = false;
                                                                        if let Some(func_obj) =
                                                                            m_inner
                                                                                .get("function")
                                                                                .and_then(|f| {
                                                                                    f.as_object()
                                                                                })
                                                                        {
                                                                            if let Some(generics) =
                                                                                func_obj
                                                                                    .get("generics")
                                                                                    .and_then(|g| {
                                                                                        g.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    params,
                                                                                ) = generics
                                                                                    .get("params")
                                                                                    .and_then(|p| {
                                                                                        p.as_array()
                                                                                    })
                                                                                {
                                                                                    if !params
                                                                                        .is_empty()
                                                                                    {
                                                                                        is_generic = true;
                                                                                    }
                                                                                }
                                                                            }
                                                                            if let Some(header) =
                                                                                func_obj
                                                                                    .get("header")
                                                                                    .and_then(|h| {
                                                                                        h.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                is_async = header
                                                                                    .get("is_async")
                                                                                    .and_then(|v| {
                                                                                        v.as_bool()
                                                                                    })
                                                                                    .unwrap_or(
                                                                                        false,
                                                                                    );
                                                                            }
                                                                            if let Some(sig) =
                                                                                func_obj
                                                                                    .get("sig")
                                                                                    .and_then(|s| {
                                                                                        s.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    inputs,
                                                                                ) = sig
                                                                                    .get("inputs")
                                                                                    .and_then(|i| {
                                                                                        i.as_array()
                                                                                    })
                                                                                {
                                                                                    for input in
                                                                                        inputs
                                                                                    {
                                                                                        if let Some(
                                                                                        arr,
                                                                                    ) = input
                                                                                        .as_array()
                                                                                    {
                                                                                        if arr.len()
                                                                                            == 2
                                                                                        {
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
                                                                                if let Some(
                                                                                    output,
                                                                                ) = sig
                                                                                    .get("output")
                                                                                {
                                                                                    m_return_type =
                                                                                        parse_type(
                                                                                            output,
                                                                                        );
                                                                                } else {
                                                                                    m_return_type =
                                                                                    "()".to_string(
                                                                                    );
                                                                                }
                                                                            }
                                                                        }
                                                                        if let Some(func_obj) =
                                                                            m_inner
                                                                                .get("function")
                                                                                .and_then(|f| {
                                                                                    f.as_object()
                                                                                })
                                                                        {
                                                                            if let Some(generics) =
                                                                                func_obj
                                                                                    .get("generics")
                                                                                    .and_then(|g| {
                                                                                        g.as_object(
                                                                                        )
                                                                                    })
                                                                            {
                                                                                if let Some(
                                                                                    params,
                                                                                ) = generics
                                                                                    .get("params")
                                                                                    .and_then(|p| {
                                                                                        p.as_array()
                                                                                    })
                                                                                {
                                                                                    if !params
                                                                                        .is_empty()
                                                                                    {
                                                                                        is_generic = true;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }

                                                                        let receiver = if !is_static
                                                                        {
                                                                            if consumes_self {
                                                                                Some(
                                                                                    "self"
                                                                                        .to_string(
                                                                                        ),
                                                                                )
                                                                            } else {
                                                                                Some(
                                                                                    "&mut self"
                                                                                        .to_string(
                                                                                        ),
                                                                                )
                                                                            }
                                                                        } else {
                                                                            None
                                                                        };
                                                                        let is_constructor =
                                                                            is_static
                                                                                && (m_return_type
                                                                                    == "Self"
                                                                                    || m_return_type
                                                                                        == name);
                                                                        s_methods.push(FlameFunctionMeta {
                                                                            name: m_name.to_string(),
                                                                            flame_name: m_name.to_string(),
                                                                            is_generic,
                                                                            is_async,
                                                                            is_constructor,
                                                                            requires: vec![],
                                                                            permissions: vec![],
                                                                            persistent_runtime: false,
                                                                            receiver,
                                                                            params: m_param_types.iter().enumerate().map(|(idx, pt)| FlameParamMeta {
                                                                                name: format!("arg{}", idx),
                                                                                type_name: pt.clone(),
                                                                                is_callback: pt == "Callback" || pt == "FlameCallback",
                                                                                is_ref: false,
                                                                                is_mut: false,
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
                            structs.push(FlameStructMeta {
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

    FlameMeta {
        module: target.to_string(),
        kind: "native".to_string(),
        lib: Some(lib_filename),
        functions,
        structs,
        docs: None,
    }
}

fn parse_type(ty: &serde_json::Value) -> String {
    if let Some(bref) = ty.get("borrowed_ref").and_then(|b| b.as_object()) {
        if let Some(inner) = bref.get("type") {
            let mut ty_str = String::from("&");
            if let Some(lt) = bref.get("lifetime").and_then(|l| l.as_str()) {
                if lt == "'static" {
                    ty_str.push_str("'static ");
                }
            }
            if bref
                .get("mutable")
                .and_then(|m| m.as_bool())
                .unwrap_or(false)
            {
                ty_str.push_str("mut ");
            }
            ty_str.push_str(&parse_type(inner));
            return ty_str;
        }
    }
    if let Some(prim) = ty.get("primitive").and_then(|p| p.as_str()) {
        return prim.to_string();
    }
    if let Some(res) = ty.get("resolved_path").and_then(|p| p.as_object()) {
        if let Some(name) = res
            .get("name")
            .or_else(|| res.get("path"))
            .and_then(|n| n.as_str())
        {
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
        return generic.to_string();
    }
    "NativeObject".to_string()
}

pub fn enrich_with_syn(meta: &mut FlameMeta, plugin_path: &Path) {
    let src_dir = plugin_path.join("src");
    if !src_dir.exists() {
        return;
    }

    let mut files_to_scan = Vec::new();
    if let Ok(entries) = fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                files_to_scan.push(p);
            }
        }
    }

    for file_path in files_to_scan {
        if let Ok(code) = fs::read_to_string(&file_path) {
            if let Ok(syntax_tree) = syn::parse_file(&code) {
                for item in syntax_tree.items {
                    match item {
                        syn::Item::Fn(fn_item) => {
                            let (rename, skip, constructor, persistent_runtime) =
                                parse_flame_attrs(&fn_item.attrs);
                            if skip || !matches!(fn_item.vis, syn::Visibility::Public(_)) {
                                continue;
                            }
                            let fn_name = fn_item.sig.ident.to_string();
                            let flame_name = rename.unwrap_or_else(|| fn_name.clone());
                            let is_async = fn_item.sig.asyncness.is_some();
                            let return_type = parse_syn_return(&fn_item.sig.output);
                            let is_constructor = constructor || return_type == meta.module;
                            let params =
                                parse_syn_params(&fn_item.sig.inputs, &fn_item.sig.generics);

                            if let Some(existing) =
                                meta.functions.iter_mut().find(|f| f.name == fn_name)
                            {
                                existing.flame_name = flame_name;
                                existing.is_async = is_async;
                                existing.is_constructor = is_constructor;
                                existing.persistent_runtime = persistent_runtime;
                                existing.params = params;
                                existing.return_type = return_type;
                            } else {
                                meta.functions.push(FlameFunctionMeta {
                                    name: fn_name,
                                    flame_name,
                                    params,
                                    return_type,
                                    is_static: true,
                                    is_generic: !fn_item.sig.generics.params.is_empty(),
                                    requires: vec![],
                                    permissions: vec![],
                                    is_async,
                                    is_constructor,
                                    persistent_runtime,
                                    receiver: None,
                                    docs: None,
                                });
                            }
                        }
                        syn::Item::Struct(struct_item) => {
                            let (_, skip, _, _) = parse_flame_attrs(&struct_item.attrs);
                            if skip || !matches!(struct_item.vis, syn::Visibility::Public(_)) {
                                continue;
                            }
                            let struct_name = struct_item.ident.to_string();
                            if !meta.structs.iter().any(|s| s.name == struct_name) {
                                meta.structs.push(FlameStructMeta {
                                    name: struct_name,
                                    methods: Vec::new(),
                                    docs: None,
                                });
                            }
                        }
                        syn::Item::Impl(impl_item) => {
                            let struct_name = quote::quote!(#impl_item.self_ty)
                                .to_string()
                                .replace(" ", "");
                            let struct_name_simple = struct_name
                                .rsplit("::")
                                .next()
                                .unwrap_or(&struct_name)
                                .split('<')
                                .next()
                                .unwrap_or(&struct_name)
                                .to_string();
                            if let Some(struct_meta) = meta.structs.iter_mut().find(|s| {
                                s.name == struct_name
                                    || s.name == struct_name_simple
                                    || struct_name.ends_with(&s.name)
                            }) {
                                for item_in_impl in impl_item.items {
                                    if let syn::ImplItem::Fn(method_item) = item_in_impl {
                                        let (rename, skip, constructor, persistent_runtime) =
                                            parse_flame_attrs(&method_item.attrs);
                                        if skip {
                                            continue;
                                        }
                                        let is_pub =
                                            matches!(method_item.vis, syn::Visibility::Public(_));
                                        if !is_pub {
                                            continue;
                                        }

                                        let m_name = method_item.sig.ident.to_string();
                                        let flame_name = rename.unwrap_or_else(|| m_name.clone());
                                        let is_async = method_item.sig.asyncness.is_some();
                                        let return_type = parse_syn_return(&method_item.sig.output);

                                        let mut receiver = None;
                                        let mut is_static = true;
                                        if let Some(first_arg) = method_item.sig.inputs.first() {
                                            if let syn::FnArg::Receiver(rec) = first_arg {
                                                is_static = false;
                                                let rec_str = quote::quote!(#rec).to_string();
                                                if rec_str.contains("mut") {
                                                    receiver = Some("&mut self".to_string());
                                                } else if rec_str.contains('&') {
                                                    receiver = Some("&self".to_string());
                                                } else {
                                                    receiver = Some("self".to_string());
                                                }
                                            }
                                        }

                                        let is_constructor = constructor
                                            || (is_static
                                                && (return_type == "Self"
                                                    || return_type == struct_name));
                                        let params = parse_syn_params(
                                            &method_item.sig.inputs,
                                            &method_item.sig.generics,
                                        );

                                        if let Some(existing) = struct_meta
                                            .methods
                                            .iter_mut()
                                            .find(|m| m.name == m_name)
                                        {
                                            existing.flame_name = flame_name;
                                            existing.is_async = is_async;
                                            existing.is_constructor = is_constructor;
                                            existing.receiver = receiver;
                                            existing.persistent_runtime = persistent_runtime;
                                            existing.params = params;
                                            existing.return_type = return_type;
                                            existing.is_static = is_static;
                                        } else {
                                            struct_meta.methods.push(FlameFunctionMeta {
                                                name: m_name,
                                                flame_name,
                                                params,
                                                return_type,
                                                is_static,
                                                is_generic: !method_item
                                                    .sig
                                                    .generics
                                                    .params
                                                    .is_empty(),
                                                requires: vec![],
                                                permissions: vec![],
                                                is_async,
                                                is_constructor,
                                                persistent_runtime,
                                                receiver,
                                                docs: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn parse_flame_attrs(attrs: &[syn::Attribute]) -> (Option<String>, bool, bool, bool) {
    let mut rename = None;
    let mut skip = false;
    let mut constructor = false;
    let mut persistent_runtime = false;
    for attr in attrs {
        if attr.path().is_ident("flame") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("constructor") {
                    constructor = true;
                } else if meta.path.is_ident("runtime") || meta.path.is_ident("daemon") {
                    persistent_runtime = true;
                } else if meta.path.is_ident("rename") {
                    if let Ok(value) = meta.value() {
                        if let Ok(s) = value.parse::<syn::LitStr>() {
                            rename = Some(s.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }
    (rename, skip, constructor, persistent_runtime)
}

fn parse_syn_return(output: &syn::ReturnType) -> String {
    match output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => {
            let ty_str = quote::quote!(#ty).to_string();
            ty_str.replace(" ", "")
        }
    }
}

fn parse_syn_params(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    generics: &syn::Generics,
) -> Vec<FlameParamMeta> {
    let mut params = Vec::new();
    let callback_generics = find_callback_generics(generics);

    for input in inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let name = match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => format!("arg{}", params.len()),
            };
            let ty_node = &*pat_type.ty;
            let ty_str = quote::quote!(#ty_node).to_string();
            let clean_ty = ty_str.replace(" ", "");
            let is_ref = clean_ty.starts_with('&') && !clean_ty.starts_with("&mut");
            let is_mut = clean_ty.starts_with("&mut") || clean_ty.starts_with("mut");
            let is_callback = clean_ty.contains("Callback")
                || clean_ty.contains("Handler")
                || clean_ty.contains("Fn(")
                || callback_generics.contains(&clean_ty);

            params.push(FlameParamMeta {
                name,
                type_name: clean_ty,
                is_callback,
                is_ref,
                is_mut,
            });
        }
    }
    params
}

fn find_callback_generics(generics: &syn::Generics) -> Vec<String> {
    let mut cb_generics = Vec::new();
    for param in &generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let ident = type_param.ident.to_string();
            for bound in &type_param.bounds {
                let b_str = quote::quote!(#bound).to_string();
                if b_str.contains("Handler") || b_str.contains("Fn") || b_str.contains("Callback") {
                    cb_generics.push(ident.clone());
                }
            }
        }
    }
    if let Some(where_clause) = &generics.where_clause {
        for pred in &where_clause.predicates {
            if let syn::WherePredicate::Type(pred_type) = pred {
                let target = quote::quote!(#pred_type.bounded_ty).to_string();
                for bound in &pred_type.bounds {
                    let b_str = quote::quote!(#bound).to_string();
                    if b_str.contains("Handler")
                        || b_str.contains("Fn")
                        || b_str.contains("Callback")
                    {
                        cb_generics.push(target.replace(" ", ""));
                    }
                }
            }
        }
    }
    cb_generics
}
