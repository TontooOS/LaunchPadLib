# ServiceConfig

LaunchPad uses YAML files for service configuration. Each service has a
`.service` file in `/Library/System/Launchpads/`.

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
| `type` | enum | yes | `sys`, `default`, or `low` |
| `user` | string | yes | User to run as (`root`, `arlo`, etc.) |
| `depends_on` | list | no | Services that must start first |
| `restart` | bool | no | Auto-restart on crash (default: `true`) |

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
/Library/System/Launchpads/
├── seatd.service
├── networkmanager.service
├── pipewire.service
├── pipewire-pulse.service
├── wireplumber.service
├── compositor.service
├── dock.service
├── topbar.service
└── ollama.service
```

## Hot Reload

The daemon watches the launchpads directory with inotify. When a `.service`
file is created or modified, the config is reloaded automatically.

## Cross References

- [Daemon.md](Daemon.md) - Boot sequence and service management
- [AppBundle.md](AppBundle.md) - Low app bundle format
