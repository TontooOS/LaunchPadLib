# IpcProtocol

LaunchPad uses a Unix socket at `/run/launchpad.sock` for IPC. The protocol
is newline-delimited JSON.

## Connection

Each request opens a new connection, sends one JSON message, reads one JSON
response, and closes the connection.

## Request Format

```json
{
    "action": "start",
    "service": "compositor",
    "options": {
        "head": null,
        "app_path": null
    }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `action` | string | yes | Action to perform |
| `service` | string | no | Service name (required for most actions) |
| `options` | object | no | Additional options |

### Actions

| Action | Requires `service` | Description |
|---|---|---|
| `start` | yes | Start a service |
| `stop` | yes | Stop a service (graceful) |
| `restart` | yes | Restart a service |
| `kill` | yes | Kill a service immediately |
| `activate` | yes | Enable autostart |
| `deactivate` | yes | Disable autostart |
| `list` | no | List all services |
| `log` | yes | Get service log |
| `start_app` | no | Start an .app bundle (uses `app_path`) |

### Options

| Field | Type | Used by | Description |
|---|---|---|---|
| `head` | int | `log` | Return last N lines |
| `app_path` | string | `start_app` | Path to .app bundle |

## Response Format

```json
{
    "success": true,
    "message": null,
    "data": {
        "services": [
            {
                "name": "compositor",
                "service_type": "sys",
                "state": "running",
                "pid": 480,
                "user": "root"
            }
        ]
    }
}
```

| Field | Type | Description |
|---|---|---|
| `success` | bool | Whether the action succeeded |
| `message` | string | Error message or log output |
| `data` | object | Response data (service list, etc.) |

## Error Response

```json
{
    "success": false,
    "message": "Service 'foo' not found",
    "data": null
}
```

## Socket Permissions

The socket is created with mode `0o666` (world-readable/writable) so any
user can communicate with the daemon.

## Cross References

- [Daemon.md](Daemon.md) - Socket server implementation
- [Launchctl.md](Launchctl.md) - CLI usage
- [LaunchpadLib.md](LaunchpadLib.md) - Client library
