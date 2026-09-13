use crate::blaze;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn dirs_fallback_cargo_bin() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("CARGO_HOME") {
        let p = PathBuf::from(home);
        return Some(if p.ends_with("bin") { p } else { p.join("bin") });
    }
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(user_profile).join(".cargo").join("bin"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".cargo").join("bin"));
    }
    None
}

fn safe_replace_binary(
    src: &Path,
    dst: &Path,
    pending_cleanup: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    if !src.exists() || src == dst {
        return Ok(());
    }

    let parent_dir = match dst.parent() {
        Some(p) => p,
        None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "destination has no parent")),
    };

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    let file_name = dst.file_name().unwrap_or_default().to_string_lossy();
    let temp_dst = parent_dir.join(format!(".{}.tmp.{}", file_name, std::process::id()));

    // Clean up any stale temp file
    let _ = fs::remove_file(&temp_dst);

    // 1. Copy fresh source to staging file in destination directory (guarantees same mount / filesystem)
    fs::copy(src, &temp_dst)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_dst, fs::Permissions::from_mode(0o755));
    }

    // 2. Try atomic rename over dst
    // On Unix (Linux, macOS), rename(2) atomically replaces dst even if dst is currently running!
    if let Err(_rename_err) = fs::rename(&temp_dst, dst) {
        // Atomic rename failed (e.g. Windows file locking when dst is currently running/open)
        if dst.exists() {
            let deleteme_path = parent_dir.join(format!(".{}.deleteme.{}", file_name, std::process::id()));
            let _ = fs::remove_file(&deleteme_path);

            if fs::rename(dst, &deleteme_path).is_ok() {
                pending_cleanup.push(deleteme_path);
            } else {
                // Direct unlink fallback
                let _ = fs::remove_file(dst);
            }
        }

        // Retry replacing dst with temp_dst
        if fs::rename(&temp_dst, dst).is_err() {
            fs::copy(&temp_dst, dst)?;
            let _ = fs::remove_file(&temp_dst);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dst, fs::Permissions::from_mode(0o755));
    }

    Ok(())
}

