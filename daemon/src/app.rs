#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppBundle {
    pub path: PathBuf,
    pub name: String,
    pub executable: PathBuf,
}

pub fn parse_app_bundle(app_path: &str) -> Result<AppBundle, String> {
    let path = Path::new(app_path);

    if !path.exists() {
        return Err(format!("App bundle not found: {}", app_path));
    }

    if !path.is_dir() {
        return Err(format!("App path is not a directory: {}", app_path));
    }

    // Extract app name from directory name (remove .app suffix)
    let dir_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid app path: {}", app_path))?;

    let app_name = if dir_name.ends_with(".app") {
        dir_name[..dir_name.len() - 4].to_string()
    } else {
        dir_name.to_string()
    };

    // Try to read Info.plist
    let plist_path = path.join("Info.plist");
    let executable = if plist_path.exists() {
        parse_info_plist(&plist_path, &app_name)?
    } else {
        // Fallback: look for binary in bin/ directory
        find_binary_in_bundle(path, &app_name)?
    };

    Ok(AppBundle {
        path: path.to_path_buf(),
        name: app_name,
        executable,
    })
}

fn parse_info_plist(plist_path: &Path, fallback_name: &str) -> Result<PathBuf, String> {
    let content = std::fs::read_to_string(plist_path)
        .map_err(|e| format!("Cannot read Info.plist: {}", e))?;

    // Simple XML plist parsing without external dependencies
    let executable = extract_plist_value(&content, "CFBundleExecutable")
        .unwrap_or_else(|| fallback_name.to_lowercase());

    // Look for executable in bin/ directory first
    let bundle_dir = plist_path.parent().unwrap();
    let bin_path = bundle_dir.join("bin").join(&executable);
    if bin_path.exists() {
        return Ok(bin_path);
    }

    // Fallback to bundle root
    let root_path = bundle_dir.join(&executable);
    if root_path.exists() {
        return Ok(root_path);
    }

    Err(format!(
        "Executable '{}' not found in app bundle",
        executable
    ))
}

fn find_binary_in_bundle(bundle_path: &Path, app_name: &str) -> Result<PathBuf, String> {
    // Try common locations
    let candidates = [
        bundle_path.join("bin").join(app_name.to_lowercase()),
        bundle_path.join("bin").join(app_name),
        bundle_path.join(app_name.to_lowercase()),
        bundle_path.join(app_name),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    Err(format!(
        "No executable found in app bundle: {}",
        bundle_path.display()
    ))
}

fn extract_plist_value(xml: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let rest = &xml[pos + key_tag.len()..];
        // Look for next <string> tag
        if let Some(start) = rest.find("<string>") {
            let after_open = &rest[start + 8..];
            if let Some(end) = after_open.find("</string>") {
                return Some(after_open[..end].to_string());
            }
        }
    }
    None
}

pub fn app_display_name(bundle: &AppBundle) -> String {
    bundle
        .path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}
