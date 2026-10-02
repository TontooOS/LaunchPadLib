# ServiceConfig

LaunchPad uses YAML files for service configuration. Each service has a
`.service` file in `/System/services/` (selectable via the daemon
`--services-dir` flag; legacy default `/Library/System/Launchpads/`).

## Format

```yaml
name: compositor
execute: /usr/bin/tontoo-compositor
type: sys
user: root
depends_on:
  - seatd
  - pipewire
restart: true
```

## Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Unique service identifier |
| `execute` | string | yes | Binary path and arguments |
| `type` | enum | yes | `sys`, `default`, or `low` (case-insensitive) |
| `user` | string | yes | User to run as (`root`, `arlo`, `session`, etc.) |
| `depends_on` | list | no | Services that must start first |
| `restart` | bool | no | Auto-restart on crash (default: `true`) |
| `no_new_privs` | bool | no | Block privilege gain across `execve` (default: `false`) |
| `capabilities` | list | no | `CAP_*` allowlist kept after the bounding set drop |
| `private_tmp` | bool | no | Private `tmpfs` on `/tmp` (default: `false`) |
| `protect_system` | enum | no | `off`, `full` or `strict` (default: `off`) |
| `protect_home` | bool | no | Hide `/home`, `/root`, `/run/user` (default: `false`) |
| `read_only_paths` | list | no | Paths bind-mounted read-only |
| `inaccessible_paths` | list | no | Paths hidden behind an empty directory |
| `memory_max` | byte size | no | `memory.max` in the service cgroup |
| `tasks_max` | integer | no | `pids.max` in the service cgroup |
| `cpu_quota` | number | no | `cpu.max` percent in the service cgroup |
| `device_allow` | list | no | Device filter rules, see [Hardening.md](Hardening.md) |
| `env_allow` | list | no | Environment variables kept from the daemon |

The hardening fields are documented in [Hardening.md](Hardening.md).

## Session User

`user: session` is a placeholder, not an account. The daemon resolves it to the
owner of the active seat session when the service is spawned, so one file works
on the live ISO and on an installed system.

```yaml
name: pipewire
execute: /usr/bin/pipewire
type: sys
user: session
```

`ServiceConfig::is_session_user` reports whether the placeholder is used; the
spelling is matched case-insensitively.

## API

Service files are read with Foundation's YAML reader. The library owns the
parsing; the daemon only calls these:

```rust
impl ServiceConfig {
    pub fn from_yaml_str(text: &str) -> Result<Self, String>
    pub fn to_yaml_string(&self) -> String
    pub fn from_file(path: &Path) -> Result<Self, String>
    pub fn is_session_user(&self) -> bool
    pub fn is_hardened(&self) -> bool
}

impl ServiceType {
    pub fn from_config_str(s: &str) -> Option<Self>   // case-insensitive
    pub fn as_config_str(&self) -> &'static str      // "sys" | "default" | "low"
    pub fn as_wire_str(&self) -> &'static str        // "Sys" | "Default" | "Low"
    pub fn from_wire_str(s: &str) -> Option<Self>    // socket protocol
}
```

Behavior:

- `name`, `execute`, `type` and `user` are required; a missing field, an
  empty `name` or an empty `execute` returns `Err`.
- `type` is matched case-insensitively, so both `sys` and `Sys` load. Only
  `ServiceType::from_wire_str` is case-sensitive, because the socket
  protocol fixes the spelling.
- `depends_on` defaults to an empty list and must be a list of strings.
- `restart` defaults to `true`; the strings `true` / `yes` / `1` and
  `false` / `no` / `0` are also accepted.
- Every hardening field is optional and defaults to no sandboxing, so a file
  written before hardening existed still loads and reports
  `is_hardened() == false`.
- A file holding more than one `---` document returns `Err`; an empty file
  or a file with only comments returns `Err("Service file is empty")`.
- `to_yaml_string` round-trips through `from_yaml_str` and omits unset
  hardening fields.

## Service Types

### sys

System services that run at boot and cannot be deactivated. These are core
components like the compositor, dock, seatd, pipewire.

```yaml
name: compositor
execute: /usr/bin/tontoo-compositor
type: sys
user: root
```

### default

User services that start at login. Can be activated/deactivated by the user.

```yaml
name: ollama
execute: /usr/bin/ollama
type: default
user: arlo
```

### low

On-demand apps started by Dock or Launcher. These have no YAML files and are
tracked in memory with auto-generated IDs.

Low apps are created dynamically when the Dock calls `start_app()`. The daemon
generates an ID like `1_low_terminal`, `2_low_terminal`, etc.

## File Location

All service files live in:

```
/System/services/
├── seatd.service
├── networkmanager.service
├── pipewire.service
├── pipewire-pulse.service
├── wireplumber.service
├── compositor.service
├── menubar.service
└── FishPerms.service
```

## Hot Reload

The daemon watches the launchpads directory with inotify. When a `.service`
file is created or modified, the config is reloaded automatically.

## Cross References

- [Hardening.md](Hardening.md) - Sandbox fields, capability table, device rules
- [Daemon.md](Daemon.md) - Boot sequence and service management
- [AppBundle.md](AppBundle.md) - Low app bundle format
