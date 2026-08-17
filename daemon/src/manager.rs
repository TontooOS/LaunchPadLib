#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use nix::sys::signal::Signal;
use nix::unistd::Pid;

use launchpad_lib::types::{ServiceConfig, ServiceInfo, ServiceState, ServiceType};

use crate::app;
use crate::dependency::DependencyGraph;
use crate::logger;
use crate::process;

#[derive(Debug, Clone)]
pub struct ManagedService {
    pub config: ServiceConfig,
    pub state: ServiceState,
    pub pid: Option<Pid>,
    pub uid: u32,
    pub started_at: Option<Instant>,
    pub crash_count: u32,
    pub last_crash: Option<Instant>,
}

impl ManagedService {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            state: ServiceState::Stopped,
            pid: None,
            uid: 0,
            started_at: None,
            crash_count: 0,
            last_crash: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LowAppEntry {
    pub id: String,
    pub app_name: String,
    pub service_type: ServiceType,
    pub state: ServiceState,
    pub pid: Option<Pid>,
    pub user: String,
}

#[derive(Debug)]
pub struct ServiceManager {
    services: HashMap<String, ManagedService>,
    low_apps: Vec<LowAppEntry>,
    next_low_id: HashMap<String, u32>,
    launchpads_dir: PathBuf,
    log_dir: PathBuf,
}

impl ServiceManager {
    pub fn services_mut(&mut self) -> &mut HashMap<String, ManagedService> {
        &mut self.services
    }

    pub fn new(launchpads_dir: PathBuf, log_dir: PathBuf) -> Self {
        Self {
            services: HashMap::new(),
            low_apps: Vec::new(),
            next_low_id: HashMap::new(),
            launchpads_dir,
            log_dir,
        }
    }

    pub fn load_services(&mut self) -> Result<(), String> {
        let configs = crate::service::load_services(&self.launchpads_dir)?;

        for (name, config) in configs {
            let state = if config.service_type == ServiceType::Low {
                ServiceState::Stopped
            } else {
                ServiceState::Stopped
            };

            self.services.insert(
                name.clone(),
                ManagedService {
                    config,
                    state,
                    pid: None,
                    uid: 0,
                    started_at: None,
                    crash_count: 0,
                    last_crash: None,
                },
            );
        }

        Ok(())
    }

    pub fn boot(&mut self) -> Result<Vec<String>, String> {
        self.load_services()?;

        let graph = DependencyGraph::new(
            &self.services.iter().map(|(k, v)| (k.clone(), v.config.clone())).collect(),
        );

        let order = graph.resolve_boot_order()?;
        let mut started = Vec::new();

        for name in &order {
            if let Some(service) = self.services.get(name) {
                if service.config.service_type == ServiceType::Sys
                    || service.config.service_type == ServiceType::Default
                {
                    match self.start(name) {
                        Ok(()) => started.push(name.clone()),
                        Err(e) => log::warn!("Failed to start {}: {}", name, e),
                    }
                }
            }
        }

        Ok(started)
    }

    pub fn start(&mut self, name: &str) -> Result<(), String> {
        let config = self
            .services
            .get(name)
            .ok_or_else(|| format!("Service '{}' not found", name))?
            .config
            .clone();

        // Check if already running
        if let Some(service) = self.services.get(name) {
            if service.state == ServiceState::Running {
                return Ok(());
            }
        }

        // Start dependencies first
        for dep in &config.depends_on {
            if let Some(dep_service) = self.services.get(dep) {
                if dep_service.state != ServiceState::Running {
                    self.start(dep)?;
                }
            }
        }

        let log_path = self.log_dir.join(format!("{}.log", name));

        self.services.entry(name.to_string()).and_modify(|s| {
            s.state = ServiceState::Starting;
        });

        let child = process::spawn_process(
            &config.execute,
            &config.user,
            &log_path,
            &[("LAUNCHPAD_SERVICE", name)],
        )?;

        self.services.entry(name.to_string()).and_modify(|s| {
            s.state = ServiceState::Running;
            s.pid = Some(child.pid);
            s.uid = child.uid;
            s.started_at = Some(Instant::now());
        });

        log::info!("Service '{}' started (PID {})", name, child.pid);
        Ok(())
    }

