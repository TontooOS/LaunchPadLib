#[cfg(target_os = "linux")]
mod app;
#[cfg(target_os = "linux")]
mod dependency;
#[cfg(target_os = "linux")]
mod logger;
#[cfg(target_os = "linux")]
mod manager;
#[cfg(target_os = "linux")]
mod process;
#[cfg(target_os = "linux")]
mod service;
#[cfg(target_os = "linux")]
mod signal;
#[cfg(target_os = "linux")]
mod socket;
#[cfg(target_os = "linux")]
mod watcher;

#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use manager::ServiceManager;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("LaunchPad daemon starting (PID 1)");

    if let Err(e) = signal::setup_signal_handlers() {
        log::error!("Failed to setup signal handlers: {}", e);
        std::process::exit(1);
    }

    let launchpads_dir = PathBuf::from(
        std::env::var("LAUNCHPAD_DIR")
            .unwrap_or_else(|_| "/Library/System/Launchpads".to_string()),
    );

    let log_dir = PathBuf::from(
        std::env::var("LAUNCHPAD_LOG_DIR")
            .unwrap_or_else(|_| "/Library/System/Launchpads".to_string()),
    );

    let _ = std::fs::create_dir_all(&launchpads_dir);
    let _ = std::fs::create_dir_all(&log_dir);

    mount_essential_fs();

    let manager = Arc::new(Mutex::new(ServiceManager::new(
        launchpads_dir.clone(),
        log_dir.clone(),
    )));

    {
        let mut mgr = manager.lock().unwrap();
        match mgr.boot() {
            Ok(started) => {
                log::info!("Boot complete, {} services started", started.len());
                for name in &started {
                    log::info!("  + {}", name);
                }
            }
            Err(e) => {
                log::error!("Boot failed: {}", e);
            }
        }
    }

    let mgr_clone = manager.clone();
    let watch_dir = launchpads_dir.clone();
    thread::spawn(move || {
        if let Err(e) = watcher::start_watcher(&watch_dir, mgr_clone) {
            log::error!("Watcher error: {}", e);
        }
    });

    let mgr_clone = manager.clone();
    thread::spawn(move || {
        loop {
            {
                let mut mgr = mgr_clone.lock().unwrap();
                mgr.reap_zombies();
            }
            thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let mgr_clone = manager.clone();
    thread::spawn(move || {
        if let Err(e) = socket::start_socket_server(mgr_clone) {
            log::error!("Socket server error: {}", e);
        }
    });

    log::info!("LaunchPad daemon running. Press Ctrl+C or send SIGTERM to stop.");
    loop {
        if signal::should_shutdown() {
            log::info!("Shutdown signal received");
            break;
        }
        thread::sleep(std::time::Duration::from_millis(500));
    }

    {
        let mut mgr = manager.lock().unwrap();
        mgr.shutdown();
    }

    let _ = std::fs::remove_file("/run/launchpad.sock");
    log::info!("LaunchPad daemon stopped");
    std::process::exit(0);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("launchpad-daemon is only supported on Linux");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn mount_essential_fs() {
    use std::process::Command;

    let mounts = [
        ("/proc", "proc"),
        ("/sys", "sysfs"),
        ("/dev", "devtmpfs"),
        ("/dev/pts", "devpts"),
        ("/dev/shm", "tmpfs"),
    ];

    for (target, fs_type) in &mounts {
        if !std::path::Path::new(target).exists() {
            let _ = std::fs::create_dir_all(target);
        }

        let output = Command::new("mount")
            .args(["-t", fs_type, fs_type, target])
            .output();

        match output {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.contains("already mounted") {
                    log::warn!("Failed to mount {}: {}", target, stderr.trim());
                }
            }
            Err(e) => {
                log::warn!("Failed to run mount for {}: {}", target, e);
            }
        }
    }
}
