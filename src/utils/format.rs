/// Format byte sizes to human-readable strings (B, KB, MB, GB).
pub fn format_byte_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format transfer speed to human-readable units (B/s, KB/s, MB/s, GB/s).
pub fn format_transfer_speed(bytes: usize, elapsed_secs: f64) -> String {
    if elapsed_secs <= 0.04 {
        return "-- MB/s".to_string();
    }
    let bps = bytes as f64 / elapsed_secs;
    if bps >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB/s", bps / (1024.0 * 1024.0 * 1024.0))
    } else if bps >= 1024.0 * 1024.0 {
        format!("{:.2} MB/s", bps / (1024.0 * 1024.0))
    } else if bps >= 1024.0 {
        format!("{:.2} KB/s", bps / 1024.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Clean markdown table formatting for hover/documentation display.
pub fn clean_table_borders(doc: &str) -> String {
    if !doc.contains('|') {
        return doc.to_string();
    }
    let lines = doc.lines().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut in_table = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cells: Vec<String> = trimmed
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Check if separator line (| --- | --- |)
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
                in_table = true;
                continue;
            }

            if !in_table {
                in_table = true;
                continue;
            }

            // Row in table: convert to borderless list item
            if cells.len() >= 3 {
                let col1 = cells[0].trim_matches('`');
                let col2 = cells[1].trim_matches('`');
                let col3 = cells[2..].join(" — ");
                result.push(format!("- `{}: {}` — {}", col1, col2, col3));
            } else if cells.len() == 2 {
                let col1 = cells[0].trim_matches('`');
                let col2 = &cells[1];
                result.push(format!("- `{}`: {}", col1, col2));
            } else if !cells.is_empty() {
                result.push(format!("- `{}`", cells[0]));
            }
        } else {
            in_table = false;
            result.push(line.to_string());
        }
    }

    result.join("\n")
}
