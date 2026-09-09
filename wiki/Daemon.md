# Daemon

The LaunchPad daemon is the PID 1 init process for TontooOS. It manages all
system and user services, handles process lifecycle, and provides the Unix
socket IPC interface.

## Boot Sequence

1. Mount essential filesystems (`/proc`, `/sys`, `/dev`)
2. Scan `/System/services/*.service` YAML files (`--services-dir` boot arg,
   default `/Library/System/Launchpads`)
3. Resolve dependencies via topological sort
4. Start all `sys` services in dependency order
5. Start all `default` services
6. Start Unix socket server at `/run/launchpad.sock`
7. Start inotify file watcher on launchpads directory
8. Enter main loop (zombie reaper + shutdown signal check)

## Dependency Resolution

Services declare dependencies via `depends_on` in their YAML config. The daemon
performs a topological sort to determine boot order. Circular dependencies cause
a panic at boot.

Example boot order:

```
seatd -> pipewire -> pipewire-pulse -> wireplumber -> compositor -> dock -> topbar
```

## Crash Recovery

When a service crashes:

1. State set to `Crashed`
2. If `restart: true`, auto-restart with exponential backoff:
   - Attempt 1: 3 seconds
   - Attempt 2: 5 seconds
   - Attempt 3: 10 seconds
   - Attempt 4: 30 seconds
   - Attempt 5+: give up (logged as error)
3. Counter resets after successful run > 60 seconds

## Signal Handling

| Signal | Action |
|---|---|
| `SIGTERM` | Graceful shutdown: stop all services, cleanup, exit |
| `SIGINT` | Same as SIGTERM |
| `SIGCHLD` | Reap zombie processes (auto via `SA_NOCLDWAIT`) |

## Shutdown

On shutdown signal:

1. Stop all `low` apps (SIGTERM)
2. Stop all services in reverse dependency order
3. Remove `/run/launchpad.sock`
4. Exit

## Configuration

The daemon takes these CLI flags (kernel boot args for PID 1):

| Flag | Default | Description |
|---|---|---|
| `--services-dir` | `/Library/System/Launchpads` | Service YAML directory (TontooOS boots with `/System/services`) |
| `--control-socket` | `/run/launchpad/control` | Path to the control socket |
| `--log-dir` | `/var/log/launchpad` | Directory for service logs |

## Cross References

- [Launchctl.md](Launchctl.md) - CLI interface
- [ServiceConfig.md](ServiceConfig.md) - YAML format
- [IpcProtocol.md](IpcProtocol.md) - Socket protocol