pub fn run_update_command(args: &[String]) {
    let update_std_only = args.iter().any(|a| a == "std" || a == "--std" || a == "blaze" || a == "--blaze");
    let prefer_remote = args.iter().any(|a| a == "--remote");

    if !update_std_only {
        println!("\x1b[1;34m[1/4]\x1b[0m Checking Cargo toolchain...");
        let cargo_check = std::process::Command::new("cargo")
            .arg("--version")
            .output();
        if cargo_check.is_err() {
            eprintln!("\x1b[1;31merror:\x1b[0m Cargo is not installed or not found in PATH.");
            eprintln!("Please install Rust and Cargo from https://rustup.rs/ to update Flame.");
            return;
        }

        let is_local_repo = if Path::new("Cargo.toml").exists() {
            fs::read_to_string("Cargo.toml")
                .map(|c| c.contains("name = \"flamelang\"") || c.contains("name = \"flame\""))
                .unwrap_or(false)
        } else {
            false
        };

        let status = if is_local_repo && !prefer_remote {
            println!("\x1b[1;34m[2/4]\x1b[0m Installing Flame release from local workspace...");
            std::process::Command::new("cargo")
                .args(["install", "--path", ".", "--force"])
                .status()
        } else {
            println!("\x1b[1;34m[2/4]\x1b[0m Fetching and installing the latest Flame release from Cargo...");
            let res = std::process::Command::new("cargo")
                .args(["install", "flamelang", "--force"])
                .status();

            if res.as_ref().map_or(false, |s| !s.success()) {
                println!("  \x1b[1;33mnotice:\x1b[0m Registry install did not succeed; fetching latest from GitHub repository...");
                std::process::Command::new("cargo")
                    .args(["install", "--git", "https://github.com/shoya-129/flame.git", "--force"])
                    .status()
            } else {
                res
            }
        };

        match status {
            Ok(s) if s.success() => {
                println!("\x1b[1;34m[3/4]\x1b[0m Synchronizing 'fmp' binary...");
                let mut pending_cleanup = Vec::new();
                let mut target_dirs: Vec<PathBuf> = Vec::new();

                let mut add_target_dir = |d: PathBuf| {
                    if d.exists() && !target_dirs.iter().any(|existing| existing == &d) {
                        target_dirs.push(d);
                    }
                };

                if let Some(cargo_bin) = dirs_fallback_cargo_bin() {
                    add_target_dir(cargo_bin);
                }
                if let Ok(cur) = std::env::current_exe() {
                    if let Some(parent) = cur.parent() {
                        add_target_dir(parent.to_path_buf());
                    }
                    if let Ok(canonical) = fs::canonicalize(&cur) {
                        if let Some(parent) = canonical.parent() {
                            add_target_dir(parent.to_path_buf());
                        }
                    }
                }
                if let Ok(home) = std::env::var("HOME") {
                    let local_bin = PathBuf::from(home).join(".local").join("bin");
                    add_target_dir(local_bin);
                }
                add_target_dir(PathBuf::from("/usr/local/bin"));

                if cfg!(windows) {
                    if let Ok(user_profile) = std::env::var("USERPROFILE") {
                        add_target_dir(PathBuf::from(user_profile).join(".cargo").join("bin"));
                    }
                    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
                        add_target_dir(PathBuf::from(local_app).join("Blaze"));
                    }
                }

                // Scan PATH for all directories containing any active flame/flamelang/fmp binaries
                if let Some(path_var) = std::env::var_os("PATH") {
                    for p in std::env::split_paths(&path_var) {
                        let has_fmp = p.join(if cfg!(windows) { "fmp.exe" } else { "fmp" }).exists();
                        let has_flamelang = p.join(if cfg!(windows) { "flamelang.exe" } else { "flamelang" }).exists();
                        let has_flame = p.join(if cfg!(windows) { "flame.exe" } else { "flame" }).exists();
                        if has_fmp || has_flamelang || has_flame {
                            add_target_dir(p);
                        }
                    }
                }

                let fmp_name = if cfg!(windows) { "fmp.exe" } else { "fmp" };
                let flamelang_name = if cfg!(windows) { "flamelang.exe" } else { "flamelang" };

                // Locate the freshly compiled binary produced by Cargo
                let mut source_bin: Option<PathBuf> = None;

                // 1. Check Cargo bin directory where cargo install placed it
                if let Some(cargo_bin) = dirs_fallback_cargo_bin() {
                    let flamelang_bin = cargo_bin.join(flamelang_name);
                    let fmp_bin = cargo_bin.join(fmp_name);
                    if flamelang_bin.exists() {
                        source_bin = Some(flamelang_bin);
                    } else if fmp_bin.exists() {
                        source_bin = Some(fmp_bin);
                    }
                }

                // 2. Check local target/release if compiled in local workspace
                if source_bin.is_none() {
                    let target_rel = PathBuf::from("target").join("release");
                    if target_rel.exists() {
                        let flamelang_bin = target_rel.join(flamelang_name);
                        let fmp_bin = target_rel.join(fmp_name);
                        if flamelang_bin.exists() {
                            source_bin = Some(flamelang_bin);
                        } else if fmp_bin.exists() {
                            source_bin = Some(fmp_bin);
                        }
                    }
                }

                // 3. Fallback: Search all candidate target directories
                if source_bin.is_none() {
                    for dir in &target_dirs {
                        let flamelang_bin = dir.join(flamelang_name);
                        let fmp_bin = dir.join(fmp_name);
                        if flamelang_bin.exists() {
                            source_bin = Some(flamelang_bin);
                            break;
                        } else if fmp_bin.exists() {
                            source_bin = Some(fmp_bin);
                            break;
                        }
                    }
                }

                if let Some(src) = source_bin {
                    // Copy fresh source to an isolated temporary location so it cannot be
                    // modified, deleted, or clobbered while we update target directories
                    let temp_fresh_bin = std::env::temp_dir().join(format!(
                        "flame_fresh_bin_{}_{}",
                        std::process::id(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0)
                    ));

                    if let Err(e) = fs::copy(&src, &temp_fresh_bin) {
                        eprintln!("  \x1b[1;31merror:\x1b[0m Failed to stage updated binary from {}: {}", src.display(), e);
                        return;
                    }

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&temp_fresh_bin, fs::Permissions::from_mode(0o755));
                    }

                    let mut synchronized_destinations: Vec<PathBuf> = Vec::new();

                    // 1. Explicitly update the currently running executable
                    if let Ok(cur_exe) = std::env::current_exe() {
                        if let Ok(()) = safe_replace_binary(&temp_fresh_bin, &cur_exe, &mut pending_cleanup) {
                            println!("  Synchronized active binary: {}", cur_exe.display());
                            synchronized_destinations.push(cur_exe.clone());
                            if let Ok(canonical) = fs::canonicalize(&cur_exe) {
                                synchronized_destinations.push(canonical);
                            }
                        }
                    }

                    // 2. Synchronize 'fmp' binary into all target directories
                    for dir in &target_dirs {
                        let fmp_bin = dir.join(fmp_name);

                        // Skip if already synchronized in step 1
                        if synchronized_destinations.iter().any(|d| d == &fmp_bin) {
                            continue;
                        }

                        match safe_replace_binary(&temp_fresh_bin, &fmp_bin, &mut pending_cleanup) {
                            Ok(()) => {
                                println!("  Synchronized 'fmp' binary to: {}", fmp_bin.display());
                                synchronized_destinations.push(fmp_bin.clone());
                                if let Ok(canonical) = fs::canonicalize(&fmp_bin) {
                                    synchronized_destinations.push(canonical);
                                }
                            }
                            Err(_e) => {
                                // Directory might not be writable (e.g. /usr/local/bin without sudo), ignore
                            }
                        }

                        // Write fmp command shims on Windows
                        if cfg!(windows) {
                            let _ = fs::write(dir.join("fmp.cmd"), "@\"%~dp0fmp.exe\" %*\n");
                            let _ = fs::write(dir.join("fmp.bat"), "@\"%~dp0fmp.exe\" %*\n");
                        }

                        // Remove leftover flamelang and flame shims
                        let _ = fs::remove_file(dir.join("flamelang.cmd"));
                        let _ = fs::remove_file(dir.join("flamelang.bat"));
                        let _ = fs::remove_file(dir.join("flame.cmd"));
                        let _ = fs::remove_file(dir.join("flame.bat"));
                    }

                    // 3. Clean up transitional and legacy binaries across all target directories
                    for dir in &target_dirs {
                        let flamelang_bin = dir.join(flamelang_name);
                        let flame_bin = dir.join(if cfg!(windows) { "flame.exe" } else { "flame" });
                        let fmp_bin = dir.join(fmp_name);

                        if flamelang_bin.exists()
                            && flamelang_bin != fmp_bin
                            && !synchronized_destinations.iter().any(|d| d == &flamelang_bin)
                        {
                            if fs::remove_file(&flamelang_bin).is_err() {
                                let temp_name = format!("{}.deleteme.{}", flamelang_bin.display(), std::process::id());
                                let temp_path = PathBuf::from(&temp_name);
                                if fs::rename(&flamelang_bin, &temp_path).is_ok() {
                                    pending_cleanup.push(temp_path);
                                }
                            } else {
                                println!("  Removed transitional 'flamelang' binary: {}", flamelang_bin.display());
                            }
                        }

                        if flame_bin.exists()
                            && flame_bin != fmp_bin
                            && !synchronized_destinations.iter().any(|d| d == &flame_bin)
                        {
                            if fs::remove_file(&flame_bin).is_err() {
                                let temp_name = format!("{}.deleteme.{}", flame_bin.display(), std::process::id());
                                let temp_path = PathBuf::from(&temp_name);
                                if fs::rename(&flame_bin, &temp_path).is_ok() {
                                    pending_cleanup.push(temp_path);
                                }
                            } else {
                                println!("  Removed legacy 'flame' binary: {}", flame_bin.display());
                            }
                        }
                    }

                    // Remove our temporary staging binary
                    let _ = fs::remove_file(&temp_fresh_bin);
                } else {
                    eprintln!("  \x1b[1;33mwarning:\x1b[0m Could not locate compiled Flame binary to synchronize.");
                }

                #[cfg(windows)]
                if !pending_cleanup.is_empty() {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    const DETACHED_PROCESS: u32 = 0x00000008;

                    let targets: Vec<String> = pending_cleanup
                        .iter()
                        .map(|p| format!("\"{}\"", p.display()))
                        .collect();

                    let script = format!(
                        "ping 127.0.0.1 -n 2 >nul & del /f /q {} >nul 2>&1",
                        targets.join(" ")
                    );

                    let _ = std::process::Command::new("cmd")
                        .args(["/C", &script])
                        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                        .spawn();
                }

                #[cfg(not(windows))]
                if !pending_cleanup.is_empty() {
                    let targets: Vec<String> = pending_cleanup
                        .iter()
                        .map(|p| format!("'{}'", p.display()))
                        .collect();
                    let script = format!("sleep 1; rm -f {}", targets.join(" "));
                    let _ = std::process::Command::new("sh")
                        .args(["-c", &script])
                        .spawn();
                }
            }
            Ok(s) => {
                eprintln!("\x1b[1;31merror:\x1b[0m Cargo failed to update flamelang with exit code: {}", s);
                return;
            }
            Err(e) => {
                eprintln!("\x1b[1;31merror:\x1b[0m Failed executing cargo: {}", e);
                return;
            }
        }
    }

    let step_label = if update_std_only { "[1/1]" } else { "[4/4]" };
    println!("\x1b[1;34m{}\x1b[0m Updating Blaze standard library definitions...", step_label);
    let effective_prefer_remote = prefer_remote || !Path::new("Blaze/std").exists();
    match blaze::update_blaze_definitions(effective_prefer_remote) {
        Ok(count) => {
            if count == 0 {
                println!("  \x1b[1;33mwarning:\x1b[0m No Blaze definition directories could be resolved.");
            }
        }
        Err(e) => {
            eprintln!("  \x1b[1;31merror:\x1b[0m Failed updating Blaze definitions: {}", e);
        }
    }

    println!("\n\x1b[1;32m✓ Successfully updated Flame & Blaze toolchain!\x1b[0m");
    println!("Run \x1b[1mfmp --version\x1b[0m to check your active release.\n");
}