    pub fn stop(&mut self, name: &str) -> Result<(), String> {
        let service = self
            .services
            .get_mut(name)
            .ok_or_else(|| format!("Service '{}' not found", name))?;

        if service.state != ServiceState::Running {
            return Ok(());
        }

        let pid = service.pid.ok_or_else(|| format!("Service '{}' has no PID", name))?;

        // Graceful: SIGTERM, wait 5s, then SIGKILL
        log::info!("Stopping service '{}' (SIGTERM)...", name);
        let _ = process::send_signal(pid, Signal::SIGTERM);

        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if !process::is_process_running(pid) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if process::is_process_running(pid) {
            log::info!("Service '{}' did not stop gracefully, sending SIGKILL...", name);
            let _ = process::send_signal(pid, Signal::SIGKILL);
            std::thread::sleep(Duration::from_millis(100));
        }

        service.state = ServiceState::Stopped;
        service.pid = None;
        service.started_at = None;

        log::info!("Service '{}' stopped", name);
        Ok(())
    }

    pub fn restart(&mut self, name: &str) -> Result<(), String> {
        self.stop(name)?;
        self.start(name)
    }

    pub fn kill(&mut self, name: &str) -> Result<(), String> {
        let service = self
            .services
            .get_mut(name)
            .ok_or_else(|| format!("Service '{}' not found", name))?;

        if let Some(pid) = service.pid {
            log::info!("Killing service '{}' (SIGKILL)...", name);
            let _ = process::send_signal(pid, Signal::SIGKILL);
            std::thread::sleep(Duration::from_millis(100));
        }

        service.state = ServiceState::Stopped;
        service.pid = None;
        service.started_at = None;

        Ok(())
    }

    pub fn activate(&mut self, name: &str) -> Result<(), String> {
        let service = self
            .services
            .get_mut(name)
            .ok_or_else(|| format!("Service '{}' not found", name))?;

        if service.config.service_type == ServiceType::Sys {
            return Err("Cannot deactivate system services".to_string());
        }

        service.state = ServiceState::Running;
        Ok(())
    }

    pub fn deactivate(&mut self, name: &str) -> Result<(), String> {
        let service = self
            .services
            .get_mut(name)
            .ok_or_else(|| format!("Service '{}' not found", name))?;

        if service.config.service_type == ServiceType::Sys {
            return Err("Cannot deactivate system services".to_string());
        }

        service.state = ServiceState::Deactivated;
        Ok(())
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        let mut result: Vec<ServiceInfo> = self
            .services
            .values()
            .map(|s| ServiceInfo {
                name: s.config.name.clone(),
                service_type: s.config.service_type.clone(),
                state: s.state.clone(),
                pid: s.pid.map(|p| p.as_raw() as u32),
                user: s.config.user.clone(),
            })
            .collect();

        for app in &self.low_apps {
            result.push(ServiceInfo {
                name: app.id.clone(),
                service_type: app.service_type.clone(),
                state: app.state.clone(),
                pid: app.pid.map(|p| p.as_raw() as u32),
                user: app.user.clone(),
            });
        }

        result
    }

    pub fn log_service(&self, name: &str, head: Option<usize>) -> Result<Vec<String>, String> {
        let log_path = self.log_dir.join(format!("{}.log", name));
        logger::read_log(&log_path, head)
    }

