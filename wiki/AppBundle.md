# AppBundle

TontooOS uses macOS-style `.app` bundles. When launched, the daemon reads
the bundle's `Info.plist` and executes the binary inside.

## Structure

```
Terminal.app/
├── Info.plist
└── bin/
    └── terminal
```

## Info.plist

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Terminal</string>
    <key>CFBundleExecutable</key>
    <string>terminal</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
</dict>
</plist>
```

## Execution Flow

1. Parse `Info.plist` from the `.app` directory
2. Read `CFBundleExecutable` key
3. Resolve binary path: `<app_dir>/bin/<executable>`
4. Fork and exec the binary
5. Track as low-type service with auto-generated ID

## Fallback

If `Info.plist` is missing, the daemon looks for a binary in:

1. `<app_dir>/bin/<app_name_lowercase>`
2. `<app_dir>/bin/<app_name>`
3. `<app_dir>/<app_name_lowercase>`
4. `<app_dir>/<app_name>`

## Launching

### Via launchctl

```bash
launchctl start-app /Applications/Terminal.app
```

### Via Rust Library

```rust
use launchpad_lib::client::LaunchpadClient;

let client = LaunchpadClient::new()?;
let info = client.start_app("/Applications/Terminal.app")?;
// info.name = "1_low_terminal"
// info.state = Running
// info.pid = Some(523)
```

### Via Dock

The Dock calls `LaunchpadClient::start_app()` when a user clicks an app icon.
The daemon creates a new process independent of the Dock, so if the Dock
crashes, the app continues running.

## Low App IDs

Each low app gets a unique ID:

```
1_low_terminal    (first Terminal instance)
2_low_terminal    (second Terminal instance)
1_low_vscode      (first VS Code instance)
```

The counter is per-app-name and persists across restarts (reset only on daemon
restart).

## Cross References

- [ServiceConfig.md](ServiceConfig.md) - YAML config format
- [LaunchpadLib.md](LaunchpadLib.md) - Client library API
- [Daemon.md](Daemon.md) - Process management
