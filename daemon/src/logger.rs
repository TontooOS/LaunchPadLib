#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use chrono::Local;

pub fn write_log(log_path: &Path, stream: &str, line: &str) {
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let formatted = format!("[{}] [{}] {}\n", timestamp, stream, line);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = file.write_all(formatted.as_bytes());
    }
}

pub fn read_log(log_path: &Path, head: Option<usize>) -> Result<Vec<String>, String> {
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(log_path).map_err(|e| format!("Cannot open log: {}", e))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Cannot read log: {}", e))?;

    match head {
        Some(n) => {
            let start = lines.len().saturating_sub(n);
            Ok(lines[start..].to_vec())
        }
        None => Ok(lines),
    }
}

pub fn truncate_log(log_path: &Path, max_bytes: usize) -> Result<(), String> {
    if !log_path.exists() {
        return Ok(());
    }

    let meta = fs::metadata(log_path).map_err(|e| format!("Cannot stat log: {}", e))?;
    if (meta.len() as usize) <= max_bytes {
        return Ok(());
    }

    let file = fs::File::open(log_path).map_err(|e| format!("Cannot open log: {}", e))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Cannot read log: {}", e))?;

    // Keep last ~50% of lines
    let keep = lines.len() / 2;
    let kept: Vec<String> = lines.into_iter().skip(keep).collect();

    let mut file = fs::File::create(log_path).map_err(|e| format!("Cannot create log: {}", e))?;
    for line in kept {
        let _ = writeln!(file, "{}", line);
    }

    Ok(())
}
