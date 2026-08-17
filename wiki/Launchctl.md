# Launchctl

The `launchctl` CLI provides command-line access to the LaunchPad daemon. It
communicates with the daemon via the Unix socket at `/run/launchpad.sock`.

## Commands

| Command | Description |
|---|---|
| `launchctl start <service>` | Start a service |
| `launchctl stop <service>` | Stop a service (graceful: SIGTERM -> 5s -> SIGKILL) |
| `launchctl restart <service>` | Restart a service |
| `launchctl kill <service>` | Kill a service immediately (SIGKILL) |
| `launchctl activate <service>` | Enable autostart (default services only) |
| `launchctl deactivate <service>` | Disable autostart (default services only) |
| `launchctl list` | List all services with status |
| `launchctl log <service>` | Show full service log |
| `launchctl log <service> --head` | Show last 50 lines of log |
| `launchctl log <service> -n 100` | Show last 100 lines of log |
| `launchctl start-app <path>` | Start an .app bundle |

## Output Format

The `list` command outputs a formatted table:

```
NAME                          TYPE       STATE        PID        USER
------------------------------------------------------------------------
seatd                         sys        running      452        root
networkmanager                sys        running      458        root
pipewire                      sys        running      461        root
compositor                    sys        running      480        root
dock                          sys        running      495        root
topbar                        sys        running      498        root
ollama                        default    running      512        arlo
1_low_terminal                low        running      523        arlo
```

## Error Handling

All errors are printed to stderr with a non-zero exit code. Connection errors
indicate the daemon is not running.

## Cross References

- [Daemon.md](Daemon.md) - The daemon process
- [IpcProtocol.md](IpcProtocol.md) - Socket protocol details
- [LaunchpadLib.md](LaunchpadLib.md) - Rust library equivalent
