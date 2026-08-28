# LaunchPad

Custom init system (PID 1) for TontooOS. Replaces systemd with a
YAML-based service manager, Rust client library, and `launchctl` CLI.

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

Add to your `Cargo.toml`:

```toml
[dependencies]
launchpad-lib = { path = "/Library/System/launchpad" }
```

## License

TCL v26.1