    pub fn handle_crash(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            service.crash_count += 1;
            service.last_crash = Some(Instant::now());
            service.state = ServiceState::Crashed;
            service.pid = None;

            if service.config.restart && service.crash_count < 5 {
                let backoff = Self::crash_backoff(service.crash_count);
                log::warn!(
                    "Service '{}' crashed, restarting in {}s (attempt {}/5)",
                    name,
                    backoff,
                    service.crash_count
                );

                let _name = name.to_string();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(backoff));
                    // Re-trigger start through the manager
                });
            } else if service.crash_count >= 5 {
                log::error!(
                    "Service '{}' crashed too many times ({}), giving up",
                    name,
                    service.crash_count
                );
            }
        }
    }

    fn crash_backoff(crash_count: u32) -> u64 {
        match crash_count {
            1 => 3,
            2 => 5,
            3 => 10,
            4 => 30,
            _ => 60,
        }
    }

    pub fn reap_zombies(&mut self) {
        let reaped = process::reap_children();
        for pid in reaped {
            // Find which service this was
            for (name, service) in &mut self.services {
                if service.pid == Some(pid) {
                    log::warn!("Service '{}' (PID {}) exited", name, pid);
                    service.state = ServiceState::Crashed;
                    service.pid = None;
                    service.crash_count += 1;
                    service.last_crash = Some(Instant::now());

                    if service.config.restart && service.crash_count < 5 {
                        let backoff = Self::crash_backoff(service.crash_count);
                        log::info!(
                            "Auto-restarting '{}' in {}s (attempt {}/5)",
                            name,
                            backoff,
                            service.crash_count
                        );
                    }
                    break;
                }
            }

            // Check low apps
            for app in &mut self.low_apps {
                if app.pid == Some(pid) {
                    log::warn!("Low app '{}' (PID {}) exited", app.id, pid);
                    app.state = ServiceState::Crashed;
                    app.pid = None;
                    break;
                }
            }
        }
    }

    pub fn start_app_bundle(&mut self, app_path: &str) -> Result<String, String> {
        let bundle = app::parse_app_bundle(app_path)?;

        let app_name = bundle.name.to_lowercase();

        // Check if already running
        for app in &self.low_apps {
            if app.app_name == app_name && app.state == ServiceState::Running {
                return Ok(app.id.clone());
            }
        }

        // Generate new ID
        let id_num = self.next_low_id.entry(app_name.clone()).or_insert(0);
        *id_num += 1;
        let id = format!("{}_low_{}", id_num, app_name);

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

        let log_path = self.log_dir.join(format!("{}.log", id));

        let child = process::spawn_process(
            bundle.executable.to_str().ok_or("Invalid executable path")?,
            &user,
            &log_path,
            &[("LAUNCHPAD_APP", &id)],
        )?;

        self.low_apps.push(LowAppEntry {
            id: id.clone(),
            app_name: app_name.clone(),
            service_type: ServiceType::Low,
            state: ServiceState::Running,
            pid: Some(child.pid),
            user,
        });

        log::info!("App '{}' started as '{}' (PID {})", bundle.name, id, child.pid);
        Ok(id)
    }

    pub fn stop_low_app(&mut self, id: &str) -> Result<(), String> {
        let app = self
            .low_apps
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("Low app '{}' not found", id))?;

        if let Some(pid) = app.pid {
            let _ = process::send_signal(pid, Signal::SIGTERM);
            std::thread::sleep(Duration::from_millis(200));
            if process::is_process_running(pid) {
                let _ = process::send_signal(pid, Signal::SIGKILL);
            }
        }

        app.state = ServiceState::Stopped;
        app.pid = None;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        log::info!("Shutting down all services...");

        // Stop low apps first
        let low_ids: Vec<String> = self.low_apps.iter().map(|a| a.id.clone()).collect();
        for id in &low_ids {
            let _ = self.stop_low_app(id);
        }

        // Stop all services in reverse order
        let names: Vec<String> = self.services.keys().cloned().collect();
        for name in names.iter().rev() {
            let _ = self.stop(name);
        }

        log::info!("All services stopped");
    }
}
