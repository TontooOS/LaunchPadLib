# LaunchpadLib

`launchpad-lib` is the Rust client library for communicating with the LaunchPad
daemon. Use it in your apps (Dock, Launcher, etc.) to start, stop, and manage
services programmatically.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
launchpad-lib = { path = "/Library/System/launchpad.library" }
```

## API

### LaunchpadClient

```rust
use launchpad_lib::client::LaunchpadClient;

let client = LaunchpadClient::new()?;
```

| Method | Description |
|---|---|
| `new()` | Connect to LaunchPad daemon |
| `with_socket(path)` | Connect to custom socket path |
| `start(service)` | Start a service |
| `stop(service)` | Stop a service |
| `restart(service)` | Restart a service |
| `kill(service)` | Kill a service immediately |
| `activate(service)` | Enable autostart |
| `deactivate(service)` | Disable autostart |
| `list()` | List all services |
| `log(service, head)` | Get service log lines |
| `start_app(app_path)` | Start an .app bundle |
| `is_running(app_name)` | Check if app is running |

### Types

```rust
use launchpad_lib::types::{ServiceInfo, ServiceState, ServiceType};

// ServiceInfo contains: name, service_type, state, pid, user
// ServiceState: Stopped, Starting, Running, Deactivated, Crashed
// ServiceType: Sys, Default, Low
```

## Example: Starting an App from Dock

```rust
use launchpad_lib::client::LaunchpadClient;

fn on_dock_click(app_path: &str) {
    let client = LaunchpadClient::new().expect("Cannot connect to LaunchPad");

    match client.start_app(app_path) {
        Ok(info) => {
            println!("Started '{}' as '{}' (PID: {})",
                app_path, info.name,
                info.pid.map(|p| p.to_string()).unwrap_or_default()
            );
        }
        Err(e) => eprintln!("Failed to start app: {}", e),
    }
}
```

## Cross References

- [Daemon.md](Daemon.md) - The daemon process
- [Launchctl.md](Launchctl.md) - CLI equivalent
- [AppBundle.md](AppBundle.md) - .app bundle format
