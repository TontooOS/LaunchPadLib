# Hardening

LaunchPad sandboxes every service it spawns. Because LaunchPad replaces
systemd, the kernel hardening that a unit file would normally request has to be
declared in the `.service` file itself. This page documents the declarative
side: the fields, their defaults and the exact rules the parser enforces.

The daemon side (cgroup v2 setup, namespace order, capability drop) is
documented in the LaunchPad daemon wiki, page `Hardening.md`.

## Why This Exists

systemd contributes two things to a service: lifecycle and confinement. On
TontooOS only the lifecycle half is replaced, and every service that ran as
`root` would otherwise run unconfined. With no `ProtectSystem=`,
`NoNewPrivileges=`, `CapabilityBoundingSet=` or `MemoryMax=`, a parser bug in a
root daemon such as NetworkManager or PipeWire is an instant root compromise,
and one runaway process can exhaust the whole machine because there are no
cgroups either.

## Fields

All hardening fields are optional. A `.service` file that declares none of them
parses and spawns exactly as before, so existing files keep working.

| Field | Type | Default | Effect |
|---|---|---|---|
| `no_new_privs` | bool | `false` | Sets `PR_SET_NO_NEW_PRIVS`, so no `execve` can gain privilege |
| `capabilities` | list of `CAP_*` | empty | Capabilities kept after the bounding set is dropped |
| `private_tmp` | bool | `false` | Mount namespace with a private `tmpfs` on `/tmp` |
| `protect_system` | enum | `off` | Remounts `/usr` and `/boot` read-only |
| `protect_home` | bool | `false` | Makes `/home`, `/root` and `/run/user` inaccessible |
| `read_only_paths` | list of paths | empty | Bind-mounted read-only inside the namespace |
| `inaccessible_paths` | list of paths | empty | Bind-mounted over with an empty, inaccessible directory |
| `memory_max` | byte size | none | `memory.max` in the service cgroup |
| `tasks_max` | integer | none | `pids.max` in the service cgroup |
| `cpu_quota` | number (percent) | none | `cpu.max` in the service cgroup |
| `device_allow` | list of rules | empty | Device filter program for the service cgroup |
| `env_allow` | list of names | empty | Variables kept from the daemon environment |

## Example

```yaml
name: pipewire
execute: /usr/bin/pipewire
type: sys
user: session
depends_on: []
restart: true
no_new_privs: true
capabilities:
  - CAP_DAC_READ_SEARCH
  - CAP_SYS_NICE
private_tmp: true
protect_system: full
protect_home: true
read_only_paths:
  - /usr
  - /System
memory_max: 256M
tasks_max: 512
cpu_quota: 100
device_allow:
  - char 189 rwm
  - /dev/snd/* rw
env_allow:
  - PATH
  - XDG_RUNTIME_DIR
```

## protect_system

| Value | Meaning |
|---|---|
| `off` | Nothing is remounted; the default |
| `full` | `/usr` and `/boot` are read-only |
| `strict` | `full`, plus the whole tree except `/dev`, `/proc` and `/sys` |

`no`, `false` and `yes` are accepted as aliases for `off` and `full`. The
spelling is matched case-insensitively. `ServiceConfig::protect_system` is
`ProtectSystem::Off` when the field is absent.

## capabilities

Entries are `CAP_*` names from the kernel table, matched case-insensitively.
An unknown name is an error at load time, not a silent no-op, so a typo can
never grant the wrong privilege or quietly leave a capability in the bounding
set.

| API | Behavior |
|---|---|
| `Capability::from_config_str` | `Ok(Some)` for a known name, `None` for anything else |
| `Capability::as_config_str` | Canonical `CAP_*` spelling |
| `Capability::as_raw` | Numeric value for `capset` and `PR_CAPBSET_DROP` |
| `Capability::from_raw` | `Err` when the number is not a known capability |
| `Capability::all` | All 41 capabilities in kernel order |

`ServiceConfig::capabilities` lists the capabilities a service **keeps**. Every
capability outside the list is dropped from the bounding set, so the field is
an allowlist and not a delta.

## device_allow

Each rule is `<selector> <perms>`. The permission group is optional and
defaults to `rw`.

| Permission | Meaning |
|---|---|
| `r` | Read the device node |
| `w` | Write to the device node |
| `m` | Create the device node (`mknod`) |

| Selector | Matches |
|---|---|
| `char 189` | Character devices with major 189 |
| `char-major:189` | Same, colon spelling |
| `char 189 rwm` | Selector plus permissions in one rule |
| `block 8` | Block devices with major 8 |
| `char-*` | Every character device |
| `block-*` | Every block device |
| `/dev/snd/*` | An absolute path, `*` is a glob wildcard |

A rule that is not a major spec and not an absolute path returns `Err`, as does
an unknown permission character, a missing major, a relative path and trailing
words. `DeviceRule::to_config_string` renders a rule back to its file spelling
and round-trips through `DeviceRule::parse`.

The daemon turns the list into a cgroup v2 device filter program. Because that
program denies by default, the daemon always adds a baseline of
`/dev/null`, `/dev/zero`, `/dev/random`, `/dev/urandom`, `/dev/full`, `/dev/tty`
and `/dev/ptmx`; a service never has to list them.

## Byte Sizes

`memory_max` accepts a plain integer or a suffix, in the short or long
spelling. The suffix is case-insensitive.

| Spelling | Multiplier |
|---|---|
| `1048576`, `1048576b` | 1 |
| `512K`, `512KB`, `512KiB` | 1024 |
| `256M`, `256MB`, `256MiB` | 1024^2 |
| `1G`, `1GB`, `1GiB` | 1024^3 |

An unknown suffix, a negative value and a non-numeric string each return
`Err`. A value that overflows a `u64` after the multiplier returns
`Err("field `memory_max` overflows a byte count")`.

## Session User

`user: session` is kept verbatim by the parser. The daemon resolves it to the
owner of the active seat session at spawn time, so an audio or shell service
follows the logged-in user on both the live ISO and an installed system.

| API | Behavior |
|---|---|
| `ServiceConfig::is_session_user` | `true` for `session`, `SESSION` and `Session` |
| `SESSION_USER` | The canonical spelling, `session` |

The check is case-insensitive. Any other value, such as `root` or `liveuser`,
is a fixed account name and `is_session_user` returns `false`.

## Validation

| API | Behavior |
|---|---|
| `ServiceConfig::is_hardened` | `true` when any sandboxing field is set |
| `ServiceConfig::from_yaml_str` | `Err` on an unknown capability, a bad device rule, a bad enum value, a wrong type or a non-numeric limit |
| `ServiceConfig::to_yaml_string` | Writes only the fields that are set, so an unhardened file round-trips to the short form |

Booleans accept `true` / `yes` / `1` and `false` / `no` / `0` as strings, the
same as `restart`. A misspelled boolean is an error, never a silent `false`.

## Cross References

- [ServiceConfig.md](ServiceConfig.md) - Full `.service` format and base fields
- [Daemon.md](Daemon.md) - Boot sequence that applies the sandbox
- [IpcProtocol.md](IpcProtocol.md) - Socket protocol
