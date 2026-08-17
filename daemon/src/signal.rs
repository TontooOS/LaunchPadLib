#![cfg(target_os = "linux")]

use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::signal::{Signal, sigaction, SaFlags, SigAction, SigHandler, SigSet};

pub static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

pub fn setup_signal_handlers() -> Result<(), String> {
    unsafe {
        let term_action = SigAction::new(
            SigHandler::Handler(handle_sigterm),
            SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        sigaction(Signal::SIGTERM, &term_action)
            .map_err(|e| format!("Cannot set SIGTERM handler: {}", e))?;

        let int_action = SigAction::new(
            SigHandler::Handler(handle_sigint),
            SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        sigaction(Signal::SIGINT, &int_action)
            .map_err(|e| format!("Cannot set SIGINT handler: {}", e))?;

        let child_action = SigAction::new(
            SigHandler::SigIgn,
            SaFlags::SA_RESTART | SaFlags::SA_NOCLDWAIT,
            SigSet::empty(),
        );
        sigaction(Signal::SIGCHLD, &child_action)
            .map_err(|e| format!("Cannot set SIGCHLD handler: {}", e))?;
    }

    Ok(())
}

extern "C" fn handle_sigterm(_sig: i32) {
    log::info!("Received SIGTERM, initiating shutdown...");
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
}

extern "C" fn handle_sigint(_sig: i32) {
    log::info!("Received SIGINT, initiating shutdown...");
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
}

pub fn should_shutdown() -> bool {
    SHUTDOWN_FLAG.load(Ordering::SeqCst)
}