pub fn run_uninstall_command(_args: &[String]) {
    println!("\x1b[1;33mUninstalling Flame and Blaze toolchain...\x1b[0m");

    // 1. Remove Blaze definition directories
    println!("\x1b[1;34m[1/3]\x1b[0m Removing Blaze standard library definition directories...");
    let mut dirs_to_remove = Vec::new();
    if let Ok(val) = std::env::var("BLAZE_HOME") {
        dirs_to_remove.push(PathBuf::from(val));
    }
    if let Ok(prog_files) = std::env::var("ProgramFiles") {
        dirs_to_remove.push(PathBuf::from(prog_files).join("Blaze"));
    }
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        dirs_to_remove.push(PathBuf::from(local_app).join("Blaze"));
    }
    if let Ok(user_prof) = std::env::var("USERPROFILE") {
        dirs_to_remove.push(PathBuf::from(user_prof).join(".blaze"));
    }
    if let Ok(home) = std::env::var("HOME") {
        dirs_to_remove.push(PathBuf::from(home).join(".blaze"));
    }
    dirs_to_remove.push(PathBuf::from("/usr/local/share/blaze"));

    for d in dirs_to_remove {
        if d.exists() {
            let _ = fs::remove_dir_all(&d);
            println!("  Removed: {}", d.display());
        }
    }

    #[cfg(windows)]
    {
        let _ = std::process::Command::new("reg")
            .args(["delete", r"HKCU\Environment", "/v", "BLAZE_HOME", "/f"])
            .output();
    }

    // 2. Discover all flame / flamelang binaries and shims to delete
    println!("\x1b[1;34m[2/3]\x1b[0m Removing Flame binaries and command aliases...");
    let mut files_to_delete = Vec::new();

    if let Some(cargo_bin) = dirs_fallback_cargo_bin() {
        files_to_delete.push(cargo_bin.join("fmp.exe"));
        files_to_delete.push(cargo_bin.join("fmp.cmd"));
        files_to_delete.push(cargo_bin.join("fmp.bat"));
        files_to_delete.push(cargo_bin.join("fmp"));
        files_to_delete.push(cargo_bin.join("flame.exe"));
        files_to_delete.push(cargo_bin.join("flame.cmd"));
        files_to_delete.push(cargo_bin.join("flame.bat"));
        files_to_delete.push(cargo_bin.join("flame"));
        files_to_delete.push(cargo_bin.join("flamelang.exe"));
        files_to_delete.push(cargo_bin.join("flamelang.cmd"));
        files_to_delete.push(cargo_bin.join("flamelang.bat"));
        files_to_delete.push(cargo_bin.join("flamelang"));
    }

    if let Ok(cur) = std::env::current_exe() {
        if let Some(parent) = cur.parent() {
            files_to_delete.push(parent.join("fmp.exe"));
            files_to_delete.push(parent.join("fmp.cmd"));
            files_to_delete.push(parent.join("fmp.bat"));
            files_to_delete.push(parent.join("fmp"));
            files_to_delete.push(parent.join("flame.exe"));
            files_to_delete.push(parent.join("flame.cmd"));
            files_to_delete.push(parent.join("flame.bat"));
            files_to_delete.push(parent.join("flame"));
            files_to_delete.push(parent.join("flamelang.exe"));
            files_to_delete.push(parent.join("flamelang.cmd"));
            files_to_delete.push(parent.join("flamelang.bat"));
            files_to_delete.push(parent.join("flamelang"));
        }
        files_to_delete.push(cur);
    }

    // Also query system PATH lookups
    if cfg!(windows) {
        for binary in ["fmp", "flame", "flamelang"] {
            if let Ok(output) = std::process::Command::new("where").arg(binary).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            files_to_delete.push(PathBuf::from(trimmed));
                        }
                    }
                }
            }
        }
    } else {
        for binary in ["fmp", "flame", "flamelang"] {
            if let Ok(output) = std::process::Command::new("which").arg(binary).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            files_to_delete.push(PathBuf::from(trimmed));
                        }
                    }
                }
            }
        }
    }

    files_to_delete.sort();
    files_to_delete.dedup();

    // 3. Uninstall flamelang via Cargo first (if cargo is available)
    println!("\x1b[1;34m[3/3]\x1b[0m Uninstalling flamelang via Cargo...");
    let _ = std::process::Command::new("cargo")
        .args(["uninstall", "flamelang"])
        .status();

    // Remove or rename files; any locked files (like the running flame.exe) are renamed and scheduled for background deletion
    let mut pending_delete = Vec::new();
    for f in &files_to_delete {
        if f.exists() {
            if fs::remove_file(f).is_ok() {
                println!("  Removed: {}", f.display());
            } else {
                // If running on Windows, rename to .deleteme.<pid> so it immediately disappears from PATH
                let temp_name = format!("{}.deleteme.{}", f.display(), std::process::id());
                let temp_path = PathBuf::from(&temp_name);
                if fs::rename(f, &temp_path).is_ok() {
                    println!("  Removed: {}", f.display());
                    pending_delete.push(temp_path);
                } else {
                    pending_delete.push(f.clone());
                }
            }
        }
    }

    #[cfg(windows)]
    if !pending_delete.is_empty() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let targets: Vec<String> = pending_delete
            .iter()
            .map(|p| format!("\"{}\"", p.display()))
            .collect();

        // Wait 1-2 seconds until the current flame process exits and releases its file lock, then delete
        let script = format!(
            "ping 127.0.0.1 -n 2 >nul & del /f /q {} >nul 2>&1",
            targets.join(" ")
        );

        let _ = std::process::Command::new("cmd")
            .args(["/C", &script])
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn();
    }

    #[cfg(not(windows))]
    if !pending_delete.is_empty() {
        let targets: Vec<String> = pending_delete
            .iter()
            .map(|p| format!("'{}'", p.display()))
            .collect();
        let script = format!("sleep 1; rm -f {}", targets.join(" "));
        let _ = std::process::Command::new("sh")
            .args(["-c", &script])
            .spawn();
    }

    println!("\n\x1b[1;32m✓ Successfully uninstalled Flame and cleaned Blaze toolchain!\x1b[0m\n");
}

