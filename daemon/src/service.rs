#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use launchpad_lib::types::{ServiceConfig, ServiceType};

pub fn load_services(dir: &Path) -> Result<HashMap<String, ServiceConfig>, String> {
    let mut services = HashMap::new();

    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("Cannot create launchpads dir: {}", e))?;
        return Ok(services);
    }

    let entries =
        fs::read_dir(dir).map_err(|e| format!("Cannot read launchpads dir: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("service") {
            match load_service_file(&path) {
                Ok(config) => {
                    services.insert(config.name.clone(), config);
                }
                Err(e) => {
                    log::warn!("Failed to load {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(services)
}

pub fn load_service_file(path: &Path) -> Result<ServiceConfig, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

    let config: ServiceConfig =
        serde_yaml::from_str(&content).map_err(|e| format!("YAML parse error: {}", e))?;

    if config.name.is_empty() {
        return Err("Service name is empty".to_string());
    }
    if config.execute.is_empty() {
        return Err("Service execute path is empty".to_string());
    }

    Ok(config)
}

pub fn service_config_path(launchpads_dir: &Path, name: &str) -> PathBuf {
    launchpads_dir.join(format!("{}.service", name))
}

pub fn service_log_path(launchpads_dir: &Path, name: &str) -> PathBuf {
    launchpads_dir.join(format!("{}.log", name))
}

pub fn is_system_service(config: &ServiceConfig) -> bool {
    config.service_type == ServiceType::Sys
}

pub fn can_deactivate(config: &ServiceConfig) -> bool {
    config.service_type != ServiceType::Sys
}
