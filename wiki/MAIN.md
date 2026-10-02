# LaunchPad - Wiki

Custom init system (PID 1) for TontooOS. Replaces systemd with a lightweight
YAML-based service manager, Rust client library, and `launchctl` CLI.

- Repository: https://github.com/TontooOS/LaunchPad
- License: TCL v27.0
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Daemon | [Daemon.md](Daemon.md) | PID 1 init daemon and boot sequence |
| Launchctl | [Launchctl.md](Launchctl.md) | CLI for service management |
| LaunchpadLib | [LaunchpadLib.md](LaunchpadLib.md) | Client library for apps |
| ServiceConfig | [ServiceConfig.md](ServiceConfig.md) | YAML service configuration format |
| AppBundle | [AppBundle.md](AppBundle.md) | .app bundle format and execution |
| IpcProtocol | [IpcProtocol.md](IpcProtocol.md) | Unix socket IPC protocol |

## Quick Start

Build and run the daemon (as root):

```bash
cargo build
sudo ./target/debug/launchpad-daemon
```

List all services:

```bash
./target/debug/launchctl list
```

Start a service:

```bash
./target/debug/launchctl start compositor
```

Use the client library in your app:

```rust
use launchpad_lib::client::LaunchpadClient;

let client = LaunchpadClient::new()?;
let info = client.start_app("/Applications/Terminal.app")?;
```

See [Daemon.md](Daemon.md) for the boot sequence and [ServiceConfig.md](ServiceConfig.md)
for the YAML format.

## Changelog

- 2026-10-02: `serde` and `serde_yaml` removed. `ServiceConfig` is parsed by `from_yaml_str` / `from_file` on top of `foundation::yaml`, and rendered by `to_yaml_string`. The `type` field is now case-insensitive, which also fixes real `.service` files (they use `type: sys` while the old serde derive only accepted `Sys`). Foundation is the only dependency. See [ServiceConfig.md](ServiceConfig.md) and [IpcProtocol.md](IpcProtocol.md).
- 2026-08-13: Initial wiki, created with LaunchPad implementation.