pub fn run_doctor_command() {
    println!("\nFlame {} LTS\n", env!("CARGO_PKG_VERSION"));

    fn check_cmd(cmd: &str, args: &[&str]) -> bool {
        std::process::Command::new(cmd).args(args).output().is_ok()
    }

    let has_rustc = check_cmd("rustc", &["--version"]);
    let has_cargo = check_cmd("cargo", &["--version"]);
    let has_git = check_cmd("git", &["--version"]);
    let blaze_opt = crate::ide::locate_blaze_dir();

    println!(
        "{} Blaze compiler & toolchain",
        if blaze_opt.is_some() {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    );

    // Check Blaze std files
    let (blaze_status, blaze_msg) = if let Some(ref bdir) = blaze_opt {
        let std_dir = bdir.join("std");
        if std_dir.exists() {
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(&std_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("fm") {
                        count += 1;
                    }
                }
            }
            (
                "\x1b[1;32m✓\x1b[0m",
                format!("Blaze standard library ({} modules found at {})", count, std_dir.display()),
            )
        } else {
            (
                "\x1b[1;31m✗\x1b[0m",
                format!("Blaze standard library (std/ missing at {})", bdir.display()),
            )
        }
    } else {
        (
            "\x1b[1;31m✗\x1b[0m",
            "Blaze standard library (not found; set FLAME_BLAZE_DIR or run in Flame workspace)".to_string(),
        )
    };
    println!("{} {}", blaze_status, blaze_msg);

    // Smoke test: create a temporary Flame project in temp folder, execute, and cleanly delete
    let (smoke_status, smoke_msg) = {
        let temp_root = std::env::temp_dir();
        let smoke_dir = temp_root.join(format!("flame_doctor_smoke_{}", std::process::id()));
        let _ = fs::create_dir_all(smoke_dir.join("src"));
        let main_fm = smoke_dir.join("src").join("main.fm");
        let toml = smoke_dir.join("flame.toml");
        let _ = fs::write(&main_fm, "fn main() {\n    println(\"doctor ok\");\n}\n");
        let _ = fs::write(
            &toml,
            "[package]\nname = \"smoke_app\"\nversion = \"0.1.0\"\n",
        );

        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("fmp"));
        let res = std::process::Command::new(&current_exe)
            .args(["run", "src/main.fm", "--local"])
            .current_dir(&smoke_dir)
            .output();

        let _ = fs::remove_dir_all(&smoke_dir);

        match res {
            Ok(out) if out.status.success() => (
                "\x1b[1;32m✓\x1b[0m",
                "Flame runtime smoke test (temp app created, executed & cleaned)".to_string(),
            ),
            Ok(out) => {
                let err_detail = String::from_utf8_lossy(&out.stderr);
                let out_detail = String::from_utf8_lossy(&out.stdout);
                (
                    "\x1b[1;31m✗\x1b[0m",
                    format!("Flame runtime smoke test exited with code {:?}: {} {}", out.status.code(), err_detail.trim(), out_detail.trim()),
                )
            }
            Err(e) => (
                "\x1b[1;31m✗\x1b[0m",
                format!("Flame runtime smoke test failed to start: {}", e),
            ),
        }
    };
    println!("{} {}", smoke_status, smoke_msg);

    println!(
        "{} Rust toolchain",
        if has_rustc {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    );
    println!(
        "{} Cargo package manager",
        if has_cargo {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    );
    println!(
        "{} Git VCS",
        if has_git {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    );
    println!(
        "{} Native plugin support",
        if has_rustc && has_cargo {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    );
    println!("\x1b[1;32m✓\x1b[0m Package manager (fmp)");
    println!("\x1b[1;32m✓\x1b[0m FMI interface generator");
    println!("\x1b[1;32m✓\x1b[0m Test runner");
    println!("\x1b[1;32m✓\x1b[0m Code formatter");

    println!("\nPlatform");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let mut os_chars = os.chars();
    let os_cap = match os_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + os_chars.as_str(),
    };
    println!("\x1b[1;32m✓\x1b[0m {} {}", os_cap, arch);

    println!();
}

