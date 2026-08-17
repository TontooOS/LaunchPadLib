#![cfg(target_os = "linux")]

use std::path::Path;
use std::sync::{Arc, Mutex};

use inotify::{Inotify, WatchMask};

use crate::manager::ServiceManager;

pub fn start_watcher(
    launchpads_dir: &Path,
    manager: Arc<Mutex<ServiceManager>>,
) -> Result<(), String> {
    let mut inotify = Inotify::init().map_err(|e| format!("Inotify init failed: {}", e))?;

    inotify
        .watches()
        .add(launchpads_dir, WatchMask::CREATE | WatchMask::MODIFY | WatchMask::DELETE)
        .map_err(|e| format!("Cannot watch launchpads dir: {}", e))?;

    log::info!("File watcher started on {}", launchpads_dir.display());

    let mut buffer = [0u8; 4096];

    loop {
        match inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    if let Some(name) = event.name {
                        let name = name.to_string_lossy().to_string();
                        if name.ends_with(".service") {
                            let service_name = name.trim_end_matches(".service");
                            log::info!("Service file changed: {}", service_name);

                            // Reload and optionally restart the service
                            let mut mgr = manager.lock().unwrap();
                            let config_path = launchpads_dir.join(&name);
                            match crate::service::load_service_file(&config_path) {
                                Ok(config) => {
                                    mgr.services_mut().insert(
                                        service_name.to_string(),
                                        crate::manager::ManagedService::new(config),
                                    );
                                    log::info!("Reloaded service config: {}", service_name);
                                }
                                Err(e) => {
                                    log::warn!("Failed to reload {}: {}", name, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                log::warn!("Inotify read error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
