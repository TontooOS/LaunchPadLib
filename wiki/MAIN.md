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
| Hardening | [Hardening.md](Hardening.md) | Sandbox fields, capabilities, cgroup limits, device rules |
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

- 2026-10-02: Service sandboxing added to the `.service` format, because
  LaunchPad replaces systemd and every root service would otherwise run
  unconfined. `ServiceConfig` gained `no_new_privs`, `capabilities`,
  `private_tmp`, `protect_system`, `protect_home`, `read_only_paths`,
  `inaccessible_paths`, `memory_max`, `tasks_max`, `cpu_quota`,
  `device_allow` and `env_allow`, plus `is_hardened()` and `is_session_user()`.
  New types `ProtectSystem`, `Capability` (all 41 `CAP_*` names, an unknown
  name is rejected at load time) and `DeviceRule` / `DeviceSelector` /
  `DevicePerms` for the cgroup device filter. `user: session` resolves to the
  seat session owner at spawn time so one file fits the live ISO and an
  installed system. `memory_max` accepts `K` / `M` / `G` suffixes. All fields
  default to no sandboxing, so existing files keep loading, and
  `to_yaml_string` omits unset fields. Still Foundation only, no `serde`. See
  [Hardening.md](Hardening.md) and [ServiceConfig.md](ServiceConfig.md).
- 2026-10-02: `serde` and `serde_yaml` removed. `ServiceConfig` is parsed by `from_yaml_str` / `from_file` on top of `foundation::yaml`, and rendered by `to_yaml_string`. The `type` field is now case-insensitive, which also fixes real `.service` files (they use `type: sys` while the old serde derive only accepted `Sys`). Foundation is the only dependency. See [ServiceConfig.md](ServiceConfig.md) and [IpcProtocol.md](IpcProtocol.md).
- 2026-08-13: Initial wiki, created with LaunchPad implementation.
