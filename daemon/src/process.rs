#![cfg(target_os = "linux")]

use std::ffi::CString;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Gid, Pid, Uid, User, setgid, setgroups, setuid};

#[derive(Debug, Clone)]
pub struct ChildProcess {
    pub pid: Pid,
    pub uid: u32,
}

pub fn spawn_process(
    execute: &str,
    user: &str,
    log_path: &Path,
    env_vars: &[(&str, &str)],
) -> Result<ChildProcess, String> {
    let args: Vec<&str> = execute.split_whitespace().collect();
    if args.is_empty() {
        return Err("Execute path is empty".to_string());
    }

    let argv: Vec<CString> = args
        .iter()
        .map(|a| CString::new(*a).map_err(|e| format!("CString error: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    let uid = resolve_user(user)?;

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            log::info!(
                "Spawned process {} (PID {}) as user {}",
                execute,
                child,
                user
            );
            Ok(ChildProcess { pid: child, uid })
        }
        Ok(ForkResult::Child) => {
            // Redirect stdout/stderr to log file
            if let Ok(log_file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let fd = log_file.as_raw_fd();
                let _ = nix::unistd::dup2(fd, 1);
                let _ = nix::unistd::dup2(fd, 2);
            }

            // Set user if not root
            if uid != 0 {
                if let Err(e) = set_uid(uid) {
                    eprintln!("Failed to setuid {}: {}", uid, e);
                    std::process::exit(1);
                }
            }

            // Set environment variables
            for (key, val) in env_vars {
                std::env::set_var(key, val);
            }

            // Exec
            let err = nix::unistd::execvp(&argv[0], &argv);
            eprintln!("execvp failed: {:?}", err);
            std::process::exit(1);
        }
        Err(e) => Err(format!("Fork failed: {}", e)),
    }
}

fn resolve_user(user: &str) -> Result<u32, String> {
    if user == "root" {
        return Ok(0);
    }

    User::from_name(user)
        .map_err(|e| format!("User lookup error: {}", e))?
        .ok_or_else(|| format!("User '{}' not found", user))
        .map(|u| u.uid.as_raw())
}

fn set_uid(uid: u32) -> Result<(), String> {
    let user = User::from_uid(Uid::from_raw(uid))
        .map_err(|e| format!("User lookup error: {}", e))?
        .ok_or_else(|| format!("User with UID {} not found", uid))?;

    // Get groups for user
    let c_name = CString::new(user.name.as_str())
        .map_err(|e| format!("CString conversion error: {}", e))?;
    let groups: Vec<Gid> = nix::unistd::getgrouplist(&c_name, user.gid)
        .map_err(|e| format!("getgrouplist failed: {}", e))?;

    if !groups.is_empty() {
        setgroups(&groups).map_err(|e| format!("setgroups failed: {}", e))?;
    }

    setgid(user.gid).map_err(|e| format!("setgid failed: {}", e))?;
    setuid(Uid::from_raw(uid)).map_err(|e| format!("setuid failed: {}", e))?;

    Ok(())
}

pub fn send_signal(pid: Pid, signal: Signal) -> Result<(), String> {
    kill(pid, signal).map_err(|e| format!("Signal {} to PID {} failed: {}", signal, pid, e))
}

pub fn reap_children() -> Vec<Pid> {
    let mut reaped = Vec::new();
    loop {
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, _)) | Ok(WaitStatus::Signaled(pid, _, _)) => {
                reaped.push(pid);
            }
            Ok(WaitStatus::StillAlive) => break,
            Err(_) => break,
            _ => {}
        }
    }
    reaped
}

pub fn is_process_running(pid: Pid) -> bool {
    kill(pid, Signal::SIGCONT).is_ok()
}
