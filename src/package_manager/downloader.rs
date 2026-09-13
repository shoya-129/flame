use std::fs;
use std::path::Path;
#[cfg(feature = "cli")]
use std::io::{Read, Write};

pub use crate::utils::format::{format_byte_size, format_transfer_speed};

// Line-based downloading loader
#[cfg(feature = "cli")]
pub fn download_archive_with_loader(
    target: &str,
    source_url: &str,
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut url = source_url.to_string();
    let mut version = None;
    if let Some(idx) = url.rfind('@') {
        if idx > url.rfind('/').unwrap_or(0) {
            version = Some(url[idx + 1..].to_string());
            url = url[..idx].to_string();
        }
    }

    let download_url = if let Some(v) = &version {
        if url.contains("github.com") {
            let repo = url
                .replace("https://github.com/", "")
                .replace("github.com/", "");
            format!("https://api.github.com/repos/{}/zipball/{}", repo, v)
        } else {
            format!("{}/archive/refs/tags/{}.zip", url.trim_end_matches('/'), v)
        }
    } else {
        if url.contains("github.com") {
            let repo = url
                .replace("https://github.com/", "")
                .replace("github.com/", "");
            format!("https://api.github.com/repos/{}/zipball/HEAD", repo)
        } else {
            format!("{}/archive/refs/heads/main.zip", url.trim_end_matches('/'))
        }
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Flamelang-Package-Manager")
        .build()?;

    let fetch_with_loader = |u: &str| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut req = client.get(u);
        if u.contains("api.github.com") {
            if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let target_name = target.to_string();
        let spin_handle = std::thread::spawn(move || {
            let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut i = 0;
            while rx.try_recv().is_err() {
                print!(
                    "\r   \x1b[1;36m⚙\x1b[0m \x1b[1;36m{}\x1b[0m Connecting  to '{}'...\x1b[K",
                    spinners[i], target_name
                );
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(80));
                i = (i + 1) % spinners.len();
            }
        });

        let resp_res = req.send();
        let _ = tx.send(());
        let _ = spin_handle.join();

        let mut resp = match resp_res {
            Ok(r) => r,
            Err(e) => {
                if e.is_connect() {
                    return Err(format!("Network connection failed (could not reach server). Check your internet connection: {}", e).into());
                } else if e.is_timeout() {
                    return Err(format!(
                        "Request timed out. The remote host took too long to respond: {}",
                        e
                    )
                    .into());
                } else {
                    return Err(format!("Network transfer error: {}", e).into());
                }
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(format!(
                    "Package not found (HTTP 404). Check if repository or tag exists: {}",
                    u
                )
                .into());
            } else if status == reqwest::StatusCode::FORBIDDEN
                || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            {
                return Err(format!("GitHub API rate limit exceeded or access forbidden (HTTP {}). Try setting the GITHUB_TOKEN environment variable.", status).into());
            } else if status.is_server_error() {
                return Err(format!(
                    "Remote server error (HTTP {}). The repository host is currently unavailable.",
                    status
                )
                .into());
            } else {
                return Err(format!("HTTP error {} ({})", status, u).into());
            }
        }

        let total_size = resp.content_length();
        let mut data = Vec::new();
        let mut chunk = [0u8; 16384];
        let start = std::time::Instant::now();
        let mut last_draw = std::time::Instant::now();
        let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut spin = 0;

        loop {
            let n = resp.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            data.extend_from_slice(&chunk[..n]);

            if last_draw.elapsed().as_millis() >= 80 {
                last_draw = std::time::Instant::now();
                spin = (spin + 1) % spinners.len();
                let elapsed = start.elapsed().as_secs_f64();
                let speed = format_transfer_speed(data.len(), elapsed);
                let size_disp = if let Some(tot) = total_size {
                    let pct = (data.len() as f64 / tot as f64 * 100.0).clamp(0.0, 100.0);
                    format!(
                        "{} / {} ({:.1}%)",
                        format_byte_size(data.len()),
                        format_byte_size(tot as usize),
                        pct
                    )
                } else {
                    format_byte_size(data.len())
                };
                print!(
                    "\r   \x1b[1;36m⚙\x1b[0m \x1b[1;36m{}\x1b[0m Downloading '{}' [{}] at {}\x1b[K",
                    spinners[spin], target, size_disp, speed
                );
                let _ = std::io::stdout().flush();
            }
        }

        let total_elapsed = start.elapsed().as_secs_f64();
        let duration_str = if total_elapsed < 1.0 {
            let ms = start.elapsed().as_millis().max(1);
            format!("{}ms", ms)
        } else {
            format!("{:.2}s", total_elapsed)
        };
        print!(
            "\r   \x1b[1;32m✓\x1b[0m Downloaded  '{}' ({} in {})\x1b[K\n",
            target,
            format_byte_size(data.len()),
            duration_str
        );
        let _ = std::io::stdout().flush();
        Ok(data)
    };

    let mut bytes = fetch_with_loader(&download_url);
    if bytes.is_err() && version.is_none() {
        let fallback = format!(
            "{}/archive/refs/heads/master.zip",
            url.trim_end_matches('/')
        );
        bytes = fetch_with_loader(&fallback);
    }

    let data = bytes?;
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let mut components = outpath.components();
        components.next(); // skip root dir in zip
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
                    let _ = fs::create_dir_all(p);
                }
            }
            let mut outfile = fs::File::create(&target_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

